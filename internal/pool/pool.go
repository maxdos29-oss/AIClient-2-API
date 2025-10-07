package pool

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"sync"
	"time"

	"github.com/justlovemaki/AIClient-2-API/internal/adapter"
	"github.com/justlovemaki/AIClient-2-API/internal/common"
)

// ProviderStatus holds the status of a provider instance
type ProviderStatus struct {
	Config         map[string]interface{} `json:"config"`
	UUID           string                 `json:"uuid"`
	IsHealthy      bool                   `json:"isHealthy"`
	LastUsed       *time.Time             `json:"lastUsed"`
	UsageCount     int                    `json:"usageCount"`
	ErrorCount     int                    `json:"errorCount"`
	LastErrorTime  *time.Time             `json:"lastErrorTime"`
}

// ProviderPoolManager manages pools of API service providers
type ProviderPoolManager struct {
	pools                map[string][]ProviderStatus
	roundRobinIndex      map[string]int
	maxErrorCount        int
	healthCheckInterval  time.Duration
	poolsFilePath        string
	mu                   sync.RWMutex
}

// NewProviderPoolManager creates a new provider pool manager
func NewProviderPoolManager(poolsFilePath string) (*ProviderPoolManager, error) {
	data, err := os.ReadFile(poolsFilePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read pools file: %w", err)
	}

	var rawPools map[string][]map[string]interface{}
	if err := json.Unmarshal(data, &rawPools); err != nil {
		return nil, fmt.Errorf("failed to parse pools file: %w", err)
	}

	manager := &ProviderPoolManager{
		pools:               make(map[string][]ProviderStatus),
		roundRobinIndex:     make(map[string]int),
		maxErrorCount:       3,
		healthCheckInterval: 30 * time.Minute,
		poolsFilePath:       poolsFilePath,
	}

	// Initialize provider status
	for providerType, configs := range rawPools {
		statuses := make([]ProviderStatus, 0, len(configs))
		for _, config := range configs {
			status := ProviderStatus{
				Config:        config,
				IsHealthy:     true,
				UsageCount:    0,
				ErrorCount:    0,
			}

			// Extract UUID from config
			if uuid, ok := config["uuid"].(string); ok {
				status.UUID = uuid
			}

			// Load existing status from config if available
			if isHealthy, ok := config["isHealthy"].(bool); ok {
				status.IsHealthy = isHealthy
			}
			if usageCount, ok := config["usageCount"].(float64); ok {
				status.UsageCount = int(usageCount)
			}
			if errorCount, ok := config["errorCount"].(float64); ok {
				status.ErrorCount = int(errorCount)
			}

			statuses = append(statuses, status)
		}
		manager.pools[providerType] = statuses
	}

	log.Println("[ProviderPoolManager] Initialized provider statuses")

	return manager, nil
}

// SelectProvider selects a healthy provider from the pool using round-robin
func (pm *ProviderPoolManager) SelectProvider(providerType string) map[string]interface{} {
	pm.mu.Lock()
	defer pm.mu.Unlock()

	statuses, exists := pm.pools[providerType]
	if !exists || len(statuses) == 0 {
		log.Printf("[ProviderPoolManager] No providers available for type: %s", providerType)
		return nil
	}

	// Filter healthy providers
	var healthyProviders []int
	for i, status := range statuses {
		if status.IsHealthy {
			healthyProviders = append(healthyProviders, i)
		}
	}

	if len(healthyProviders) == 0 {
		log.Printf("[ProviderPoolManager] No healthy providers available for type: %s", providerType)
		return nil
	}

	// Round-robin selection
	currentIndex := pm.roundRobinIndex[providerType]
	selectedIndex := healthyProviders[currentIndex%len(healthyProviders)]
	pm.roundRobinIndex[providerType] = (currentIndex + 1) % len(healthyProviders)

	// Update usage stats
	now := time.Now()
	statuses[selectedIndex].LastUsed = &now
	statuses[selectedIndex].UsageCount++
	pm.pools[providerType] = statuses

	// Save to file
	go pm.savePoolsToFile()

	log.Printf("[ProviderPoolManager] Selected provider for %s: %s (usage: %d)",
		providerType, statuses[selectedIndex].UUID, statuses[selectedIndex].UsageCount)

	return statuses[selectedIndex].Config
}

// MarkProviderUnhealthy marks a provider as unhealthy
func (pm *ProviderPoolManager) MarkProviderUnhealthy(providerType string, config map[string]interface{}) {
	pm.mu.Lock()
	defer pm.mu.Unlock()

	statuses, exists := pm.pools[providerType]
	if !exists {
		return
	}

	uuid, _ := config["uuid"].(string)
	for i, status := range statuses {
		if status.UUID == uuid {
			statuses[i].ErrorCount++
			now := time.Now()
			statuses[i].LastErrorTime = &now

			if statuses[i].ErrorCount >= pm.maxErrorCount {
				statuses[i].IsHealthy = false
				log.Printf("[ProviderPoolManager] Marked provider as unhealthy: %s (errors: %d)",
					uuid, statuses[i].ErrorCount)
			} else {
				log.Printf("[ProviderPoolManager] Provider %s error count: %d/%d",
					uuid, statuses[i].ErrorCount, pm.maxErrorCount)
			}

			pm.pools[providerType] = statuses
			go pm.savePoolsToFile()
			break
		}
	}
}

// MarkProviderHealthy marks a provider as healthy
func (pm *ProviderPoolManager) MarkProviderHealthy(providerType string, config map[string]interface{}) {
	pm.mu.Lock()
	defer pm.mu.Unlock()

	statuses, exists := pm.pools[providerType]
	if !exists {
		return
	}

	uuid, _ := config["uuid"].(string)
	for i, status := range statuses {
		if status.UUID == uuid {
			statuses[i].IsHealthy = true
			statuses[i].ErrorCount = 0
			statuses[i].LastErrorTime = nil

			log.Printf("[ProviderPoolManager] Marked provider as healthy: %s", uuid)
			pm.pools[providerType] = statuses
			go pm.savePoolsToFile()
			break
		}
	}
}

// PerformHealthChecks performs health checks on all providers
func (pm *ProviderPoolManager) PerformHealthChecks() {
	log.Println("[ProviderPoolManager] Performing health checks on all providers...")

	pm.mu.RLock()
	poolsCopy := make(map[string][]ProviderStatus)
	for k, v := range pm.pools {
		poolsCopy[k] = append([]ProviderStatus{}, v...)
	}
	pm.mu.RUnlock()

	for providerType, statuses := range poolsCopy {
		for _, status := range statuses {
			// Skip recently failed providers
			if !status.IsHealthy && status.LastErrorTime != nil {
				if time.Since(*status.LastErrorTime) < pm.healthCheckInterval {
					log.Printf("[ProviderPoolManager] Skipping health check for %s (%s). Last error too recent.",
						status.UUID, providerType)
					continue
				}
			}

			// Perform health check
			go pm.checkProviderHealth(providerType, status.Config)
		}
	}
}

// checkProviderHealth checks the health of a single provider
func (pm *ProviderPoolManager) checkProviderHealth(providerType string, config map[string]interface{}) {
	// Create a temporary adapter for health check
	tempConfig := pm.configFromMap(providerType, config)
	
	adapter, err := adapter.GetAdapter(tempConfig)
	if err != nil {
		log.Printf("[ProviderPoolManager] Failed to get adapter for health check: %v", err)
		pm.MarkProviderUnhealthy(providerType, config)
		return
	}

	// Try to list models as a simple health check
	_, err = adapter.ListModels()
	if err != nil {
		log.Printf("[ProviderPoolManager] Health check failed for %s: %v", config["uuid"], err)
		pm.MarkProviderUnhealthy(providerType, config)
	} else {
		log.Printf("[ProviderPoolManager] Health check passed for %s", config["uuid"])
		pm.MarkProviderHealthy(providerType, config)
	}
}

// configFromMap converts a map to a Config struct
func (pm *ProviderPoolManager) configFromMap(providerType string, m map[string]interface{}) *common.Config {
	config := &common.Config{
		ModelProvider: providerType,
	}

	if uuid, ok := m["uuid"].(string); ok {
		config.UUID = uuid
	}
	if key, ok := m["OPENAI_API_KEY"].(string); ok {
		config.OpenAIAPIKey = key
	}
	if url, ok := m["OPENAI_BASE_URL"].(string); ok {
		config.OpenAIBaseURL = url
	}
	if key, ok := m["CLAUDE_API_KEY"].(string); ok {
		config.ClaudeAPIKey = key
	}
	if url, ok := m["CLAUDE_BASE_URL"].(string); ok {
		config.ClaudeBaseURL = url
	}
	if path, ok := m["GEMINI_OAUTH_CREDS_FILE_PATH"].(string); ok {
		config.GeminiOAuthCredsFilePath = path
	}
	if id, ok := m["PROJECT_ID"].(string); ok {
		config.ProjectID = id
	}
	if path, ok := m["KIRO_OAUTH_CREDS_FILE_PATH"].(string); ok {
		config.KiroOAuthCredsFilePath = path
	}
	if path, ok := m["QWEN_OAUTH_CREDS_FILE_PATH"].(string); ok {
		config.QwenOAuthCredsFilePath = path
	}

	return config
}

// savePoolsToFile saves the current pool state to file
func (pm *ProviderPoolManager) savePoolsToFile() {
	pm.mu.RLock()
	defer pm.mu.RUnlock()

	// Convert pools to save format
	saveData := make(map[string][]map[string]interface{})
	for providerType, statuses := range pm.pools {
		configs := make([]map[string]interface{}, 0, len(statuses))
		for _, status := range statuses {
			config := make(map[string]interface{})
			for k, v := range status.Config {
				config[k] = v
			}
			config["isHealthy"] = status.IsHealthy
			config["usageCount"] = status.UsageCount
			config["errorCount"] = status.ErrorCount
			if status.LastUsed != nil {
				config["lastUsed"] = status.LastUsed.Format(time.RFC3339)
			}
			if status.LastErrorTime != nil {
				config["lastErrorTime"] = status.LastErrorTime.Format(time.RFC3339)
			}
			configs = append(configs, config)
		}
		saveData[providerType] = configs
	}

	data, err := json.MarshalIndent(saveData, "", "  ")
	if err != nil {
		log.Printf("[ProviderPoolManager] Failed to marshal pools: %v", err)
		return
	}

	if err := os.WriteFile(pm.poolsFilePath, data, 0644); err != nil {
		log.Printf("[ProviderPoolManager] Failed to write pools file: %v", err)
		return
	}
}

