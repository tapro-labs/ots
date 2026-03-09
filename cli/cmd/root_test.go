package cmd

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// executeCmd builds a fresh root command, runs it with the given args, captures
// stdout, and returns the trimmed output together with any RunE error.
func executeCmd(t *testing.T, args ...string) (string, error) {
	t.Helper()
	cmd := newRootCmd()
	buf := new(bytes.Buffer)
	cmd.SetOut(buf)
	cmd.SetErr(new(bytes.Buffer)) // suppress error output in test logs
	cmd.SetArgs(args)
	err := cmd.Execute()
	return strings.TrimSpace(buf.String()), err
}

// startFakeServer starts an httptest.Server that always responds with the given
// secretId and status code. It is closed automatically at the end of the test.
func startFakeServer(t *testing.T, secretID string, status int) *httptest.Server {
	t.Helper()
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(status)
		if status == http.StatusOK {
			json.NewEncoder(w).Encode(map[string]string{"secretId": secretID})
		} else {
			json.NewEncoder(w).Encode(map[string]string{"message": "error from server"})
		}
	}))
	t.Cleanup(srv.Close)
	return srv
}

// overrideHome sets the HOME env var for the duration of the test.
func overrideHome(t *testing.T, dir string) {
	t.Helper()
	orig, set := os.LookupEnv("HOME")
	os.Setenv("HOME", dir)
	t.Cleanup(func() {
		if set {
			os.Setenv("HOME", orig)
		} else {
			os.Unsetenv("HOME")
		}
	})
}

// ---------------------------------------------------------------------------
// Expiry validation
// ---------------------------------------------------------------------------

func TestRun_InvalidExpiry_ReturnsError(t *testing.T) {
	srv := startFakeServer(t, "irrelevant", http.StatusOK)
	_, err := executeCmd(t, "--text", "hello", "--url", srv.URL, "--expiry", "999")
	if err == nil {
		t.Fatal("expected error for invalid expiry, got nil")
	}
	if !strings.Contains(err.Error(), "999") {
		t.Errorf("expected error message to mention 999, got: %v", err)
	}
}

func TestRun_ValidExpiry_Succeeds(t *testing.T) {
	for _, expiry := range []string{"300", "1800", "3600", "14400", "43200", "86400", "172800", "432000"} {
		t.Run("expiry="+expiry, func(t *testing.T) {
			srv := startFakeServer(t, "test-id", http.StatusOK)
			out, err := executeCmd(t, "--text", "hello", "--url", srv.URL, "--expiry", expiry)
			if err != nil {
				t.Fatalf("expiry %s: unexpected error: %v", expiry, err)
			}
			if out == "" {
				t.Errorf("expiry %s: expected non-empty output", expiry)
			}
		})
	}
}

// ---------------------------------------------------------------------------
// URL resolution
// ---------------------------------------------------------------------------

func TestRun_NoURL_ReturnsError(t *testing.T) {
	// Point HOME at an empty dir so there is no config file.
	overrideHome(t, t.TempDir())
	_, err := executeCmd(t, "--text", "hello")
	if err == nil {
		t.Fatal("expected error when no URL is configured, got nil")
	}
}

func TestRun_URLFromConfigFile(t *testing.T) {
	srv := startFakeServer(t, "from-config", http.StatusOK)

	dir := t.TempDir()
	cfgDir := filepath.Join(dir, ".config", "ots")
	os.MkdirAll(cfgDir, 0755)
	b, _ := json.Marshal(map[string]string{"url": srv.URL})
	os.WriteFile(filepath.Join(cfgDir, "config.json"), b, 0600)
	overrideHome(t, dir)

	out, err := executeCmd(t, "--text", "hello")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !strings.Contains(out, "from-config") {
		t.Errorf("expected secretId in URL, got: %q", out)
	}
}

// ---------------------------------------------------------------------------
// Input sources
// ---------------------------------------------------------------------------

func TestRun_TextFlag(t *testing.T) {
	srv := startFakeServer(t, "text-id", http.StatusOK)
	out, err := executeCmd(t, "--text", "my secret", "--url", srv.URL)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !strings.Contains(out, "text-id") {
		t.Errorf("expected secretId in output, got: %q", out)
	}
}

func TestRun_FileFlag(t *testing.T) {
	srv := startFakeServer(t, "file-id", http.StatusOK)

	f, _ := os.CreateTemp(t.TempDir(), "secret-*.txt")
	f.WriteString("file content here")
	f.Close()

	out, err := executeCmd(t, "--file", f.Name(), "--url", srv.URL)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !strings.Contains(out, "file-id") {
		t.Errorf("expected secretId in output, got: %q", out)
	}
}

func TestRun_FileFlag_NonExistentFile_ReturnsError(t *testing.T) {
	srv := startFakeServer(t, "irrelevant", http.StatusOK)
	_, err := executeCmd(t, "--file", "/nonexistent/file.txt", "--url", srv.URL)
	if err == nil {
		t.Fatal("expected error for non-existent file, got nil")
	}
}

// ---------------------------------------------------------------------------
// Output format
// ---------------------------------------------------------------------------

func TestRun_OutputIsValidSecretURL(t *testing.T) {
	srv := startFakeServer(t, "my-secret-id", http.StatusOK)
	out, err := executeCmd(t, "--text", "hello", "--url", srv.URL)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if !strings.HasPrefix(out, "http://") && !strings.HasPrefix(out, "https://") {
		t.Errorf("output should be a URL, got: %q", out)
	}
	if !strings.Contains(out, "/secret/my-secret-id") {
		t.Errorf("expected /secret/my-secret-id in URL, got: %q", out)
	}
	if !strings.Contains(out, "#") {
		t.Errorf("expected URL fragment (#), got: %q", out)
	}
	parts := strings.SplitN(out, "#", 2)
	if len(parts) < 2 || parts[1] == "" {
		t.Errorf("expected non-empty fragment, got: %q", out)
	}
}

func TestRun_APIPathStrippedFromOrigin(t *testing.T) {
	// When baseURL is "http://host/api", the secret URL should use "http://host"
	// as the origin, not include "/api" in the path.
	srv := startFakeServer(t, "strip-id", http.StatusOK)
	out, err := executeCmd(t, "--text", "hello", "--url", srv.URL+"/api")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if strings.Contains(out, "/api/secret/") {
		t.Errorf("API path should be stripped from origin, got: %q", out)
	}
	if !strings.Contains(out, "/secret/strip-id") {
		t.Errorf("expected /secret/strip-id in URL, got: %q", out)
	}
}

// ---------------------------------------------------------------------------
// Server errors propagate correctly
// ---------------------------------------------------------------------------

func TestRun_ServerReturns422_ReturnsError(t *testing.T) {
	srv := startFakeServer(t, "", http.StatusUnprocessableEntity)
	_, err := executeCmd(t, "--text", "hello", "--url", srv.URL)
	if err == nil {
		t.Fatal("expected error for 422, got nil")
	}
}

func TestRun_ServerReturns500_ReturnsError(t *testing.T) {
	srv := startFakeServer(t, "", http.StatusInternalServerError)
	_, err := executeCmd(t, "--text", "hello", "--url", srv.URL)
	if err == nil {
		t.Fatal("expected error for 500, got nil")
	}
}
