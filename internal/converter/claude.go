package converter

import "github.com/justlovemaki/AIClient-2-API/internal/common"

// Claude Request Conversions

func convertOpenAIToClaudeRequest(openaiReq map[string]interface{}) map[string]interface{} {
	claudeReq := map[string]interface{}{
		"messages": []interface{}{},
		"model":    getStringOrDefault(openaiReq, "model", "claude-3-7-sonnet-20250219"),
	}

	// Add generation config
	claudeReq["max_tokens"] = getIntOrDefault(openaiReq, "max_tokens", common.DefaultMaxTokens)
	claudeReq["temperature"] = getFloatOrDefault(openaiReq, "temperature", common.DefaultTemperature)
	claudeReq["top_p"] = getFloatOrDefault(openaiReq, "top_p", common.DefaultTopP)

	// Process messages
	if messages, ok := openaiReq["messages"].([]interface{}); ok {
		var systemText string
		var claudeMessages []interface{}

		for _, msg := range messages {
			if msgMap, ok := msg.(map[string]interface{}); ok {
				role := getStringOrDefault(msgMap, "role", "user")
				content := extractText(msgMap["content"])

				if role == "system" {
					systemText = content
				} else {
					claudeMessages = append(claudeMessages, map[string]interface{}{
						"role": role,
						"content": []interface{}{
							map[string]interface{}{
								"type": "text",
								"text": content,
							},
						},
					})
				}
			}
		}

		// Add system message if present
		if systemText != "" {
			claudeReq["system"] = systemText
		}

		claudeReq["messages"] = claudeMessages
	}

	return claudeReq
}

func convertGeminiToClaudeRequest(geminiReq map[string]interface{}) map[string]interface{} {
	claudeReq := map[string]interface{}{
		"messages": []interface{}{},
		"model":    "claude-3-7-sonnet-20250219",
	}

	// Add generation config
	if maxTokens, ok := geminiReq["max_tokens"]; ok {
		claudeReq["max_tokens"] = maxTokens
	} else {
		claudeReq["max_tokens"] = common.DefaultMaxTokens
	}

	if temp, ok := geminiReq["temperature"]; ok {
		claudeReq["temperature"] = temp
	}

	if topP, ok := geminiReq["top_p"]; ok {
		claudeReq["top_p"] = topP
	}

	// Process system instruction
	if sysInstr, ok := geminiReq["systemInstruction"].(map[string]interface{}); ok {
		if parts, ok := sysInstr["parts"].([]interface{}); ok {
			systemText := extractPartsText(parts)
			if systemText != "" {
				claudeReq["system"] = systemText
			}
		}
	}

	// Process contents
	if contents, ok := geminiReq["contents"].([]interface{}); ok {
		var messages []interface{}
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
							"role": role,
							"content": []interface{}{
								map[string]interface{}{
									"type": "text",
									"text": text,
								},
							},
						})
					}
				}
			}
		}
		claudeReq["messages"] = messages
	}

	return claudeReq
}

// Claude Response Conversions

func convertOpenAIToClaudeResponse(openaiResp map[string]interface{}, model string) map[string]interface{} {
	claudeResp := map[string]interface{}{
		"id":           generateID("msg"),
		"type":         "message",
		"role":         "assistant",
		"content":      []interface{}{},
		"model":        model,
		"stop_reason":  "end_turn",
		"stop_sequence": nil,
		"usage": map[string]interface{}{
			"input_tokens":  0,
			"output_tokens": 0,
		},
	}

	// Extract content from choices
	if choices, ok := openaiResp["choices"].([]interface{}); ok && len(choices) > 0 {
		if choice, ok := choices[0].(map[string]interface{}); ok {
			var contentText string
			if message, ok := choice["message"].(map[string]interface{}); ok {
				contentText = getStringOrDefault(message, "content", "")
			}

			claudeResp["content"] = []interface{}{
				map[string]interface{}{
					"type": "text",
					"text": contentText,
				},
			}

			// Map finish reason
			if finishReason, ok := choice["finish_reason"].(string); ok {
				switch finishReason {
				case "stop":
					claudeResp["stop_reason"] = "end_turn"
				case "length":
					claudeResp["stop_reason"] = "max_tokens"
				default:
					claudeResp["stop_reason"] = "end_turn"
				}
			}
		}
	}

	// Add usage
	if usage, ok := openaiResp["usage"].(map[string]interface{}); ok {
		claudeUsage := claudeResp["usage"].(map[string]interface{})
		claudeUsage["input_tokens"] = getIntOrDefault(usage, "prompt_tokens", 0)
		claudeUsage["output_tokens"] = getIntOrDefault(usage, "completion_tokens", 0)
	}

	return claudeResp
}

func convertGeminiToClaudeResponse(geminiResp map[string]interface{}, model string) map[string]interface{} {
	claudeResp := map[string]interface{}{
		"id":            generateID("msg"),
		"type":          "message",
		"role":          "assistant",
		"content":       []interface{}{},
		"model":         model,
		"stop_reason":   "end_turn",
		"stop_sequence": nil,
		"usage": map[string]interface{}{
			"input_tokens":  0,
			"output_tokens": 0,
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

			// Map finish reason
			if finishReason, ok := candidate["finishReason"].(string); ok {
				switch finishReason {
				case "STOP":
					claudeResp["stop_reason"] = "end_turn"
				case "MAX_TOKENS":
					claudeResp["stop_reason"] = "max_tokens"
				default:
					claudeResp["stop_reason"] = "end_turn"
				}
			}
		}
	}

	claudeResp["content"] = []interface{}{
		map[string]interface{}{
			"type": "text",
			"text": contentText,
		},
	}

	// Add usage metadata
	if usageMetadata, ok := geminiResp["usageMetadata"].(map[string]interface{}); ok {
		usage := claudeResp["usage"].(map[string]interface{})
		usage["input_tokens"] = getIntOrDefault(usageMetadata, "promptTokenCount", 0)
		usage["output_tokens"] = getIntOrDefault(usageMetadata, "candidatesTokenCount", 0)
	}

	return claudeResp
}

// Claude Stream Chunk Conversions

func convertOpenAIToClaudeStreamChunk(openaiChunk map[string]interface{}, model string) map[string]interface{} {
	text := ""
	if choices, ok := openaiChunk["choices"].([]interface{}); ok && len(choices) > 0 {
		if choice, ok := choices[0].(map[string]interface{}); ok {
			if delta, ok := choice["delta"].(map[string]interface{}); ok {
				text = getStringOrDefault(delta, "content", "")
			}
		}
	}

	return map[string]interface{}{
		"type":  "content_block_delta",
		"index": 0,
		"delta": map[string]interface{}{
			"type": "text_delta",
			"text": text,
		},
	}
}

func convertGeminiToClaudeStreamChunk(geminiChunk map[string]interface{}, model string) map[string]interface{} {
	text := ""
	if textVal, ok := geminiChunk["text"]; ok {
		text = extractText(textVal)
	} else if candidates, ok := geminiChunk["candidates"].([]interface{}); ok && len(candidates) > 0 {
		if candidate, ok := candidates[0].(map[string]interface{}); ok {
			if content, ok := candidate["content"].(map[string]interface{}); ok {
				if parts, ok := content["parts"].([]interface{}); ok {
					text = extractPartsText(parts)
				}
			}
		}
	}

	return map[string]interface{}{
		"type":  "content_block_delta",
		"index": 0,
		"delta": map[string]interface{}{
			"type": "text_delta",
			"text": text,
		},
	}
}

