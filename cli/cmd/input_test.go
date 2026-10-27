package cmd

import (
	"bytes"
	"crypto/aes"
	"crypto/cipher"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/KaloyanYosifov/ots/cli/crypto"
)

// capturedRequest is what the fake server received.
type capturedRequest struct {
	Secret        string `json:"secret"`
	ExpirySeconds int    `json:"expiry_seconds"`
}

// startCapturingServer returns a server that records the request body and
// answers with the given secretId.
func startCapturingServer(t *testing.T, secretID string) (*httptest.Server, *capturedRequest) {
	t.Helper()
	got := &capturedRequest{}
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if err := json.NewDecoder(r.Body).Decode(got); err != nil {
			t.Errorf("decoding request: %v", err)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]string{"secretId": secretID})
	}))
	t.Cleanup(srv.Close)
	return srv, got
}

// executeCmdWithStdin runs the root command with the given stdin content.
func executeCmdWithStdin(t *testing.T, stdin io.Reader, args ...string) (string, string, error) {
	t.Helper()
	cmd := newRootCmd()
	out, errOut := new(bytes.Buffer), new(bytes.Buffer)
	cmd.SetOut(out)
	cmd.SetErr(errOut)
	cmd.SetIn(stdin)
	cmd.SetArgs(args)
	err := cmd.Execute()
	return strings.TrimSpace(out.String()), errOut.String(), err
}

// decryptFromURL takes the printed secret URL and the payload the server got,
// checks the fragment checksum and decrypts it the way Secret.vue does. It returns the plaintext and the
// secretInfo from the fragment.
func decryptFromURL(t *testing.T, secretURL, payload string) ([]byte, crypto.SecretInfo) {
	t.Helper()
	_, fragment, ok := strings.Cut(secretURL, "#")
	if !ok {
		t.Fatalf("no fragment in %q", secretURL)
	}
	raw, err := base64.StdEncoding.DecodeString(fragment)
	if err != nil {
		t.Fatalf("fragment is not base64: %v", err)
	}
	var details struct {
		SecretKey  crypto.JWK        `json:"secretKey"`
		SecretInfo crypto.SecretInfo `json:"secretInfo"`
		Checksum   string            `json:"checksum"`
	}
	if err := json.Unmarshal(raw, &details); err != nil {
		t.Fatalf("fragment JSON: %v", err)
	}
	sum := sha256.Sum256([]byte(payload))
	if want := hex.EncodeToString(sum[:]); details.Checksum != want {
		t.Fatalf("fragment checksum %q does not match the sha256 of the sent payload %q", details.Checksum, want)
	}
	key, err := base64.RawURLEncoding.DecodeString(details.SecretKey.K)
	if err != nil {
		t.Fatalf("JWK k: %v", err)
	}
	block, _ := aes.NewCipher(key)
	gcm, _ := cipher.NewGCM(block)

	var out []byte
	for _, chunk := range strings.Split(strings.TrimSuffix(payload, "$_$"), "$_$") {
		sealed, err := base64.StdEncoding.DecodeString(chunk)
		if err != nil {
			t.Fatalf("chunk base64: %v", err)
		}
		plain, err := gcm.Open(nil, sealed[:12], sealed[12:], nil)
		if err != nil {
			t.Fatalf("decrypt: %v", err)
		}
		data, err := base64.StdEncoding.DecodeString(string(plain))
		if err != nil {
			t.Fatalf("inner base64: %v", err)
		}
		out = append(out, data...)
	}
	return out, details.SecretInfo
}

// ---------------------------------------------------------------------------
// End to end: what is printed decrypts what was sent
// ---------------------------------------------------------------------------

func TestRun_TextFlag_EndToEndDecrypts(t *testing.T) {
	srv, got := startCapturingServer(t, "e2e-id")
	out, _, err := executeCmdWithStdin(t, strings.NewReader(""), "-t", "s3cr3t ünïcode", "-e", "300", "--url", srv.URL+"/api")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if got.ExpirySeconds != 300 {
		t.Errorf("expected expiry_seconds=300, got %d", got.ExpirySeconds)
	}
	if !strings.HasPrefix(out, srv.URL+"/secret/e2e-id#") {
		t.Errorf("unexpected URL %q (origin with port must be kept)", out)
	}
	plain, info := decryptFromURL(t, out, got.Secret)
	if string(plain) != "s3cr3t ünïcode" {
		t.Errorf("decrypted %q", plain)
	}
	if info.Type != "plain" || info.Info != nil {
		t.Errorf("unexpected secretInfo %+v", info)
	}
}

func TestRun_FileFlag_EndToEndDecrypts(t *testing.T) {
	srv, got := startCapturingServer(t, "file-e2e")
	content := bytes.Repeat([]byte{0, 1, 2, 250}, 10000)
	path := filepath.Join(t.TempDir(), "dump.bin")
	if err := os.WriteFile(path, content, 0600); err != nil {
		t.Fatal(err)
	}
	out, _, err := executeCmdWithStdin(t, strings.NewReader(""), "-f", path, "--url", srv.URL)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if got.ExpirySeconds != 86400 {
		t.Errorf("expected default expiry 86400, got %d", got.ExpirySeconds)
	}
	plain, info := decryptFromURL(t, out, got.Secret)
	if !bytes.Equal(plain, content) {
		t.Error("decrypted file content differs")
	}
	if info.Type != "file" || info.Info == nil || info.Info.Name != "dump.bin" {
		t.Errorf("unexpected secretInfo %+v", info)
	}
}

// ---------------------------------------------------------------------------
// Stdin input
// ---------------------------------------------------------------------------

func TestRun_Stdin(t *testing.T) {
	cases := []struct {
		name, stdin, want string
	}{
		{"single line with newline", "hello\n", "hello"},
		{"single line without newline", "hello", "hello"},
		{"multi line", "line1\nline2\n", "line1\nline2"},
		{"keeps inner blank lines", "a\n\nb\n", "a\n\nb"},
		{"keeps one of two trailing newlines", "a\n\n", "a\n"},
		{"strips CRLF line ending", "hello\r\n", "hello"},
		{"keeps surrounding spaces", "  pad  \n", "  pad  "},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			srv, got := startCapturingServer(t, "stdin-id")
			out, _, err := executeCmdWithStdin(t, strings.NewReader(tc.stdin), "--url", srv.URL)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			plain, _ := decryptFromURL(t, out, got.Secret)
			if string(plain) != tc.want {
				t.Errorf("got %q, want %q", plain, tc.want)
			}
		})
	}
}

func TestRun_Stdin_LongSingleLine(t *testing.T) {
	// bufio.Scanner rejects lines longer than 64 KB ("token too long").
	// A long one-line secret (for example a base64 blob) must still work.
	long := strings.Repeat("k", 100*1024)
	srv, got := startCapturingServer(t, "long-id")
	out, _, err := executeCmdWithStdin(t, strings.NewReader(long+"\n"), "--url", srv.URL)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	plain, _ := decryptFromURL(t, out, got.Secret)
	if string(plain) != long {
		t.Errorf("decrypted %d bytes, want %d", len(plain), len(long))
	}
}

func TestRun_Stdin_Empty_ReturnsError(t *testing.T) {
	for name, stdin := range map[string]string{"empty": "", "only newline": "\n"} {
		t.Run(name, func(t *testing.T) {
			srv, _ := startCapturingServer(t, "unused")
			_, _, err := executeCmdWithStdin(t, strings.NewReader(stdin), "--url", srv.URL)
			if err == nil || !strings.Contains(err.Error(), "no secret provided") {
				t.Fatalf("expected 'no secret provided' error, got %v", err)
			}
		})
	}
}

func TestRun_Stdin_ReadError(t *testing.T) {
	srv, _ := startCapturingServer(t, "unused")
	_, _, err := executeCmdWithStdin(t, io.MultiReader(strings.NewReader("part"), failingReader{}), "--url", srv.URL)
	if err == nil || !strings.Contains(err.Error(), "reading stdin") {
		t.Fatalf("expected reading stdin error, got %v", err)
	}
}

type failingReader struct{}

func (failingReader) Read([]byte) (int, error) { return 0, io.ErrClosedPipe }

func TestRun_Stdin_FromPipeFile(t *testing.T) {
	// An *os.File that is not a terminal (pipe) is read without a prompt.
	r, w, err := os.Pipe()
	if err != nil {
		t.Fatal(err)
	}
	go func() {
		w.WriteString("piped secret\n")
		w.Close()
	}()
	defer r.Close()

	srv, got := startCapturingServer(t, "pipe-id")
	out, stderr, err := executeCmdWithStdin(t, r, "--url", srv.URL)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if stderr != "" {
		t.Errorf("expected no prompt for piped input, got %q", stderr)
	}
	plain, _ := decryptFromURL(t, out, got.Secret)
	if string(plain) != "piped secret" {
		t.Errorf("got %q", plain)
	}
}

func TestRun_Stdin_ClosedFile_ReturnsError(t *testing.T) {
	f, err := os.CreateTemp(t.TempDir(), "stdin")
	if err != nil {
		t.Fatal(err)
	}
	f.Close() // Stat on a closed file fails
	srv, _ := startCapturingServer(t, "unused")
	_, _, err = executeCmdWithStdin(t, f, "--url", srv.URL)
	if err == nil || !strings.Contains(err.Error(), "checking stdin") {
		t.Fatalf("expected checking stdin error, got %v", err)
	}
}

// ---------------------------------------------------------------------------
// Flag handling
// ---------------------------------------------------------------------------

func TestRun_TextAndFile_AreMutuallyExclusive(t *testing.T) {
	srv, _ := startCapturingServer(t, "unused")
	path := filepath.Join(t.TempDir(), "a.txt")
	os.WriteFile(path, []byte("file"), 0600)
	_, _, err := executeCmdWithStdin(t, strings.NewReader(""), "--text", "x", "--file", path, "--url", srv.URL)
	if err == nil {
		t.Fatal("expected an error when both --text and --file are given")
	}
}

func TestRun_PositionalArgs_ReturnsError(t *testing.T) {
	// `ots my secret` must not silently ignore the words and wait on stdin.
	srv, _ := startCapturingServer(t, "unused")
	_, _, err := executeCmdWithStdin(t, strings.NewReader("from stdin"), "my", "secret", "--url", srv.URL)
	if err == nil {
		t.Fatal("expected an error for positional arguments")
	}
}

func TestRun_UnknownFlag_ReturnsError(t *testing.T) {
	_, _, err := executeCmdWithStdin(t, strings.NewReader(""), "--nope")
	if err == nil {
		t.Fatal("expected an error for an unknown flag")
	}
}

func TestRun_NonNumericExpiry_ReturnsError(t *testing.T) {
	_, _, err := executeCmdWithStdin(t, strings.NewReader(""), "-t", "x", "-e", "1h", "--url", "http://unused")
	if err == nil {
		t.Fatal("expected an error for a non-numeric --expiry")
	}
}

func TestRun_InvalidExpiry_DoesNotCallServer(t *testing.T) {
	called := false
	srv := httptest.NewServer(http.HandlerFunc(func(http.ResponseWriter, *http.Request) { called = true }))
	defer srv.Close()
	_, _, err := executeCmdWithStdin(t, strings.NewReader(""), "-t", "x", "-e", "0", "--url", srv.URL)
	if err == nil {
		t.Fatal("expected error")
	}
	if called {
		t.Error("server must not be called when --expiry is invalid")
	}
}

func TestRun_Version(t *testing.T) {
	out, _, err := executeCmdWithStdin(t, strings.NewReader(""), "--version")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !strings.Contains(out, "1.0.0") {
		t.Errorf("expected version in output, got %q", out)
	}
}

func TestRun_URLFlagOverridesConfigFile(t *testing.T) {
	dir := t.TempDir()
	cfgDir := filepath.Join(dir, ".config", "ots")
	os.MkdirAll(cfgDir, 0755)
	os.WriteFile(filepath.Join(cfgDir, "config.json"), []byte(`{"url":"http://127.0.0.1:1"}`), 0600)
	overrideHome(t, dir)

	srv, _ := startCapturingServer(t, "flag-wins")
	out, _, err := executeCmdWithStdin(t, strings.NewReader(""), "-t", "x", "--url", srv.URL)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !strings.Contains(out, "flag-wins") {
		t.Errorf("got %q", out)
	}
}

func TestRun_MalformedConfigFile_ReturnsError(t *testing.T) {
	dir := t.TempDir()
	cfgDir := filepath.Join(dir, ".config", "ots")
	os.MkdirAll(cfgDir, 0755)
	os.WriteFile(filepath.Join(cfgDir, "config.json"), []byte(`{not json`), 0600)
	overrideHome(t, dir)

	_, _, err := executeCmdWithStdin(t, strings.NewReader(""), "-t", "x")
	if err == nil {
		t.Fatal("expected error for malformed config file")
	}
}

func TestRun_ServerError_MessageSurfaced(t *testing.T) {
	srv := startFakeServer(t, "", http.StatusUnprocessableEntity)
	_, _, err := executeCmdWithStdin(t, strings.NewReader(""), "-t", "x", "--url", srv.URL)
	if err == nil || !strings.Contains(err.Error(), "error from server") {
		t.Fatalf("expected server message in error, got %v", err)
	}
}

func TestRun_EncryptFileError_IsWrapped(t *testing.T) {
	srv, _ := startCapturingServer(t, "unused")
	path := filepath.Join(t.TempDir(), "empty.txt")
	os.WriteFile(path, nil, 0600)
	_, _, err := executeCmdWithStdin(t, strings.NewReader(""), "-f", path, "--url", srv.URL)
	if err == nil || !strings.Contains(err.Error(), "encrypting file") {
		t.Fatalf("expected encrypting file error, got %v", err)
	}
}
