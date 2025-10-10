/*!
 * Detailed Format Conversion Implementations
 *
 * Complete conversion logic between OpenAI, Claude, and Gemini formats.
 */

use anyhow::Result;
use serde_json::{json, Value};
use uuid::Uuid;

const DEFAULT_MAX_TOKENS: u32 = 8192;
#[allow(dead_code)]
const DEFAULT_GEMINI_MAX_TOKENS: u32 = 65536;
#[allow(dead_code)]
const DEFAULT_TEMPERATURE: f32 = 1.0;
#[allow(dead_code)]
const DEFAULT_TOP_P: f32 = 0.9;

// ============================================================================
// OpenAI <-> Gemini Conversions
// ============================================================================

pub fn openai_request_to_gemini(openai_req: Value) -> Result<Value> {
    let mut gemini_req = json!({});
    
    // Extract system messages
    let (system_instruction, non_system_messages) = extract_system_messages(&openai_req)?;
    
    if let Some(system) = system_instruction {
        gemini_req["systemInstruction"] = system;
    }
    
    // Convert messages to Gemini contents
    let mut contents = Vec::new();
    let mut last_role = String::new();
    let mut accumulated_parts = Vec::new();
    
    for msg in non_system_messages {
        let role = msg.get("role")
            .and_then(|r| r.as_str())
            .unwrap_or("user");
        
        let gemini_role = if role == "assistant" { "model" } else { role };
        
        // Handle tool responses
        if role == "tool" {
            if !accumulated_parts.is_empty() {
                contents.push(json!({
                    "role": last_role,
                    "parts": accumulated_parts
                }));
                accumulated_parts = Vec::new();
            }
            
            contents.push(json!({
                "role": "function",
                "parts": [{
                    "functionResponse": {
                        "name": msg.get("name").and_then(|n| n.as_str()).unwrap_or("unknown"),
                        "response": {"content": msg.get("content").unwrap_or(&json!(""))}
                    }
                }]
            }));
            
            last_role = String::new();
            continue;
        }
        
        // Convert content to parts
        let parts = convert_openai_content_to_gemini_parts(msg.get("content").unwrap_or(&json!("")))?;
        
        // Merge consecutive messages from same role
        if gemini_role == last_role {
            accumulated_parts.extend(parts);
        } else {
            if !accumulated_parts.is_empty() {
                contents.push(json!({
                    "role": last_role,
                    "parts": accumulated_parts
                }));
            }
            accumulated_parts = parts;
            last_role = gemini_role.to_string();
        }
    }
    
    if !accumulated_parts.is_empty() {
        contents.push(json!({
            "role": last_role,
            "parts": accumulated_parts
        }));
    }
    
    gemini_req["contents"] = json!(contents);
    
    // Generation config
    let mut gen_config = json!({});
    if let Some(temp) = openai_req.get("temperature") {
        gen_config["temperature"] = temp.clone();
    }
    if let Some(max_tokens) = openai_req.get("max_tokens") {
        gen_config["maxOutputTokens"] = max_tokens.clone();
    }
    if let Some(top_p) = openai_req.get("top_p") {
        gen_config["topP"] = top_p.clone();
    }
    
    if !gen_config.as_object().unwrap().is_empty() {
        gemini_req["generationConfig"] = gen_config;
    }
    
    Ok(gemini_req)
}

pub fn gemini_response_to_openai(gemini_resp: Value, model: &str) -> Result<Value> {
    let content = extract_gemini_response_content(&gemini_resp);
    
    let usage = if let Some(usage_meta) = gemini_resp.get("usageMetadata") {
        json!({
            "prompt_tokens": usage_meta.get("promptTokenCount").unwrap_or(&json!(0)),
            "completion_tokens": usage_meta.get("candidatesTokenCount").unwrap_or(&json!(0)),
            "total_tokens": usage_meta.get("totalTokenCount").unwrap_or(&json!(0))
        })
    } else {
        json!({"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0})
    };
    
    Ok(json!({
        "id": format!("chatcmpl-{}", Uuid::new_v4()),
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content
            },
            "finish_reason": "stop"
        }],
        "usage": usage
    }))
}

// ============================================================================
// OpenAI <-> Claude Conversions
// ============================================================================

pub fn openai_request_to_claude(openai_req: Value) -> Result<Value> {
    let mut claude_req = json!({});
    
    // Extract system message
    let (system_instruction, non_system_messages) = extract_system_messages(&openai_req)?;
    
    if let Some(system) = system_instruction {
        if let Some(parts) = system.get("parts").and_then(|p| p.as_array()) {
            if let Some(first_part) = parts.first() {
                if let Some(text) = first_part.get("text") {
                    claude_req["system"] = text.clone();
                }
            }
        }
    }
    
    // Convert messages
    let mut claude_messages = Vec::new();
    
    for msg in non_system_messages {
        let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
        
        if role == "tool" {
            // Tool result
            claude_messages.push(json!({
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": msg.get("tool_call_id").and_then(|id| id.as_str()).unwrap_or(""),
                    "content": msg.get("content").and_then(|c| c.as_str()).unwrap_or("")
                }]
            }));
        } else {
            let claude_role = if role == "assistant" { "assistant" } else { "user" };
            let content = convert_openai_content_to_claude_content(msg.get("content").unwrap_or(&json!("")))?;
            
            if !content.as_array().map(|a| a.is_empty()).unwrap_or(false) {
                claude_messages.push(json!({
                    "role": claude_role,
                    "content": content
                }));
            }
        }
    }
    
    claude_req["messages"] = json!(claude_messages);
    claude_req["model"] = openai_req.get("model").cloned().unwrap_or(json!("claude-3-opus"));
    claude_req["max_tokens"] = openai_req.get("max_tokens").cloned().unwrap_or(json!(DEFAULT_MAX_TOKENS));
    
    if let Some(temp) = openai_req.get("temperature") {
        claude_req["temperature"] = temp.clone();
    }
    if let Some(top_p) = openai_req.get("top_p") {
        claude_req["top_p"] = top_p.clone();
    }
    
    Ok(claude_req)
}

pub fn claude_response_to_openai(claude_resp: Value, model: &str) -> Result<Value> {
    let content = if let Some(content_arr) = claude_resp.get("content").and_then(|c| c.as_array()) {
        content_arr.iter()
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
    };
    
    let finish_reason = match claude_resp.get("stop_reason").and_then(|r| r.as_str()) {
        Some("end_turn") => "stop",
        Some(other) => other,
        None => "stop"
    };
    
    let usage = if let Some(usage) = claude_resp.get("usage") {
        json!({
            "prompt_tokens": usage.get("input_tokens").unwrap_or(&json!(0)),
            "completion_tokens": usage.get("output_tokens").unwrap_or(&json!(0)),
            "total_tokens": usage.get("input_tokens").unwrap_or(&json!(0)).as_i64().unwrap_or(0) 
                + usage.get("output_tokens").unwrap_or(&json!(0)).as_i64().unwrap_or(0)
        })
    } else {
        json!({"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0})
    };
    
    Ok(json!({
        "id": format!("chatcmpl-{}", Uuid::new_v4()),
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content
            },
            "finish_reason": finish_reason
        }],
        "usage": usage
    }))
}

// ============================================================================
// Claude <-> Gemini Conversions
// ============================================================================

pub fn claude_request_to_gemini(claude_req: Value) -> Result<Value> {
    let mut gemini_req = json!({});
    
    // System instruction
    if let Some(system) = claude_req.get("system") {
        gemini_req["systemInstruction"] = json!({
            "parts": [{"text": system}]
        });
    }
    
    // Convert messages
    let mut contents = Vec::new();
    
    if let Some(messages) = claude_req.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let gemini_role = if role == "assistant" { "model" } else { "user" };
            
            let parts = convert_claude_content_to_gemini_parts(msg.get("content").unwrap_or(&json!([])))?;
            
            if !parts.as_array().map(|a| a.is_empty()).unwrap_or(true) {
                contents.push(json!({
                    "role": gemini_role,
                    "parts": parts
                }));
            }
        }
    }
    
    gemini_req["contents"] = json!(contents);
    
    // Generation config
    let mut gen_config = json!({});
    if let Some(max_tokens) = claude_req.get("max_tokens") {
        gen_config["maxOutputTokens"] = max_tokens.clone();
    }
    if let Some(temp) = claude_req.get("temperature") {
        gen_config["temperature"] = temp.clone();
    }
    if let Some(top_p) = claude_req.get("top_p") {
        gen_config["topP"] = top_p.clone();
    }
    
    if !gen_config.as_object().unwrap().is_empty() {
        gemini_req["generationConfig"] = gen_config;
    }
    
    Ok(gemini_req)
}

pub fn gemini_response_to_claude(gemini_resp: Value, model: &str) -> Result<Value> {
    let mut content_blocks = Vec::new();
    
    if let Some(candidates) = gemini_resp.get("candidates").and_then(|c| c.as_array()) {
        for candidate in candidates {
            if let Some(parts) = candidate.get("content")
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.as_array()) 
            {
                for part in parts {
                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                        content_blocks.push(json!({
                            "type": "text",
                            "text": text
                        }));
                    }
                }
            }
        }
    }
    
    let usage = if let Some(usage_meta) = gemini_resp.get("usageMetadata") {
        json!({
            "input_tokens": usage_meta.get("promptTokenCount").unwrap_or(&json!(0)),
            "output_tokens": usage_meta.get("candidatesTokenCount").unwrap_or(&json!(0))
        })
    } else {
        json!({"input_tokens": 0, "output_tokens": 0})
    };
    
    Ok(json!({
        "id": format!("msg_{}", Uuid::new_v4()),
        "type": "message",
        "role": "assistant",
        "content": content_blocks,
        "model": model,
        "stop_reason": "end_turn",
        "usage": usage
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn extract_system_messages(openai_req: &Value) -> Result<(Option<Value>, Vec<Value>)> {
    let mut system_parts = Vec::new();
    let mut non_system = Vec::new();
    
    if let Some(messages) = openai_req.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            if msg.get("role").and_then(|r| r.as_str()) == Some("system") {
                if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                    system_parts.push(json!({"text": content}));
                }
            } else {
                non_system.push(msg.clone());
            }
        }
    }
    
    let system_instruction = if !system_parts.is_empty() {
        Some(json!({"parts": system_parts}))
    } else {
        None
    };
    
    Ok((system_instruction, non_system))
}

fn convert_openai_content_to_gemini_parts(content: &Value) -> Result<Vec<Value>> {
    let mut parts = Vec::new();
    
    if let Some(text) = content.as_str() {
        parts.push(json!({"text": text}));
    } else if let Some(arr) = content.as_array() {
        for item in arr {
            if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                match item_type {
                    "text" => {
                        if let Some(text) = item.get("text") {
                            parts.push(json!({"text": text}));
                        }
                    }
                    "image_url" => {
                        if let Some(image_url) = item.get("image_url") {
                            let url = if let Some(url_str) = image_url.as_str() {
                                url_str
                            } else {
                                image_url.get("url").and_then(|u| u.as_str()).unwrap_or("")
                            };
                            
                            if url.starts_with("data:") {
                                // Base64 image
                                if let Some((header, data)) = url.split_once(',') {
                                    let mime_type = header.strip_prefix("data:")
                                        .and_then(|s| s.split(';').next())
                                        .unwrap_or("image/jpeg");
                                    
                                    parts.push(json!({
                                        "inlineData": {
                                            "mimeType": mime_type,
                                            "data": data
                                        }
                                    }));
                                }
                            } else {
                                // URL
                                parts.push(json!({
                                    "fileData": {
                                        "mimeType": "image/jpeg",
                                        "fileUri": url
                                    }
                                }));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    Ok(parts)
}

fn convert_openai_content_to_claude_content(content: &Value) -> Result<Value> {
    let mut content_blocks = Vec::new();
    
    if let Some(text) = content.as_str() {
        if !text.is_empty() {
            content_blocks.push(json!({"type": "text", "text": text}));
        }
    } else if let Some(arr) = content.as_array() {
        for item in arr {
            if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                match item_type {
                    "text" => {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            if !text.is_empty() {
                                content_blocks.push(json!({"type": "text", "text": text}));
                            }
                        }
                    }
                    "image_url" => {
                        if let Some(image_url) = item.get("image_url") {
                            let url = if let Some(url_str) = image_url.as_str() {
                                url_str
                            } else {
                                image_url.get("url").and_then(|u| u.as_str()).unwrap_or("")
                            };
                            
                            if url.starts_with("data:") {
                                if let Some((header, data)) = url.split_once(',') {
                                    let media_type = header.strip_prefix("data:")
                                        .and_then(|s| s.split(';').next())
                                        .unwrap_or("image/jpeg");
                                    
                                    content_blocks.push(json!({
                                        "type": "image",
                                        "source": {
                                            "type": "base64",
                                            "media_type": media_type,
                                            "data": data
                                        }
                                    }));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    Ok(json!(content_blocks))
}

fn convert_claude_content_to_gemini_parts(content: &Value) -> Result<Value> {
    let mut parts = Vec::new();
    
    if let Some(text) = content.as_str() {
        parts.push(json!({"text": text}));
    } else if let Some(arr) = content.as_array() {
        for block in arr {
            if let Some(block_type) = block.get("type").and_then(|t| t.as_str()) {
                match block_type {
                    "text" => {
                        if let Some(text) = block.get("text") {
                            parts.push(json!({"text": text}));
                        }
                    }
                    "image" => {
                        if let Some(source) = block.get("source") {
                            if source.get("type").and_then(|t| t.as_str()) == Some("base64") {
                                parts.push(json!({
                                    "inlineData": {
                                        "mimeType": source.get("media_type").unwrap_or(&json!("image/jpeg")),
                                        "data": source.get("data").unwrap_or(&json!(""))
                                    }
                                }));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    Ok(json!(parts))
}

fn extract_gemini_response_content(gemini_resp: &Value) -> String {
    if let Some(candidates) = gemini_resp.get("candidates").and_then(|c| c.as_array()) {
        candidates.iter()
            .filter_map(|candidate| {
                let parts = candidate.get("content")?
                    .get("parts")?
                    .as_array()?;
                
                let texts: Vec<&str> = parts.iter()
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

