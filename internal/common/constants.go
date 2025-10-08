package common

// Model provider constants
const (
	ModelProviderGeminiCLI     = "gemini-cli-oauth"
	ModelProviderOpenAICustom  = "openai-custom"
	ModelProviderClaudeCustom  = "claude-custom"
	ModelProviderKiroAPI       = "claude-kiro-oauth"
	ModelProviderQwenAPI       = "openai-qwen-oauth"
)

// Model protocol prefix constants
const (
	ModelProtocolGemini = "gemini"
	ModelProtocolOpenAI = "openai"
	ModelProtocolClaude = "claude"
)

// API actions
const (
	APIActionGenerateContent       = "generateContent"
	APIActionStreamGenerateContent = "streamGenerateContent"
)

// Endpoint types
const (
	EndpointTypeOpenAIChat      = "openai_chat"
	EndpointTypeGeminiContent   = "gemini_content"
	EndpointTypeClaudeMessage   = "claude_message"
	EndpointTypeOpenAIModelList = "openai_model_list"
	EndpointTypeGeminiModelList = "gemini_model_list"
)

// File paths
const (
	FetchSystemPromptFile = "fetch_system_prompt.txt"
	InputSystemPromptFile = "input_system_prompt.txt"
)

// Default generation config values
const (
	DefaultMaxTokens       = 8192
	DefaultGeminiMaxTokens = 65536
	DefaultTemperature     = 1.0
	DefaultTopP            = 0.9
)

// GetProtocolPrefix extracts the protocol prefix from a provider string
func GetProtocolPrefix(provider string) string {
	if len(provider) == 0 {
		return provider
	}

	// Find the first hyphen
	for i, c := range provider {
		if c == '-' {
			return provider[:i]
		}
	}

	return provider
}

// AllModelProviders returns all valid model provider names
func AllModelProviders() []string {
	return []string{
		ModelProviderGeminiCLI,
		ModelProviderOpenAICustom,
		ModelProviderClaudeCustom,
		ModelProviderKiroAPI,
		ModelProviderQwenAPI,
	}
}

