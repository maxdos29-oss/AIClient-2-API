package adapter

import (
	"bytes"
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/justlovemaki/AIClient-2-API/internal/common"
)

// Kiro constants
const (
	kiroRefreshURL    = "https://prod.{{region}}.auth.desktop.kiro.dev/refreshToken"
	kiroRefreshIDCURL = "https://oidc.{{region}}.amazonaws.com/token"
	kiroBaseURL       = "https://codewhisperer.{{region}}.amazonaws.com/generateAssistantResponse"
	kiroAmazonQURL    = "https://codewhisperer.{{region}}.amazonaws.com/SendMessageStreaming"
	kiroAuthMethodSocial = "social"
	kiroDefaultModel  = "claude-sonnet-4-5-20250929"
)

// Kiro model mapping
var kiroModelMapping = map[string]string{
	"claude-sonnet-4-20250514":            "CLAUDE_SONNET_4_20250514_V1_0",
	"claude-sonnet-4-5-20250929":          "CLAUDE_SONNET_4_5_20250929_V1_0",
	"claude-3-7-sonnet-20250219":          "CLAUDE_3_7_SONNET_20250219_V1_0",
	"amazonq-claude-sonnet-4-20250514":    "CLAUDE_SONNET_4_20250514_V1_0",
	"amazonq-claude-sonnet-4-5-20250929":  "CLAUDE_SONNET_4_5_20250929_V1_0",
	"amazonq-claude-3-7-sonnet-20250219":  "CLAUDE_3_7_SONNET_20250219_V1_0",
}

// KiroAdapter implements the ApiServiceAdapter interface for Kiro API
type KiroAdapter struct {
	config       *common.Config
	client       *http.Client
	accessToken  string
	refreshToken string
	clientID     string
	clientSecret string
	profileARN   string
	region       string
	authMethod   string
	expiresAt    time.Time
	baseURL      string
	amazonQURL   string
	refreshURL   string
	refreshIDCURL string
	macSHA256    string
	initialized  bool
}

// NewKiroAdapter creates a new Kiro adapter
func NewKiroAdapter(config *common.Config) (*KiroAdapter, error) {
	adapter := &KiroAdapter{
		config:     config,
		client:     &http.Client{Timeout: 120 * time.Second},
		region:     "us-east-1", // Default region
		authMethod: kiroAuthMethodSocial,
		initialized: false,
	}

	// Get MAC address SHA256
	if err := adapter.initMacAddress(); err != nil {
		fmt.Printf("[Kiro] Warning: Failed to get MAC address: %v\n", err)
	}

	// Initialize authentication
	if err := adapter.initializeAuth(false); err != nil {
		return nil, fmt.Errorf("failed to initialize Kiro auth: %w", err)
	}

	adapter.initialized = true
	return adapter, nil
}

// initMacAddress gets the MAC address and computes SHA256
func (k *KiroAdapter) initMacAddress() error {
	interfaces, err := net.Interfaces()
	if err != nil {
		return err
	}

	var macAddress string
	for _, iface := range interfaces {
		// Skip loopback and interfaces without MAC
		if iface.Flags&net.FlagLoopback != 0 || len(iface.HardwareAddr) == 0 {
			continue
		}
		macAddress = iface.HardwareAddr.String()
		if macAddress != "" && macAddress != "00:00:00:00:00:00" {
			break
		}
	}

	if macAddress == "" {
		macAddress = "00:00:00:00:00:00"
	}

	hash := sha256.Sum256([]byte(macAddress))
	k.macSHA256 = fmt.Sprintf("%x", hash)
	return nil
}

// initializeAuth initializes Kiro OAuth authentication
func (k *KiroAdapter) initializeAuth(forceRefresh bool) error {
	if k.accessToken != "" && !forceRefresh {
		return nil
	}

	// Load credentials
	var credsData map[string]interface{}
	var err error

	// Try Base64 first
	if k.config.KiroOAuthCredsBase64 != "" {
		data, err := common.DecodeBase64(k.config.KiroOAuthCredsBase64)
		if err != nil {
			return fmt.Errorf("failed to decode base64 credentials: %w", err)
		}
		if err := json.Unmarshal(data, &credsData); err != nil {
			return fmt.Errorf("failed to parse base64 credentials: %w", err)
		}
	} else {
		// Try file path
		var filePath string
		if k.config.KiroOAuthCredsFilePath != "" {
			filePath = common.ExpandHomePath(k.config.KiroOAuthCredsFilePath)
		} else {
			// Try default path
			home, _ := os.UserHomeDir()
			filePath = filepath.Join(home, ".aws", "sso", "cache", "kiro-auth-token.json")
		}

		data, err := os.ReadFile(filePath)
		if err != nil {
			return fmt.Errorf("failed to read credentials file: %w", err)
		}
		if err := json.Unmarshal(data, &credsData); err != nil {
			return fmt.Errorf("failed to parse credentials: %w", err)
		}
	}

	// Extract credentials
	if token, ok := credsData["accessToken"].(string); ok {
		k.accessToken = token
	}
	if token, ok := credsData["refreshToken"].(string); ok {
		k.refreshToken = token
	}
	if id, ok := credsData["clientId"].(string); ok {
		k.clientID = id
	}
	if secret, ok := credsData["clientSecret"].(string); ok {
		k.clientSecret = secret
	}
	if arn, ok := credsData["profileArn"].(string); ok {
		k.profileARN = arn
	}
	if region, ok := credsData["region"].(string); ok && region != "" {
		k.region = region
	}
	if method, ok := credsData["authMethod"].(string); ok {
		k.authMethod = method
	}
	if expiresAt, ok := credsData["expiresAt"].(string); ok {
		if t, err := time.Parse(time.RFC3339, expiresAt); err == nil {
			k.expiresAt = t
		}
	}

	// Set region-specific URLs
	k.baseURL = strings.ReplaceAll(kiroBaseURL, "{{region}}", k.region)
	k.amazonQURL = strings.ReplaceAll(kiroAmazonQURL, "{{region}}", k.region)
	k.refreshURL = strings.ReplaceAll(kiroRefreshURL, "{{region}}", k.region)
	k.refreshIDCURL = strings.ReplaceAll(kiroRefreshIDCURL, "{{region}}", k.region)

	// Refresh token if needed
	if forceRefresh || k.accessToken == "" {
		if err := k.refreshAccessToken(); err != nil {
			return fmt.Errorf("failed to refresh token: %w", err)
		}
	}

	return nil
}

// refreshAccessToken refreshes the Kiro access token
func (k *KiroAdapter) refreshAccessToken() error {
	if k.refreshToken == "" {
		return fmt.Errorf("no refresh token available")
	}

	var refreshURL string
	var requestBody map[string]interface{}

	if k.authMethod == kiroAuthMethodSocial {
		refreshURL = k.refreshURL
		requestBody = map[string]interface{}{
			"refreshToken": k.refreshToken,
		}
	} else {
		refreshURL = k.refreshIDCURL
		requestBody = map[string]interface{}{
			"refreshToken": k.refreshToken,
			"clientId":     k.clientID,
			"clientSecret": k.clientSecret,
			"grantType":    "refresh_token",
		}
	}

	jsonData, err := json.Marshal(requestBody)
	if err != nil {
		return fmt.Errorf("failed to marshal request: %w", err)
	}

	req, err := http.NewRequest("POST", refreshURL, bytes.NewBuffer(jsonData))
	if err != nil {
		return fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")

	resp, err := k.client.Do(req)
	if err != nil {
		return fmt.Errorf("failed to refresh token: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("failed to read response: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("token refresh failed with status %d: %s", resp.StatusCode, string(body))
	}

	var result map[string]interface{}
	if err := json.Unmarshal(body, &result); err != nil {
		return fmt.Errorf("failed to parse response: %w", err)
	}

	// Update tokens
	if token, ok := result["accessToken"].(string); ok {
		k.accessToken = token
	}
	if token, ok := result["refreshToken"].(string); ok {
		k.refreshToken = token
	}
	if arn, ok := result["profileArn"].(string); ok {
		k.profileARN = arn
	}
	if expiresIn, ok := result["expiresIn"].(float64); ok {
		k.expiresAt = time.Now().Add(time.Duration(expiresIn) * time.Second)
	}

	fmt.Println("[Kiro] Access token refreshed successfully")
	return nil
}

// GenerateContent generates content using Kiro API
func (k *KiroAdapter) GenerateContent(model string, requestBody map[string]interface{}) (map[string]interface{}, error) {
	if !k.initialized {
		return nil, fmt.Errorf("kiro adapter not initialized")
	}

	// Build CodeWhisperer request
	cwRequest, err := k.buildCodeWhispererRequest(model, requestBody)
	if err != nil {
		return nil, fmt.Errorf("failed to build request: %w", err)
	}

	// Call API
	response, err := k.callKiroAPI(model, cwRequest)
	if err != nil {
		return nil, err
	}

	// Parse response and build Claude format
	return k.buildClaudeResponse(response, model, false)
}

// GenerateContentStream generates content using Kiro API in streaming mode
// Note: Kiro API doesn't truly stream, so we simulate streaming
func (k *KiroAdapter) GenerateContentStream(model string, requestBody map[string]interface{}) (<-chan interface{}, error) {
	if !k.initialized {
		return nil, fmt.Errorf("kiro adapter not initialized")
	}

	ch := make(chan interface{}, 10)

	go func() {
		defer close(ch)

		// Build CodeWhisperer request
		cwRequest, err := k.buildCodeWhispererRequest(model, requestBody)
		if err != nil {
			ch <- map[string]interface{}{"error": err.Error()}
			return
		}

		// Call API (Kiro doesn't actually stream, returns full response)
		response, err := k.callKiroAPI(model, cwRequest)
		if err != nil {
			ch <- map[string]interface{}{"error": err.Error()}
			return
		}

		// Build Claude streaming events
		events, err := k.buildClaudeStreamingEvents(response, model)
		if err != nil {
			ch <- map[string]interface{}{"error": err.Error()}
			return
		}

		// Send all events
		for _, event := range events {
			ch <- event
		}
	}()

	return ch, nil
}

// ListModels lists available Kiro models
func (k *KiroAdapter) ListModels() (map[string]interface{}, error) {
	if !k.initialized {
		return nil, fmt.Errorf("kiro adapter not initialized")
	}

	models := make([]map[string]interface{}, 0, len(kiroModelMapping))
	for modelID := range kiroModelMapping {
		models = append(models, map[string]interface{}{
			"id":   modelID,
			"name": modelID,
		})
	}

	return map[string]interface{}{
		"models": models,
	}, nil
}

// RefreshToken refreshes the Kiro OAuth token if needed
func (k *KiroAdapter) RefreshToken() error {
	if !k.expiresAt.IsZero() && time.Until(k.expiresAt) < 10*time.Minute {
		return k.initializeAuth(true)
	}
	return nil
}

// IsInitialized returns whether the adapter is initialized
func (k *KiroAdapter) IsInitialized() bool {
	return k.initialized
}

// buildCodeWhispererRequest builds a CodeWhisperer-format request from Claude messages
func (k *KiroAdapter) buildCodeWhispererRequest(model string, requestBody map[string]interface{}) (map[string]interface{}, error) {
	messages, ok := requestBody["messages"].([]interface{})
	if !ok || len(messages) == 0 {
		return nil, fmt.Errorf("no messages in request")
	}

	// Get CodeWhisperer model name
	cwModel, ok := kiroModelMapping[model]
	if !ok {
		cwModel = kiroModelMapping[kiroDefaultModel]
	}

	conversationID := uuid.New().String()
	history := []map[string]interface{}{}

	// Process messages except the last one (which becomes currentMessage)
	for i := 0; i < len(messages)-1; i++ {
		msgMap, ok := messages[i].(map[string]interface{})
		if !ok {
			continue
		}

		role, _ := msgMap["role"].(string)
		content := extractContentText(msgMap["content"])

		if role == "user" {
			history = append(history, map[string]interface{}{
				"userInputMessage": map[string]interface{}{
					"content": content,
					"modelId": cwModel,
					"origin":  "AI_EDITOR",
				},
			})
		} else if role == "assistant" {
			history = append(history, map[string]interface{}{
				"assistantResponseMessage": map[string]interface{}{
					"content": content,
				},
			})
		}
	}

	// Build current message (last message)
	currentMsg := messages[len(messages)-1].(map[string]interface{})
	currentRole, _ := currentMsg["role"].(string)
	currentContent := extractContentText(currentMsg["content"])

	// Handle system prompt
	if system, ok := requestBody["system"].(string); ok && system != "" {
		// Prepend system prompt to first user message
		if currentRole == "user" {
			currentContent = system + "\n\n" + currentContent
		}
	}

	currentMessage := map[string]interface{}{}
	if currentRole == "user" {
		currentMessage["userInputMessage"] = map[string]interface{}{
			"content": currentContent,
			"modelId": cwModel,
			"origin":  "AI_EDITOR",
		}
	} else {
		currentMessage["assistantResponseMessage"] = map[string]interface{}{
			"content": currentContent,
		}
	}

	request := map[string]interface{}{
		"conversationState": map[string]interface{}{
			"chatTriggerType": "MANUAL",
			"conversationId":  conversationID,
			"currentMessage":  currentMessage,
			"history":         history,
		},
	}

	// Add profileArn for social auth
	if k.authMethod == kiroAuthMethodSocial && k.profileARN != "" {
		request["profileArn"] = k.profileARN
	}

	return request, nil
}

// callKiroAPI makes the actual API call to Kiro
func (k *KiroAdapter) callKiroAPI(model string, requestBody map[string]interface{}) (string, error) {
	// Determine URL based on model
	url := k.baseURL
	if strings.HasPrefix(model, "amazonq") {
		url = k.amazonQURL
	}

	jsonData, err := json.Marshal(requestBody)
	if err != nil {
		return "", fmt.Errorf("failed to marshal request: %w", err)
	}

	req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
	if err != nil {
		return "", fmt.Errorf("failed to create request: %w", err)
	}

	// Set headers
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", k.accessToken))
	req.Header.Set("amz-sdk-invocation-id", uuid.New().String())
	req.Header.Set("x-amz-user-agent", fmt.Sprintf("aws-sdk-js/1.0.7 KiroIDE-0.1.25-%s", k.macSHA256))
	req.Header.Set("User-Agent", fmt.Sprintf("aws-sdk-js/1.0.7 ua/2.1 os/linux md/nodejs#20.16.0 api/codewhispererstreaming#1.0.7 m/E KiroIDE-0.1.25-%s", k.macSHA256))
	req.Header.Set("amz-sdk-request", "attempt=1; max=1")
	req.Header.Set("x-amzn-kiro-agent-mode", "vibe")

	resp, err := k.client.Do(req)
	if err != nil {
		return "", fmt.Errorf("failed to make request: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", fmt.Errorf("failed to read response: %w", err)
	}

	// Handle 403 - token may need refresh
	if resp.StatusCode == 403 {
		fmt.Println("[Kiro] Received 403, refreshing token and retrying...")
		if err := k.initializeAuth(true); err != nil {
			return "", fmt.Errorf("token refresh failed: %w", err)
		}
		// Retry once
		return k.callKiroAPI(model, requestBody)
	}

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("API request failed with status %d: %s", resp.StatusCode, string(body))
	}

	return string(body), nil
}

// buildClaudeResponse builds a Claude-format response from Kiro response
func (k *KiroAdapter) buildClaudeResponse(rawResponse string, model string, isStream bool) (map[string]interface{}, error) {
	// Parse the event stream response
	content := k.parseEventStreamContent(rawResponse)

	messageID := uuid.New().String()
	estimateTokens := func(text string) int {
		return len(text) / 4
	}

	return map[string]interface{}{
		"id":            messageID,
		"type":          "message",
		"role":          "assistant",
		"model":         model,
		"content": []interface{}{
			map[string]interface{}{
				"type": "text",
				"text": content,
			},
		},
		"stop_reason":   "end_turn",
		"stop_sequence": nil,
		"usage": map[string]interface{}{
			"input_tokens":  0,
			"output_tokens": estimateTokens(content),
		},
	}, nil
}

// buildClaudeStreamingEvents builds Claude streaming events from Kiro response
func (k *KiroAdapter) buildClaudeStreamingEvents(rawResponse string, model string) ([]map[string]interface{}, error) {
	content := k.parseEventStreamContent(rawResponse)
	messageID := uuid.New().String()
	
	events := []map[string]interface{}{
		// message_start
		{
			"type": "message_start",
			"message": map[string]interface{}{
				"id":    messageID,
				"type":  "message",
				"role":  "assistant",
				"model": model,
				"usage": map[string]interface{}{
					"input_tokens":  0,
					"output_tokens": 0,
				},
				"content": []interface{}{},
			},
		},
		// content_block_start
		{
			"type":  "content_block_start",
			"index": 0,
			"content_block": map[string]interface{}{
				"type": "text",
				"text": "",
			},
		},
		// content_block_delta
		{
			"type":  "content_block_delta",
			"index": 0,
			"delta": map[string]interface{}{
				"type": "text_delta",
				"text": content,
			},
		},
		// content_block_stop
		{
			"type":  "content_block_stop",
			"index": 0,
		},
		// message_delta
		{
			"type": "message_delta",
			"delta": map[string]interface{}{
				"stop_reason":   "end_turn",
				"stop_sequence": nil,
			},
			"usage": map[string]interface{}{
				"output_tokens": len(content) / 4,
			},
		},
		// message_stop
		{
			"type": "message_stop",
		},
	}

	return events, nil
}

// parseEventStreamContent parses Kiro's event stream format
func (k *KiroAdapter) parseEventStreamContent(rawData string) string {
	var fullContent strings.Builder

	// Simple parsing - extract content from event blocks
	// Kiro returns event{...}event{...} format
	parts := strings.Split(rawData, "event{")
	for _, part := range parts {
		if part == "" {
			continue
		}

		// Find the end of JSON
		endIdx := strings.Index(part, "}")
		if endIdx == -1 {
			continue
		}

		jsonStr := "{" + part[:endIdx+1]
		
		var eventData map[string]interface{}
		if err := json.Unmarshal([]byte(jsonStr), &eventData); err != nil {
			continue
		}

		// Extract content
		if content, ok := eventData["content"].(string); ok {
			// Decode \n to actual newlines
			content = strings.ReplaceAll(content, "\\n", "\n")
			fullContent.WriteString(content)
		}
	}

	return fullContent.String()
}

// extractContentText extracts text from message content (various formats)
func extractContentText(content interface{}) string {
	switch v := content.(type) {
	case string:
		return v
	case []interface{}:
		var texts []string
		for _, item := range v {
			if itemMap, ok := item.(map[string]interface{}); ok {
				if itemMap["type"] == "text" {
					if text, ok := itemMap["text"].(string); ok {
						texts = append(texts, text)
					}
				}
			}
		}
		return strings.Join(texts, "")
	}
	return ""
}

