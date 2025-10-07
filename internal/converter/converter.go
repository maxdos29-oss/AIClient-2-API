package converter

import (
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/justlovemaki/AIClient-2-API/internal/common"
)

// Converter handles data conversion between different API formats
type Converter struct{}

// NewConverter creates a new converter instance
func NewConverter() *Converter {
	return &Converter{}
}

// ConvertRequest converts a request from one format to another
func (c *Converter) ConvertRequest(data map[string]interface{}, fromProtocol, toProtocol string) (map[string]interface{}, error) {
	if fromProtocol == toProtocol {
		return data, nil
	}

	switch toProtocol {
	case common.ModelProtocolOpenAI:
		return c.toOpenAIRequest(data, fromProtocol)
	case common.ModelProtocolGemini:
		return c.toGeminiRequest(data, fromProtocol)
	case common.ModelProtocolClaude:
		return c.toClaudeRequest(data, fromProtocol)
	default:
		return nil, fmt.Errorf("unsupported target protocol: %s", toProtocol)
	}
}

// ConvertResponse converts a response from one format to another
func (c *Converter) ConvertResponse(data map[string]interface{}, fromProtocol, toProtocol, model string) (map[string]interface{}, error) {
	if fromProtocol == toProtocol {
		return data, nil
	}

	switch toProtocol {
	case common.ModelProtocolOpenAI:
		return c.toOpenAIResponse(data, fromProtocol, model)
	case common.ModelProtocolGemini:
		return c.toGeminiResponse(data, fromProtocol, model)
	case common.ModelProtocolClaude:
		return c.toClaudeResponse(data, fromProtocol, model)
	default:
		return nil, fmt.Errorf("unsupported target protocol: %s", toProtocol)
	}
}

// ConvertStreamChunk converts a streaming chunk from one format to another
func (c *Converter) ConvertStreamChunk(data interface{}, fromProtocol, toProtocol, model string) (interface{}, error) {
	if fromProtocol == toProtocol {
		return data, nil
	}

	// Convert to map if it's a string
	var dataMap map[string]interface{}
	switch v := data.(type) {
	case string:
		dataMap = map[string]interface{}{"text": v}
	case map[string]interface{}:
		dataMap = v
	default:
		return nil, fmt.Errorf("unsupported data type for stream chunk")
	}

	switch toProtocol {
	case common.ModelProtocolOpenAI:
		return c.toOpenAIStreamChunk(dataMap, fromProtocol, model)
	case common.ModelProtocolGemini:
		return c.toGeminiStreamChunk(dataMap, fromProtocol, model)
	case common.ModelProtocolClaude:
		return c.toClaudeStreamChunk(dataMap, fromProtocol, model)
	default:
		return nil, fmt.Errorf("unsupported target protocol: %s", toProtocol)
	}
}

// Helper functions to get protocol-specific converters
func (c *Converter) toOpenAIRequest(data map[string]interface{}, fromProtocol string) (map[string]interface{}, error) {
	switch fromProtocol {
	case common.ModelProtocolGemini:
		return convertGeminiToOpenAIRequest(data), nil
	case common.ModelProtocolClaude:
		return convertClaudeToOpenAIRequest(data), nil
	default:
		return nil, fmt.Errorf("unsupported source protocol: %s", fromProtocol)
	}
}

func (c *Converter) toGeminiRequest(data map[string]interface{}, fromProtocol string) (map[string]interface{}, error) {
	switch fromProtocol {
	case common.ModelProtocolOpenAI:
		return convertOpenAIToGeminiRequest(data), nil
	case common.ModelProtocolClaude:
		return convertClaudeToGeminiRequest(data), nil
	default:
		return nil, fmt.Errorf("unsupported source protocol: %s", fromProtocol)
	}
}

func (c *Converter) toClaudeRequest(data map[string]interface{}, fromProtocol string) (map[string]interface{}, error) {
	switch fromProtocol {
	case common.ModelProtocolOpenAI:
		return convertOpenAIToClaudeRequest(data), nil
	case common.ModelProtocolGemini:
		return convertGeminiToClaudeRequest(data), nil
	default:
		return nil, fmt.Errorf("unsupported source protocol: %s", fromProtocol)
	}
}

func (c *Converter) toOpenAIResponse(data map[string]interface{}, fromProtocol, model string) (map[string]interface{}, error) {
	switch fromProtocol {
	case common.ModelProtocolGemini:
		return convertGeminiToOpenAIResponse(data, model), nil
	case common.ModelProtocolClaude:
		return convertClaudeToOpenAIResponse(data, model), nil
	default:
		return nil, fmt.Errorf("unsupported source protocol: %s", fromProtocol)
	}
}

func (c *Converter) toGeminiResponse(data map[string]interface{}, fromProtocol, model string) (map[string]interface{}, error) {
	return nil, fmt.Errorf("gemini response conversion not yet implemented")
}

func (c *Converter) toClaudeResponse(data map[string]interface{}, fromProtocol, model string) (map[string]interface{}, error) {
	switch fromProtocol {
	case common.ModelProtocolOpenAI:
		return convertOpenAIToClaudeResponse(data, model), nil
	case common.ModelProtocolGemini:
		return convertGeminiToClaudeResponse(data, model), nil
	default:
		return nil, fmt.Errorf("unsupported source protocol: %s", fromProtocol)
	}
}

func (c *Converter) toOpenAIStreamChunk(data map[string]interface{}, fromProtocol, model string) (interface{}, error) {
	switch fromProtocol {
	case common.ModelProtocolGemini:
		return convertGeminiToOpenAIStreamChunk(data, model), nil
	case common.ModelProtocolClaude:
		return convertClaudeToOpenAIStreamChunk(data, model), nil
	default:
		return nil, fmt.Errorf("unsupported source protocol: %s", fromProtocol)
	}
}

func (c *Converter) toGeminiStreamChunk(data map[string]interface{}, fromProtocol, model string) (interface{}, error) {
	return nil, fmt.Errorf("gemini stream chunk conversion not yet implemented")
}

func (c *Converter) toClaudeStreamChunk(data map[string]interface{}, fromProtocol, model string) (interface{}, error) {
	switch fromProtocol {
	case common.ModelProtocolOpenAI:
		return convertOpenAIToClaudeStreamChunk(data, model), nil
	case common.ModelProtocolGemini:
		return convertGeminiToClaudeStreamChunk(data, model), nil
	default:
		return nil, fmt.Errorf("unsupported source protocol: %s", fromProtocol)
	}
}

// Helper function to generate IDs
func generateID(prefix string) string {
	return fmt.Sprintf("%s_%s", prefix, uuid.New().String())
}

// Helper function to get current timestamp
func getCurrentTimestamp() int64 {
	return time.Now().Unix()
}

// Helper function to extract text from content
func extractText(content interface{}) string {
	switch v := content.(type) {
	case string:
		return v
	case []interface{}:
		var texts []string
		for _, item := range v {
			if itemMap, ok := item.(map[string]interface{}); ok {
				if text, ok := itemMap["text"].(string); ok {
					texts = append(texts, text)
				}
			}
		}
		return joinStrings(texts, "\n")
	case map[string]interface{}:
		if text, ok := v["text"].(string); ok {
			return text
		}
	}
	return ""
}

// Helper function to join strings
func joinStrings(strs []string, sep string) string {
	if len(strs) == 0 {
		return ""
	}
	result := strs[0]
	for i := 1; i < len(strs); i++ {
		result += sep + strs[i]
	}
	return result
}

// Helper function to get value with default
func getValueOrDefault(m map[string]interface{}, key string, defaultValue interface{}) interface{} {
	if val, ok := m[key]; ok && val != nil {
		return val
	}
	return defaultValue
}

// Helper function to get int with default
func getIntOrDefault(m map[string]interface{}, key string, defaultValue int) int {
	if val, ok := m[key]; ok {
		switch v := val.(type) {
		case int:
			return v
		case float64:
			return int(v)
		}
	}
	return defaultValue
}

// Helper function to get float with default
func getFloatOrDefault(m map[string]interface{}, key string, defaultValue float64) float64 {
	if val, ok := m[key]; ok {
		switch v := val.(type) {
		case float64:
			return v
		case int:
			return float64(v)
		}
	}
	return defaultValue
}

// Helper function to get string with default
func getStringOrDefault(m map[string]interface{}, key string, defaultValue string) string {
	if val, ok := m[key]; ok {
		if str, ok := val.(string); ok {
			return str
		}
	}
	return defaultValue
}
