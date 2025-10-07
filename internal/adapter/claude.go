package adapter

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	"github.com/justlovemaki/AIClient-2-API/internal/common"
)

// ClaudeAdapter implements the ApiServiceAdapter interface for Claude API
type ClaudeAdapter struct {
	config      *common.Config
	client      *http.Client
	initialized bool
}

// NewClaudeAdapter creates a new Claude adapter
func NewClaudeAdapter(config *common.Config) (*ClaudeAdapter, error) {
	if config.ClaudeAPIKey == "" {
		return nil, fmt.Errorf("claude API key is required")
	}

	adapter := &ClaudeAdapter{
		config:      config,
		client:      &http.Client{Timeout: 30 * time.Second},
		initialized: true,
	}

	return adapter, nil
}

// GenerateContent generates content using Claude API
func (c *ClaudeAdapter) GenerateContent(model string, requestBody map[string]interface{}) (map[string]interface{}, error) {
	if !c.initialized {
		return nil, fmt.Errorf("claude adapter not initialized")
	}

	baseURL := c.config.ClaudeBaseURL
	if baseURL == "" {
		baseURL = "https://api.anthropic.com"
	}

	url := fmt.Sprintf("%s/v1/messages", baseURL)
	
	// Set model in request body
	requestBody["model"] = model
	// Set API version
	requestBody["anthropic-version"] = "2023-06-01"

	jsonData, err := json.Marshal(requestBody)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("x-api-key", c.config.ClaudeAPIKey)
	req.Header.Set("anthropic-version", "2023-06-01")

	resp, err := c.client.Do(req)
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

// GenerateContentStream generates content using Claude API in streaming mode
func (c *ClaudeAdapter) GenerateContentStream(model string, requestBody map[string]interface{}) (<-chan interface{}, error) {
	if !c.initialized {
		return nil, fmt.Errorf("claude adapter not initialized")
	}

	baseURL := c.config.ClaudeBaseURL
	if baseURL == "" {
		baseURL = "https://api.anthropic.com"
	}

	url := fmt.Sprintf("%s/v1/messages", baseURL)
	
	// Set model and streaming in request body
	requestBody["model"] = model
	requestBody["stream"] = true

	jsonData, err := json.Marshal(requestBody)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("x-api-key", c.config.ClaudeAPIKey)
	req.Header.Set("anthropic-version", "2023-06-01")

	resp, err := c.client.Do(req)
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

			// Claude SSE format: "event: type\ndata: {json}"
			// We need to parse both event and data lines
			var eventType string
			var data []byte

			if bytes.HasPrefix(line, []byte("event: ")) {
				eventType = string(bytes.TrimPrefix(line, []byte("event: ")))
				continue
			}

			if bytes.HasPrefix(line, []byte("data: ")) {
				data = bytes.TrimPrefix(line, []byte("data: "))

				var chunk map[string]interface{}
				if err := json.Unmarshal(data, &chunk); err != nil {
					continue
				}

				// Add event type to chunk if available
				if eventType != "" {
					chunk["type"] = eventType
					eventType = ""
				}

				ch <- chunk
			}
		}
	}()

	return ch, nil
}

// ListModels lists available Claude models
func (c *ClaudeAdapter) ListModels() (map[string]interface{}, error) {
	if !c.initialized {
		return nil, fmt.Errorf("claude adapter not initialized")
	}

	// TODO: Implement actual Claude list models
	return map[string]interface{}{
		"models": []map[string]interface{}{
			{"id": "claude-3-7-sonnet-20250219"},
			{"id": "claude-3-5-sonnet-20241022"},
		},
	}, nil
}

// RefreshToken is a no-op for Claude as API keys are static
func (c *ClaudeAdapter) RefreshToken() error {
	return nil
}

// IsInitialized returns whether the adapter is initialized
func (c *ClaudeAdapter) IsInitialized() bool {
	return c.initialized
}

