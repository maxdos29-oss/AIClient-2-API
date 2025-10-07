package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"os/signal"
	"strconv"
	"strings"
	"syscall"
	"time"

	"github.com/justlovemaki/AIClient-2-API/internal/adapter"
	"github.com/justlovemaki/AIClient-2-API/internal/common"
	"github.com/justlovemaki/AIClient-2-API/internal/pool"
	"github.com/justlovemaki/AIClient-2-API/internal/server"
)

func main() {
	// Parse command line flags
	config := parseFlags()

	// Initialize configuration
	if err := common.InitializeConfig(config); err != nil {
		log.Fatalf("[Config Error] Failed to initialize config: %v", err)
	}

	// Print configuration
	printConfig(config)

	// Initialize provider pool manager
	var poolManager *pool.ProviderPoolManager
	if config.ProviderPoolsFilePath != "" {
		var err error
		poolManager, err = pool.NewProviderPoolManager(config.ProviderPoolsFilePath)
		if err != nil {
			log.Printf("[Warning] Failed to initialize provider pool manager: %v", err)
		} else {
			log.Println("[Initialization] ProviderPoolManager initialized with configured pools.")
			// Perform initial health checks
			go poolManager.PerformHealthChecks()
		}
	}

	// Initialize service adapters
	if err := adapter.InitializeAdapters(config); err != nil {
		log.Printf("[Warning] Failed to initialize some adapters: %v", err)
	}

	// Start token refresh cron if enabled
	if config.CronRefreshToken {
		go startTokenRefreshCron(config, poolManager)
	}

	// Start HTTP server
	srv := server.NewServer(config, poolManager)
	log.Printf("\n[Server] Starting Unified API Server on http://%s:%d\n", config.Host, config.ServerPort)
	
	// Setup graceful shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Start server in goroutine
	serverErr := make(chan error, 1)
	go func() {
		if err := srv.Start(); err != nil {
			serverErr <- err
		}
	}()

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)

	select {
	case err := <-serverErr:
		log.Fatalf("[Server] Failed to start server: %v", err)
	case sig := <-sigChan:
		log.Printf("\n[Server] Received signal %v, initiating graceful shutdown...", sig)
		
		// Create shutdown context with timeout
		shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer shutdownCancel()

		// Shutdown server
		if err := srv.Shutdown(shutdownCtx); err != nil {
			log.Printf("[Server] Error during shutdown: %v", err)
		} else {
			log.Println("[Server] Graceful shutdown completed")
		}
	}
}

func parseFlags() *common.Config {
	config := &common.Config{}

	// Server configuration
	flag.StringVar(&config.Host, "host", "localhost", "Server listening address")
	flag.IntVar(&config.ServerPort, "port", 3000, "Server listening port")
	flag.StringVar(&config.RequiredAPIKey, "api-key", "123456", "Required API key for authentication")

	// Model provider configuration
	modelProvider := flag.String("model-provider", "gemini-cli-oauth", "AI model provider")

	// OpenAI configuration
	flag.StringVar(&config.OpenAIAPIKey, "openai-api-key", "", "OpenAI API key")
	flag.StringVar(&config.OpenAIBaseURL, "openai-base-url", "", "OpenAI API base URL")

	// Claude configuration
	flag.StringVar(&config.ClaudeAPIKey, "claude-api-key", "", "Claude API key")
	flag.StringVar(&config.ClaudeBaseURL, "claude-base-url", "", "Claude API base URL")

	// Gemini OAuth configuration
	flag.StringVar(&config.GeminiOAuthCredsBase64, "gemini-oauth-creds-base64", "", "Gemini OAuth credentials as Base64 string")
	flag.StringVar(&config.GeminiOAuthCredsFilePath, "gemini-oauth-creds-file", "", "Gemini OAuth credentials file path")
	flag.StringVar(&config.ProjectID, "project-id", "", "Google Cloud Project ID")

	// Kiro OAuth configuration
	flag.StringVar(&config.KiroOAuthCredsBase64, "kiro-oauth-creds-base64", "", "Kiro OAuth credentials as Base64 string")
	flag.StringVar(&config.KiroOAuthCredsFilePath, "kiro-oauth-creds-file", "", "Kiro OAuth credentials file path")

	// Qwen OAuth configuration
	flag.StringVar(&config.QwenOAuthCredsFilePath, "qwen-oauth-creds-file", "", "Qwen OAuth credentials file path")

	// System prompt configuration
	flag.StringVar(&config.SystemPromptFilePath, "system-prompt-file", "input_system_prompt.txt", "System prompt file path")
	flag.StringVar(&config.SystemPromptMode, "system-prompt-mode", "overwrite", "System prompt mode: overwrite or append")

	// Logging configuration
	flag.StringVar(&config.PromptLogMode, "log-prompts", "none", "Prompt logging mode: console, file, or none")
	flag.StringVar(&config.PromptLogBaseName, "prompt-log-base-name", "prompt_log", "Base name for prompt log files")

	// Retry configuration
	flag.IntVar(&config.RequestMaxRetries, "request-max-retries", 3, "Max retries for API requests")
	flag.IntVar(&config.RequestBaseDelay, "request-base-delay", 1000, "Base delay in milliseconds between retries")

	// Cron configuration
	flag.IntVar(&config.CronNearMinutes, "cron-near-minutes", 15, "Interval for OAuth token refresh task in minutes")
	flag.BoolVar(&config.CronRefreshToken, "cron-refresh-token", true, "Enable automatic OAuth token refresh")

	// Provider pools configuration
	flag.StringVar(&config.ProviderPoolsFilePath, "provider-pools-file", "", "Provider pools configuration file path")

	flag.Parse()

	// Parse model provider
	config.ModelProvider = *modelProvider
	providers := strings.Split(*modelProvider, ",")
	config.DefaultModelProviders = make([]string, 0, len(providers))
	for _, p := range providers {
		p = strings.TrimSpace(p)
		if p != "" {
			config.DefaultModelProviders = append(config.DefaultModelProviders, p)
		}
	}

	return config
}

func printConfig(config *common.Config) {
	log.Println("--- Unified API Server Configuration ---")
	log.Printf("  Primary Model Provider: %s", config.ModelProvider)
	if len(config.DefaultModelProviders) > 1 {
		log.Printf("  Additional Model Providers: %v", config.DefaultModelProviders[1:])
	}
	
	// Print provider-specific details
	for _, provider := range config.DefaultModelProviders {
		printProviderDetails(provider, config)
	}
	
	log.Printf("  System Prompt File: %s", config.SystemPromptFilePath)
	log.Printf("  System Prompt Mode: %s", config.SystemPromptMode)
	log.Printf("  Host: %s", config.Host)
	log.Printf("  Port: %d", config.ServerPort)
	log.Printf("  Required API Key: %s", config.RequiredAPIKey)
	log.Printf("  Prompt Logging: %s", config.PromptLogMode)
	
	if config.CronRefreshToken {
		log.Printf("  Cron Near Minutes: %d", config.CronNearMinutes)
		log.Printf("  Cron Refresh Token: %t", config.CronRefreshToken)
	}
	
	log.Println("------------------------------------------")
}

func printProviderDetails(provider string, config *common.Config) {
	switch provider {
	case common.ModelProviderOpenAICustom:
		apiKey := "Not Set"
		if config.OpenAIAPIKey != "" {
			apiKey = "******"
		}
		log.Printf("  [%s] API Key: %s", provider, apiKey)
		baseURL := "Default"
		if config.OpenAIBaseURL != "" {
			baseURL = config.OpenAIBaseURL
		}
		log.Printf("  [%s] Base URL: %s", provider, baseURL)
		
	case common.ModelProviderClaudeCustom:
		apiKey := "Not Set"
		if config.ClaudeAPIKey != "" {
			apiKey = "******"
		}
		log.Printf("  [%s] API Key: %s", provider, apiKey)
		baseURL := "Default"
		if config.ClaudeBaseURL != "" {
			baseURL = config.ClaudeBaseURL
		}
		log.Printf("  [%s] Base URL: %s", provider, baseURL)
		
	case common.ModelProviderGeminiCLI:
		if config.GeminiOAuthCredsFilePath != "" {
			log.Printf("  [%s] OAuth Creds File Path: %s", provider, config.GeminiOAuthCredsFilePath)
		} else if config.GeminiOAuthCredsBase64 != "" {
			log.Printf("  [%s] OAuth Creds Source: Provided via Base64 string", provider)
		} else {
			log.Printf("  [%s] OAuth Creds: Default discovery", provider)
		}
		projectID := "Auto-discovered"
		if config.ProjectID != "" {
			projectID = config.ProjectID
		}
		log.Printf("  [%s] Project ID: %s", provider, projectID)
		
	case common.ModelProviderKiroAPI:
		if config.KiroOAuthCredsFilePath != "" {
			log.Printf("  [%s] OAuth Creds File Path: %s", provider, config.KiroOAuthCredsFilePath)
		} else if config.KiroOAuthCredsBase64 != "" {
			log.Printf("  [%s] OAuth Creds Source: Provided via Base64 string", provider)
		} else {
			log.Printf("  [%s] OAuth Creds: Default", provider)
		}
		
	case common.ModelProviderQwenAPI:
		credsPath := "Default"
		if config.QwenOAuthCredsFilePath != "" {
			credsPath = config.QwenOAuthCredsFilePath
		}
		log.Printf("  [%s] OAuth Creds File Path: %s", provider, credsPath)
	}
}

func startTokenRefreshCron(config *common.Config, poolManager *pool.ProviderPoolManager) {
	ticker := time.NewTicker(time.Duration(config.CronNearMinutes) * time.Minute)
	defer ticker.Stop()

	for range ticker.C {
		log.Printf("[Heartbeat] Server is running. Current time: %s", time.Now().Format(time.RFC3339))
		
		// Perform health checks if pool manager exists
		if poolManager != nil {
			poolManager.PerformHealthChecks()
		}
		
		// Refresh tokens for all adapters
		for providerType, adapterInstance := range adapter.GetAllAdapters() {
			if err := adapterInstance.RefreshToken(); err != nil {
				log.Printf("[Token Refresh Error] Failed to refresh token for %s: %v", providerType, err)
			} else {
				log.Printf("[Token Refresh] Successfully refreshed token for %s", providerType)
			}
		}
	}
}

// Helper function to get environment variable with default value
func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

// Helper function to get environment variable as int with default value
func getEnvInt(key string, defaultValue int) int {
	if value := os.Getenv(key); value != "" {
		if intValue, err := strconv.Atoi(value); err == nil {
			return intValue
		}
	}
	return defaultValue
}

// Helper function to get environment variable as bool with default value
func getEnvBool(key string, defaultValue bool) bool {
	if value := os.Getenv(key); value != "" {
		if boolValue, err := strconv.ParseBool(value); err == nil {
			return boolValue
		}
	}
	return defaultValue
}

