/*!
 * Gemini API Service Implementation
 */

use crate::adapter::ApiServiceAdapter;
use crate::common::*;
use anyhow::{Context, Result};
use async_stream::stream;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
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
use tracing::{debug, info, warn};

// Constants
const CODE_ASSIST_ENDPOINT: &str = "https://cloudcode-pa.googleapis.com";
const CODE_ASSIST_API_VERSION: &str = "v1internal";
const OAUTH_CLIENT_ID: &str = "681255809395-oo8ft2oprdrnp9e3aqf6av3hmdib135j.apps.googleusercontent.com";
const OAUTH_CLIENT_SECRET: &str = "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";
const GEMINI_MODELS: &[&str] = &["gemini-2.5-flash", "gemini-2.5-flash-lite", "gemini-2.5-pro"];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OAuthCredentials {
    access_token: String,
    refresh_token: Option<String>,
    expiry_date: Option<i64>,
    token_type: Option<String>,
}

pub struct GeminiApiService {
    client: Client,
    credentials: Arc<RwLock<OAuthCredentials>>,
    credentials_path: PathBuf,
    project_id: Arc<RwLock<Option<String>>>,
    available_models: Vec<String>,
    max_retries: u32,
    base_delay: u64,
}

impl GeminiApiService {
    pub async fn new(
        oauth_creds_base64: Option<String>,
        oauth_creds_file: Option<PathBuf>,
        project_id: Option<String>,
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

        // Determine credentials path
        let credentials_path = oauth_creds_file.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".gemini")
                .join("oauth_creds.json")
        });

        // Load credentials
        let credentials = if let Some(base64_creds) = oauth_creds_base64 {
            Self::load_credentials_from_base64(&base64_creds)?
        } else {
            Self::load_credentials_from_file(&credentials_path).await?
        };

        let mut service = Self {
            client,
            credentials: Arc::new(RwLock::new(credentials)),
            credentials_path,
            project_id: Arc::new(RwLock::new(project_id)),
            available_models: GEMINI_MODELS.iter().map(|s| s.to_string()).collect(),
            max_retries,
            base_delay,
        };

        // Discover project ID if not provided
        if service.project_id.read().await.is_none() {
            let discovered_id = service.discover_project_id().await?;
            *service.project_id.write().await = Some(discovered_id);
        }

        info!("Gemini API Service initialized successfully");
        Ok(service)
    }

    fn load_credentials_from_base64(base64_str: &str) -> Result<OAuthCredentials> {
        let decoded = general_purpose::STANDARD
            .decode(base64_str)
            .context("Failed to decode base64 credentials")?;
        let creds: OAuthCredentials = serde_json::from_slice(&decoded)
            .context("Failed to parse credentials JSON")?;
        Ok(creds)
    }

    async fn load_credentials_from_file(path: &PathBuf) -> Result<OAuthCredentials> {
        let content = fs::read_to_string(path)
            .await
            .context("Failed to read credentials file")?;
        let creds: OAuthCredentials = serde_json::from_str(&content)
            .context("Failed to parse credentials JSON")?;
        Ok(creds)
    }

    async fn save_credentials(&self) -> Result<()> {
        let creds = self.credentials.read().await;
        let json = serde_json::to_string_pretty(&*creds)?;
        fs::write(&self.credentials_path, json).await?;
        Ok(())
    }

    fn is_token_expired(&self, creds: &OAuthCredentials) -> bool {
        if let Some(expiry) = creds.expiry_date {
            let now = Utc::now().timestamp();
            expiry <= now + 300 // Refresh if expiring within 5 minutes
        } else {
            false
        }
    }

    async fn refresh_access_token(&self) -> Result<()> {
        let creds = self.credentials.read().await;
        
        if let Some(refresh_token) = &creds.refresh_token {
            info!("Refreshing Gemini access token...");
            
            let params = [
                ("client_id", OAUTH_CLIENT_ID),
                ("client_secret", OAUTH_CLIENT_SECRET),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ];

            let response = self.client
                .post("https://oauth2.googleapis.com/token")
                .form(&params)
                .send()
                .await
                .context("Failed to refresh token")?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                anyhow::bail!("Token refresh failed: {}", error_text);
            }

            let token_response: serde_json::Value = response.json().await?;
            
            drop(creds);
            let mut creds_write = self.credentials.write().await;
            
            if let Some(access_token) = token_response.get("access_token").and_then(|v| v.as_str()) {
                creds_write.access_token = access_token.to_string();
            }
            
            if let Some(expires_in) = token_response.get("expires_in").and_then(|v| v.as_i64()) {
                creds_write.expiry_date = Some(Utc::now().timestamp() + expires_in);
            }
            
            drop(creds_write);
            self.save_credentials().await?;
            
            info!("Gemini access token refreshed successfully");
        } else {
            warn!("No refresh token available");
        }

        Ok(())
    }

    async fn discover_project_id(&self) -> Result<String> {
        info!("Discovering Gemini project ID...");
        
        let response = self.call_api("loadCodeAssist", json!({
            "metadata": { "pluginType": "GEMINI" }
        })).await?;

        if let Some(project) = response.get("cloudaicompanionProject").and_then(|v| v.as_str()) {
            info!("Discovered project ID: {}", project);
            return Ok(project.to_string());
        }

        // If no project exists, onboard user
        warn!("No project found, onboarding user...");
        
        let default_tier = response
            .get("allowedTiers")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.iter().find(|t| t.get("isDefault").and_then(|v| v.as_bool()).unwrap_or(false)))
            .ok_or_else(|| anyhow::anyhow!("No default tier found"))?;

        let tier_id = default_tier
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("free-tier");

        let onboard_request = json!({
            "tierId": tier_id,
            "metadata": { "pluginType": "GEMINI" },
            "cloudaicompanionProject": "default"
        });

        let mut lro = self.call_api("onboardUser", onboard_request.clone()).await?;
        
        // Poll until operation completes
        while !lro.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            lro = self.call_api("onboardUser", onboard_request.clone()).await?;
        }

        let project_id = lro
            .get("response")
            .and_then(|v| v.get("cloudaicompanionProject"))
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Failed to get project ID from onboarding response"))?;

        info!("Project onboarded successfully: {}", project_id);
        Ok(project_id.to_string())
    }

    async fn call_api(&self, method: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        self.call_api_with_retry(method, body, 0).await
    }

    fn call_api_with_retry<'a>(
        &'a self,
        method: &'a str,
        body: serde_json::Value,
        retry_count: u32,
    ) -> BoxFuture<'a, Result<serde_json::Value>> {
        Box::pin(async move {
        // Check and refresh token if needed
        {
            let creds = self.credentials.read().await;
            if self.is_token_expired(&creds) {
                drop(creds);
                self.refresh_access_token().await?;
            }
        }

        let project_id = self.project_id.read().await;
        let project_id = project_id.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Project ID not available"))?;

        let url = format!(
            "{}/{}/projects/{}/locations/us-central1/cloudaicompanion:{}",
            CODE_ASSIST_ENDPOINT, CODE_ASSIST_API_VERSION, project_id, method
        );

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

        // Handle retryable errors
        if (status.as_u16() == 429 || status.is_server_error()) && retry_count < self.max_retries {
            let delay = self.base_delay * 2_u64.pow(retry_count);
            warn!("Request failed with status {}, retrying in {}ms...", status, delay);
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            return self.call_api_with_retry(method, body, retry_count + 1).await;
        }

        let error_text = response.text().await?;
        anyhow::bail!("API call failed ({}): {}", status, error_text)
        })
    }
}

#[async_trait]
impl ApiServiceAdapter for GeminiApiService {
    async fn generate_content(
        &self,
        model: &str,
        request_body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Generating content with model: {}", model);
        
        let response = self.call_api("generateContent", request_body).await?;
        
        // Transform to Gemini-compliant format
        let compliant = json!({
            "candidates": response.get("candidates"),
            "usageMetadata": response.get("usageMetadata"),
            "promptFeedback": response.get("promptFeedback"),
        });
        
        Ok(compliant)
    }

    async fn generate_content_stream(
        &self,
        model: &str,
        request_body: serde_json::Value,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>> {
        debug!("Generating streaming content with model: {}", model);
        
        // For now, implement as non-streaming and convert
        // TODO: Implement true streaming
        let response = self.generate_content(model, request_body).await?;
        
        let stream = stream! {
            yield Ok(response);
        };
        
        Ok(Box::pin(stream))
    }

    async fn list_models(&self) -> Result<ModelListResponse> {
        debug!("Listing Gemini models");
        
        let models: Vec<ModelInfo> = self.available_models
            .iter()
            .map(|model_id| ModelInfo {
                id: Some(format!("models/{}", model_id)),
                name: Some(model_id.clone()),
                object: Some("model".to_string()),
                created: Some(Utc::now().timestamp()),
                owned_by: Some("google".to_string()),
                extra: Default::default(),
            })
            .collect();

        Ok(ModelListResponse {
            object: Some("list".to_string()),
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

