/*!
 * Claude API Service Implementation
 */

use crate::adapter::ApiServiceAdapter;
use crate::common::*;
use anyhow::Result;
use async_stream::stream;
use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::Stream;
use reqwest::Client;
use serde_json::json;
use std::pin::Pin;
use tokio_stream::StreamExt;
use tracing::{debug, warn};

const CLAUDE_MODELS: &[&str] = &[
    "claude-4-sonnet",
    "claude-sonnet-4-20250514",
    "claude-opus-4-20250514",
    "claude-3-7-sonnet-20250219",
    "claude-3-5-sonnet-20241022",
    "claude-3-5-haiku-20241022",
    "claude-3-opus-20240229",
    "claude-3-haiku-20240307",
];

pub struct ClaudeApiService {
    client: Client,
    api_key: String,
    base_url: String,
    max_retries: u32,
    base_delay: u64,
}

impl ClaudeApiService {
    pub fn new(api_key: String, base_url: Option<String>, max_retries: u32, base_delay: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        let base_url = base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string());

        Ok(Self {
            client,
            api_key,
            base_url,
            max_retries,
            base_delay,
        })
    }

    fn call_api_with_retry<'a>(
        &'a self,
        endpoint: &'a str,
        body: serde_json::Value,
        retry_count: u32,
    ) -> BoxFuture<'a, Result<serde_json::Value>> {
        Box::pin(async move {
        let url = format!("{}{}", self.base_url, endpoint);

        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let result: serde_json::Value = response.json().await?;
            return Ok(result);
        }

        // Handle retryable errors
        if (status.as_u16() == 429 || status.is_server_error()) && retry_count < self.max_retries {
            let delay = self.base_delay * 2_u64.pow(retry_count);
            warn!("Request failed with status {}, retrying in {}ms...", status, delay);
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            return self.call_api_with_retry(endpoint, body, retry_count + 1).await;
        }

        let error_text = response.text().await?;
        anyhow::bail!("API call failed ({}): {}", status, error_text)
        })
    }
}

#[async_trait]
impl ApiServiceAdapter for ClaudeApiService {
    async fn generate_content(
        &self,
        _model: &str,
        request_body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Claude generate_content");
        self.call_api_with_retry("/v1/messages", request_body, 0).await
    }

    async fn generate_content_stream(
        &self,
        _model: &str,
        mut request_body: serde_json::Value,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>> {
        debug!("Claude generate_content_stream");

        // Ensure stream flag is set
        if let Some(obj) = request_body.as_object_mut() {
            obj.insert("stream".to_string(), json!(true));
        }

        let url = format!("{}/v1/messages", self.base_url);
        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Stream API call failed: {}", error_text);
        }

        let byte_stream = response.bytes_stream();
        
        let stream = stream! {
            let mut bytes_stream = byte_stream;
            let mut buffer = String::new();
            
            while let Some(chunk_result) = bytes_stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));
                        
                        while let Some(boundary) = buffer.find("\n\n") {
                            let event_block = buffer[..boundary].to_string();
                            buffer = buffer[boundary + 2..].to_string();
                            
                            let mut data = String::new();
                            for line in event_block.lines() {
                                if line.starts_with("data: ") {
                                    data = line[6..].to_string();
                                    break;
                                }
                            }
                            
                            if !data.is_empty() {
                                match serde_json::from_str(&data) {
                                    Ok(parsed) => yield Ok(parsed),
                                    Err(e) => {
                                        warn!("Failed to parse chunk: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(anyhow::anyhow!("Stream error: {}", e));
                        return;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    async fn list_models(&self) -> Result<ModelListResponse> {
        debug!("Claude list_models");
        
        // Claude doesn't have a models endpoint, return hardcoded list
        let models: Vec<ModelInfo> = CLAUDE_MODELS
            .iter()
            .map(|model_id| ModelInfo {
                id: Some(model_id.to_string()),
                name: Some(model_id.to_string()),
                object: Some("model".to_string()),
                created: Some(chrono::Utc::now().timestamp()),
                owned_by: Some("anthropic".to_string()),
                extra: Default::default(),
            })
            .collect();

        Ok(ModelListResponse {
            object: None,
            data: None,
            models: Some(models),
        })
    }

    async fn refresh_token(&self) -> Result<()> {
        // Claude uses static API keys, no refresh needed
        Ok(())
    }
}

