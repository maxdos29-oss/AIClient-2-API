/*!
 * Configuration module
 *
 * Handles loading and managing server configuration from files and command-line arguments.
 */

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// CLI configuration from command-line arguments
#[derive(Debug, Default)]
struct CliConfig {
    config_file: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    api_key: Option<String>,
    model_provider: Option<String>,
    openai_api_key: Option<String>,
    openai_base_url: Option<String>,
    claude_api_key: Option<String>,
    claude_base_url: Option<String>,
    kiro_oauth_creds_file: Option<PathBuf>,
    kiro_oauth_creds_base64: Option<String>,
    gemini_oauth_creds_file: Option<PathBuf>,
    project_id: Option<String>,
    qwen_oauth_creds_file: Option<PathBuf>,
    prompt_log_mode: Option<String>,
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server host address
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Required API key for authentication
    #[serde(default = "default_api_key")]
    pub required_api_key: String,

    /// Primary model provider
    #[serde(default = "default_model_provider")]
    pub model_provider: String,

    /// List of all configured model providers
    #[serde(default)]
    pub default_model_providers: Vec<String>,

    /// OpenAI configuration
    #[serde(default)]
    pub openai_api_key: Option<String>,
    #[serde(default)]
    pub openai_base_url: Option<String>,

    /// Claude configuration
    #[serde(default)]
    pub claude_api_key: Option<String>,
    #[serde(default)]
    pub claude_base_url: Option<String>,

    /// Gemini OAuth configuration
    #[serde(default)]
    pub gemini_oauth_creds_base64: Option<String>,
    #[serde(default)]
    pub gemini_oauth_creds_file_path: Option<PathBuf>,
    #[serde(default)]
    pub project_id: Option<String>,

    /// Kiro OAuth configuration
    #[serde(default)]
    pub kiro_oauth_creds_base64: Option<String>,
    #[serde(default)]
    pub kiro_oauth_creds_file_path: Option<PathBuf>,

    /// Qwen OAuth configuration
    #[serde(default)]
    pub qwen_oauth_creds_file_path: Option<PathBuf>,

    /// System prompt configuration
    #[serde(default = "default_system_prompt_file")]
    pub system_prompt_file_path: PathBuf,
    #[serde(default = "default_system_prompt_mode")]
    pub system_prompt_mode: String,
    #[serde(default)]
    pub system_prompt_content: Option<String>,

    /// Logging configuration
    #[serde(default = "default_prompt_log_mode")]
    pub prompt_log_mode: String,
    #[serde(default = "default_prompt_log_base_name")]
    pub prompt_log_base_name: String,

    /// Retry configuration
    #[serde(default = "default_max_retries")]
    pub request_max_retries: u32,
    #[serde(default = "default_base_delay")]
    pub request_base_delay: u64,

    /// Cron configuration
    #[serde(default = "default_cron_near_minutes")]
    pub cron_near_minutes: u64,
    #[serde(default = "default_cron_refresh_token")]
    pub cron_refresh_token: bool,

    /// Provider pools configuration
    #[serde(default)]
    pub provider_pools_file_path: Option<PathBuf>,
    #[serde(default)]
    pub provider_pools: HashMap<String, Vec<ProviderConfig>>,
}

/// Provider configuration for pool management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub uuid: String,
    
    #[serde(flatten)]
    pub credentials: HashMap<String, serde_json::Value>,
    
    #[serde(default)]
    pub check_model_name: Option<String>,
    
    #[serde(default = "default_healthy")]
    pub is_healthy: bool,
    
    #[serde(default)]
    pub last_used: Option<String>,
    
    #[serde(default)]
    pub usage_count: u64,
    
    #[serde(default)]
    pub error_count: u32,
    
    #[serde(default)]
    pub last_error_time: Option<String>,
}

// Default value functions
fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_api_key() -> String {
    "123456".to_string()
}

fn default_model_provider() -> String {
    "gemini-cli-oauth".to_string()
}

fn default_system_prompt_file() -> PathBuf {
    PathBuf::from("input_system_prompt.txt")
}

fn default_system_prompt_mode() -> String {
    "overwrite".to_string()
}

fn default_prompt_log_mode() -> String {
    "none".to_string()
}

fn default_prompt_log_base_name() -> String {
    "prompt_log".to_string()
}

fn default_max_retries() -> u32 {
    3
}

fn default_base_delay() -> u64 {
    1000
}

fn default_cron_near_minutes() -> u64 {
    15
}

fn default_cron_refresh_token() -> bool {
    true
}

fn default_healthy() -> bool {
    true
}

impl Config {
    /// Load configuration from config file, environment, and command-line arguments
    pub fn load() -> Result<Self> {
        Self::load_with_args(&std::env::args().collect::<Vec<_>>())
    }

    /// Load configuration with custom arguments (for testing)
    pub fn load_with_args(args: &[String]) -> Result<Self> {
        // Parse command-line arguments
        let cli_config = Self::parse_cli_args(args)?;
        
        let config_path = cli_config.config_file.as_deref().unwrap_or("config.json");
        
        let mut config: Config = if let Ok(content) = fs::read_to_string(config_path) {
            serde_json::from_str(&content)
                .context("Failed to parse config.json")?
        } else {
            // Use default configuration if file doesn't exist
            Self::default()
        };

        // Load system prompt content if file exists
        if config.system_prompt_file_path.exists() {
            config.system_prompt_content = fs::read_to_string(&config.system_prompt_file_path).ok();
        }

        // Load provider pools if configured
        if let Some(ref pools_path) = config.provider_pools_file_path {
            if pools_path.exists() {
                let pools_content = fs::read_to_string(pools_path)
                    .context("Failed to read provider pools file")?;
                config.provider_pools = serde_json::from_str(&pools_content)
                    .context("Failed to parse provider pools file")?;
            }
        }

        // Merge CLI arguments into config
        config.merge_cli_args(cli_config);

        // Normalize provider configuration
        config.normalize_providers();

        Ok(config)
    }

    /// Parse command-line arguments
    fn parse_cli_args(args: &[String]) -> Result<CliConfig> {
        let mut cli_config = CliConfig::default();
        let mut i = 1; // Skip program name
        
        while i < args.len() {
            match args[i].as_str() {
                "--host" if i + 1 < args.len() => {
                    cli_config.host = Some(args[i + 1].clone());
                    i += 2;
                }
                "--port" if i + 1 < args.len() => {
                    cli_config.port = Some(args[i + 1].parse()?);
                    i += 2;
                }
                "--api-key" if i + 1 < args.len() => {
                    cli_config.api_key = Some(args[i + 1].clone());
                    i += 2;
                }
                "--model-provider" if i + 1 < args.len() => {
                    cli_config.model_provider = Some(args[i + 1].clone());
                    i += 2;
                }
                "--openai-api-key" if i + 1 < args.len() => {
                    cli_config.openai_api_key = Some(args[i + 1].clone());
                    i += 2;
                }
                "--openai-base-url" if i + 1 < args.len() => {
                    cli_config.openai_base_url = Some(args[i + 1].clone());
                    i += 2;
                }
                "--claude-api-key" if i + 1 < args.len() => {
                    cli_config.claude_api_key = Some(args[i + 1].clone());
                    i += 2;
                }
                "--claude-base-url" if i + 1 < args.len() => {
                    cli_config.claude_base_url = Some(args[i + 1].clone());
                    i += 2;
                }
                "--kiro-oauth-creds-file" if i + 1 < args.len() => {
                    cli_config.kiro_oauth_creds_file = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                }
                "--kiro-oauth-creds-base64" if i + 1 < args.len() => {
                    cli_config.kiro_oauth_creds_base64 = Some(args[i + 1].clone());
                    i += 2;
                }
                "--gemini-oauth-creds-file" if i + 1 < args.len() => {
                    cli_config.gemini_oauth_creds_file = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                }
                "--project-id" if i + 1 < args.len() => {
                    cli_config.project_id = Some(args[i + 1].clone());
                    i += 2;
                }
                "--qwen-oauth-creds-file" if i + 1 < args.len() => {
                    cli_config.qwen_oauth_creds_file = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                }
                "--config" if i + 1 < args.len() => {
                    cli_config.config_file = Some(args[i + 1].clone());
                    i += 2;
                }
                "--log-prompts" if i + 1 < args.len() => {
                    cli_config.prompt_log_mode = Some(args[i + 1].clone());
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }
        
        Ok(cli_config)
    }

    /// Merge CLI arguments into config
    fn merge_cli_args(&mut self, cli: CliConfig) {
        if let Some(host) = cli.host {
            self.host = host;
        }
        if let Some(port) = cli.port {
            self.port = port;
        }
        if let Some(key) = cli.api_key {
            self.required_api_key = key;
        }
        if let Some(provider) = cli.model_provider {
            self.model_provider = provider;
        }
        if let Some(key) = cli.openai_api_key {
            self.openai_api_key = Some(key);
        }
        if let Some(url) = cli.openai_base_url {
            self.openai_base_url = Some(url);
        }
        if let Some(key) = cli.claude_api_key {
            self.claude_api_key = Some(key);
        }
        if let Some(url) = cli.claude_base_url {
            self.claude_base_url = Some(url);
        }
        if let Some(file) = cli.kiro_oauth_creds_file {
            self.kiro_oauth_creds_file_path = Some(file);
        }
        if let Some(base64) = cli.kiro_oauth_creds_base64 {
            self.kiro_oauth_creds_base64 = Some(base64);
        }
        if let Some(file) = cli.gemini_oauth_creds_file {
            self.gemini_oauth_creds_file_path = Some(file);
        }
        if let Some(id) = cli.project_id {
            self.project_id = Some(id);
        }
        if let Some(file) = cli.qwen_oauth_creds_file {
            self.qwen_oauth_creds_file_path = Some(file);
        }
        if let Some(mode) = cli.prompt_log_mode {
            self.prompt_log_mode = mode;
        }
    }

    /// Normalize and validate provider configuration
    fn normalize_providers(&mut self) {
        if self.default_model_providers.is_empty() {
            self.default_model_providers.push(self.model_provider.clone());
        }
        
        // Deduplicate providers
        self.default_model_providers.sort();
        self.default_model_providers.dedup();
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            required_api_key: default_api_key(),
            model_provider: default_model_provider(),
            default_model_providers: vec![],
            openai_api_key: None,
            openai_base_url: None,
            claude_api_key: None,
            claude_base_url: None,
            gemini_oauth_creds_base64: None,
            gemini_oauth_creds_file_path: None,
            project_id: None,
            kiro_oauth_creds_base64: None,
            kiro_oauth_creds_file_path: None,
            qwen_oauth_creds_file_path: None,
            system_prompt_file_path: default_system_prompt_file(),
            system_prompt_mode: default_system_prompt_mode(),
            system_prompt_content: None,
            prompt_log_mode: default_prompt_log_mode(),
            prompt_log_base_name: default_prompt_log_base_name(),
            request_max_retries: default_max_retries(),
            request_base_delay: default_base_delay(),
            cron_near_minutes: default_cron_near_minutes(),
            cron_refresh_token: default_cron_refresh_token(),
            provider_pools_file_path: None,
            provider_pools: HashMap::new(),
        }
    }
}

