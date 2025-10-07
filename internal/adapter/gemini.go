package adapter

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"time"

	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
	"google.golang.org/api/option"
	"google.golang.org/api/generativeai/v1beta"

	"github.com/justlovemaki/AIClient-2-API/internal/common"
)

// GeminiAdapter implements the ApiServiceAdapter interface for Gemini API
type GeminiAdapter struct {
	config       *common.Config
	client       *http.Client
	oauthConfig  *oauth2.Config
	token        *oauth2.Token
	tokenExpiry  time.Time
	apiEndpoint  string
	initialized  bool
}

// NewGeminiAdapter creates a new Gemini adapter
func NewGeminiAdapter(config *common.Config) (*GeminiAdapter, error) {
	adapter := &GeminiAdapter{
		config:      config,
		apiEndpoint: "https://generativelanguage.googleapis.com/v1beta",
		initialized: false,
	}

	// Initialize OAuth if credentials are provided
	if err := adapter.initializeAuth(); err != nil {
		return nil, fmt.Errorf("failed to initialize auth: %w", err)
	}

	adapter.initialized = true
	return adapter, nil
}

// initializeAuth initializes OAuth authentication
func (g *GeminiAdapter) initializeAuth() error {
	var credsData []byte
	var err error

	// Try to load credentials from various sources
	if g.config.GeminiOAuthCredsBase64 != "" {
		// Load from base64 string
		credsData, err = common.DecodeBase64(g.config.GeminiOAuthCredsBase64)
		if err != nil {
			return fmt.Errorf("failed to decode base64 credentials: %w", err)
		}
	} else if g.config.GeminiOAuthCredsFilePath != "" {
		// Load from file path
		filePath := common.ExpandHomePath(g.config.GeminiOAuthCredsFilePath)
		credsData, err = os.ReadFile(filePath)
		if err != nil {
			return fmt.Errorf("failed to read credentials file: %w", err)
		}
	} else {
		// Try default path
		home, err := os.UserHomeDir()
		if err == nil {
			defaultPath := filepath.Join(home, ".gemini", "oauth_creds.json")
			if _, err := os.Stat(defaultPath); err == nil {
				credsData, _ = os.ReadFile(defaultPath)
			}
		}
	}

	if len(credsData) == 0 {
		return fmt.Errorf("no Gemini OAuth credentials found")
	}

	// Parse credentials
	var creds map[string]interface{}
	if err := json.Unmarshal(credsData, &creds); err != nil {
		return fmt.Errorf("failed to parse credentials: %w", err)
	}

	// Create OAuth config
	g.oauthConfig, err = google.ConfigFromJSON(credsData, generativeai.CloudPlatformScope)
	if err != nil {
		return fmt.Errorf("failed to create OAuth config: %w", err)
	}

	// Load or refresh token
	if err := g.loadOrRefreshToken(); err != nil {
		return fmt.Errorf("failed to load token: %w", err)
	}

	// Create HTTP client with OAuth token
	ctx := context.Background()
	g.client = g.oauthConfig.Client(ctx, g.token)

	return nil
}

// loadOrRefreshToken loads an existing token or refreshes it if expired
func (g *GeminiAdapter) loadOrRefreshToken() error {
	// Try to load existing token
	if g.token != nil && g.token.Valid() {
		return nil
	}

	// For simplicity, we'll create a new token
	// In production, you'd want to persist and reload tokens
	ctx := context.Background()
	
	// This is a simplified version - actual implementation would need
	// proper OAuth flow with token storage
	if g.token != nil && g.token.RefreshToken != "" {
		tokenSource := g.oauthConfig.TokenSource(ctx, g.token)
		newToken, err := tokenSource.Token()
		if err != nil {
			return fmt.Errorf("failed to refresh token: %w", err)
		}
		g.token = newToken
		g.tokenExpiry = newToken.Expiry
	}

	return nil
}

// GenerateContent generates content using Gemini API
func (g *GeminiAdapter) GenerateContent(model string, requestBody map[string]interface{}) (map[string]interface{}, error) {
	if !g.initialized {
		return nil, fmt.Errorf("gemini adapter not initialized")
	}

	url := fmt.Sprintf("%s/models/%s:generateContent", g.apiEndpoint, model)
	
	jsonData, err := json.Marshal(requestBody)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")

	resp, err := g.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to make request: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("API request failed with status %d: %s", resp.StatusCode, string(body))
	}

	var result map[string]interface{}
	if err := json.Unmarshal(body, &result); err != nil {
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}

	return result, nil
}

// GenerateContentStream generates content using Gemini API in streaming mode
func (g *GeminiAdapter) GenerateContentStream(model string, requestBody map[string]interface{}) (<-chan interface{}, error) {
	if !g.initialized {
		return nil, fmt.Errorf("gemini adapter not initialized")
	}

	url := fmt.Sprintf("%s/models/%s:streamGenerateContent", g.apiEndpoint, model)
	
	jsonData, err := json.Marshal(requestBody)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")

	resp, err := g.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to make request: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		resp.Body.Close()
		return nil, fmt.Errorf("API request failed with status %d: %s", resp.StatusCode, string(body))
	}

	ch := make(chan interface{}, 10)

	go func() {
		defer close(ch)
		defer resp.Body.Close()

		reader := bufio.NewReader(resp.Body)
		for {
			line, err := reader.ReadBytes('\n')
			if err != nil {
				if err != io.EOF {
					ch <- map[string]interface{}{
						"error": err.Error(),
					}
				}
				break
			}

			line = bytes.TrimSpace(line)
			if len(line) == 0 {
				continue
			}

			var chunk map[string]interface{}
			if err := json.Unmarshal(line, &chunk); err != nil {
				continue
			}

			ch <- chunk
		}
	}()

	return ch, nil
}

// ListModels lists available Gemini models
func (g *GeminiAdapter) ListModels() (map[string]interface{}, error) {
	if !g.initialized {
		return nil, fmt.Errorf("gemini adapter not initialized")
	}

	// TODO: Implement actual Gemini list models API call
	return map[string]interface{}{
		"models": []map[string]interface{}{
			{
				"name": "models/gemini-2.5-flash",
			},
			{
				"name": "models/gemini-2.5-pro",
			},
		},
	}, nil
}

// RefreshToken refreshes the Gemini OAuth token if needed
func (g *GeminiAdapter) RefreshToken() error {
	if g.token != nil && time.Until(g.tokenExpiry) < 5*time.Minute {
		return g.loadOrRefreshToken()
	}
	return nil
}

// IsInitialized returns whether the adapter is initialized
func (g *GeminiAdapter) IsInitialized() bool {
	return g.initialized
}

