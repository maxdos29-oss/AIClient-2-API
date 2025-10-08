package common

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// Config holds all server configuration
type Config struct {
	RequiredAPIKey            string   `json:"REQUIRED_API_KEY"`
	ServerPort                int      `json:"SERVER_PORT"`
	Host                      string   `json:"HOST"`
	ModelProvider             string   `json:"MODEL_PROVIDER"`
	DefaultModelProviders     []string `json:"-"`
	OpenAIAPIKey              string   `json:"OPENAI_API_KEY"`
	OpenAIBaseURL             string   `json:"OPENAI_BASE_URL"`
	ClaudeAPIKey              string   `json:"CLAUDE_API_KEY"`
	ClaudeBaseURL             string   `json:"CLAUDE_BASE_URL"`
	GeminiOAuthCredsBase64    string   `json:"GEMINI_OAUTH_CREDS_BASE64"`
	GeminiOAuthCredsFilePath  string   `json:"GEMINI_OAUTH_CREDS_FILE_PATH"`
	KiroOAuthCredsBase64      string   `json:"KIRO_OAUTH_CREDS_BASE64"`
	KiroOAuthCredsFilePath    string   `json:"KIRO_OAUTH_CREDS_FILE_PATH"`
	QwenOAuthCredsFilePath    string   `json:"QWEN_OAUTH_CREDS_FILE_PATH"`
	ProjectID                 string   `json:"PROJECT_ID"`
	SystemPromptFilePath      string   `json:"SYSTEM_PROMPT_FILE_PATH"`
	SystemPromptMode          string   `json:"SYSTEM_PROMPT_MODE"`
	SystemPromptContent       string   `json:"-"`
	PromptLogBaseName         string   `json:"PROMPT_LOG_BASE_NAME"`
	PromptLogMode             string   `json:"PROMPT_LOG_MODE"`
	PromptLogFilename         string   `json:"-"`
	RequestMaxRetries         int      `json:"REQUEST_MAX_RETRIES"`
	RequestBaseDelay          int      `json:"REQUEST_BASE_DELAY"`
	CronNearMinutes           int      `json:"CRON_NEAR_MINUTES"`
	CronRefreshToken          bool     `json:"CRON_REFRESH_TOKEN"`
	ProviderPoolsFilePath     string   `json:"PROVIDER_POOLS_FILE_PATH"`
	ProviderPools             map[string][]map[string]interface{} `json:"-"`
	UUID                      string   `json:"-"` // For provider pool instances
}

// InitializeConfig loads configuration from file and merges with provided config
func InitializeConfig(config *Config) error {
	// Try to load config.json
	configFile := "config.json"
	if _, err := os.Stat(configFile); err == nil {
		data, err := os.ReadFile(configFile)
		if err == nil {
			var fileConfig Config
			if err := json.Unmarshal(data, &fileConfig); err == nil {
				// Merge file config with provided config (command line flags take precedence)
				mergeConfig(config, &fileConfig)
				fmt.Println("[Config] Loaded configuration from config.json")
			}
		}
	}

	// Set defaults if not set
	if config.RequiredAPIKey == "" {
		config.RequiredAPIKey = "123456"
	}
	if config.ServerPort == 0 {
		config.ServerPort = 3000
	}
	if config.Host == "" {
		config.Host = "localhost"
	}
	if config.ModelProvider == "" {
		config.ModelProvider = ModelProviderGeminiCLI
	}
	if config.SystemPromptFilePath == "" {
		config.SystemPromptFilePath = "input_system_prompt.txt"
	}
	if config.SystemPromptMode == "" {
		config.SystemPromptMode = "overwrite"
	}
	if config.PromptLogBaseName == "" {
		config.PromptLogBaseName = "prompt_log"
	}
	if config.PromptLogMode == "" {
		config.PromptLogMode = "none"
	}
	if config.RequestMaxRetries == 0 {
		config.RequestMaxRetries = 3
	}
	if config.RequestBaseDelay == 0 {
		config.RequestBaseDelay = 1000
	}
	if config.CronNearMinutes == 0 {
		config.CronNearMinutes = 15
	}

	// Normalize model providers
	normalizeConfiguredProviders(config)

	// Load system prompt content
	if config.SystemPromptFilePath != "" {
		content, err := loadSystemPromptFile(config.SystemPromptFilePath)
		if err == nil {
			config.SystemPromptContent = content
		}
	}

	// Set prompt log filename if file logging is enabled
	if config.PromptLogMode == "file" {
		now := time.Now()
		timestamp := now.Format("20060102-150405")
		config.PromptLogFilename = fmt.Sprintf("%s-%s.log", config.PromptLogBaseName, timestamp)
	}

	// Load provider pools if configured
	if config.ProviderPoolsFilePath != "" {
		data, err := os.ReadFile(config.ProviderPoolsFilePath)
		if err == nil {
			var pools map[string][]map[string]interface{}
			if err := json.Unmarshal(data, &pools); err == nil {
				config.ProviderPools = pools
				fmt.Printf("[Config] Loaded provider pools from %s\n", config.ProviderPoolsFilePath)
			} else {
				fmt.Printf("[Config Error] Failed to parse provider pools: %v\n", err)
			}
		} else {
			fmt.Printf("[Config Error] Failed to load provider pools: %v\n", err)
		}
	}

	return nil
}

// mergeConfig merges fileConfig into config, but only for fields that are not set in config
func mergeConfig(config, fileConfig *Config) {
	if config.RequiredAPIKey == "" && fileConfig.RequiredAPIKey != "" {
		config.RequiredAPIKey = fileConfig.RequiredAPIKey
	}
	if config.ServerPort == 0 && fileConfig.ServerPort != 0 {
		config.ServerPort = fileConfig.ServerPort
	}
	if config.Host == "" && fileConfig.Host != "" {
		config.Host = fileConfig.Host
	}
	if config.ModelProvider == "" && fileConfig.ModelProvider != "" {
		config.ModelProvider = fileConfig.ModelProvider
	}
	if config.OpenAIAPIKey == "" && fileConfig.OpenAIAPIKey != "" {
		config.OpenAIAPIKey = fileConfig.OpenAIAPIKey
	}
	if config.OpenAIBaseURL == "" && fileConfig.OpenAIBaseURL != "" {
		config.OpenAIBaseURL = fileConfig.OpenAIBaseURL
	}
	if config.ClaudeAPIKey == "" && fileConfig.ClaudeAPIKey != "" {
		config.ClaudeAPIKey = fileConfig.ClaudeAPIKey
	}
	if config.ClaudeBaseURL == "" && fileConfig.ClaudeBaseURL != "" {
		config.ClaudeBaseURL = fileConfig.ClaudeBaseURL
	}
	if config.GeminiOAuthCredsBase64 == "" && fileConfig.GeminiOAuthCredsBase64 != "" {
		config.GeminiOAuthCredsBase64 = fileConfig.GeminiOAuthCredsBase64
	}
	if config.GeminiOAuthCredsFilePath == "" && fileConfig.GeminiOAuthCredsFilePath != "" {
		config.GeminiOAuthCredsFilePath = fileConfig.GeminiOAuthCredsFilePath
	}
	if config.ProjectID == "" && fileConfig.ProjectID != "" {
		config.ProjectID = fileConfig.ProjectID
	}
	if config.KiroOAuthCredsBase64 == "" && fileConfig.KiroOAuthCredsBase64 != "" {
		config.KiroOAuthCredsBase64 = fileConfig.KiroOAuthCredsBase64
	}
	if config.KiroOAuthCredsFilePath == "" && fileConfig.KiroOAuthCredsFilePath != "" {
		config.KiroOAuthCredsFilePath = fileConfig.KiroOAuthCredsFilePath
	}
	if config.QwenOAuthCredsFilePath == "" && fileConfig.QwenOAuthCredsFilePath != "" {
		config.QwenOAuthCredsFilePath = fileConfig.QwenOAuthCredsFilePath
	}
	if config.SystemPromptFilePath == "" && fileConfig.SystemPromptFilePath != "" {
		config.SystemPromptFilePath = fileConfig.SystemPromptFilePath
	}
	if config.SystemPromptMode == "" && fileConfig.SystemPromptMode != "" {
		config.SystemPromptMode = fileConfig.SystemPromptMode
	}
	if config.PromptLogBaseName == "" && fileConfig.PromptLogBaseName != "" {
		config.PromptLogBaseName = fileConfig.PromptLogBaseName
	}
	if config.PromptLogMode == "" && fileConfig.PromptLogMode != "" {
		config.PromptLogMode = fileConfig.PromptLogMode
	}
	if config.RequestMaxRetries == 0 && fileConfig.RequestMaxRetries != 0 {
		config.RequestMaxRetries = fileConfig.RequestMaxRetries
	}
	if config.RequestBaseDelay == 0 && fileConfig.RequestBaseDelay != 0 {
		config.RequestBaseDelay = fileConfig.RequestBaseDelay
	}
	if config.CronNearMinutes == 0 && fileConfig.CronNearMinutes != 0 {
		config.CronNearMinutes = fileConfig.CronNearMinutes
	}
	if config.ProviderPoolsFilePath == "" && fileConfig.ProviderPoolsFilePath != "" {
		config.ProviderPoolsFilePath = fileConfig.ProviderPoolsFilePath
	}
}

// normalizeConfiguredProviders normalizes the configured providers
func normalizeConfiguredProviders(config *Config) {
	if len(config.DefaultModelProviders) == 0 {
		config.DefaultModelProviders = []string{config.ModelProvider}
	}

	// Deduplicate and validate providers
	validProviders := []string{
		ModelProviderOpenAICustom,
		ModelProviderClaudeCustom,
		ModelProviderGeminiCLI,
		ModelProviderKiroAPI,
		ModelProviderQwenAPI,
	}

	deduped := make([]string, 0)
	seen := make(map[string]bool)

	for _, provider := range config.DefaultModelProviders {
		provider = strings.TrimSpace(provider)
		if provider == "" {
			continue
		}

		// Check if valid
		isValid := false
		for _, valid := range validProviders {
			if strings.EqualFold(provider, valid) {
				provider = valid // Use the canonical form
				isValid = true
				break
			}
		}

		if !isValid {
			fmt.Printf("[Config Warning] Unknown model provider '%s'. This entry will be ignored.\n", provider)
			continue
		}

		if !seen[provider] {
			deduped = append(deduped, provider)
			seen[provider] = true
		}
	}

	if len(deduped) == 0 {
		deduped = []string{ModelProviderGeminiCLI}
	}

	config.DefaultModelProviders = deduped
	config.ModelProvider = deduped[0]
}

// loadSystemPromptFile loads system prompt content from file
func loadSystemPromptFile(filePath string) (string, error) {
	if filePath == "" {
		return "", nil
	}

	// Expand home directory if needed
	if strings.HasPrefix(filePath, "~/") {
		home, err := os.UserHomeDir()
		if err == nil {
			filePath = filepath.Join(home, filePath[2:])
		}
	}

	data, err := os.ReadFile(filePath)
	if err != nil {
		if os.IsNotExist(err) {
			fmt.Printf("[System Prompt] Specified system prompt file not found: %s\n", filePath)
		} else {
			fmt.Printf("[System Prompt] Error reading system prompt file %s: %v\n", filePath, err)
		}
		return "", err
	}

	content := string(data)
	if strings.TrimSpace(content) == "" {
		return "", nil
	}

	fmt.Printf("[System Prompt] Loaded system prompt from %s\n", filePath)
	return content, nil
}

