/*!
 * Conversion Tests
 *
 * Unit tests for format conversion functions.
 */

use aiclient2api_rust::convert_detailed::*;
use serde_json::json;

#[test]
fn test_openai_to_gemini_basic() {
    let openai_req = json!({
        "model": "gpt-4",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant"},
            {"role": "user", "content": "Hello"}
        ]
    });

    let result = openai_request_to_gemini(openai_req).unwrap();
    
    // Check system instruction
    assert!(result.get("systemInstruction").is_some());
    
    // Check contents
    let contents = result.get("contents").unwrap().as_array().unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0]["role"], "user");
}

#[test]
fn test_gemini_to_openai_response() {
    let gemini_resp = json!({
        "candidates": [{
            "content": {
                "parts": [{"text": "Hello, how can I help you?"}],
                "role": "model"
            },
            "finishReason": "STOP"
        }],
        "usageMetadata": {
            "promptTokenCount": 10,
            "candidatesTokenCount": 8,
            "totalTokenCount": 18
        }
    });

    let result = gemini_response_to_openai(gemini_resp, "gemini-2.5-flash").unwrap();
    
    // Check structure
    assert_eq!(result["object"], "chat.completion");
    assert_eq!(result["model"], "gemini-2.5-flash");
    
    // Check content
    let choices = result["choices"].as_array().unwrap();
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0]["message"]["role"], "assistant");
    assert!(choices[0]["message"]["content"].as_str().unwrap().contains("Hello"));
    
    // Check usage
    assert_eq!(result["usage"]["prompt_tokens"], 10);
    assert_eq!(result["usage"]["completion_tokens"], 8);
}

#[test]
fn test_openai_to_claude_basic() {
    let openai_req = json!({
        "model": "gpt-4",
        "messages": [
            {"role": "user", "content": "Hello"},
            {"role": "assistant", "content": "Hi there!"},
            {"role": "user", "content": "How are you?"}
        ],
        "max_tokens": 100
    });

    let result = openai_request_to_claude(openai_req).unwrap();
    
    // Check messages
    let messages = result.get("messages").unwrap().as_array().unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0]["role"], "user");
    assert_eq!(messages[1]["role"], "assistant");
    
    // Check max_tokens
    assert_eq!(result["max_tokens"], 100);
}

#[test]
fn test_claude_to_openai_response() {
    let claude_resp = json!({
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": "I'm doing well, thank you!"
            }
        ],
        "model": "claude-3-opus",
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 15,
            "output_tokens": 12
        }
    });

    let result = claude_response_to_openai(claude_resp, "claude-3-opus").unwrap();
    
    // Check structure
    assert_eq!(result["object"], "chat.completion");
    assert_eq!(result["model"], "claude-3-opus");
    
    // Check content
    let choices = result["choices"].as_array().unwrap();
    assert_eq!(choices[0]["message"]["content"], "I'm doing well, thank you!");
    assert_eq!(choices[0]["finish_reason"], "stop");
    
    // Check usage
    assert_eq!(result["usage"]["prompt_tokens"], 15);
    assert_eq!(result["usage"]["completion_tokens"], 12);
}

#[test]
fn test_claude_to_gemini_basic() {
    let claude_req = json!({
        "model": "claude-3-opus",
        "system": "You are helpful",
        "messages": [
            {
                "role": "user",
                "content": [{"type": "text", "text": "Hello"}]
            }
        ],
        "max_tokens": 200
    });

    let result = claude_request_to_gemini(claude_req).unwrap();
    
    // Check system instruction
    assert!(result.get("systemInstruction").is_some());
    
    // Check contents
    let contents = result.get("contents").unwrap().as_array().unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0]["role"], "user");
    
    // Check generation config
    let gen_config = result.get("generationConfig").unwrap();
    assert_eq!(gen_config["maxOutputTokens"], 200);
}

#[test]
fn test_gemini_to_claude_response() {
    let gemini_resp = json!({
        "candidates": [{
            "content": {
                "parts": [{"text": "This is a test response"}],
                "role": "model"
            }
        }],
        "usageMetadata": {
            "promptTokenCount": 20,
            "candidatesTokenCount": 10
        }
    });

    let result = gemini_response_to_claude(gemini_resp, "claude-3-opus").unwrap();
    
    // Check structure
    assert_eq!(result["type"], "message");
    assert_eq!(result["role"], "assistant");
    assert_eq!(result["model"], "claude-3-opus");
    
    // Check content
    let content = result["content"].as_array().unwrap();
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[0]["text"], "This is a test response");
    
    // Check usage
    assert_eq!(result["usage"]["input_tokens"], 20);
    assert_eq!(result["usage"]["output_tokens"], 10);
}

#[test]
fn test_multimodal_conversion() {
    let openai_req = json!({
        "model": "gpt-4-vision",
        "messages": [{
            "role": "user",
            "content": [
                {"type": "text", "text": "What's in this image?"},
                {
                    "type": "image_url",
                    "image_url": {
                        "url": "data:image/jpeg;base64,/9j/4AAQ..."
                    }
                }
            ]
        }]
    });

    let result = openai_request_to_gemini(openai_req).unwrap();
    
    // Check that parts include both text and image
    let contents = result.get("contents").unwrap().as_array().unwrap();
    let parts = contents[0]["parts"].as_array().unwrap();
    
    assert!(parts.len() >= 2);
    assert!(parts[0].get("text").is_some());
    assert!(parts[1].get("inlineData").is_some());
}

