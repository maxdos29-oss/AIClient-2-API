package converter

import "github.com/justlovemaki/AIClient-2-API/internal/common"

// Gemini Request Conversions

func convertOpenAIToGeminiRequest(openaiReq map[string]interface{}) map[string]interface{} {
	geminiReq := map[string]interface{}{
		"contents": []interface{}{},
	}

	// Process messages
	if messages, ok := openaiReq["messages"].([]interface{}); ok {
		var systemText string
		var contents []interface{}

		for _, msg := range messages {
			if msgMap, ok := msg.(map[string]interface{}); ok {
				role := getStringOrDefault(msgMap, "role", "user")
				content := extractText(msgMap["content"])

				if role == "system" {
					systemText = content
				} else {
					geminiRole := role
					if role == "assistant" {
						geminiRole = "model"
					}

					contents = append(contents, map[string]interface{}{
						"role": geminiRole,
						"parts": []interface{}{
							map[string]interface{}{
								"text": content,
							},
						},
					})
				}
			}
		}

		// Add system instruction if present
		if systemText != "" {
			geminiReq["systemInstruction"] = map[string]interface{}{
				"parts": []interface{}{
					map[string]interface{}{
						"text": systemText,
					},
				},
			}
		}

		geminiReq["contents"] = contents
	}

	// Add generation config
	generationConfig := map[string]interface{}{}

	if maxTokens, ok := openaiReq["max_tokens"]; ok {
		generationConfig["maxOutputTokens"] = maxTokens
	} else {
		generationConfig["maxOutputTokens"] = common.DefaultGeminiMaxTokens
	}

	if temp, ok := openaiReq["temperature"]; ok {
		generationConfig["temperature"] = temp
	}

	if topP, ok := openaiReq["top_p"]; ok {
		generationConfig["topP"] = topP
	}

	if len(generationConfig) > 0 {
		geminiReq["generationConfig"] = generationConfig
	}

	return geminiReq
}

func convertClaudeToGeminiRequest(claudeReq map[string]interface{}) map[string]interface{} {
	geminiReq := map[string]interface{}{
		"contents": []interface{}{},
	}

	// Add system instruction if present
	if system, ok := claudeReq["system"]; ok {
		systemText := extractText(system)
		if systemText != "" {
			geminiReq["systemInstruction"] = map[string]interface{}{
				"parts": []interface{}{
					map[string]interface{}{
						"text": systemText,
					},
				},
			}
		}
	}

	// Process messages
	if messages, ok := claudeReq["messages"].([]interface{}); ok {
		var contents []interface{}
		for _, msg := range messages {
			if msgMap, ok := msg.(map[string]interface{}); ok {
				role := getStringOrDefault(msgMap, "role", "user")
				if role == "assistant" {
					role = "model"
				}

				content := extractText(msgMap["content"])
				contents = append(contents, map[string]interface{}{
					"role": role,
					"parts": []interface{}{
						map[string]interface{}{
							"text": content,
						},
					},
				})
			}
		}
		geminiReq["contents"] = contents
	}

	// Add generation config
	generationConfig := map[string]interface{}{}
	generationConfig["maxOutputTokens"] = getIntOrDefault(claudeReq, "max_tokens", common.DefaultGeminiMaxTokens)
	generationConfig["temperature"] = getFloatOrDefault(claudeReq, "temperature", common.DefaultTemperature)
	generationConfig["topP"] = getFloatOrDefault(claudeReq, "top_p", common.DefaultTopP)

	if len(generationConfig) > 0 {
		geminiReq["generationConfig"] = generationConfig
	}

	return geminiReq
}

// Gemini Response Conversions (not commonly needed but included for completeness)

func convertOpenAIToGeminiResponse(openaiResp map[string]interface{}, model string) map[string]interface{} {
	geminiResp := map[string]interface{}{
		"candidates": []interface{}{},
	}

	// Extract content from choices
	if choices, ok := openaiResp["choices"].([]interface{}); ok && len(choices) > 0 {
		if choice, ok := choices[0].(map[string]interface{}); ok {
			var contentText string
			if message, ok := choice["message"].(map[string]interface{}); ok {
				contentText = getStringOrDefault(message, "content", "")
			}

			geminiResp["candidates"] = []interface{}{
				map[string]interface{}{
					"content": map[string]interface{}{
						"parts": []interface{}{
							map[string]interface{}{
								"text": contentText,
							},
						},
						"role": "model",
					},
					"finishReason": "STOP",
				},
			}
		}
	}

	// Add usage metadata
	if usage, ok := openaiResp["usage"].(map[string]interface{}); ok {
		geminiResp["usageMetadata"] = map[string]interface{}{
			"promptTokenCount":     getIntOrDefault(usage, "prompt_tokens", 0),
			"candidatesTokenCount": getIntOrDefault(usage, "completion_tokens", 0),
			"totalTokenCount":      getIntOrDefault(usage, "total_tokens", 0),
		}
	}

	return geminiResp
}

func convertClaudeToGeminiResponse(claudeResp map[string]interface{}, model string) map[string]interface{} {
	geminiResp := map[string]interface{}{
		"candidates": []interface{}{},
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

	geminiResp["candidates"] = []interface{}{
		map[string]interface{}{
			"content": map[string]interface{}{
				"parts": []interface{}{
					map[string]interface{}{
						"text": contentText,
					},
				},
				"role": "model",
			},
			"finishReason": "STOP",
		},
	}

	// Add usage metadata
	if usage, ok := claudeResp["usage"].(map[string]interface{}); ok {
		inputTokens := getIntOrDefault(usage, "input_tokens", 0)
		outputTokens := getIntOrDefault(usage, "output_tokens", 0)
		geminiResp["usageMetadata"] = map[string]interface{}{
			"promptTokenCount":     inputTokens,
			"candidatesTokenCount": outputTokens,
			"totalTokenCount":      inputTokens + outputTokens,
		}
	}

	return geminiResp
}

// Gemini Stream Chunk Conversions

func convertOpenAIToGeminiStreamChunk(openaiChunk map[string]interface{}, model string) map[string]interface{} {
	text := ""
	if choices, ok := openaiChunk["choices"].([]interface{}); ok && len(choices) > 0 {
		if choice, ok := choices[0].(map[string]interface{}); ok {
			if delta, ok := choice["delta"].(map[string]interface{}); ok {
				text = getStringOrDefault(delta, "content", "")
			}
		}
	}

	return map[string]interface{}{
		"candidates": []interface{}{
			map[string]interface{}{
				"content": map[string]interface{}{
					"parts": []interface{}{
						map[string]interface{}{
							"text": text,
						},
					},
					"role": "model",
				},
			},
		},
	}
}

func convertClaudeToGeminiStreamChunk(claudeChunk map[string]interface{}, model string) map[string]interface{} {
	text := ""
	if delta, ok := claudeChunk["delta"].(map[string]interface{}); ok {
		text = getStringOrDefault(delta, "text", "")
	} else if textVal, ok := claudeChunk["text"]; ok {
		text = extractText(textVal)
	}

	return map[string]interface{}{
		"candidates": []interface{}{
			map[string]interface{}{
				"content": map[string]interface{}{
					"parts": []interface{}{
						map[string]interface{}{
							"text": text,
						},
					},
					"role": "model",
				},
			},
		},
	}
}
