package api

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

// startServer spins up an httptest.Server with the given handler and returns it.
// The server is shut down automatically at the end of the test.
func startServer(t *testing.T, handler http.HandlerFunc) *httptest.Server {
	t.Helper()
	srv := httptest.NewServer(handler)
	t.Cleanup(srv.Close)
	return srv
}

func TestCreateSecret_Success(t *testing.T) {
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			t.Errorf("expected POST, got %s", r.Method)
		}
		if r.URL.Path != "/secrets" {
			t.Errorf("expected path /secrets, got %s", r.URL.Path)
		}
		if ct := r.Header.Get("Content-Type"); ct != "application/json" {
			t.Errorf("expected Content-Type application/json, got %q", ct)
		}

		var req createSecretRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			t.Errorf("decoding request body: %v", err)
		}
		if req.Secret != "encrypted-payload" {
			t.Errorf("expected secret=encrypted-payload, got %q", req.Secret)
		}
		if req.ExpirySeconds != 3600 {
			t.Errorf("expected expiry_seconds=3600, got %d", req.ExpirySeconds)
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(createSecretResponse{SecretID: "abc-123"})
	})

	secretID, err := CreateSecret(srv.URL, "encrypted-payload", 3600)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if secretID != "abc-123" {
		t.Errorf("expected secretId=abc-123, got %q", secretID)
	}
}

func TestCreateSecret_TrimsTrailingSlashFromBaseURL(t *testing.T) {
	called := false
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		called = true
		if r.URL.Path != "/secrets" {
			t.Errorf("expected /secrets, got %q", r.URL.Path)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(createSecretResponse{SecretID: "id-1"})
	})

	// URL with trailing slash
	_, err := CreateSecret(srv.URL+"/", "data", 86400)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !called {
		t.Error("server was never called")
	}
}

func TestCreateSecret_422_ReturnsValidationError(t *testing.T) {
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusUnprocessableEntity)
		json.NewEncoder(w).Encode(errorResponse{Message: "Invalid expiry_seconds value: 999"})
	})

	_, err := CreateSecret(srv.URL, "data", 999)
	if err == nil {
		t.Fatal("expected error for 422, got nil")
	}
	if !contains(err.Error(), "Invalid expiry_seconds value: 999") {
		t.Errorf("expected validation message in error, got: %v", err)
	}
}

func TestCreateSecret_422_NoBody_ReturnsGenericError(t *testing.T) {
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnprocessableEntity)
	})

	_, err := CreateSecret(srv.URL, "data", 999)
	if err == nil {
		t.Fatal("expected error for 422, got nil")
	}
}

func TestCreateSecret_500_ReturnsError(t *testing.T) {
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	})

	_, err := CreateSecret(srv.URL, "data", 86400)
	if err == nil {
		t.Fatal("expected error for 500, got nil")
	}
}

func TestCreateSecret_404_ReturnsError(t *testing.T) {
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
	})

	_, err := CreateSecret(srv.URL, "data", 86400)
	if err == nil {
		t.Fatal("expected error for 404, got nil")
	}
}

func TestCreateSecret_EmptySecretID_ReturnsError(t *testing.T) {
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(createSecretResponse{SecretID: ""})
	})

	_, err := CreateSecret(srv.URL, "data", 86400)
	if err == nil {
		t.Fatal("expected error for empty secretId, got nil")
	}
}

func TestCreateSecret_MalformedResponseJSON_ReturnsError(t *testing.T) {
	srv := startServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte("not json{{"))
	})

	_, err := CreateSecret(srv.URL, "data", 86400)
	if err == nil {
		t.Fatal("expected error for malformed JSON response, got nil")
	}
}

func TestCreateSecret_UnreachableServer_ReturnsError(t *testing.T) {
	_, err := CreateSecret("http://127.0.0.1:1", "data", 86400)
	if err == nil {
		t.Fatal("expected error for unreachable server, got nil")
	}
}

func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(substr) == 0 ||
		func() bool {
			for i := 0; i <= len(s)-len(substr); i++ {
				if s[i:i+len(substr)] == substr {
					return true
				}
			}
			return false
		}())
}
