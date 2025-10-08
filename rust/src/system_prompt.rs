/*!
 * System Prompt Management
 *
 * Handles system prompt file operations and injection into requests.
 */

use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info, warn};

pub struct SystemPromptManager {
    file_path: Option<PathBuf>,
    mode: String,
    pub content: Option<String>,
}

impl SystemPromptManager {
    pub async fn new(file_path: Option<PathBuf>, mode: String) -> Result<Self> {
        let content = if let Some(ref path) = file_path {
            Self::load_from_file(path).await.ok()
        } else {
            None
        };

        Ok(Self {
            file_path,
            mode,
            content,
        })
    }

    async fn load_from_file(path: &PathBuf) -> Result<String> {
        let content = fs::read_to_string(path).await?;
        if content.trim().is_empty() {
            anyhow::bail!("System prompt file is empty");
        }
        info!("Loaded system prompt from: {:?}", path);
        Ok(content)
    }

    /// Apply system prompt to OpenAI format request
    pub fn apply_to_openai(&self, mut request: Value) -> Result<Value> {
        if let Some(ref prompt_content) = self.content {
            if let Some(messages) = request.get_mut("messages").and_then(|v| v.as_array_mut()) {
                match self.mode.as_str() {
                    "overwrite" => {
                        // Remove existing system messages
                        messages.retain(|msg| {
                            msg.get("role").and_then(|r| r.as_str()) != Some("system")
                        });
                        
                        // Insert new system message at the beginning
                        messages.insert(0, serde_json::json!({
                            "role": "system",
                            "content": prompt_content
                        }));
                        
                        debug!("System prompt overwritten in OpenAI request");
                    }
                    "append" => {
                        // Find last system message or create new one
                        let has_system = messages.iter().any(|msg| {
                            msg.get("role").and_then(|r| r.as_str()) == Some("system")
                        });
                        
                        if has_system {
                            // Append to existing system message
                            for msg in messages.iter_mut() {
                                if msg.get("role").and_then(|r| r.as_str()) == Some("system") {
                                    if let Some(content) = msg.get_mut("content").and_then(|c| c.as_str()) {
                                        let new_content = format!("{}\n\n{}", content, prompt_content);
                                        *msg = serde_json::json!({
                                            "role": "system",
                                            "content": new_content
                                        });
                                    }
                                    break;
                                }
                            }
                        } else {
                            // Add new system message
                            messages.insert(0, serde_json::json!({
                                "role": "system",
                                "content": prompt_content
                            }));
                        }
                        
                        debug!("System prompt appended in OpenAI request");
                    }
                    _ => {
                        warn!("Unknown system prompt mode: {}", self.mode);
                    }
                }
            }
        }
        
        Ok(request)
    }

    /// Apply system prompt to Claude format request
    pub fn apply_to_claude(&self, mut request: Value) -> Result<Value> {
        if let Some(ref prompt_content) = self.content {
            match self.mode.as_str() {
                "overwrite" => {
                    request["system"] = serde_json::json!(prompt_content);
                    debug!("System prompt overwritten in Claude request");
                }
                "append" => {
                    let existing = request.get("system")
                        .and_then(|s| s.as_str())
                        .unwrap_or("");
                    
                    let new_content = if existing.is_empty() {
                        prompt_content.clone()
                    } else {
                        format!("{}\n\n{}", existing, prompt_content)
                    };
                    
                    request["system"] = serde_json::json!(new_content);
                    debug!("System prompt appended in Claude request");
                }
                _ => {
                    warn!("Unknown system prompt mode: {}", self.mode);
                }
            }
        }
        
        Ok(request)
    }

    /// Apply system prompt to Gemini format request
    pub fn apply_to_gemini(&self, mut request: Value) -> Result<Value> {
        if let Some(ref prompt_content) = self.content {
            match self.mode.as_str() {
                "overwrite" => {
                    request["systemInstruction"] = serde_json::json!({
                        "parts": [{ "text": prompt_content }]
                    });
                    debug!("System prompt overwritten in Gemini request");
                }
                "append" => {
                    let existing = request.get("systemInstruction")
                        .and_then(|si| si.get("parts"))
                        .and_then(|p| p.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|p| p.get("text"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    
                    let new_content = if existing.is_empty() {
                        prompt_content.clone()
                    } else {
                        format!("{}\n\n{}", existing, prompt_content)
                    };
                    
                    request["systemInstruction"] = serde_json::json!({
                        "parts": [{ "text": new_content }]
                    });
                    debug!("System prompt appended in Gemini request");
                }
                _ => {
                    warn!("Unknown system prompt mode: {}", self.mode);
                }
            }
        }
        
        Ok(request)
    }

    /// Save incoming system prompt to file for monitoring
    pub async fn save_incoming_prompt(&self, prompt_text: &str) -> Result<()> {
        if let Some(ref path) = self.file_path {
            let fetch_path = path.parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join("fetch_system_prompt.txt");
            
            // Read current content
            let current = fs::read_to_string(&fetch_path).await.unwrap_or_default();
            
            // Only write if different
            if current != prompt_text {
                fs::write(&fetch_path, prompt_text).await?;
                info!("Saved incoming system prompt to: {:?}", fetch_path);
            }
        }
        
        Ok(())
    }
}

