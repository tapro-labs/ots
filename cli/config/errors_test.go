package config

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// writeHomeConfig writes content to $HOME/.config/ots/config.json in a fresh
// temp HOME and returns the config file path.
func writeHomeConfig(t *testing.T, content string) string {
	t.Helper()
	home := t.TempDir()
	dir := filepath.Join(home, configDir)
	if err := os.MkdirAll(dir, 0755); err != nil {
		t.Fatal(err)
	}
	path := filepath.Join(dir, configFile)
	if err := os.WriteFile(path, []byte(content), 0600); err != nil {
		t.Fatal(err)
	}
	overrideHome(t, home)
	return path
}

func TestLoadURL_NoHome_ReturnsError(t *testing.T) {
	// os.UserHomeDir fails when $HOME is empty on Unix.
	overrideHome(t, "")
	if _, err := LoadURL(""); err == nil {
		t.Fatal("expected error when the home directory cannot be found")
	}
}

func TestLoadURL_NoHome_FlagStillWorks(t *testing.T) {
	overrideHome(t, "")
	got, err := LoadURL("https://x.example.com/api")
	if err != nil || got != "https://x.example.com/api" {
		t.Fatalf("got %q, %v", got, err)
	}
}

func TestLoadURL_NoConfigFile_ErrorExplainsHowToFix(t *testing.T) {
	overrideHome(t, t.TempDir())
	_, err := LoadURL("")
	if err == nil || !strings.Contains(err.Error(), "--url") || !strings.Contains(err.Error(), "config.json") {
		t.Fatalf("expected a helpful error, got %v", err)
	}
}

func TestLoadURL_MalformedConfig_ErrorNamesTheFile(t *testing.T) {
	// A bare JSON syntax error does not tell the user which file is broken.
	path := writeHomeConfig(t, `{"url": `)
	_, err := LoadURL("")
	if err == nil || !strings.Contains(err.Error(), path) {
		t.Fatalf("expected error to mention %s, got %v", path, err)
	}
}

func TestLoadURL_WrongJSONType_ReturnsError(t *testing.T) {
	path := writeHomeConfig(t, `{"url": 42}`)
	_, err := LoadURL("")
	if err == nil || !strings.Contains(err.Error(), path) {
		t.Fatalf("expected error to mention %s, got %v", path, err)
	}
}

func TestLoadURL_ConfigIsJSONArray_ReturnsError(t *testing.T) {
	writeHomeConfig(t, `["https://x.example.com"]`)
	if _, err := LoadURL(""); err == nil {
		t.Fatal("expected error for a JSON array config")
	}
}

func TestLoadURL_UnknownFieldsIgnored(t *testing.T) {
	writeHomeConfig(t, `{"url":"https://x.example.com/api","theme":"dark"}`)
	got, err := LoadURL("")
	if err != nil || got != "https://x.example.com/api" {
		t.Fatalf("got %q, %v", got, err)
	}
}

func TestLoadURL_ConfigPathIsDirectory_ReturnsError(t *testing.T) {
	home := t.TempDir()
	if err := os.MkdirAll(filepath.Join(home, configDir, configFile), 0755); err != nil {
		t.Fatal(err)
	}
	overrideHome(t, home)
	if _, err := LoadURL(""); err == nil {
		t.Fatal("expected error when config.json is a directory")
	}
}

func TestLoadURL_UnreadableConfig_ReturnsError(t *testing.T) {
	if os.Geteuid() == 0 {
		t.Skip("root ignores file permissions")
	}
	path := writeHomeConfig(t, `{"url":"https://x.example.com/api"}`)
	if err := os.Chmod(path, 0); err != nil {
		t.Fatal(err)
	}
	_, err := LoadURL("")
	if err == nil {
		t.Fatal("expected error for an unreadable config file")
	}
	if strings.Contains(err.Error(), "no OTS URL configured") {
		t.Errorf("permission error must not be reported as missing config: %v", err)
	}
}
