/*!
 * System Prompt Tests
 *
 * Unit tests for system prompt management.
 */

use serde_json::json;

#[tokio::test]
async fn test_apply_to_openai_overwrite() {
    use aiclient2api_rust::system_prompt::SystemPromptManager;
    use std::path::PathBuf;
    
    let manager = SystemPromptManager::new(
        None,
        "overwrite".to_string()
    ).await.unwrap();
    
    // Manually set content for testing
    let mut manager = manager;
    manager.content = Some("New system prompt".to_string());
    
    let request = json!({
        "messages": [
            {"role": "system", "content": "Old system prompt"},
            {"role": "user", "content": "Hello"}
        ]
    });
    
    let result = manager.apply_to_openai(request).unwrap();
    let messages = result["messages"].as_array().unwrap();
    
    // Should have only one system message
    let system_messages: Vec<_> = messages.iter()
        .filter(|m| m["role"] == "system")
        .collect();
    
    assert_eq!(system_messages.len(), 1);
    assert_eq!(system_messages[0]["content"], "New system prompt");
}

#[tokio::test]
async fn test_apply_to_openai_append() {
    use aiclient2api_rust::system_prompt::SystemPromptManager;
    
    let mut manager = SystemPromptManager::new(
        None,
        "append".to_string()
    ).await.unwrap();
    
    manager.content = Some("Additional instructions".to_string());
    
    let request = json!({
        "messages": [
            {"role": "system", "content": "Base system prompt"},
            {"role": "user", "content": "Hello"}
        ]
    });
    
    let result = manager.apply_to_openai(request).unwrap();
    let messages = result["messages"].as_array().unwrap();
    
    let system_msg = messages.iter()
        .find(|m| m["role"] == "system")
        .unwrap();
    
    let content = system_msg["content"].as_str().unwrap();
    assert!(content.contains("Base system prompt"));
    assert!(content.contains("Additional instructions"));
}

#[tokio::test]
async fn test_apply_to_claude() {
    use aiclient2api_rust::system_prompt::SystemPromptManager;
    
    let mut manager = SystemPromptManager::new(
        None,
        "overwrite".to_string()
    ).await.unwrap();
    
    manager.content = Some("Claude system prompt".to_string());
    
    let request = json!({
        "messages": [
            {"role": "user", "content": "Hello"}
        ]
    });
    
    let result = manager.apply_to_claude(request).unwrap();
    
    assert_eq!(result["system"], "Claude system prompt");
}

#[tokio::test]
async fn test_apply_to_gemini() {
    use aiclient2api_rust::system_prompt::SystemPromptManager;
    
    let mut manager = SystemPromptManager::new(
        None,
        "overwrite".to_string()
    ).await.unwrap();
    
    manager.content = Some("Gemini system prompt".to_string());
    
    let request = json!({
        "contents": [
            {"role": "user", "parts": [{"text": "Hello"}]}
        ]
    });
    
    let result = manager.apply_to_gemini(request).unwrap();
    
    let system_instruction = result["systemInstruction"].as_object().unwrap();
    let parts = system_instruction["parts"].as_array().unwrap();
    
    assert_eq!(parts[0]["text"], "Gemini system prompt");
}

