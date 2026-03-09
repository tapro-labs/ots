package config

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

// writeConfigFile creates a temporary config file with the given content and
// returns the directory it was written to, along with a cleanup function.
func writeConfigFile(t *testing.T, content string) string {
	t.Helper()
	dir := t.TempDir()
	path := filepath.Join(dir, configFile)
	if err := os.WriteFile(path, []byte(content), 0600); err != nil {
		t.Fatalf("writing temp config: %v", err)
	}
	return dir
}

// overrideConfigDir monkey-patches the configFilePath function for the duration
// of a test by redirecting the home directory lookup via HOME env var.
func overrideHome(t *testing.T, home string) {
	t.Helper()
	orig, set := os.LookupEnv("HOME")
	os.Setenv("HOME", home)
	t.Cleanup(func() {
		if set {
			os.Setenv("HOME", orig)
		} else {
			os.Unsetenv("HOME")
		}
	})
}

func TestLoadURL_FlagTakesPrecedence(t *testing.T) {
	// Even if a config file exists, --url flag should win.
	dir := t.TempDir()
	configPath := filepath.Join(dir, configDir)
	if err := os.MkdirAll(configPath, 0755); err != nil {
		t.Fatal(err)
	}
	b, _ := json.Marshal(Config{URL: "https://from-file.example.com/api"})
	os.WriteFile(filepath.Join(configPath, configFile), b, 0600)

	overrideHome(t, dir)

	got, err := LoadURL("https://from-flag.example.com/api")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if got != "https://from-flag.example.com/api" {
		t.Errorf("expected flag URL, got %q", got)
	}
}

func TestLoadURL_ReadsConfigFile(t *testing.T) {
	dir := t.TempDir()
	configPath := filepath.Join(dir, configDir)
	if err := os.MkdirAll(configPath, 0755); err != nil {
		t.Fatal(err)
	}
	b, _ := json.Marshal(Config{URL: "https://from-file.example.com/api"})
	os.WriteFile(filepath.Join(configPath, configFile), b, 0600)

	overrideHome(t, dir)

	got, err := LoadURL("")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if got != "https://from-file.example.com/api" {
		t.Errorf("expected config file URL, got %q", got)
	}
}

func TestLoadURL_NoConfigFile_NoFlag_ReturnsError(t *testing.T) {
	dir := t.TempDir() // empty — no config file
	overrideHome(t, dir)

	_, err := LoadURL("")
	if err == nil {
		t.Fatal("expected error, got nil")
	}
}

func TestLoadURL_EmptyURLInConfigFile_ReturnsError(t *testing.T) {
	dir := t.TempDir()
	configPath := filepath.Join(dir, configDir)
	if err := os.MkdirAll(configPath, 0755); err != nil {
		t.Fatal(err)
	}
	b, _ := json.Marshal(Config{URL: ""})
	os.WriteFile(filepath.Join(configPath, configFile), b, 0600)

	overrideHome(t, dir)

	_, err := LoadURL("")
	if err == nil {
		t.Fatal("expected error for empty URL in config, got nil")
	}
}

func TestLoadURL_MalformedConfigFile_ReturnsError(t *testing.T) {
	dir := t.TempDir()
	configPath := filepath.Join(dir, configDir)
	if err := os.MkdirAll(configPath, 0755); err != nil {
		t.Fatal(err)
	}
	os.WriteFile(filepath.Join(configPath, configFile), []byte("not json{{{{"), 0600)

	overrideHome(t, dir)

	_, err := LoadURL("")
	if err == nil {
		t.Fatal("expected error for malformed JSON, got nil")
	}
}
