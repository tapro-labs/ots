package cmd

import (
	"fmt"
	"io"
	"net/url"
	"os"
	"slices"
	"strings"

	"github.com/KaloyanYosifov/ots/cli/api"
	"github.com/KaloyanYosifov/ots/cli/config"
	"github.com/KaloyanYosifov/ots/cli/crypto"
	"github.com/spf13/cobra"
)

// allowedExpiry mirrors the backend's ALLOWED_EXPIRY_SECONDS constant.
var allowedExpiry = []int{300, 1800, 3600, 14400, 43200, 86400, 172800, 432000}

// newRootCmd constructs and returns a fresh root command. Separating construction
// from the package-level variable makes the command independently testable.
func newRootCmd() *cobra.Command {
	var (
		flagText   string
		flagFile   string
		flagExpiry int
		flagURL    string
	)

	cmd := &cobra.Command{
		Use:   "ots",
		Short: "Create one-time secret links from the command line",
		Long: `ots encrypts your secret locally and sends only the ciphertext to the server.
The decryption key never leaves your machine — it is embedded in the URL fragment (#)
which is never sent to the server.

Configuration:
  Set the API URL in ~/.config/ots/config.json:  { "url": "https://your-ots.example.com/api" }
  Or pass --url per invocation.

Examples:
  ots --text "my secret password"
  echo "my secret" | ots
  ots --file ./secret.pdf --expiry 3600
  ots --text "hello" --url https://ots.example.com/api`,
		Version:      "1.0.0",
		SilenceUsage: true,
		// Reject `ots my secret`: without this the words are ignored and stdin is read.
		Args: cobra.NoArgs,
	}

	cmd.Flags().StringVarP(&flagText, "text", "t", "", "Secret text to encrypt")
	cmd.Flags().StringVarP(&flagFile, "file", "f", "", "Path to a file to encrypt and share")
	cmd.Flags().IntVarP(&flagExpiry, "expiry", "e", 86400, "Expiry in seconds (allowed: 300, 1800, 3600, 14400, 43200, 86400, 172800, 432000)")
	cmd.Flags().StringVar(&flagURL, "url", "", "OTS API base URL (overrides ~/.config/ots/config.json)")
	cmd.MarkFlagsMutuallyExclusive("text", "file")

	cmd.RunE = func(cmd *cobra.Command, _ []string) error {
		return runCmd(cmd, flagText, flagFile, flagExpiry, flagURL)
	}

	return cmd
}

// Execute is the binary entry point.
func Execute() {
	if err := newRootCmd().Execute(); err != nil {
		os.Exit(1)
	}
}

// runCmd contains all the orchestration logic, kept separate so it is easily
// unit-tested without having to re-parse cobra flags.
func runCmd(cmd *cobra.Command, flagText, flagFile string, flagExpiry int, flagURL string) error {
	// 1. Validate expiry
	if !slices.Contains(allowedExpiry, flagExpiry) {
		return fmt.Errorf("invalid --expiry %d\nAllowed values: 300 (5m), 1800 (30m), 3600 (1h), 14400 (4h), 43200 (12h), 86400 (24h), 172800 (2d), 432000 (5d)", flagExpiry)
	}

	// 2. Resolve API URL
	baseURL, err := config.LoadURL(flagURL)
	if err != nil {
		return err
	}

	// 3. Determine input source and encrypt
	var result crypto.EncryptResult

	switch {
	case flagFile != "":
		result, err = crypto.EncryptFile(flagFile)
		if err != nil {
			return fmt.Errorf("encrypting file: %w", err)
		}

	case flagText != "":
		result, err = crypto.EncryptText(flagText)
		if err != nil {
			return fmt.Errorf("encrypting text: %w", err)
		}

	default:
		// No flags — read from stdin (piped or interactive terminal)
		text, readErr := readInput(cmd.InOrStdin(), cmd.ErrOrStderr())
		if readErr != nil {
			return readErr
		}
		result, err = crypto.EncryptText(text)
		if err != nil {
			return fmt.Errorf("encrypting text: %w", err)
		}
	}

	// 4. POST to API
	secretID, err := api.CreateSecret(baseURL, result.Payload, flagExpiry)
	if err != nil {
		return err
	}

	// 5. Build the secret URL.
	// Strip the API path from baseURL to get just the origin,
	// e.g. "https://ots.example.com/api" → "https://ots.example.com"
	parsed, err := url.Parse(baseURL)
	if err != nil {
		return fmt.Errorf("parsing base URL: %w", err)
	}
	origin := parsed.Scheme + "://" + parsed.Host

	fragment, err := crypto.BuildFragment(result.JWK, result.SecretInfo, crypto.Checksum(result.Payload))
	if err != nil {
		return fmt.Errorf("building fragment: %w", err)
	}

	secretURL := origin + "/secret/" + secretID + "#" + fragment
	fmt.Fprintln(cmd.OutOrStdout(), secretURL)

	return nil
}

// readInput reads the secret text from in (stdin by default).
// If in is a TTY (interactive terminal), the user is prompted on prompt to type their secret.
// If in is piped, it reads all content directly. One trailing newline is removed.
func readInput(in io.Reader, prompt io.Writer) (string, error) {
	if f, ok := in.(*os.File); ok {
		stat, err := f.Stat()
		if err != nil {
			return "", fmt.Errorf("checking stdin: %w", err)
		}
		if stat.Mode()&os.ModeCharDevice != 0 {
			fmt.Fprint(prompt, "Enter secret (press Enter then Ctrl+D when done):\n")
		}
	}

	// io.ReadAll instead of bufio.Scanner: the scanner fails on lines over 64 KB.
	data, err := io.ReadAll(in)
	if err != nil {
		return "", fmt.Errorf("reading stdin: %w", err)
	}

	text := strings.TrimSuffix(string(data), "\n")
	text = strings.TrimSuffix(text, "\r")
	if text == "" {
		return "", fmt.Errorf("no secret provided: use --text, --file, or pipe input via stdin")
	}

	return text, nil
}
