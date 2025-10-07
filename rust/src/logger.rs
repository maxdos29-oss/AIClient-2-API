/*!
 * Logging Module
 *
 * Handles conversation logging to console or file.
 */

use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tracing::info;

pub struct ConversationLogger {
    mode: LogMode,
    file_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum LogMode {
    None,
    Console,
    File,
}

impl ConversationLogger {
    pub fn new(mode_str: &str, base_name: &str) -> Self {
        let mode = match mode_str {
            "console" => LogMode::Console,
            "file" => LogMode::File,
            _ => LogMode::None,
        };

        let file_path = if matches!(mode, LogMode::File) {
            Some(Self::generate_log_filename(base_name))
        } else {
            None
        };

        Self { mode, file_path }
    }

    fn generate_log_filename(base_name: &str) -> PathBuf {
        let now = Utc::now();
        let timestamp = now.format("%Y%m%d-%H%M%S");
        PathBuf::from(format!("{}-{}.log", base_name, timestamp))
    }

    pub async fn log_conversation(&self, log_type: &str, content: &str) -> Result<()> {
        if matches!(self.mode, LogMode::None) {
            return Ok(());
        }

        if content.is_empty() {
            return Ok(());
        }

        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!(
            "{} [{}]:\n{}\n--------------------------------------\n",
            timestamp, log_type.to_uppercase(), content
        );

        match self.mode {
            LogMode::Console => {
                info!("{}", log_entry);
            }
            LogMode::File => {
                if let Some(ref path) = self.file_path {
                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path)
                        .await?;
                    
                    file.write_all(log_entry.as_bytes()).await?;
                    file.flush().await?;
                }
            }
            LogMode::None => {}
        }

        Ok(())
    }

    pub async fn log_input(&self, content: &str) -> Result<()> {
        self.log_conversation("INPUT", content).await
    }

    pub async fn log_output(&self, content: &str) -> Result<()> {
        self.log_conversation("OUTPUT", content).await
    }

    pub async fn log_error(&self, content: &str) -> Result<()> {
        self.log_conversation("ERROR", content).await
    }
}

/// Extract prompt text from request for logging
pub fn extract_prompt_from_request(request: &serde_json::Value, protocol: &str) -> String {
    match protocol {
        "openai" => {
            if let Some(messages) = request.get("messages").and_then(|v| v.as_array()) {
                messages
                    .iter()
                    .filter_map(|msg| {
                        let role = msg.get("role")?.as_str()?;
                        let content = msg.get("content")?.as_str()?;
                        Some(format!("{}: {}", role, content))
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                String::new()
            }
        }
        "claude" => {
            let mut result = Vec::new();
            
            if let Some(system) = request.get("system").and_then(|v| v.as_str()) {
                result.push(format!("system: {}", system));
            }
            
            if let Some(messages) = request.get("messages").and_then(|v| v.as_array()) {
                for msg in messages {
                    if let (Some(role), Some(content)) = (
                        msg.get("role").and_then(|r| r.as_str()),
                        msg.get("content")
                    ) {
                        let content_str = if content.is_array() {
                            content.as_array()
                                .unwrap()
                                .iter()
                                .filter_map(|c| c.get("text").and_then(|t| t.as_str()))
                                .collect::<Vec<_>>()
                                .join(" ")
                        } else {
                            content.as_str().unwrap_or("").to_string()
                        };
                        result.push(format!("{}: {}", role, content_str));
                    }
                }
            }
            
            result.join("\n")
        }
        "gemini" => {
            let mut result = Vec::new();
            
            if let Some(system_instruction) = request.get("systemInstruction") {
                if let Some(parts) = system_instruction.get("parts").and_then(|p| p.as_array()) {
                    for part in parts {
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            result.push(format!("system: {}", text));
                        }
                    }
                }
            }
            
            if let Some(contents) = request.get("contents").and_then(|v| v.as_array()) {
                for content in contents {
                    if let (Some(role), Some(parts)) = (
                        content.get("role").and_then(|r| r.as_str()),
                        content.get("parts").and_then(|p| p.as_array())
                    ) {
                        let text_parts: Vec<String> = parts
                            .iter()
                            .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
                            .map(|s| s.to_string())
                            .collect();
                        
                        if !text_parts.is_empty() {
                            result.push(format!("{}: {}", role, text_parts.join(" ")));
                        }
                    }
                }
            }
            
            result.join("\n")
        }
        _ => String::new(),
    }
}

/// Extract response text from response for logging
pub fn extract_text_from_response(response: &serde_json::Value, protocol: &str) -> String {
    match protocol {
        "openai" => {
            if let Some(choices) = response.get("choices").and_then(|v| v.as_array()) {
                choices
                    .iter()
                    .filter_map(|choice| {
                        choice.get("message")
                            .and_then(|msg| msg.get("content"))
                            .and_then(|c| c.as_str())
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                String::new()
            }
        }
        "claude" => {
            if let Some(content) = response.get("content").and_then(|v| v.as_array()) {
                content
                    .iter()
                    .filter_map(|block| {
                        if block.get("type")?.as_str()? == "text" {
                            block.get("text")?.as_str()
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                String::new()
            }
        }
        "gemini" => {
            if let Some(candidates) = response.get("candidates").and_then(|v| v.as_array()) {
                candidates
                    .iter()
                    .filter_map(|candidate| {
                        let parts = candidate.get("content")?
                            .get("parts")?
                            .as_array()?;
                        
                        let texts: Vec<&str> = parts
                            .iter()
                            .filter_map(|p| p.get("text")?.as_str())
                            .collect();
                        
                        Some(texts.join(" "))
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

