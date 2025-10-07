/*!
 * Provider Tests
 *
 * Integration tests for different AI service providers.
 */

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn test_model_provider_parsing() {
        use aiclient2api_rust::common::ModelProvider;
        
        assert_eq!(ModelProvider::from_str("gemini-cli-oauth"), Some(ModelProvider::GeminiCliOAuth));
        assert_eq!(ModelProvider::from_str("openai-custom"), Some(ModelProvider::OpenAICustom));
        assert_eq!(ModelProvider::from_str("claude-custom"), Some(ModelProvider::ClaudeCustom));
        assert_eq!(ModelProvider::from_str("invalid"), None);
    }

    #[test]
    fn test_model_protocol_extraction() {
        use aiclient2api_rust::common::{ModelProvider, ModelProtocol};
        
        assert_eq!(ModelProvider::GeminiCliOAuth.protocol(), ModelProtocol::Gemini);
        assert_eq!(ModelProvider::OpenAICustom.protocol(), ModelProtocol::OpenAI);
        assert_eq!(ModelProvider::ClaudeCustom.protocol(), ModelProtocol::Claude);
        assert_eq!(ModelProvider::ClaudeKiroOAuth.protocol(), ModelProtocol::Claude);
        assert_eq!(ModelProvider::OpenAIQwenOAuth.protocol(), ModelProtocol::OpenAI);
    }

    #[test]
    fn test_authorization_check() {
        use aiclient2api_rust::common::is_authorized;
        
        let required_key = "test-key-123";
        
        // Test Bearer token
        assert!(is_authorized(
            Some("Bearer test-key-123"),
            None,
            None,
            None,
            required_key
        ));
        
        // Test x-api-key header
        assert!(is_authorized(
            None,
            Some("test-key-123"),
            None,
            None,
            required_key
        ));
        
        // Test x-goog-api-key header
        assert!(is_authorized(
            None,
            None,
            Some("test-key-123"),
            None,
            required_key
        ));
        
        // Test query parameter
        assert!(is_authorized(
            None,
            None,
            None,
            Some("test-key-123"),
            required_key
        ));
        
        // Test invalid key
        assert!(!is_authorized(
            Some("Bearer wrong-key"),
            None,
            None,
            None,
            required_key
        ));
        
        // Test no key
        assert!(!is_authorized(None, None, None, None, required_key));
    }
}

