/*!
 * Common types and utilities
 */

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model protocol prefixes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelProtocol {
    Gemini,
    OpenAI,
    Claude,
}

impl ModelProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::OpenAI => "openai",
            Self::Claude => "claude",
        }
    }
}

/// Model provider identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelProvider {
    GeminiCliOAuth,
    OpenAICustom,
    ClaudeCustom,
    ClaudeKiroOAuth,
    OpenAIQwenOAuth,
}

impl ModelProvider {
    pub fn as_str(&self) -> &str {
        match self {
            Self::GeminiCliOAuth => "gemini-cli-oauth",
            Self::OpenAICustom => "openai-custom",
            Self::ClaudeCustom => "claude-custom",
            Self::ClaudeKiroOAuth => "claude-kiro-oauth",
            Self::OpenAIQwenOAuth => "openai-qwen-oauth",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "gemini-cli-oauth" => Some(Self::GeminiCliOAuth),
            "openai-custom" => Some(Self::OpenAICustom),
            "claude-custom" => Some(Self::ClaudeCustom),
            "claude-kiro-oauth" => Some(Self::ClaudeKiroOAuth),
            "openai-qwen-oauth" => Some(Self::OpenAIQwenOAuth),
            _ => None,
        }
    }

    pub fn protocol(&self) -> ModelProtocol {
        match self {
            Self::GeminiCliOAuth => ModelProtocol::Gemini,
            Self::OpenAICustom | Self::OpenAIQwenOAuth => ModelProtocol::OpenAI,
            Self::ClaudeCustom | Self::ClaudeKiroOAuth => ModelProtocol::Claude,
        }
    }
}

/// API endpoint types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointType {
    OpenAIChat,
    GeminiContent,
    ClaudeMessage,
    OpenAIModelList,
    GeminiModelList,
}

/// API actions
#[derive(Debug, Clone, Copy)]
pub enum ApiAction {
    GenerateContent,
    StreamGenerateContent,
}

impl ApiAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GenerateContent => "generateContent",
            Self::StreamGenerateContent => "streamGenerateContent",
        }
    }
}

/// Generic API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    pub model: Option<String>,
    pub messages: Option<Vec<Message>>,
    pub contents: Option<Vec<Content>>,
    pub system: Option<serde_json::Value>,
    pub system_instruction: Option<SystemInstruction>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stream: Option<bool>,
    pub tools: Option<Vec<serde_json::Value>>,
    
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Message structure (OpenAI/Claude format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Message content (can be string or array of content parts)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// Content part
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
    
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

/// Tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    
    #[serde(rename = "type")]
    pub tool_type: String,
    
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String,
}

/// Content structure (Gemini format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

/// Gemini part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_data: Option<InlineData>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<FileData>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_response: Option<FunctionResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineData {
    pub mime_type: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileData {
    pub mime_type: String,
    pub file_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}

/// System instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInstruction {
    pub parts: Vec<Part>,
}

/// Generic API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

/// Model list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelListResponse {
    pub object: Option<String>,
    pub data: Option<Vec<ModelInfo>>,
    pub models: Option<Vec<ModelInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: Option<String>,
    pub name: Option<String>,
    pub object: Option<String>,
    pub created: Option<i64>,
    pub owned_by: Option<String>,
    
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Format expiry timestamp as human-readable string
pub fn format_expiry_time(expiry_timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff_secs = expiry_timestamp - now;
    
    if diff_secs <= 0 {
        return "Token has expired".to_string();
    }
    
    let hours = diff_secs / 3600;
    let minutes = (diff_secs % 3600) / 60;
    let seconds = diff_secs % 60;
    
    format!("{:02}h {:02}m {:02}s", hours, minutes, seconds)
}

/// Check if authorization is valid
pub fn is_authorized(
    auth_header: Option<&str>,
    api_key_header: Option<&str>,
    goog_api_key: Option<&str>,
    query_key: Option<&str>,
    required_key: &str,
) -> bool {
    // Check Bearer token
    if let Some(auth) = auth_header {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            if token == required_key {
                return true;
            }
        }
    }
    
    // Check x-api-key header (Claude)
    if let Some(key) = api_key_header {
        if key == required_key {
            return true;
        }
    }
    
    // Check x-goog-api-key header (Gemini)
    if let Some(key) = goog_api_key {
        if key == required_key {
            return true;
        }
    }
    
    // Check query parameter
    if let Some(key) = query_key {
        if key == required_key {
            return true;
        }
    }
    
    false
}

