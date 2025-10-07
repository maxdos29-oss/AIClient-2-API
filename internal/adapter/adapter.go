package adapter

import (
	"fmt"
	"sync"

	"github.com/justlovemaki/AIClient-2-API/internal/common"
)

// ApiServiceAdapter defines the interface for all API service adapters
type ApiServiceAdapter interface {
	// GenerateContent generates content using the API
	GenerateContent(model string, requestBody map[string]interface{}) (map[string]interface{}, error)

	// GenerateContentStream generates content using the API in streaming mode
	GenerateContentStream(model string, requestBody map[string]interface{}) (<-chan interface{}, error)

	// ListModels lists available models
	ListModels() (map[string]interface{}, error)

	// RefreshToken refreshes the authentication token if needed
	RefreshToken() error

	// IsInitialized returns whether the adapter is initialized
	IsInitialized() bool
}

var (
	adapterInstances = make(map[string]ApiServiceAdapter)
	adapterMutex     sync.RWMutex
)

// GetAdapter returns an adapter instance for the given configuration
func GetAdapter(config *common.Config) (ApiServiceAdapter, error) {
	provider := config.ModelProvider
	providerKey := provider
	if config.UUID != "" {
		providerKey = provider + config.UUID
	}

	adapterMutex.RLock()
	if adapter, exists := adapterInstances[providerKey]; exists {
		adapterMutex.RUnlock()
		return adapter, nil
	}
	adapterMutex.RUnlock()

	// Create new adapter
	adapterMutex.Lock()
	defer adapterMutex.Unlock()

	// Double-check after acquiring write lock
	if adapter, exists := adapterInstances[providerKey]; exists {
		return adapter, nil
	}

	var adapter ApiServiceAdapter
	var err error

	switch provider {
	case common.ModelProviderGeminiCLI:
		adapter, err = NewGeminiAdapter(config)
	case common.ModelProviderOpenAICustom:
		adapter, err = NewOpenAIAdapter(config)
	case common.ModelProviderClaudeCustom:
		adapter, err = NewClaudeAdapter(config)
	case common.ModelProviderKiroAPI:
		adapter, err = NewKiroAdapter(config)
	case common.ModelProviderQwenAPI:
		adapter, err = NewQwenAdapter(config)
	default:
		return nil, fmt.Errorf("unsupported model provider: %s", provider)
	}

	if err != nil {
		return nil, fmt.Errorf("failed to create adapter for %s: %w", provider, err)
	}

	adapterInstances[providerKey] = adapter
	fmt.Printf("[Adapter] Created adapter for %s (key: %s)\n", provider, providerKey)

	return adapter, nil
}

// GetAllAdapters returns all initialized adapter instances
func GetAllAdapters() map[string]ApiServiceAdapter {
	adapterMutex.RLock()
	defer adapterMutex.RUnlock()

	result := make(map[string]ApiServiceAdapter, len(adapterInstances))
	for k, v := range adapterInstances {
		result[k] = v
	}

	return result
}

// InitializeAdapters initializes adapters for all configured providers
func InitializeAdapters(config *common.Config) error {
	var errors []error

	// Initialize adapters for all default providers
	for _, provider := range config.DefaultModelProviders {
		providerConfig := &common.Config{
			ModelProvider:             provider,
			RequiredAPIKey:            config.RequiredAPIKey,
			ServerPort:                config.ServerPort,
			Host:                      config.Host,
			OpenAIAPIKey:              config.OpenAIAPIKey,
			OpenAIBaseURL:             config.OpenAIBaseURL,
			ClaudeAPIKey:              config.ClaudeAPIKey,
			ClaudeBaseURL:             config.ClaudeBaseURL,
			GeminiOAuthCredsBase64:    config.GeminiOAuthCredsBase64,
			GeminiOAuthCredsFilePath:  config.GeminiOAuthCredsFilePath,
			KiroOAuthCredsBase64:      config.KiroOAuthCredsBase64,
			KiroOAuthCredsFilePath:    config.KiroOAuthCredsFilePath,
			QwenOAuthCredsFilePath:    config.QwenOAuthCredsFilePath,
			ProjectID:                 config.ProjectID,
			SystemPromptFilePath:      config.SystemPromptFilePath,
			SystemPromptMode:          config.SystemPromptMode,
			SystemPromptContent:       config.SystemPromptContent,
			PromptLogBaseName:         config.PromptLogBaseName,
			PromptLogMode:             config.PromptLogMode,
			PromptLogFilename:         config.PromptLogFilename,
			RequestMaxRetries:         config.RequestMaxRetries,
			RequestBaseDelay:          config.RequestBaseDelay,
			CronNearMinutes:           config.CronNearMinutes,
			CronRefreshToken:          config.CronRefreshToken,
			DefaultModelProviders:     config.DefaultModelProviders,
		}

		fmt.Printf("[Initialization] Initializing adapter for %s...\n", provider)
		if _, err := GetAdapter(providerConfig); err != nil {
			fmt.Printf("[Initialization Warning] Failed to initialize adapter for %s: %v\n", provider, err)
			errors = append(errors, err)
		}
	}

	if len(errors) > 0 {
		return fmt.Errorf("failed to initialize %d adapter(s)", len(errors))
	}

	return nil
}

