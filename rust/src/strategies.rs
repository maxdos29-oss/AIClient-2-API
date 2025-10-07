/*!
 * Provider Strategy Pattern
 *
 * Defines strategy interfaces for handling different provider protocols.
 */

use crate::common::*;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ProviderStrategy: Send + Sync {
    /// Extract model name and stream info from request
    fn extract_model_and_stream_info(
        &self,
        request: &serde_json::Value,
    ) -> Result<(String, bool)>;

    /// Extract response text
    fn extract_response_text(&self, response: &serde_json::Value) -> Result<String>;

    /// Extract prompt text from request
    fn extract_prompt_text(&self, request: &serde_json::Value) -> Result<String>;

    /// Apply system prompt from file
    async fn apply_system_prompt_from_file(
        &self,
        request: serde_json::Value,
        system_prompt: Option<&str>,
        mode: &str,
    ) -> Result<serde_json::Value>;
}

pub struct GeminiStrategy;
pub struct OpenAIStrategy;
pub struct ClaudeStrategy;

#[async_trait]
impl ProviderStrategy for GeminiStrategy {
    fn extract_model_and_stream_info(
        &self,
        request: &serde_json::Value,
    ) -> Result<(String, bool)> {
        // TODO: Implement
        Ok(("gemini-2.5-flash".to_string(), false))
    }

    fn extract_response_text(&self, response: &serde_json::Value) -> Result<String> {
        // TODO: Implement
        Ok(String::new())
    }

    fn extract_prompt_text(&self, request: &serde_json::Value) -> Result<String> {
        // TODO: Implement
        Ok(String::new())
    }

    async fn apply_system_prompt_from_file(
        &self,
        request: serde_json::Value,
        system_prompt: Option<&str>,
        mode: &str,
    ) -> Result<serde_json::Value> {
        // TODO: Implement
        Ok(request)
    }
}

#[async_trait]
impl ProviderStrategy for OpenAIStrategy {
    fn extract_model_and_stream_info(
        &self,
        request: &serde_json::Value,
    ) -> Result<(String, bool)> {
        // TODO: Implement
        let model = request
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("gpt-3.5-turbo")
            .to_string();
        let stream = request
            .get("stream")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        Ok((model, stream))
    }

    fn extract_response_text(&self, response: &serde_json::Value) -> Result<String> {
        // TODO: Implement
        Ok(String::new())
    }

    fn extract_prompt_text(&self, request: &serde_json::Value) -> Result<String> {
        // TODO: Implement
        Ok(String::new())
    }

    async fn apply_system_prompt_from_file(
        &self,
        request: serde_json::Value,
        system_prompt: Option<&str>,
        mode: &str,
    ) -> Result<serde_json::Value> {
        // TODO: Implement
        Ok(request)
    }
}

#[async_trait]
impl ProviderStrategy for ClaudeStrategy {
    fn extract_model_and_stream_info(
        &self,
        request: &serde_json::Value,
    ) -> Result<(String, bool)> {
        // TODO: Implement
        let model = request
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("claude-3-opus")
            .to_string();
        let stream = request
            .get("stream")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        Ok((model, stream))
    }

    fn extract_response_text(&self, response: &serde_json::Value) -> Result<String> {
        // TODO: Implement
        Ok(String::new())
    }

    fn extract_prompt_text(&self, request: &serde_json::Value) -> Result<String> {
        // TODO: Implement
        Ok(String::new())
    }

    async fn apply_system_prompt_from_file(
        &self,
        request: serde_json::Value,
        system_prompt: Option<&str>,
        mode: &str,
    ) -> Result<serde_json::Value> {
        // TODO: Implement
        Ok(request)
    }
}

