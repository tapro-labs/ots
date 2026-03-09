package config

import (
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
)

const configDir = ".config/ots"
const configFile = "config.json"

type Config struct {
	URL string `json:"url"`
}

func configFilePath() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}

	return filepath.Join(home, configDir, configFile), nil
}

// LoadURL resolves the OTS API base URL using the following precedence:
//  1. flagURL (--url flag)
//  2. ~/.config/ots/config.json
//  3. Error
func LoadURL(flagURL string) (string, error) {
	if flagURL != "" {
		return flagURL, nil
	}

	path, err := configFilePath()
	if err != nil {
		return "", err
	}

	f, err := os.Open(path)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return "", errors.New("no OTS URL configured: use --url or set \"url\" in ~/.config/ots/config.json")
		}
		return "", err
	}
	defer f.Close()

	var cfg Config
	if err := json.NewDecoder(f).Decode(&cfg); err != nil {
		return "", err
	}

	if cfg.URL == "" {
		return "", errors.New("no OTS URL configured: use --url or set \"url\" in ~/.config/ots/config.json")
	}

	return cfg.URL, nil
}
