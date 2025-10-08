/*!
 * Logger Tests
 *
 * Unit tests for logging functionality.
 */

use aiclient2api_rust::logger::*;
use serde_json::json;

#[test]
fn test_extract_prompt_from_openai_request() {
    let request = json!({
        "messages": [
            {"role": "system", "content": "You are helpful"},
            {"role": "user", "content": "Hello"},
            {"role": "assistant", "content": "Hi there!"}
        ]
    });

    let result = extract_prompt_from_request(&request, "openai");
    
    assert!(result.contains("system: You are helpful"));
    assert!(result.contains("user: Hello"));
    assert!(result.contains("assistant: Hi there!"));
}

#[test]
fn test_extract_prompt_from_claude_request() {
    let request = json!({
        "system": "You are helpful",
        "messages": [
            {
                "role": "user",
                "content": [{"type": "text", "text": "Hello"}]
            }
        ]
    });

    let result = extract_prompt_from_request(&request, "claude");
    
    assert!(result.contains("system: You are helpful"));
    assert!(result.contains("user: Hello"));
}

#[test]
fn test_extract_prompt_from_gemini_request() {
    let request = json!({
        "systemInstruction": {
            "parts": [{"text": "You are helpful"}]
        },
        "contents": [{
            "role": "user",
            "parts": [{"text": "Hello"}]
        }]
    });

    let result = extract_prompt_from_request(&request, "gemini");
    
    assert!(result.contains("system: You are helpful"));
    assert!(result.contains("user: Hello"));
}

#[test]
fn test_extract_text_from_openai_response() {
    let response = json!({
        "choices": [{
            "message": {
                "role": "assistant",
                "content": "This is a response"
            }
        }]
    });

    let result = extract_text_from_response(&response, "openai");
    
    assert_eq!(result, "This is a response");
}

#[test]
fn test_extract_text_from_claude_response() {
    let response = json!({
        "content": [
            {"type": "text", "text": "First part"},
            {"type": "text", "text": "Second part"}
        ]
    });

    let result = extract_text_from_response(&response, "claude");
    
    assert!(result.contains("First part"));
    assert!(result.contains("Second part"));
}

#[test]
fn test_extract_text_from_gemini_response() {
    let response = json!({
        "candidates": [{
            "content": {
                "parts": [
                    {"text": "Hello"},
                    {"text": "World"}
                ]
            }
        }]
    });

    let result = extract_text_from_response(&response, "gemini");
    
    assert!(result.contains("Hello"));
    assert!(result.contains("World"));
}

#[tokio::test]
async fn test_logger_creation() {
    let logger = ConversationLogger::new("none", "test_log");
    
    // Should not panic
    let result = logger.log_input("test").await;
    assert!(result.is_ok());
}

