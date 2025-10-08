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
use tracing::{debug, error, info, warn};
use uuid::Uuid;

const CLAUDE_MODELS: &[&str] = &[
    "claude-sonnet-4-20250514",
    "claude-sonnet-4-5-20250929",
    "claude-3-7-sonnet-20250219",
    "claude-3-5-sonnet-20241022",
    "claude-3-5-haiku-20241022",
    "amazonq-claude-sonnet-4-20250514",
    "amazonq-claude-sonnet-4-5-20250929",
    "amazonq-claude-3-7-sonnet-20250219",
];

const CHAT_TRIGGER_TYPE_MANUAL: &str = "MANUAL";
const ORIGIN_AI_EDITOR: &str = "AI_EDITOR";

// Model mapping from Claude to CodeWhisperer format
fn map_model_to_codewhisperer(model: &str) -> &'static str {
    match model {
        "claude-sonnet-4-20250514" => "CLAUDE_SONNET_4_20250514_V1_0",
        "claude-sonnet-4-5-20250929" => "CLAUDE_SONNET_4_5_20250929_V1_0",
        "claude-3-7-sonnet-20250219" => "CLAUDE_3_7_SONNET_20250219_V1_0",
        "claude-3-5-sonnet-20241022" => "CLAUDE_3_5_SONNET_20241022_V1_0",
        "amazonq-claude-sonnet-4-20250514" => "CLAUDE_SONNET_4_20250514_V1_0",
        "amazonq-claude-sonnet-4-5-20250929" => "CLAUDE_SONNET_4_5_20250929_V1_0",
        "amazonq-claude-3-7-sonnet-20250219" => "CLAUDE_3_7_SONNET_20250219_V1_0",
        _ => "CLAUDE_SONNET_4_20250514_V1_0", // Default
    }
}

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
    
    /// Convert Claude messages format to CodeWhisperer request format
    async fn build_codewhisperer_request(&self, claude_request: &serde_json::Value) -> Result<serde_json::Value> {
        let messages = claude_request.get("messages")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid messages field"))?;

        if messages.is_empty() {
            anyhow::bail!("No user messages found");
        }

        let model = claude_request.get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("claude-sonnet-4-20250514");

        let codewhisperer_model = map_model_to_codewhisperer(model);
        let conversation_id = Uuid::new_v4().to_string();

        let tools_context = self.build_tools_context(claude_request.get("tools"));
        let system_prompt = claude_request.get("system")
            .map(|v| self.extract_content_text(v))
            .unwrap_or_default();

        let mut history = Vec::new();
        let mut start_index = 0;

        // Handle system prompt
        if !system_prompt.is_empty() {
            if messages.first().and_then(|m| m.get("role").and_then(|r| r.as_str())) == Some("user") {
                let first_user_content = self.extract_content_text(&messages[0]);
                history.push(json!({
                    "userInputMessage": {
                        "content": format!("{}\n\n{}", system_prompt, first_user_content),
                        "modelId": codewhisperer_model,
                        "origin": ORIGIN_AI_EDITOR,
                    }
                }));
                start_index = 1;
            } else {
                history.push(json!({
                    "userInputMessage": {
                        "content": system_prompt,
                        "modelId": codewhisperer_model,
                        "origin": ORIGIN_AI_EDITOR,
                    }
                }));
            }
        }

        // Process all but the last message into history
        for i in start_index..messages.len().saturating_sub(1) {
            let message = &messages[i];
            let role = message.get("role").and_then(|r| r.as_str()).unwrap_or("");

            match role {
                "user" => {
                    let (content, images, tool_results, tools_used) = self.extract_user_message_details(message);
                    let mut user_msg = json!({
                        "content": content,
                        "modelId": codewhisperer_model,
                        "origin": ORIGIN_AI_EDITOR,
                        "userInputMessageContext": {}
                    });

                    if !images.is_empty() {
                        user_msg["images"] = json!(images);
                    }

                    if let Some(ctx) = user_msg.get_mut("userInputMessageContext") {
                        if !tool_results.is_empty() {
                            ctx["toolResults"] = json!(tool_results);
                        }
                        ctx["tools"] = if !tools_context.is_empty() { json!(tools_context) } else { serde_json::Value::Null };
                        ctx["toolsUsed"] = if !tools_used.is_empty() { json!(tools_used) } else { serde_json::Value::Null };
                    }

                    history.push(json!({ "userInputMessage": user_msg }));
                }
                "assistant" => {
                    let (content, tool_uses, tool_results) = self.extract_assistant_message_details(message);
                    let mut assistant_msg = json!({
                        "content": content,
                        "toolUses": if tool_uses.is_empty() { serde_json::Value::Null } else { json!(tool_uses) },
                        "toolResults": if tool_results.is_empty() { serde_json::Value::Null } else { json!(tool_results) }
                    });

                    history.push(json!({
                        "assistantResponseMessage": assistant_msg
                    }));
                }
                _ => {}
            }
        }

        // Build current message from the last message in the array
        let current_message = messages.last().unwrap();
        let role = current_message.get("role").and_then(|r| r.as_str()).unwrap_or("");
        let mut conversation_state = json!({
            "chatTriggerType": CHAT_TRIGGER_TYPE_MANUAL,
            "conversationId": conversation_id,
            "history": history,
            "currentMessage": {}
        });

        match role {
            "user" => {
                let (content, images, tool_results, tools_used) = self.extract_user_message_details(current_message);
                let mut current = json!({
                    "content": if content.is_empty() { "Continue".to_string() } else { content },
                    "modelId": codewhisperer_model,
                    "origin": ORIGIN_AI_EDITOR,
                    "userInputMessageContext": {}
                });

                if !images.is_empty() {
                    current["images"] = json!(images);
                }

                if let Some(ctx) = current.get_mut("userInputMessageContext") {
                    if !tool_results.is_empty() {
                        ctx["toolResults"] = json!(tool_results);
                    }
                    ctx["tools"] = if !tools_context.is_empty() { json!(tools_context) } else { serde_json::Value::Null };
                    ctx["toolsUsed"] = if !tools_used.is_empty() { json!(tools_used) } else { serde_json::Value::Null };
                }

                conversation_state["currentMessage"]["userInputMessage"] = current;
            }
            "assistant" => {
                let (content, tool_uses, tool_results) = self.extract_assistant_message_details(current_message);
                conversation_state["currentMessage"]["assistantResponseMessage"] = json!({
                    "content": if content.is_empty() { "Continue".to_string() } else { content },
                    "toolUses": if tool_uses.is_empty() { serde_json::Value::Null } else { json!(tool_uses) },
                    "toolResults": if tool_results.is_empty() { serde_json::Value::Null } else { json!(tool_results) }
                });
            }
            _ => {}
        }

        let mut request = json!({
            "conversationState": conversation_state,
            "conversationStateMetadata": {
                "systemPrompt": if system_prompt.is_empty() { serde_json::Value::Null } else { json!(system_prompt) }
            }
        });

        if let Some(profile_arn) = self.credentials.read().await.profile_arn.clone() {
            request["profileArn"] = json!(profile_arn);
        }

        Ok(request)
    }

    fn extract_content_text(&self, value: &serde_json::Value) -> String {
        if value.is_null() {
            return String::new();
        }

        if let Some(obj) = value.as_object() {
            if let Some(array) = obj.get("content").and_then(|c| c.as_array()) {
                return array.iter()
                    .filter_map(|part| {
                        if part.get("type").and_then(|t| t.as_str()) == Some("text") {
                            part.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("");
            }
            if let Some(text) = obj.get("content").and_then(|t| t.as_str()) {
                return text.to_string();
            }
        }

        value.as_str().unwrap_or("").to_string()
    }

    fn extract_user_message_details(&self, message: &serde_json::Value) -> (String, Vec<serde_json::Value>, Vec<serde_json::Value>, Vec<serde_json::Value>) {
        let mut content = String::new();
        let mut images = Vec::new();
        let mut tool_results = Vec::new();
        let mut tools_used = Vec::new();

        if let Some(parts) = message.get("content").and_then(|c| c.as_array()) {
            for part in parts {
                match part.get("type").and_then(|t| t.as_str()).unwrap_or("") {
                    "text" => {
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            content.push_str(text);
                        }
                    }
                    "tool_result" => {
                        if let Some(tool_use_id) = part.get("tool_use_id").and_then(|id| id.as_str()) {
                            let tool_content = part.get("content").cloned().unwrap_or(json!([]));
                            let text_content = self.extract_tool_result_content(&tool_content);
                            tool_results.push(json!({
                                "content": text_content,
                                "status": "success",
                                "toolUseId": tool_use_id
                            }));
                        }
                    }
                    "tool_use" => {
                        if let Some(name) = part.get("name").and_then(|n| n.as_str()) {
                            let input = part.get("input").cloned().unwrap_or(json!({}));
                            if let Some(id) = part.get("id").and_then(|i| i.as_str()) {
                                tools_used.push(json!({
                                    "input": input,
                                    "name": name,
                                    "toolUseId": id
                                }));
                            }
                        }
                    }
                    "image" => {
                        if let Some(source) = part.get("source") {
                            let media_type = source.get("media_type").and_then(|m| m.as_str()).unwrap_or("image/png");
                            let format = media_type.split('/').nth(1).unwrap_or("png");
                            let data = source.get("data");
                            images.push(json!({
                                "format": format,
                                "source": {
                                    "bytes": data
                                }
                            }));
                        }
                    }
                    _ => {}
                }
            }
        } else {
            content = self.extract_content_text(message);
        }

        (content, images, tool_results, tools_used)
    }

    fn extract_tool_result_content(&self, value: &serde_json::Value) -> Vec<serde_json::Value> {
        if let Some(array) = value.as_array() {
            array.iter().filter_map(|item| {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    Some(json!({ "text": text }))
                } else if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                    match item_type {
                        "text" => item.get("text").map(|t| json!({ "text": t.as_str().unwrap_or("") })),
                        _ => Some(json!({ "text": self.extract_content_text(item) }))
                    }
                } else {
                    Some(json!({ "text": self.extract_content_text(item) }))
                }
            }).collect()
        } else if let Some(text) = value.as_str() {
            vec![json!({ "text": text })]
        } else {
            // For any other type, convert to text
            vec![json!({ "text": self.extract_content_text(value) })]
        }
    }

    fn extract_assistant_message_details(&self, message: &serde_json::Value) -> (String, Vec<serde_json::Value>, Vec<serde_json::Value>) {
        let mut content = String::new();
        let mut tool_uses = Vec::new();
        let mut tool_results = Vec::new();

        if let Some(parts) = message.get("content").and_then(|c| c.as_array()) {
            for part in parts {
                match part.get("type").and_then(|t| t.as_str()).unwrap_or("") {
                    "text" => {
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            content.push_str(text);
                        }
                    }
                    "tool_use" => {
                        let input = part.get("input").cloned().unwrap_or(json!({}));
                        let name = part.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        let id = part.get("id").and_then(|i| i.as_str()).unwrap_or("");
                        tool_uses.push(json!({
                            "input": input,
                            "name": name,
                            "toolUseId": id
                        }));
                    }
                    "tool_result" => {
                        if let Some(tool_use_id) = part.get("tool_use_id").and_then(|id| id.as_str()) {
                            let tool_content = part.get("content").cloned().unwrap_or(json!([]));
                            let text_content = self.extract_tool_result_content(&tool_content);
                            tool_results.push(json!({
                                "content": text_content,
                                "status": "success",
                                "toolUseId": tool_use_id
                            }));
                        }
                    }
                    _ => {}
                }
            }
        } else {
            content = self.extract_content_text(message);
        }

        (content, tool_uses, tool_results)
    }

    fn build_tools_context(&self, tools: Option<&serde_json::Value>) -> Vec<serde_json::Value> {
        tools
            .and_then(|v| v.as_array())
            .map(|tools| {
                tools.iter().filter_map(|tool| {
                    let name = tool.get("name")?.as_str()?;
                    let input_schema = tool.get("input_schema").cloned().unwrap_or(json!({}));
                    Some(json!({
                        "toolSpecification": {
                            "name": name,
                            "description": tool.get("description").and_then(|d| d.as_str()).unwrap_or(""),
                            "inputSchema": { "json": input_schema }
                        }
                    }))
                }).collect::<Vec<_>>()
            })
            .unwrap_or_default()
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
        
        let mut creds = self.credentials.write().await;
        
        // Check if we have a refresh token
        let refresh_token = creds.refresh_token.clone()
            .ok_or_else(|| anyhow::anyhow!("No refresh token available"))?;
            
        // Determine refresh URL based on auth method
        let refresh_url = if creds.auth_method.as_deref() == Some("social") {
            format!("https://prod.{}.auth.desktop.kiro.dev/refreshToken", self.region)
        } else {
            format!("https://oidc.{}.amazonaws.com/token", self.region)
        };
        
        // Build refresh request
        let mut request_body = json!({
            "refreshToken": refresh_token
        });
        
        // Add additional fields for non-social auth
        if creds.auth_method.as_deref() != Some("social") {
            // For IDC auth, we'd need clientId and clientSecret
            // These aren't stored in the current credentials format
            request_body["grantType"] = json!("refresh_token");
        }
        
        // Make refresh request
        let response = self.client
            .post(&refresh_url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Token refresh failed: {}", error_text);
        }
        
        let refresh_response: serde_json::Value = response.json().await?;
        
        // Update credentials with new token
        if let Some(new_access_token) = refresh_response.get("accessToken").and_then(|v| v.as_str()) {
            creds.access_token = new_access_token.to_string();
            
            if let Some(new_refresh_token) = refresh_response.get("refreshToken").and_then(|v| v.as_str()) {
                creds.refresh_token = Some(new_refresh_token.to_string());
            }
            
            if let Some(expires_in) = refresh_response.get("expiresIn").and_then(|v| v.as_i64()) {
                let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in);
                creds.expires_at = Some(expires_at.to_rfc3339());
            }
            
            if let Some(profile_arn) = refresh_response.get("profileArn").and_then(|v| v.as_str()) {
                creds.profile_arn = Some(profile_arn.to_string());
            }
            
            // Save updated credentials to file
            let updated_creds = json!({
                "accessToken": creds.access_token,
                "refreshToken": creds.refresh_token,
                "expiresAt": creds.expires_at,
                "profileArn": creds.profile_arn,
                "authMethod": creds.auth_method,
                "provider": creds.provider
            });
            
            if let Err(e) = tokio::fs::write(&self.credentials_path, serde_json::to_string_pretty(&updated_creds)?).await {
                warn!("Failed to save refreshed credentials to file: {}", e);
            }
            
            info!("Access token refreshed successfully");
        Ok(())
        } else {
            anyhow::bail!("Invalid refresh response: Missing accessToken")
        }
    }
    
    /// Parse CodeWhisperer event stream response to extract text content and tool calls
    /// This implementation closely follows the Node.js parseEventStreamChunk logic
    fn parse_codewhisperer_response(&self, response_text: &str) -> Result<(String, Vec<serde_json::Value>)> {
        let mut full_content = String::new();
        let mut tool_calls: Vec<serde_json::Value> = Vec::new();
        let mut current_tool_call: Option<serde_json::Value> = None;
        
        info!("Starting to parse CodeWhisperer response, length: {}", response_text.len());
        debug!("Response preview: {}", &response_text[..response_text.len().min(500)]);
        
        // Find all "event{" positions manually (Rust regex doesn't support look-ahead)
        let mut event_positions: Vec<usize> = Vec::new();
        let mut search_start = 0;
        while let Some(pos) = response_text[search_start..].find("event{") {
            event_positions.push(search_start + pos);
            search_start += pos + 6; // Move past "event{"
        }
        
        let mut found_events = 0;
        
        // Process each event block
        for (i, &event_start) in event_positions.iter().enumerate() {
            found_events += 1;
            
            // Determine the end of this event block (start of next event or end of string)
            let event_end = if i + 1 < event_positions.len() {
                event_positions[i + 1]
            } else {
                response_text.len()
            };
            
            // Extract the event block content (skip "event" prefix)
            let event_block_start = event_start + 5; // Skip "event"
            if event_block_start >= event_end {
                continue;
            }
            
            let potential_json_block = &response_text[event_block_start..event_end];
            
            // Try to find valid JSON by looking for closing braces
                    let mut search_pos = 0;
            loop {
                // Find next '}' starting from search_pos
                match potential_json_block[search_pos..].find('}') {
                    Some(relative_pos) => {
                        let brace_pos = search_pos + relative_pos;
                        let json_candidate = &potential_json_block[..=brace_pos];
                        
                        // Try to parse this as JSON
                        if let Ok(event_data) = serde_json::from_str::<serde_json::Value>(json_candidate) {
                            // Successfully parsed - check what type of event this is
                            
                            // Check for tool use events (structured)
                            if event_data.get("name").is_some() && event_data.get("toolUseId").is_some() {
                                let name = event_data.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                let tool_use_id = event_data.get("toolUseId").and_then(|id| id.as_str()).unwrap_or("");
                                
                                if current_tool_call.is_none() {
                                    // Start new tool call - input as empty string to match Node.js
                                    current_tool_call = Some(json!({
                                        "id": tool_use_id,
                                        "type": "tool_use",
                                        "name": name,
                                        "input": ""
                                    }));
                                }
                                
                                // Accumulate input if present
                                if let Some(input) = event_data.get("input").and_then(|i| i.as_str()) {
                                    if let Some(ref mut tool_call) = current_tool_call {
                                        if let Some(existing_input) = tool_call.get("input").and_then(|i| i.as_str()) {
                                            tool_call["input"] = json!(format!("{}{}", existing_input, input));
                                        } else {
                                            tool_call["input"] = json!(input);
                                        }
                                    }
                                }
                                
                                // Check if tool call is complete
                                if event_data.get("stop").and_then(|s| s.as_bool()).unwrap_or(false) {
                                    if let Some(tool_call) = current_tool_call.take() {
                                        // Keep input as string, just like Node.js version
                                        // Don't parse to JSON object to maintain compatibility
                                        tool_calls.push(tool_call);
                                    }
                                }
                            }
                            // Check for content events (not followup prompts)
                            else if event_data.get("followupPrompt").is_none() {
                                if let Some(text) = event_data.get("content").and_then(|c| c.as_str()) {
                                    let decoded = text.replace("\\n", "\n");
                                    full_content.push_str(&decoded);
                                }
                            }
                            
                            // Successfully parsed and processed, break to next event block
                            break;
                        }
                        
                        // JSON parse failed, continue searching for next '}'
                        search_pos = brace_pos + 1;
                    }
                    None => {
                        // No more '}' found in this block
                        break;
                    }
                }
            }
        }
        
        // If there's a pending tool call, add it
        if let Some(tool_call) = current_tool_call {
            tool_calls.push(tool_call);
        }
        
        info!("Parsed {} event blocks, content: {} chars, tool_calls: {}", 
              found_events, full_content.len(), tool_calls.len());
        
        // Parse bracket-style tool calls from content: [Called functionName with args: {...}]
        let bracket_tool_calls = self.parse_bracket_tool_calls(&full_content);
        if !bracket_tool_calls.is_empty() {
            info!("Found {} bracket-style tool calls in content", bracket_tool_calls.len());
            
            // Add bracket tool calls to the list
            tool_calls.extend(bracket_tool_calls.clone());
            
            // Remove tool call syntax from content
            for tool_call in &bracket_tool_calls {
                if let Some(name) = tool_call.get("name").and_then(|n| n.as_str()) {
                    // Remove [Called functionName with args: {...}] pattern
                    let pattern = format!(r"\[Called\s+{}\s+with\s+args:\s*\{{[^}}]*(?:\{{[^}}]*\}}[^}}]*)*\}}\]", regex::escape(name));
                    if let Ok(re) = regex::Regex::new(&pattern) {
                        full_content = re.replace_all(&full_content, "").to_string();
                    }
                }
            }
            // Clean up extra whitespace
            full_content = full_content.split_whitespace().collect::<Vec<_>>().join(" ");
        }
        
        if full_content.is_empty() && tool_calls.is_empty() {
            error!("Could not parse any content from CodeWhisperer response!");
            error!("Full response: {}", &response_text[..response_text.len().min(2000)]);
            // Return a helpful error message instead of empty
            full_content = "⚠️ Unable to parse response from Kiro API. Please check server logs with RUST_LOG=debug.".to_string();
        }
        
        Ok((full_content, tool_calls))
    }
    
    /// Parse bracket-style tool calls: [Called functionName with args: {...}]
    fn parse_bracket_tool_calls(&self, text: &str) -> Vec<serde_json::Value> {
        if !text.contains("[Called") {
            return Vec::new();
        }
        
        let mut tool_calls = Vec::new();
        
        // Find all [Called positions
        let mut call_positions = Vec::new();
                let mut start = 0;
        while let Some(pos) = text[start..].find("[Called") {
            call_positions.push(start + pos);
            start += pos + 1;
        }
        
        for (i, &start_pos) in call_positions.iter().enumerate() {
            let end_search_limit = if i + 1 < call_positions.len() {
                call_positions[i + 1]
            } else {
                text.len()
            };
            
            let segment = &text[start_pos..end_search_limit];
            
            // Find matching bracket
            let bracket_end = self.find_matching_bracket(segment, 0);
            
            let tool_call_text = if bracket_end != -1 {
                &segment[..=(bracket_end as usize)]
            } else {
                // Fallback: find last ']'
                if let Some(last_bracket) = segment.rfind(']') {
                    &segment[..=last_bracket]
                } else {
                    continue;
                }
            };
            
            if let Some(parsed) = self.parse_single_tool_call(tool_call_text) {
                tool_calls.push(parsed);
            }
        }
        
        tool_calls
    }
    
    /// Find matching closing bracket
    fn find_matching_bracket(&self, text: &str, start_pos: usize) -> i32 {
        let bytes = text.as_bytes();
        if start_pos >= bytes.len() || bytes[start_pos] != b'[' {
            return -1;
        }
        
        let mut bracket_count = 1;
        let mut in_string = false;
        let mut escape_next = false;
        
        for i in (start_pos + 1)..bytes.len() {
            let ch = bytes[i];
            
            if escape_next {
                escape_next = false;
                continue;
            }
            
            if ch == b'\\' && in_string {
                escape_next = true;
                continue;
            }
            
            if ch == b'"' && !escape_next {
                in_string = !in_string;
                continue;
            }
            
            if !in_string {
                if ch == b'[' {
                    bracket_count += 1;
                } else if ch == b']' {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        return i as i32;
                    }
                }
            }
        }
        
        -1
    }
    
    /// Parse single tool call from text like: [Called functionName with args: {...}]
    fn parse_single_tool_call(&self, tool_call_text: &str) -> Option<serde_json::Value> {
        // Pattern: [Called functionName with args:
        let name_pattern = regex::Regex::new(r"(?i)\[Called\s+(\w+)\s+with\s+args:").ok()?;
        let name_match = name_pattern.captures(tool_call_text)?;
        let function_name = name_match.get(1)?.as_str().trim();
        
        let args_start_marker = "with args:";
        let args_start_pos = tool_call_text.to_lowercase().find(&args_start_marker.to_lowercase())?;
        let args_start = args_start_pos + args_start_marker.len();
        let args_end = tool_call_text.rfind(']')?;
        
        if args_end <= args_start {
            return None;
        }
        
        let json_candidate = tool_call_text[args_start..args_end].trim();
        
        // Try to repair and parse JSON
        let mut repaired_json = json_candidate.to_string();
        // Remove trailing comma
        repaired_json = regex::Regex::new(r",\s*([}\]])").ok()?.replace_all(&repaired_json, "$1").to_string();
        // Add quotes to unquoted keys
        repaired_json = regex::Regex::new(r#"([{,]\s*)([a-zA-Z0-9_]+?)\s*:"#).ok()?.replace_all(&repaired_json, r#"$1"$2":"#).to_string();
        // Quote unquoted values
        repaired_json = regex::Regex::new(r":\s*([a-zA-Z0-9_]+)(?=[,\}\]])").ok()?.replace_all(&repaired_json, r#":"$1""#).to_string();
        
        match serde_json::from_str::<serde_json::Value>(&repaired_json) {
            Ok(arguments_obj) => {
                if !arguments_obj.is_object() {
                    return None;
                }
                
                let tool_call_id = format!("call_{}", Uuid::new_v4().to_string().replace("-", "")[..8].to_string());
                Some(json!({
                    "id": tool_call_id,
                    "type": "tool_use",
                    "name": function_name,
                    "input": arguments_obj
                }))
            }
            Err(e) => {
                warn!("Failed to parse tool call arguments: {}", e);
                None
            }
        }
    }

    fn call_api_with_retry<'a>(
        &'a self,
        endpoint: &'a str,
        body: serde_json::Value,
        retry_count: u32,
    ) -> BoxFuture<'a, Result<serde_json::Value>> {
        Box::pin(async move {
        self.call_api_with_retry_and_refresh(endpoint, body, retry_count, false).await
        })
    }
    
    fn call_api_with_retry_and_refresh<'a>(
        &'a self,
        endpoint: &'a str,
        body: serde_json::Value,
        retry_count: u32,
        is_retry_after_refresh: bool,
    ) -> BoxFuture<'a, Result<serde_json::Value>> {
        Box::pin(async move {
        // Check token expiration before making request
        {
            let creds = self.credentials.read().await;
            if self.is_token_expired(&creds) {
                drop(creds);
                info!("Token expired, attempting refresh...");
                if let Err(e) = self.refresh_access_token().await {
                    warn!("Token refresh failed: {}", e);
                    // Continue with expired token, will handle 403 if it occurs
                }
            }
        }

        let url = self.get_api_url(endpoint);
        let creds = self.credentials.read().await;

        info!("Calling Kiro API: {}", url);
        let request_start = std::time::Instant::now();
        
        // Convert Claude request to CodeWhisperer format
        let convert_start = std::time::Instant::now();
        let codewhisperer_request = self.build_codewhisperer_request(&body).await?;
        let convert_duration = convert_start.elapsed();
        debug!("Request conversion took: {:?}", convert_duration);
        debug!("CodeWhisperer request: {}", serde_json::to_string_pretty(&codewhisperer_request)?);

        let api_call_start = std::time::Instant::now();
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", creds.access_token))
            .header("Content-Type", "application/json")
            .header("amz-sdk-invocation-id", Uuid::new_v4().to_string())
            .json(&codewhisperer_request)
            .send()
            .await?;
        
        let api_call_duration = api_call_start.elapsed();
        info!("API call took: {:?}", api_call_duration);

        let status = response.status();

        if status.is_success() {
            // CodeWhisperer returns event stream format, not JSON
            let response_text = response.text().await?;
            info!("Raw response length: {} bytes", response_text.len());
            debug!("Raw response: {}", &response_text[..response_text.len().min(500)]);
            
            // Parse the event stream to extract the actual content and tool calls
            let parse_start = std::time::Instant::now();
            let (content, tool_calls) = self.parse_codewhisperer_response(&response_text)?;
            let parse_duration = parse_start.elapsed();
            info!("Response parsing took: {:?}", parse_duration);
            info!("Parsed content length: {}, tool_calls count: {}", content.len(), tool_calls.len());
            debug!("Parsed content: {}", &content);
            
            // Build content array with both text and tool_use blocks
            let mut content_array = Vec::new();
            
            // Add tool calls first (if any)
            for tool_call in tool_calls {
                content_array.push(tool_call);
            }
            
            // Add text content (if any)
            if !content.is_empty() {
                content_array.push(json!({
                    "type": "text",
                    "text": content
                }));
            }
            
            // Ensure we have at least some content
            if content_array.is_empty() {
                error!("No content parsed from CodeWhisperer response!");
                // Return a helpful error message instead of empty content
                // This prevents Cline from showing "unparsable response" error
                content_array.push(json!({
                    "type": "text",
                    "text": "⚠️ Unable to parse response from Kiro API. The response may be in an unexpected format. Please check the server logs with RUST_LOG=debug for details."
                }));
            }
            
            // Determine stop_reason based on content
            let stop_reason = if content_array.iter().any(|c| c.get("type").and_then(|t| t.as_str()) == Some("tool_use")) {
                "tool_use"
            } else {
                "end_turn"
            };
            
            // Calculate total output tokens
            let output_tokens: usize = content_array.iter().map(|item| {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    text.len() / 4
                } else if let Some(input) = item.get("input") {
                    serde_json::to_string(input).unwrap_or_default().len() / 4
                } else {
                    0
                }
            }).sum();
            
            // Convert to Claude response format
            let result = json!({
                "id": format!("msg_{}", Uuid::new_v4()),
                "type": "message",
                "role": "assistant",
                "content": content_array,
                "model": "claude-sonnet-4-20250514",
                "stop_reason": stop_reason,
                "stop_sequence": null,
                "usage": {
                    "input_tokens": 0,
                    "output_tokens": output_tokens
                }
            });
            
            let total_duration = request_start.elapsed();
            info!("Total request processing took: {:?}", total_duration);
            info!("Returning Claude response with {} content blocks", content_array.len());
            debug!("Response structure: {}", serde_json::to_string_pretty(&result).unwrap_or_default());
            
            // Log the actual content for debugging
            for (i, block) in content_array.iter().enumerate() {
                if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                    debug!("Content block {}: type=text, length={}, preview={}", 
                           i, text.len(), &text[..text.len().min(100)]);
                } else if let Some(tool_name) = block.get("name").and_then(|n| n.as_str()) {
                    debug!("Content block {}: type=tool_use, name={}", i, tool_name);
                }
            }
            
            return Ok(result);
        }
        
        // Handle 403 Forbidden - token might be invalid
        if status.as_u16() == 403 && !is_retry_after_refresh {
            info!("Received 403 Forbidden. Attempting token refresh and retrying...");
            drop(creds); // Release the read lock before refreshing
            
            match self.refresh_access_token().await {
                Ok(_) => {
                    info!("Token refreshed successfully, retrying request...");
                    return self.call_api_with_retry_and_refresh(endpoint, body, retry_count, true).await;
                }
                Err(e) => {
                    error!("Token refresh failed during 403 retry: {}", e);
                    let error_text = response.text().await?;
                    anyhow::bail!("API call failed (403 Forbidden) and token refresh failed: {}", error_text);
                }
            }
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
        request_body: serde_json::Value,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>> {
        debug!("Kiro generate_content_stream");

        // Note: Kiro/CodeWhisperer doesn't support true streaming
        // We'll get the full response and simulate streaming
        
        // Get the full response first
        let full_response = self.call_api_with_retry("/v1/messages", request_body, 0).await?;
        
        // Extract the content array from the response
        let content_array = full_response
            .get("content")
            .and_then(|c| c.as_array())
            .cloned()
            .unwrap_or_default();
            
        let model = full_response
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("claude-sonnet-4-20250514")
            .to_string();
            
        let stop_reason = full_response
            .get("stop_reason")
            .and_then(|s| s.as_str())
            .unwrap_or("end_turn")
            .to_string();
        
        // Create a simulated stream that returns Claude-formatted events
        let message_id = format!("msg_{}", Uuid::new_v4());
        
        let stream = stream! {
            // 1. Send message_start event
            yield Ok(json!({
                "type": "message_start",
                "message": {
                    "id": message_id.clone(),
                    "type": "message",
                    "role": "assistant",
                    "model": model.clone(),
                    "content": [],
                    "stop_reason": null,
                    "stop_sequence": null,
                    "usage": {
                        "input_tokens": 0,
                        "output_tokens": 0
                    }
                }
            }));
            
            // Estimate total output tokens
            let mut total_output_tokens = 0;
            
            // 2-4. Send events for each content block (tool_use or text)
            for (index, content_block) in content_array.iter().enumerate() {
                let content_type = content_block.get("type").and_then(|t| t.as_str()).unwrap_or("text");
                
                if content_type == "tool_use" {
                    // Handle tool_use block
                    let tool_id = content_block.get("id").and_then(|i| i.as_str()).unwrap_or("");
                    let tool_name = content_block.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    // Keep input as string to match Node.js behavior
                    let tool_input = content_block.get("input").and_then(|i| i.as_str()).unwrap_or("").to_string();
                    
                    // Send content_block_start for tool_use
            yield Ok(json!({
                "type": "content_block_start",
                        "index": index,
                        "content_block": {
                            "type": "tool_use",
                            "id": tool_id,
                            "name": tool_name,
                            "input": {}
                        }
                    }));
                    
                    // Send content_block_delta with tool input (as string)
                    yield Ok(json!({
                        "type": "content_block_delta",
                        "index": index,
                        "delta": {
                            "type": "input_json_delta",
                            "partial_json": tool_input
                        }
                    }));
                    
                    // Send content_block_stop
                    yield Ok(json!({
                        "type": "content_block_stop",
                        "index": index
                    }));
                    
                    total_output_tokens += serde_json::to_string(&tool_input).unwrap_or_default().len() / 4;
                } else {
                    // Handle text block
                    let text = content_block.get("text").and_then(|t| t.as_str()).unwrap_or("");
                    
                    // Send content_block_start for text
                    yield Ok(json!({
                        "type": "content_block_start",
                        "index": index,
                "content_block": {
                    "type": "text",
                    "text": ""
                }
            }));
            
                    // Send content_block_delta with text
            yield Ok(json!({
                "type": "content_block_delta",
                        "index": index,
                "delta": {
                    "type": "text_delta",
                            "text": text
                }
            }));
            
                    // Send content_block_stop
            yield Ok(json!({
                "type": "content_block_stop",
                        "index": index
            }));
                    
                    total_output_tokens += text.len() / 4;
                }
            }
            
            // 5. Send message_delta event
            yield Ok(json!({
                "type": "message_delta",
                "delta": {
                    "stop_reason": stop_reason,
                    "stop_sequence": null
                },
                "usage": {
                    "output_tokens": total_output_tokens
                }
            }));
            
            // 6. Send message_stop event
            yield Ok(json!({
                "type": "message_stop"
            }));
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

