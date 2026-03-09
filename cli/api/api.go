package api

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
)

type createSecretRequest struct {
	Secret        string `json:"secret"`
	ExpirySeconds int    `json:"expiry_seconds"`
}

type createSecretResponse struct {
	SecretID string `json:"secretId"`
}

type errorResponse struct {
	Message string `json:"message"`
}

// CreateSecret posts the encrypted payload to POST {baseURL}/secrets and returns
// the secretId from the response. baseURL should be the full API base, e.g.
// "https://ots.example.com/api".
func CreateSecret(baseURL, secret string, expirySeconds int) (string, error) {
	body, err := json.Marshal(createSecretRequest{
		Secret:        secret,
		ExpirySeconds: expirySeconds,
	})
	if err != nil {
		return "", fmt.Errorf("marshalling request: %w", err)
	}

	url := strings.TrimRight(baseURL, "/") + "/secrets"
	resp, err := http.Post(url, "application/json", bytes.NewReader(body)) //nolint:noctx
	if err != nil {
		return "", fmt.Errorf("posting secret: %w", err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", fmt.Errorf("reading response: %w", err)
	}

	if resp.StatusCode == http.StatusUnprocessableEntity {
		var errResp errorResponse
		if jsonErr := json.Unmarshal(respBody, &errResp); jsonErr == nil && errResp.Message != "" {
			return "", fmt.Errorf("validation error: %s", errResp.Message)
		}
		return "", fmt.Errorf("validation error (422)")
	}

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("unexpected status %d from server", resp.StatusCode)
	}

	var result createSecretResponse
	if err := json.Unmarshal(respBody, &result); err != nil {
		return "", fmt.Errorf("parsing response: %w", err)
	}

	if result.SecretID == "" {
		return "", fmt.Errorf("server returned empty secretId")
	}

	return result.SecretID, nil
}
