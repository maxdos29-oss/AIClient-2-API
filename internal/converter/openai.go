package converter

import "github.com/justlovemaki/AIClient-2-API/internal/common"

// OpenAI Request Conversions

func convertGeminiToOpenAIRequest(geminiReq map[string]interface{}) map[string]interface{} {
	openaiReq := map[string]interface{}{
		"messages": []interface{}{},
		"model":    getStringOrDefault(geminiReq, "model", "gpt-3.5-turbo"),
	}

	// Add generation config
	if maxTokens, ok := geminiReq["max_tokens"]; ok {
		openaiReq["max_tokens"] = maxTokens
	} else {
		openaiReq["max_tokens"] = common.DefaultMaxTokens
	}

	if temp, ok := geminiReq["temperature"]; ok {
		openaiReq["temperature"] = temp
	}

	if topP, ok := geminiReq["top_p"]; ok {
		openaiReq["top_p"] = topP
	}

	// Process system instruction
	if sysInstr, ok := geminiReq["systemInstruction"].(map[string]interface{}); ok {
		if parts, ok := sysInstr["parts"].([]interface{}); ok {
			systemText := extractPartsText(parts)
			if systemText != "" {
				messages := openaiReq["messages"].([]interface{})
				messages = append(messages, map[string]interface{}{
					"role":    "system",
					"content": systemText,
				})
				openaiReq["messages"] = messages
			}
		}
	}

	// Process contents
	if contents, ok := geminiReq["contents"].([]interface{}); ok {
		messages := openaiReq["messages"].([]interface{})
		for _, content := range contents {
			if contentMap, ok := content.(map[string]interface{}); ok {
				role := getStringOrDefault(contentMap, "role", "user")
				if role == "model" {
					role = "assistant"
				}

				if parts, ok := contentMap["parts"].([]interface{}); ok {
					text := extractPartsText(parts)
					if text != "" {
						messages = append(messages, map[string]interface{}{
							"role":    role,
							"content": text,
						})
					}
				}
			}
		}
		openaiReq["messages"] = messages
	}

	return openaiReq
}

func convertClaudeToOpenAIRequest(claudeReq map[string]interface{}) map[string]interface{} {
	openaiReq := map[string]interface{}{
		"messages": []interface{}{},
		"model":    getStringOrDefault(claudeReq, "model", "gpt-3.5-turbo"),
	}

	// Add generation config
	openaiReq["max_tokens"] = getIntOrDefault(claudeReq, "max_tokens", common.DefaultMaxTokens)
	openaiReq["temperature"] = getFloatOrDefault(claudeReq, "temperature", common.DefaultTemperature)
	openaiReq["top_p"] = getFloatOrDefault(claudeReq, "top_p", common.DefaultTopP)

	// Add system message if present
	if system, ok := claudeReq["system"]; ok {
		systemText := extractText(system)
		if systemText != "" {
			messages := openaiReq["messages"].([]interface{})
			messages = append(messages, map[string]interface{}{
				"role":    "system",
				"content": systemText,
			})
			openaiReq["messages"] = messages
		}
	}

	// Process messages
	if messages, ok := claudeReq["messages"].([]interface{}); ok {
		openaiMessages := openaiReq["messages"].([]interface{})
		for _, msg := range messages {
			if msgMap, ok := msg.(map[string]interface{}); ok {
				role := getStringOrDefault(msgMap, "role", "user")
				content := msgMap["content"]
				
				openaiMessages = append(openaiMessages, map[string]interface{}{
					"role":    role,
					"content": extractText(content),
				})
			}
		}
		openaiReq["messages"] = openaiMessages
	}

	return openaiReq
}

// OpenAI Response Conversions

func convertGeminiToOpenAIResponse(geminiResp map[string]interface{}, model string) map[string]interface{} {
	openaiResp := map[string]interface{}{
		"id":      generateID("chatcmpl"),
		"object":  "chat.completion",
		"created": getCurrentTimestamp(),
		"model":   model,
		"choices": []interface{}{},
		"usage": map[string]interface{}{
			"prompt_tokens":     0,
			"completion_tokens": 0,
			"total_tokens":      0,
		},
	}

	// Extract content from candidates
	var contentText string
	if candidates, ok := geminiResp["candidates"].([]interface{}); ok && len(candidates) > 0 {
		if candidate, ok := candidates[0].(map[string]interface{}); ok {
			if content, ok := candidate["content"].(map[string]interface{}); ok {
				if parts, ok := content["parts"].([]interface{}); ok {
					contentText = extractPartsText(parts)
				}
			}
		}
	}

	openaiResp["choices"] = []interface{}{
		map[string]interface{}{
			"index": 0,
			"message": map[string]interface{}{
				"role":    "assistant",
				"content": contentText,
			},
			"finish_reason": "stop",
		},
	}

	// Add usage metadata if available
	if usageMetadata, ok := geminiResp["usageMetadata"].(map[string]interface{}); ok {
		usage := openaiResp["usage"].(map[string]interface{})
		usage["prompt_tokens"] = getIntOrDefault(usageMetadata, "promptTokenCount", 0)
		usage["completion_tokens"] = getIntOrDefault(usageMetadata, "candidatesTokenCount", 0)
		usage["total_tokens"] = getIntOrDefault(usageMetadata, "totalTokenCount", 0)
	}

	return openaiResp
}

func convertClaudeToOpenAIResponse(claudeResp map[string]interface{}, model string) map[string]interface{} {
	openaiResp := map[string]interface{}{
		"id":      generateID("chatcmpl"),
		"object":  "chat.completion",
		"created": getCurrentTimestamp(),
		"model":   model,
		"choices": []interface{}{},
		"usage": map[string]interface{}{
			"prompt_tokens":     0,
			"completion_tokens": 0,
			"total_tokens":      0,
		},
	}

	// Extract content
	var contentText string
	if content, ok := claudeResp["content"].([]interface{}); ok {
		for _, block := range content {
			if blockMap, ok := block.(map[string]interface{}); ok {
				if blockMap["type"] == "text" {
					if text, ok := blockMap["text"].(string); ok {
						contentText += text
					}
				}
			}
		}
	}

	// Map finish reason
	finishReason := "stop"
	if stopReason, ok := claudeResp["stop_reason"].(string); ok {
		switch stopReason {
		case "end_turn":
			finishReason = "stop"
		case "max_tokens":
			finishReason = "length"
		default:
			finishReason = "stop"
		}
	}

	openaiResp["choices"] = []interface{}{
		map[string]interface{}{
			"index": 0,
			"message": map[string]interface{}{
				"role":    "assistant",
				"content": contentText,
			},
			"finish_reason": finishReason,
		},
	}

	// Add usage
	if usage, ok := claudeResp["usage"].(map[string]interface{}); ok {
		openaiUsage := openaiResp["usage"].(map[string]interface{})
		inputTokens := getIntOrDefault(usage, "input_tokens", 0)
		outputTokens := getIntOrDefault(usage, "output_tokens", 0)
		openaiUsage["prompt_tokens"] = inputTokens
		openaiUsage["completion_tokens"] = outputTokens
		openaiUsage["total_tokens"] = inputTokens + outputTokens
	}

	return openaiResp
}

// OpenAI Stream Chunk Conversions

func convertGeminiToOpenAIStreamChunk(geminiChunk map[string]interface{}, model string) map[string]interface{} {
	text := ""
	if textVal, ok := geminiChunk["text"]; ok {
		text = extractText(textVal)
	}

	return map[string]interface{}{
		"id":      generateID("chatcmpl"),
		"object":  "chat.completion.chunk",
		"created": getCurrentTimestamp(),
		"model":   model,
		"choices": []interface{}{
			map[string]interface{}{
				"index": 0,
				"delta": map[string]interface{}{
					"content": text,
				},
				"finish_reason": nil,
			},
		},
	}
}

func convertClaudeToOpenAIStreamChunk(claudeChunk map[string]interface{}, model string) map[string]interface{} {
	text := ""
	if textVal, ok := claudeChunk["text"]; ok {
		text = extractText(textVal)
	} else if delta, ok := claudeChunk["delta"].(map[string]interface{}); ok {
		if textVal, ok := delta["text"]; ok {
			text = extractText(textVal)
		}
	}

	return map[string]interface{}{
		"id":      generateID("chatcmpl"),
		"object":  "chat.completion.chunk",
		"created": getCurrentTimestamp(),
		"model":   model,
		"choices": []interface{}{
			map[string]interface{}{
				"index": 0,
				"delta": map[string]interface{}{
					"content": text,
				},
				"finish_reason": nil,
			},
		},
	}
}

// Helper function to extract text from parts
func extractPartsText(parts []interface{}) string {
	var texts []string
	for _, part := range parts {
		if partMap, ok := part.(map[string]interface{}); ok {
			if text, ok := partMap["text"].(string); ok {
				texts = append(texts, text)
			}
		}
	}
	return joinStrings(texts, "\n")
}

