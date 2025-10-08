/*!
 * API Service Adapter
 *
 * Defines the adapter interface and implementations for different AI service providers.
 */

use crate::common::*;
use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

/// Trait defining the interface for all AI service adapters
#[async_trait]
pub trait ApiServiceAdapter: Send + Sync {
    /// Generate content (non-streaming)
    async fn generate_content(
        &self,
        model: &str,
        request_body: serde_json::Value,
    ) -> Result<serde_json::Value>;

    /// Generate content (streaming)
    async fn generate_content_stream(
        &self,
        model: &str,
        request_body: serde_json::Value,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>>;

    /// List available models
    async fn list_models(&self) -> Result<ModelListResponse>;

    /// Refresh authentication token (if applicable)
    async fn refresh_token(&self) -> Result<()>;
}

/// Factory function to create appropriate adapter based on provider type
pub async fn create_adapter(
    provider: ModelProvider,
    config: &crate::config::Config,
) -> Result<Box<dyn ApiServiceAdapter>> {
    match provider {
        ModelProvider::GeminiCliOAuth => {
            let service = crate::providers::gemini::GeminiApiService::new(
                config.gemini_oauth_creds_base64.clone(),
                config.gemini_oauth_creds_file_path.clone(),
                config.project_id.clone(),
                config.request_max_retries,
                config.request_base_delay,
            ).await?;
            Ok(Box::new(service))
        }
        ModelProvider::OpenAICustom => {
            let api_key = config.openai_api_key.clone()
                .ok_or_else(|| anyhow::anyhow!("OpenAI API key is required"))?;
            let service = crate::providers::openai::OpenAIApiService::new(
                api_key,
                config.openai_base_url.clone(),
                config.request_max_retries,
                config.request_base_delay,
            )?;
            Ok(Box::new(service))
        }
        ModelProvider::ClaudeCustom => {
            let api_key = config.claude_api_key.clone()
                .ok_or_else(|| anyhow::anyhow!("Claude API key is required"))?;
            let service = crate::providers::claude::ClaudeApiService::new(
                api_key,
                config.claude_base_url.clone(),
                config.request_max_retries,
                config.request_base_delay,
            )?;
            Ok(Box::new(service))
        }
        ModelProvider::ClaudeKiroOAuth => {
            let service = crate::providers::kiro::KiroApiService::new(
                config.kiro_oauth_creds_base64.clone(),
                config.kiro_oauth_creds_file_path.clone(),
                config.request_max_retries,
                config.request_base_delay,
            ).await?;
            Ok(Box::new(service))
        }
        ModelProvider::OpenAIQwenOAuth => {
            let service = crate::providers::qwen::QwenApiService::new(
                config.qwen_oauth_creds_file_path.clone(),
                config.request_max_retries,
                config.request_base_delay,
            ).await?;
            Ok(Box::new(service))
        }
    }
}

