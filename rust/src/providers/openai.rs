/*!
 * OpenAI API Service Implementation
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

pub struct OpenAIApiService {
    client: Client,
    api_key: String,
    base_url: String,
    max_retries: u32,
    base_delay: u64,
}

impl OpenAIApiService {
    pub fn new(api_key: String, base_url: Option<String>, max_retries: u32, base_delay: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        let base_url = base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string());

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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
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
impl ApiServiceAdapter for OpenAIApiService {
    async fn generate_content(
        &self,
        _model: &str,
        request_body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("OpenAI generate_content");
        self.call_api_with_retry("/chat/completions", request_body, 0).await
    }

    async fn generate_content_stream(
        &self,
        _model: &str,
        mut request_body: serde_json::Value,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>> {
        debug!("OpenAI generate_content_stream");

        // Ensure stream flag is set
        if let Some(obj) = request_body.as_object_mut() {
            obj.insert("stream".to_string(), json!(true));
        }

        let url = format!("{}/chat/completions", self.base_url);
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
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
                        
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].trim().to_string();
                            buffer = buffer[newline_pos + 1..].to_string();
                            
                            if line.starts_with("data: ") {
                                let json_data = &line[6..];
                                if json_data == "[DONE]" {
                                    return;
                                }
                                
                                match serde_json::from_str(json_data) {
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
        debug!("OpenAI list_models");
        
        let url = format!("{}/models", self.base_url);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("List models failed: {}", error_text);
        }

        let result: ModelListResponse = response.json().await?;
        Ok(result)
    }

    async fn refresh_token(&self) -> Result<()> {
        // OpenAI uses static API keys, no refresh needed
        Ok(())
    }
}

