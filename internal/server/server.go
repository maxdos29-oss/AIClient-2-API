package server

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"regexp"
	"strings"
	"time"

	"github.com/justlovemaki/AIClient-2-API/internal/adapter"
	"github.com/justlovemaki/AIClient-2-API/internal/common"
	"github.com/justlovemaki/AIClient-2-API/internal/converter"
	"github.com/justlovemaki/AIClient-2-API/internal/pool"
)

// Server represents the HTTP server
type Server struct {
	config      *common.Config
	poolManager *pool.ProviderPoolManager
	converter   *converter.Converter
	mux         *http.ServeMux
	httpServer  *http.Server
}

// NewServer creates a new HTTP server
func NewServer(config *common.Config, poolManager *pool.ProviderPoolManager) *Server {
	s := &Server{
		config:      config,
		poolManager: poolManager,
		converter:   converter.NewConverter(),
		mux:         http.NewServeMux(),
	}

	s.setupRoutes()
	return s
}

// setupRoutes sets up all HTTP routes
func (s *Server) setupRoutes() {
	s.mux.HandleFunc("/health", s.handleHealth)
	s.mux.HandleFunc("/v1/models", s.handleModels)
	s.mux.HandleFunc("/v1/chat/completions", s.handleChatCompletions)
	s.mux.HandleFunc("/v1/messages", s.handleMessages)
	s.mux.HandleFunc("/v1beta/models", s.handleModelsOrGenerate)
	s.mux.HandleFunc("/", s.handleRoot)
}

// Start starts the HTTP server
func (s *Server) Start() error {
	addr := fmt.Sprintf("%s:%d", s.config.Host, s.config.ServerPort)
	
	log.Println("Supports multiple API formats:")
	log.Println("  • OpenAI-compatible: /v1/chat/completions, /v1/models")
	log.Println("  • Gemini-compatible: /v1beta/models, /v1beta/models/{model}:generateContent")
	log.Println("  • Claude-compatible: /v1/messages")
	log.Println("  • Health check: /health")
	
	s.httpServer = &http.Server{
		Addr:         addr,
		Handler:      s.corsMiddleware(s.authMiddleware(s.mux)),
		ReadTimeout:  30 * time.Second,
		WriteTimeout: 30 * time.Second,
		IdleTimeout:  120 * time.Second,
	}

	return s.httpServer.ListenAndServe()
}

// Shutdown gracefully shuts down the server
func (s *Server) Shutdown(ctx context.Context) error {
	if s.httpServer != nil {
		return s.httpServer.Shutdown(ctx)
	}
	return nil
}

// corsMiddleware adds CORS headers
func (s *Server) corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization, x-goog-api-key, x-api-key, Model-Provider")

		if r.Method == "OPTIONS" {
			w.WriteHeader(http.StatusNoContent)
			return
		}

		next.ServeHTTP(w, r)
	})
}

// authMiddleware checks API key authentication
func (s *Server) authMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Skip auth for health endpoint
		if r.URL.Path == "/health" {
			next.ServeHTTP(w, r)
			return
		}

		if !common.IsAuthorized(r, s.config.RequiredAPIKey) {
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(http.StatusUnauthorized)
			json.NewEncoder(w).Encode(map[string]interface{}{
				"error": map[string]string{
					"message": "Unauthorized: API key is invalid or missing.",
				},
			})
			return
		}

		next.ServeHTTP(w, r)
	})
}

// handleHealth handles health check requests
func (s *Server) handleHealth(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status":    "healthy",
		"timestamp": time.Now().Format(time.RFC3339),
		"provider":  s.config.ModelProvider,
	})
}

// handleModels handles model list requests
func (s *Server) handleModels(w http.ResponseWriter, r *http.Request) {
	log.Printf("[Server] Received request: %s %s", r.Method, r.URL.Path)

	// Get adapter
	currentConfig := s.getRequestConfig(r)
	adapter, err := s.getAdapter(currentConfig)
	if err != nil {
		s.handleError(w, err)
		return
	}

	// List models
	models, err := adapter.ListModels()
	if err != nil {
		s.handleError(w, err)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(models)
}

// handleChatCompletions handles OpenAI-style chat completion requests
func (s *Server) handleChatCompletions(w http.ResponseWriter, r *http.Request) {
	if r.Method != "POST" {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	log.Printf("[Server] Received request: %s %s", r.Method, r.URL.Path)

	// Get request body
	requestBody, err := common.GetRequestBody(r)
	if err != nil {
		s.handleError(w, err)
		return
	}

	// Get adapter
	currentConfig := s.getRequestConfig(r)
	adapter, err := s.getAdapter(currentConfig)
	if err != nil {
		s.handleError(w, err)
		return
	}

	// Extract model
	model, ok := requestBody["model"].(string)
	if !ok {
		s.handleError(w, fmt.Errorf("model field is required"))
		return
	}

	// Check if streaming is requested
	stream, _ := requestBody["stream"].(bool)

	if stream {
		s.handleStreamingResponse(w, adapter, model, requestBody)
	} else {
		s.handleUnaryResponse(w, adapter, model, requestBody)
	}
}

// handleMessages handles Claude-style message requests
func (s *Server) handleMessages(w http.ResponseWriter, r *http.Request) {
	if r.Method != "POST" {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	log.Printf("[Server] Received request: %s %s", r.Method, r.URL.Path)

	// Get request body
	requestBody, err := common.GetRequestBody(r)
	if err != nil {
		s.handleError(w, err)
		return
	}

	// Get adapter
	currentConfig := s.getRequestConfig(r)
	adapter, err := s.getAdapter(currentConfig)
	if err != nil {
		s.handleError(w, err)
		return
	}

	// Extract model
	model, ok := requestBody["model"].(string)
	if !ok {
		s.handleError(w, fmt.Errorf("model field is required"))
		return
	}

	// Determine protocols
	fromProtocol := common.ModelProtocolClaude // Client uses Claude format
	toProtocol := common.GetProtocolPrefix(currentConfig.ModelProvider)

	// Check if streaming is requested
	stream, _ := requestBody["stream"].(bool)

	if stream {
		s.handleStreamingResponse(w, adapter, model, requestBody, fromProtocol, toProtocol)
	} else {
		s.handleUnaryResponse(w, adapter, model, requestBody, fromProtocol, toProtocol)
	}
}

// handleModelsOrGenerate handles Gemini models list or generateContent
func (s *Server) handleModelsOrGenerate(w http.ResponseWriter, r *http.Request) {
	// Check if it's a model list request or generateContent
	geminiPattern := regexp.MustCompile(`/v1beta/models/([^/]+):(generateContent|streamGenerateContent)`)
	if matches := geminiPattern.FindStringSubmatch(r.URL.Path); len(matches) > 0 {
		// It's a generateContent request
		s.handleGeminiGenerate(w, r, matches[1], matches[2])
	} else {
		// It's a models list request
		s.handleModels(w, r)
	}
}

// handleGeminiGenerate handles Gemini generateContent requests
func (s *Server) handleGeminiGenerate(w http.ResponseWriter, r *http.Request, model, action string) {
	if r.Method != "POST" {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	log.Printf("[Server] Received Gemini request: %s %s", r.Method, r.URL.Path)

	// Get request body
	requestBody, err := common.GetRequestBody(r)
	if err != nil {
		s.handleError(w, err)
		return
	}

	// Get adapter
	currentConfig := s.getRequestConfig(r)
	adapter, err := s.getAdapter(currentConfig)
	if err != nil {
		s.handleError(w, err)
		return
	}

	// Determine protocols
	fromProtocol := common.ModelProtocolGemini // Client uses Gemini format
	toProtocol := common.GetProtocolPrefix(currentConfig.ModelProvider)

	// Check if streaming
	isStream := action == "streamGenerateContent"

	if isStream {
		s.handleStreamingResponse(w, adapter, model, requestBody, fromProtocol, toProtocol)
	} else {
		s.handleUnaryResponse(w, adapter, model, requestBody, fromProtocol, toProtocol)
	}
}

// handleRoot handles all other requests
func (s *Server) handleRoot(w http.ResponseWriter, r *http.Request) {
	log.Printf("[Server] Received request: %s %s", r.Method, r.URL.Path)

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusNotFound)
	json.NewEncoder(w).Encode(map[string]interface{}{
		"error": map[string]string{
			"message": "Not Found",
		},
	})
}

// handleUnaryResponse handles non-streaming responses
func (s *Server) handleUnaryResponse(w http.ResponseWriter, adp adapter.ApiServiceAdapter, model string, requestBody map[string]interface{}, fromProtocol, toProtocol string) {
	// Apply system prompt if configured
	requestBody = s.applySystemPrompt(requestBody, toProtocol)
	
	// Log request if enabled
	s.logRequest(requestBody, toProtocol)
	
	// Convert request if needed
	if fromProtocol != toProtocol {
		convertedReq, err := s.converter.ConvertRequest(requestBody, fromProtocol, toProtocol)
		if err != nil {
			log.Printf("[Server] Request conversion failed: %v", err)
		} else {
			requestBody = convertedReq
		}
	}

	// Call adapter
	response, err := adp.GenerateContent(model, requestBody)
	if err != nil {
		s.handleError(w, err)
		return
	}

	// Log response if enabled
	s.logResponse(response, toProtocol)

	// Convert response if needed
	if fromProtocol != toProtocol {
		convertedResp, err := s.converter.ConvertResponse(response, toProtocol, fromProtocol, model)
		if err != nil {
			log.Printf("[Server] Response conversion failed: %v", err)
		} else {
			response = convertedResp
		}
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

// handleStreamingResponse handles streaming responses
func (s *Server) handleStreamingResponse(w http.ResponseWriter, adp adapter.ApiServiceAdapter, model string, requestBody map[string]interface{}, fromProtocol, toProtocol string) {
	// Apply system prompt if configured
	requestBody = s.applySystemPrompt(requestBody, toProtocol)
	
	// Log request if enabled
	s.logRequest(requestBody, toProtocol)
	
	// Convert request if needed
	if fromProtocol != toProtocol {
		convertedReq, err := s.converter.ConvertRequest(requestBody, fromProtocol, toProtocol)
		if err != nil {
			log.Printf("[Server] Request conversion failed: %v", err)
		} else {
			requestBody = convertedReq
		}
	}

	stream, err := adp.GenerateContentStream(model, requestBody)
	if err != nil {
		s.handleError(w, err)
		return
	}

	// Set headers for SSE
	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")

	flusher, ok := w.(http.Flusher)
	if !ok {
		s.handleError(w, fmt.Errorf("streaming not supported"))
		return
	}

	var fullResponseText strings.Builder

	// Stream responses
	for chunk := range stream {
		// Convert chunk if needed
		convertedChunk := chunk
		if fromProtocol != toProtocol {
			converted, err := s.converter.ConvertStreamChunk(chunk, toProtocol, fromProtocol, model)
			if err != nil {
				log.Printf("[Server] Chunk conversion failed: %v", err)
			} else {
				convertedChunk = converted
			}
		}

		// Extract text for logging
		if chunkMap, ok := convertedChunk.(map[string]interface{}); ok {
			if choices, ok := chunkMap["choices"].([]interface{}); ok && len(choices) > 0 {
				if choice, ok := choices[0].(map[string]interface{}); ok {
					if delta, ok := choice["delta"].(map[string]interface{}); ok {
						if content, ok := delta["content"].(string); ok {
							fullResponseText.WriteString(content)
						}
					}
				}
			}
		}

		data, err := json.Marshal(convertedChunk)
		if err != nil {
			log.Printf("[Server] Error marshaling chunk: %v", err)
			continue
		}

		// Add event prefix for Claude SSE format
		if fromProtocol == common.ModelProtocolClaude {
			if chunkMap, ok := convertedChunk.(map[string]interface{}); ok {
				if eventType, ok := chunkMap["type"].(string); ok {
					fmt.Fprintf(w, "event: %s\n", eventType)
				}
			}
		}

		fmt.Fprintf(w, "data: %s\n\n", data)
		flusher.Flush()
	}

	// Send final chunk for OpenAI format
	if fromProtocol == common.ModelProtocolOpenAI {
		fmt.Fprintf(w, "data: [DONE]\n\n")
		flusher.Flush()
	}

	// Log response if enabled
	s.logResponseText(fullResponseText.String())
}

// getRequestConfig creates a config for the current request
func (s *Server) getRequestConfig(r *http.Request) *common.Config {
	config := *s.config // Copy config

	// Check for provider override in header
	if provider := r.Header.Get("Model-Provider"); provider != "" {
		config.ModelProvider = provider
		log.Printf("[Config] MODEL_PROVIDER overridden by header to: %s", provider)
	}

	// Check for provider override in path
	pathSegments := strings.Split(strings.Trim(r.URL.Path, "/"), "/")
	if len(pathSegments) > 0 {
		firstSegment := pathSegments[0]
		for _, validProvider := range common.AllModelProviders() {
			if firstSegment == validProvider {
				config.ModelProvider = validProvider
				log.Printf("[Config] MODEL_PROVIDER overridden by path to: %s", validProvider)
				break
			}
		}
	}

	return &config
}

// getAdapter gets an adapter for the given config
func (s *Server) getAdapter(config *common.Config) (adapter.ApiServiceAdapter, error) {
	// Check if we should use pool manager
	if s.poolManager != nil {
		if poolConfig := s.poolManager.SelectProvider(config.ModelProvider); poolConfig != nil {
			// Merge pool config with request config
			if uuid, ok := poolConfig["uuid"].(string); ok {
				config.UUID = uuid
			}
			// TODO: Merge other fields as needed
		}
	}

	return adapter.GetAdapter(config)
}

// handleError handles errors and sends appropriate responses
func (s *Server) handleError(w http.ResponseWriter, err error) {
	log.Printf("[Server] Error: %v", err)

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusInternalServerError)
	json.NewEncoder(w).Encode(map[string]interface{}{
		"error": map[string]string{
			"message": err.Error(),
		},
	})
}

// applySystemPrompt applies system prompt from config to request body
func (s *Server) applySystemPrompt(requestBody map[string]interface{}, protocol string) map[string]interface{} {
	if s.config.SystemPromptContent == "" {
		return requestBody
	}

	systemText := s.config.SystemPromptContent
	mode := s.config.SystemPromptMode

	switch protocol {
	case common.ModelProtocolOpenAI:
		messages, ok := requestBody["messages"].([]interface{})
		if !ok {
			return requestBody
		}

		// Find existing system message
		hasSystem := false
		for i, msg := range messages {
			if msgMap, ok := msg.(map[string]interface{}); ok {
				if msgMap["role"] == "system" {
					if mode == "append" {
						// Append to existing system message
						existing := msgMap["content"].(string)
						msgMap["content"] = existing + "\n\n" + systemText
						messages[i] = msgMap
					} else {
						// Override existing system message
						msgMap["content"] = systemText
						messages[i] = msgMap
					}
					hasSystem = true
					break
				}
			}
		}

		// Add new system message if none exists
		if !hasSystem {
			systemMsg := map[string]interface{}{
				"role":    "system",
				"content": systemText,
			}
			messages = append([]interface{}{systemMsg}, messages...)
			requestBody["messages"] = messages
		}

	case common.ModelProtocolGemini:
		if mode == "override" || requestBody["systemInstruction"] == nil {
			requestBody["systemInstruction"] = map[string]interface{}{
				"parts": []interface{}{
					map[string]interface{}{"text": systemText},
				},
			}
		} else if mode == "append" {
			if sysInstr, ok := requestBody["systemInstruction"].(map[string]interface{}); ok {
				if parts, ok := sysInstr["parts"].([]interface{}); ok {
					parts = append(parts, map[string]interface{}{"text": systemText})
					sysInstr["parts"] = parts
				}
			}
		}

	case common.ModelProtocolClaude:
		if mode == "override" || requestBody["system"] == nil {
			requestBody["system"] = systemText
		} else if mode == "append" {
			if existing, ok := requestBody["system"].(string); ok {
				requestBody["system"] = existing + "\n\n" + systemText
			}
		}
	}

	return requestBody
}

// logRequest logs the request if logging is enabled
func (s *Server) logRequest(requestBody map[string]interface{}, protocol string) {
	if s.config.PromptLogMode == "none" {
		return
	}

	// Extract prompt text based on protocol
	promptText := extractPromptText(requestBody, protocol)
	if promptText == "" {
		return
	}

	_ = common.LogConversation("input", promptText, s.config.PromptLogMode, s.config.PromptLogFilename)
}

// logResponse logs the response if logging is enabled
func (s *Server) logResponse(response map[string]interface{}, protocol string) {
	if s.config.PromptLogMode == "none" {
		return
	}

	// Extract response text based on protocol
	responseText := extractResponseText(response, protocol)
	if responseText == "" {
		return
	}

	_ = common.LogConversation("output", responseText, s.config.PromptLogMode, s.config.PromptLogFilename)
}

// logResponseText logs response text directly
func (s *Server) logResponseText(text string) {
	if s.config.PromptLogMode == "none" || text == "" {
		return
	}

	_ = common.LogConversation("output", text, s.config.PromptLogMode, s.config.PromptLogFilename)
}

// extractPromptText extracts prompt text from request based on protocol
func extractPromptText(requestBody map[string]interface{}, protocol string) string {
	switch protocol {
	case common.ModelProtocolOpenAI:
		if messages, ok := requestBody["messages"].([]interface{}); ok {
			var texts []string
			for _, msg := range messages {
				if msgMap, ok := msg.(map[string]interface{}); ok {
					if content, ok := msgMap["content"].(string); ok {
						texts = append(texts, fmt.Sprintf("[%s]: %s", msgMap["role"], content))
					}
				}
			}
			return strings.Join(texts, "\n")
		}

	case common.ModelProtocolGemini:
		if contents, ok := requestBody["contents"].([]interface{}); ok {
			var texts []string
			for _, content := range contents {
				if contentMap, ok := content.(map[string]interface{}); ok {
					if parts, ok := contentMap["parts"].([]interface{}); ok {
						for _, part := range parts {
							if partMap, ok := part.(map[string]interface{}); ok {
								if text, ok := partMap["text"].(string); ok {
									texts = append(texts, text)
								}
							}
						}
					}
				}
			}
			return strings.Join(texts, "\n")
		}

	case common.ModelProtocolClaude:
		if messages, ok := requestBody["messages"].([]interface{}); ok {
			var texts []string
			for _, msg := range messages {
				if msgMap, ok := msg.(map[string]interface{}); ok {
					role := msgMap["role"]
					if content, ok := msgMap["content"].([]interface{}); ok {
						for _, block := range content {
							if blockMap, ok := block.(map[string]interface{}); ok {
								if text, ok := blockMap["text"].(string); ok {
									texts = append(texts, fmt.Sprintf("[%s]: %s", role, text))
								}
							}
						}
					}
				}
			}
			return strings.Join(texts, "\n")
		}
	}

	return ""
}

// extractResponseText extracts response text based on protocol
func extractResponseText(response map[string]interface{}, protocol string) string {
	switch protocol {
	case common.ModelProtocolOpenAI:
		if choices, ok := response["choices"].([]interface{}); ok && len(choices) > 0 {
			if choice, ok := choices[0].(map[string]interface{}); ok {
				if message, ok := choice["message"].(map[string]interface{}); ok {
					if content, ok := message["content"].(string); ok {
						return content
					}
				}
			}
		}

	case common.ModelProtocolGemini:
		if candidates, ok := response["candidates"].([]interface{}); ok && len(candidates) > 0 {
			if candidate, ok := candidates[0].(map[string]interface{}); ok {
				if content, ok := candidate["content"].(map[string]interface{}); ok {
					if parts, ok := content["parts"].([]interface{}); ok {
						var texts []string
						for _, part := range parts {
							if partMap, ok := part.(map[string]interface{}); ok {
								if text, ok := partMap["text"].(string); ok {
									texts = append(texts, text)
								}
							}
						}
						return strings.Join(texts, "\n")
					}
				}
			}
		}

	case common.ModelProtocolClaude:
		if content, ok := response["content"].([]interface{}); ok {
			var texts []string
			for _, block := range content {
				if blockMap, ok := block.(map[string]interface{}); ok {
					if text, ok := blockMap["text"].(string); ok {
						texts = append(texts, text)
					}
				}
			}
			return strings.Join(texts, "\n")
		}
	}

	return ""
}

