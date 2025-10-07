package server

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strings"
	"time"

	"github.com/justlovemaki/AIClient-2-API/internal/adapter"
	"github.com/justlovemaki/AIClient-2-API/internal/common"
	"github.com/justlovemaki/AIClient-2-API/internal/pool"
)

// Server represents the HTTP server
type Server struct {
	config      *common.Config
	poolManager *pool.ProviderPoolManager
	mux         *http.ServeMux
}

// NewServer creates a new HTTP server
func NewServer(config *common.Config, poolManager *pool.ProviderPoolManager) *Server {
	s := &Server{
		config:      config,
		poolManager: poolManager,
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
	s.mux.HandleFunc("/v1beta/models", s.handleModels)
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
	
	server := &http.Server{
		Addr:         addr,
		Handler:      s.corsMiddleware(s.authMiddleware(s.mux)),
		ReadTimeout:  30 * time.Second,
		WriteTimeout: 30 * time.Second,
		IdleTimeout:  120 * time.Second,
	}

	return server.ListenAndServe()
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

	// Check if streaming is requested
	stream, _ := requestBody["stream"].(bool)

	if stream {
		s.handleStreamingResponse(w, adapter, model, requestBody)
	} else {
		s.handleUnaryResponse(w, adapter, model, requestBody)
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
func (s *Server) handleUnaryResponse(w http.ResponseWriter, adapter adapter.ApiServiceAdapter, model string, requestBody map[string]interface{}) {
	response, err := adapter.GenerateContent(model, requestBody)
	if err != nil {
		s.handleError(w, err)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

// handleStreamingResponse handles streaming responses
func (s *Server) handleStreamingResponse(w http.ResponseWriter, adapter adapter.ApiServiceAdapter, model string, requestBody map[string]interface{}) {
	stream, err := adapter.GenerateContentStream(model, requestBody)
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

	// Stream responses
	for chunk := range stream {
		data, err := json.Marshal(chunk)
		if err != nil {
			log.Printf("[Server] Error marshaling chunk: %v", err)
			continue
		}

		fmt.Fprintf(w, "data: %s\n\n", data)
		flusher.Flush()
	}

	fmt.Fprintf(w, "data: [DONE]\n\n")
	flusher.Flush()
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

