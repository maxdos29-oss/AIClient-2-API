package common

import (
	"fmt"
	"math"
	"time"
)

// RetryConfig holds retry configuration
type RetryConfig struct {
	MaxRetries int
	BaseDelay  time.Duration
}

// RetryableFunc is a function that can be retried
type RetryableFunc func() (interface{}, error)

// WithRetry executes a function with exponential backoff retry
func WithRetry(config *RetryConfig, fn RetryableFunc) (interface{}, error) {
	var lastErr error

	for attempt := 0; attempt <= config.MaxRetries; attempt++ {
		result, err := fn()
		if err == nil {
			return result, nil
		}

		lastErr = err

		// Don't retry on last attempt
		if attempt == config.MaxRetries {
			break
		}

		// Calculate delay with exponential backoff
		delay := time.Duration(float64(config.BaseDelay) * math.Pow(2, float64(attempt)))
		
		// Log retry attempt
		fmt.Printf("[Retry] Attempt %d failed: %v. Retrying in %v...\n", 
			attempt+1, err, delay)

		time.Sleep(delay)
	}

	return nil, fmt.Errorf("max retries (%d) exceeded: %w", config.MaxRetries, lastErr)
}

// ShouldRetry determines if an error should trigger a retry
func ShouldRetry(err error, statusCode int) bool {
	if err != nil {
		return true
	}

	// Retry on 5xx errors and 429 (rate limit)
	if statusCode >= 500 || statusCode == 429 {
		return true
	}

	return false
}

