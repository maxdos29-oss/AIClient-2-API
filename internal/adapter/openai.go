package adapter

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"

	"github.com/justlovemaki/AIClient-2-API/internal/common"
)

// OpenAIAdapter implements the ApiServiceAdapter interface for OpenAI API
type OpenAIAdapter struct {
	config      *common.Config
	client      *http.Client
	initialized bool
}

// NewOpenAIAdapter creates a new OpenAI adapter
func NewOpenAIAdapter(config *common.Config) (*OpenAIAdapter, error) {
	if config.OpenAIAPIKey == "" {
		return nil, fmt.Errorf("openai API key is required")
	}

	adapter := &OpenAIAdapter{
		config:      config,
		client:      &http.Client{},
		initialized: true,
	}

	return adapter, nil
}

// GenerateContent generates content using OpenAI API
func (o *OpenAIAdapter) GenerateContent(model string, requestBody map[string]interface{}) (map[string]interface{}, error) {
	if !o.initialized {
		return nil, fmt.Errorf("openai adapter not initialized")
	}

	baseURL := o.config.OpenAIBaseURL
	if baseURL == "" {
		baseURL = "https://api.openai.com/v1"
	}

	url := fmt.Sprintf("%s/chat/completions", baseURL)

	// Set model in request body
	requestBody["model"] = model

	jsonData, err := json.Marshal(requestBody)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", o.config.OpenAIAPIKey))

	resp, err := o.client.Do(req)
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

// GenerateContentStream generates content using OpenAI API in streaming mode
func (o *OpenAIAdapter) GenerateContentStream(model string, requestBody map[string]interface{}) (<-chan interface{}, error) {
	if !o.initialized {
		return nil, fmt.Errorf("openai adapter not initialized")
	}

	baseURL := o.config.OpenAIBaseURL
	if baseURL == "" {
		baseURL = "https://api.openai.com/v1"
	}

	url := fmt.Sprintf("%s/chat/completions", baseURL)

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
	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", o.config.OpenAIAPIKey))

	resp, err := o.client.Do(req)
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

			// SSE format: "data: {json}"
			if bytes.HasPrefix(line, []byte("data: ")) {
				data := bytes.TrimPrefix(line, []byte("data: "))

				// Check for [DONE] signal
				if bytes.Equal(data, []byte("[DONE]")) {
					break
				}

				var chunk map[string]interface{}
				if err := json.Unmarshal(data, &chunk); err != nil {
					continue // Skip invalid JSON
				}

				ch <- chunk
			}
		}
	}()

	return ch, nil
}

// ListModels lists available OpenAI models
func (o *OpenAIAdapter) ListModels() (map[string]interface{}, error) {
	if !o.initialized {
		return nil, fmt.Errorf("openai adapter not initialized")
	}

	baseURL := o.config.OpenAIBaseURL
	if baseURL == "" {
		baseURL = "https://api.openai.com/v1"
	}

	url := fmt.Sprintf("%s/models", baseURL)

	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", o.config.OpenAIAPIKey))

	resp, err := o.client.Do(req)
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

// RefreshToken is a no-op for OpenAI as API keys are static
func (o *OpenAIAdapter) RefreshToken() error {
	return nil
}

// IsInitialized returns whether the adapter is initialized
func (o *OpenAIAdapter) IsInitialized() bool {
	return o.initialized
}
