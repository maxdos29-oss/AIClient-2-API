package adapter

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"time"

	"github.com/justlovemaki/AIClient-2-API/internal/common"
)

// GeminiAdapter implements the ApiServiceAdapter interface for Gemini API
type GeminiAdapter struct {
	config      *common.Config
	client      *http.Client
	accessToken string
	tokenExpiry time.Time
	apiEndpoint string
	initialized bool
}

// NewGeminiAdapter creates a new Gemini adapter
func NewGeminiAdapter(config *common.Config) (*GeminiAdapter, error) {
	adapter := &GeminiAdapter{
		config:      config,
		apiEndpoint: "https://generativelanguage.googleapis.com/v1beta",
		client:      &http.Client{Timeout: 30 * time.Second},
		initialized: false,
	}

	// Initialize authentication if credentials are provided
	if err := adapter.initializeAuth(); err != nil {
		// For now, continue even if auth fails - allows the adapter to be created
		fmt.Printf("[Gemini] Warning: Failed to initialize auth: %v\n", err)
	}

	adapter.initialized = true
	return adapter, nil
}

// initializeAuth initializes authentication
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

	// Parse credentials to extract access token
	var creds map[string]interface{}
	if err := json.Unmarshal(credsData, &creds); err != nil {
		return fmt.Errorf("failed to parse credentials: %w", err)
	}

	// Extract access token (simplified - actual OAuth flow is more complex)
	if token, ok := creds["access_token"].(string); ok && token != "" {
		g.accessToken = token
	}

	// Extract expiry if available
	if expiry, ok := creds["expiry"].(string); ok {
		if t, err := time.Parse(time.RFC3339, expiry); err == nil {
			g.tokenExpiry = t
		}
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
	if !g.tokenExpiry.IsZero() && time.Until(g.tokenExpiry) < 5*time.Minute {
		return g.initializeAuth()
	}
	return nil
}

// IsInitialized returns whether the adapter is initialized
func (g *GeminiAdapter) IsInitialized() bool {
	return g.initialized
}

