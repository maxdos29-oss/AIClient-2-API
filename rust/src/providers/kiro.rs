/*!
 * Kiro API Service Implementation
 *
 * Kiro provides Claude API access through OAuth authentication.
 * Similar to Claude but with OAuth token management.
 */

use crate::adapter::ApiServiceAdapter;
use crate::common::*;
use anyhow::{Context, Result};
use async_stream::stream;
use async_trait::async_trait;
use base64::Engine;
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

const CLAUDE_MODELS: &[&str] = &[
    "claude-sonnet-4-20250514",
    "claude-3-7-sonnet-20250219",
    "claude-3-5-sonnet-20241022",
    "claude-3-5-haiku-20241022",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KiroOAuthCredentials {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_at: Option<String>,
    #[serde(default)]
    profile_arn: Option<String>,
    #[serde(default)]
    auth_method: Option<String>,
    #[serde(default)]
    provider: Option<String>,
}

pub struct KiroApiService {
    client: Client,
    credentials: Arc<RwLock<KiroOAuthCredentials>>,
    credentials_path: PathBuf,
    max_retries: u32,
    base_delay: u64,
    region: String,
}

impl KiroApiService {
    pub async fn new(
        oauth_creds_base64: Option<String>,
        oauth_creds_file: Option<PathBuf>,
        max_retries: u32,
        base_delay: u64,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        let credentials_path = oauth_creds_file.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".aws/sso/cache")
                .join("kiro-auth-token.json")
        });

        let credentials = if let Some(base64_creds) = oauth_creds_base64 {
            Self::load_credentials_from_base64(&base64_creds)?
        } else {
            Self::load_credentials_from_file(&credentials_path).await?
        };

        // Extract region from profileArn (e.g., "arn:aws:codewhisperer:us-east-1:...")
        let region = credentials
            .profile_arn
            .as_ref()
            .and_then(|arn| {
                let parts: Vec<&str> = arn.split(':').collect();
                if parts.len() >= 4 {
                    Some(parts[3].to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "us-east-1".to_string());

        info!("Kiro API Service initialized with region: {}", region);
        Ok(Self {
            client,
            credentials: Arc::new(RwLock::new(credentials)),
            credentials_path,
            max_retries,
            base_delay,
            region,
        })
    }
    
    fn get_api_url(&self, endpoint: &str) -> String {
        // Kiro uses AWS CodeWhisperer endpoints
        // For /v1/messages endpoint, we need to convert to CodeWhisperer format
        match endpoint {
            "/v1/messages" => {
                format!("https://codewhisperer.{}.amazonaws.com/generateAssistantResponse", self.region)
            }
            _ => {
                format!("https://codewhisperer.{}.amazonaws.com{}", self.region, endpoint)
            }
        }
    }

    fn load_credentials_from_base64(base64_str: &str) -> Result<KiroOAuthCredentials> {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(base64_str)
            .context("Failed to decode base64 credentials")?;
        let creds: KiroOAuthCredentials = serde_json::from_slice(&decoded)
            .context("Failed to parse credentials JSON")?;
        Ok(creds)
    }

    async fn load_credentials_from_file(path: &PathBuf) -> Result<KiroOAuthCredentials> {
        let content = fs::read_to_string(path)
            .await
            .context("Failed to read Kiro credentials file")?;
        let creds: KiroOAuthCredentials = serde_json::from_str(&content)
            .context("Failed to parse Kiro credentials JSON")?;
        Ok(creds)
    }

    fn is_token_expired(&self, creds: &KiroOAuthCredentials) -> bool {
        if let Some(expires_at_str) = &creds.expires_at {
            // Parse ISO 8601 format: "2025-10-06T08:48:25.579Z"
            if let Ok(expires_at) = chrono::DateTime::parse_from_rfc3339(expires_at_str) {
                let now = chrono::Utc::now();
                let expires_at_utc = expires_at.with_timezone(&chrono::Utc);
                // Refresh if expiring within 5 minutes
                expires_at_utc <= now + chrono::Duration::minutes(5)
            } else {
                false
            }
        } else {
            false
        }
    }

    async fn refresh_access_token(&self) -> Result<()> {
        info!("Refreshing Kiro access token...");
        // Token refresh would be implemented here based on Kiro's OAuth flow
        // For now, this is a placeholder
        warn!("Kiro token refresh not fully implemented - requires Kiro OAuth server access");
        Ok(())
    }

    fn call_api_with_retry<'a>(
        &'a self,
        endpoint: &'a str,
        body: serde_json::Value,
        retry_count: u32,
    ) -> BoxFuture<'a, Result<serde_json::Value>> {
        Box::pin(async move {
        // Note: Token expiration check disabled for now
        // {
        //     let creds = self.credentials.read().await;
        //     if self.is_token_expired(&creds) {
        //         drop(creds);
        //         self.refresh_access_token().await?;
        //     }
        // }

        let url = self.get_api_url(endpoint);
        let creds = self.credentials.read().await;

        debug!("Calling Kiro API: {}", url);

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
impl ApiServiceAdapter for KiroApiService {
    async fn generate_content(
        &self,
        _model: &str,
        request_body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Kiro generate_content");
        self.call_api_with_retry("/v1/messages", request_body, 0).await
    }

    async fn generate_content_stream(
        &self,
        _model: &str,
        mut request_body: serde_json::Value,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>> {
        debug!("Kiro generate_content_stream");

        if let Some(obj) = request_body.as_object_mut() {
            obj.insert("stream".to_string(), json!(true));
        }

        let creds = self.credentials.read().await;
        let url = self.get_api_url("/v1/messages");
        
        debug!("Calling Kiro API (streaming): {}", url);
        
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
        debug!("Kiro list_models");
        
        let models: Vec<ModelInfo> = CLAUDE_MODELS
            .iter()
            .map(|model_id| ModelInfo {
                id: Some(model_id.to_string()),
                name: Some(model_id.to_string()),
                object: Some("model".to_string()),
                created: Some(chrono::Utc::now().timestamp()),
                owned_by: Some("kiro".to_string()),
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
        let creds = self.credentials.read().await;
        if self.is_token_expired(&creds) {
            drop(creds);
            self.refresh_access_token().await?;
        }
        Ok(())
    }
}

