/*!
 * Qwen API Service Implementation
 *
 * Qwen Code API with OAuth authentication.
 * Uses OpenAI-compatible format.
 */

use crate::adapter::ApiServiceAdapter;
use crate::common::*;
use anyhow::{Context, Result};
use async_stream::stream;
use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tracing::{debug, info, warn};

const QWEN_API_BASE: &str = "https://api.qwen.aliyun.com/v1";
const QWEN_MODELS: &[&str] = &[
    "qwen3-coder-plus",
    "qwen3-coder-flash",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QwenOAuthCredentials {
    access_token: String,
    refresh_token: Option<String>,
    expiry_date: Option<i64>,
}

pub struct QwenApiService {
    client: Client,
    credentials: Arc<RwLock<QwenOAuthCredentials>>,
    credentials_path: PathBuf,
    max_retries: u32,
    base_delay: u64,
}

impl QwenApiService {
    pub async fn new(
        oauth_creds_file: Option<PathBuf>,
        max_retries: u32,
        base_delay: u64,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))  // 减少到60秒
            .connect_timeout(std::time::Duration::from_secs(10))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .tcp_nodelay(true)
            .build()?;

        let credentials_path = oauth_creds_file.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".qwen")
                .join("oauth_creds.json")
        });

        let credentials = Self::load_credentials_from_file(&credentials_path).await?;

        info!("Qwen API Service initialized");
        Ok(Self {
            client,
            credentials: Arc::new(RwLock::new(credentials)),
            credentials_path,
            max_retries,
            base_delay,
        })
    }

    async fn load_credentials_from_file(path: &PathBuf) -> Result<QwenOAuthCredentials> {
        let content = fs::read_to_string(path)
            .await
            .context("Failed to read Qwen credentials file")?;
        let creds: QwenOAuthCredentials = serde_json::from_str(&content)
            .context("Failed to parse Qwen credentials JSON")?;
        Ok(creds)
    }

    fn is_token_expired(&self, creds: &QwenOAuthCredentials) -> bool {
        if let Some(expiry) = creds.expiry_date {
            let now = chrono::Utc::now().timestamp();
            expiry <= now + 300
        } else {
            false
        }
    }

    async fn refresh_access_token(&self) -> Result<()> {
        info!("Refreshing Qwen access token...");
        warn!("Qwen token refresh not fully implemented - requires Qwen OAuth flow");
        Ok(())
    }

    fn call_api_with_retry<'a>(
        &'a self,
        endpoint: &'a str,
        body: serde_json::Value,
        retry_count: u32,
    ) -> BoxFuture<'a, Result<serde_json::Value>> {
        Box::pin(async move {
        {
            let creds = self.credentials.read().await;
            if self.is_token_expired(&creds) {
                drop(creds);
                self.refresh_access_token().await?;
            }
        }

        let url = format!("{}{}", QWEN_API_BASE, endpoint);
        let creds = self.credentials.read().await;

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", creds.access_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let result: serde_json::Value = response.json().await?;
            return Ok(result);
        }

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
impl ApiServiceAdapter for QwenApiService {
    async fn generate_content(
        &self,
        _model: &str,
        request_body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Qwen generate_content");
        self.call_api_with_retry("/chat/completions", request_body, 0).await
    }

    async fn generate_content_stream(
        &self,
        _model: &str,
        mut request_body: serde_json::Value,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>> {
        debug!("Qwen generate_content_stream");

        if let Some(obj) = request_body.as_object_mut() {
            obj.insert("stream".to_string(), json!(true));
        }

        let creds = self.credentials.read().await;
        let url = format!("{}/chat/completions", QWEN_API_BASE);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", creds.access_token))
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
                                    Err(e) => warn!("Failed to parse chunk: {}", e),
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
        debug!("Qwen list_models");
        
        let models: Vec<ModelInfo> = QWEN_MODELS
            .iter()
            .map(|model_id| ModelInfo {
                id: Some(model_id.to_string()),
                name: Some(model_id.to_string()),
                object: Some("model".to_string()),
                created: Some(chrono::Utc::now().timestamp()),
                owned_by: Some("qwen".to_string()),
                extra: Default::default(),
            })
            .collect();

        Ok(ModelListResponse {
            object: Some("list".to_string()),
            data: Some(models),
            models: None,
        })
    }

    async fn refresh_token(&self) -> Result<()> {
        let creds = self.credentials.read().await;
        if self.is_token_expired(&creds) {
            drop(creds);
            self.refresh_access_token().await?;
        }
        Ok(())
    }
}

