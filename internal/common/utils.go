package common

import (
	"crypto/md5"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// FormatExpiryTime formats an expiry timestamp to a human-readable duration
func FormatExpiryTime(expiryTimestamp int64) string {
	if expiryTimestamp == 0 {
		return "No expiry date available"
	}

	diffMs := expiryTimestamp - time.Now().UnixMilli()
	if diffMs <= 0 {
		return "Token has expired"
	}

	totalSeconds := diffMs / 1000
	hours := totalSeconds / 3600
	totalSeconds %= 3600
	minutes := totalSeconds / 60
	seconds := totalSeconds % 60

	return fmt.Sprintf("%02dh %02dm %02ds", hours, minutes, seconds)
}

// IsAuthorized checks if the request is authorized based on API key
func IsAuthorized(r *http.Request, requiredAPIKey string) bool {
	// Check Authorization header (Bearer token)
	authHeader := r.Header.Get("Authorization")
	if strings.HasPrefix(authHeader, "Bearer ") {
		token := strings.TrimPrefix(authHeader, "Bearer ")
		if token == requiredAPIKey {
			return true
		}
	}

	// Check x-goog-api-key header (Gemini style)
	googAPIKey := r.Header.Get("x-goog-api-key")
	if googAPIKey == requiredAPIKey {
		return true
	}

	// Check x-api-key header (Claude style)
	claudeAPIKey := r.Header.Get("x-api-key")
	if claudeAPIKey == requiredAPIKey {
		return true
	}

	// Check query parameter
	queryKey := r.URL.Query().Get("key")
	if queryKey == requiredAPIKey {
		return true
	}

	return false
}

// GetRequestBody reads and parses the request body as JSON
func GetRequestBody(r *http.Request) (map[string]interface{}, error) {
	if r.Body == nil {
		return make(map[string]interface{}), nil
	}

	body, err := io.ReadAll(r.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read request body: %w", err)
	}

	if len(body) == 0 {
		return make(map[string]interface{}), nil
	}

	var result map[string]interface{}
	if err := json.Unmarshal(body, &result); err != nil {
		return nil, fmt.Errorf("invalid JSON in request body: %w", err)
	}

	return result, nil
}

// LogConversation logs conversation to console or file
func LogConversation(logType, content, logMode, logFilename string) error {
	if logMode == "none" || content == "" {
		return nil
	}

	timestamp := time.Now().Format(time.RFC3339)
	logEntry := fmt.Sprintf("%s [%s]:\n%s\n--------------------------------------\n",
		timestamp, strings.ToUpper(logType), content)

	if logMode == "console" {
		fmt.Print(logEntry)
	} else if logMode == "file" {
		f, err := os.OpenFile(logFilename, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
		if err != nil {
			return fmt.Errorf("failed to open log file: %w", err)
		}
		defer f.Close()

		if _, err := f.WriteString(logEntry); err != nil {
			return fmt.Errorf("failed to write to log file: %w", err)
		}
	}

	return nil
}

// GetMD5Hash generates an MD5 hash for a given object
func GetMD5Hash(obj interface{}) (string, error) {
	data, err := json.Marshal(obj)
	if err != nil {
		return "", err
	}

	hash := md5.Sum(data)
	return hex.EncodeToString(hash[:]), nil
}

// ExpandHomePath expands ~ to the user's home directory
func ExpandHomePath(path string) string {
	if !strings.HasPrefix(path, "~/") {
		return path
	}

	home, err := os.UserHomeDir()
	if err != nil {
		return path
	}

	return filepath.Join(home, path[2:])
}

// EnsureDir ensures a directory exists, creating it if necessary
func EnsureDir(path string) error {
	if path == "" {
		return nil
	}

	dir := filepath.Dir(path)
	if _, err := os.Stat(dir); os.IsNotExist(err) {
		if err := os.MkdirAll(dir, 0755); err != nil {
			return fmt.Errorf("failed to create directory: %w", err)
		}
	}

	return nil
}

// ParseQueryParams parses URL query parameters
func ParseQueryParams(urlStr string) (url.Values, error) {
	u, err := url.Parse(urlStr)
	if err != nil {
		return nil, err
	}
	return u.Query(), nil
}

// CheckAndAssignOrDefault returns the value if it's not zero, otherwise returns the default
func CheckAndAssignOrDefault(value interface{}, defaultValue interface{}) interface{} {
	switch v := value.(type) {
	case int:
		if v != 0 {
			return v
		}
	case float64:
		if v != 0 {
			return v
		}
	case string:
		if v != "" {
			return v
		}
	}
	return defaultValue
}

// SafeParseJSON parses a JSON string, returning the original string on error
func SafeParseJSON(str string) interface{} {
	if str == "" {
		return str
	}

	// Handle truncated escape sequences
	cleanedStr := str
	if strings.HasSuffix(cleanedStr, "\\") && !strings.HasSuffix(cleanedStr, "\\\\") {
		cleanedStr = cleanedStr[:len(cleanedStr)-1]
	} else if strings.HasSuffix(cleanedStr, "\\u") ||
		strings.HasSuffix(cleanedStr, "\\u0") ||
		strings.HasSuffix(cleanedStr, "\\u00") {
		idx := strings.LastIndex(cleanedStr, "\\u")
		if idx >= 0 {
			cleanedStr = cleanedStr[:idx]
		}
	}

	var result interface{}
	if err := json.Unmarshal([]byte(cleanedStr), &result); err != nil {
		return str
	}

	return result
}

// ExtractTextFromMessageContent extracts text from various forms of message content
func ExtractTextFromMessageContent(content interface{}) string {
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
		return strings.Join(texts, "\n")
	}
	return ""
}

// DecodeBase64 decodes a base64 string
func DecodeBase64(encoded string) ([]byte, error) {
	decoded, err := base64.StdEncoding.DecodeString(encoded)
	if err != nil {
		return nil, fmt.Errorf("failed to decode base64: %w", err)
	}
	return decoded, nil
}
