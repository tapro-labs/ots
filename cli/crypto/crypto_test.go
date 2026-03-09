package crypto

import (
	"crypto/aes"
	"crypto/cipher"
	"encoding/base64"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// ---------------------------------------------------------------------------
// generateKey
// ---------------------------------------------------------------------------

func TestGenerateKey_ProducesValidJWK(t *testing.T) {
	keyBytes, jwk, err := generateKey()
	if err != nil {
		t.Fatalf("generateKey error: %v", err)
	}

	if len(keyBytes) != 32 {
		t.Errorf("expected 32-byte key, got %d", len(keyBytes))
	}
	if jwk.Kty != "oct" {
		t.Errorf("expected kty=oct, got %q", jwk.Kty)
	}
	if jwk.Alg != "A256GCM" {
		t.Errorf("expected alg=A256GCM, got %q", jwk.Alg)
	}
	if !jwk.Ext {
		t.Error("expected ext=true")
	}
	if len(jwk.KeyOps) != 2 {
		t.Errorf("expected 2 key_ops, got %d", len(jwk.KeyOps))
	}

	// k must be base64url (no padding) of the raw key bytes
	decoded, err := base64.RawURLEncoding.DecodeString(jwk.K)
	if err != nil {
		t.Fatalf("JWK k is not valid base64url: %v", err)
	}
	if len(decoded) != 32 {
		t.Errorf("decoded k should be 32 bytes, got %d", len(decoded))
	}
	if string(decoded) != string(keyBytes) {
		t.Error("JWK k does not match raw key bytes")
	}
}

func TestGenerateKey_ProducesUniqueKeys(t *testing.T) {
	key1, _, _ := generateKey()
	key2, _, _ := generateKey()
	if string(key1) == string(key2) {
		t.Error("two consecutive generateKey calls produced identical keys")
	}
}

// ---------------------------------------------------------------------------
// encryptChunk / round-trip decryption
// ---------------------------------------------------------------------------

func TestEncryptChunk_ProducesValidBase64(t *testing.T) {
	key, _, _ := generateKey()
	encrypted, err := encryptChunk(key, "aGVsbG8gd29ybGQ=") // base64("hello world")
	if err != nil {
		t.Fatalf("encryptChunk error: %v", err)
	}
	if _, err := base64.StdEncoding.DecodeString(encrypted); err != nil {
		t.Errorf("output is not valid base64: %v", err)
	}
}

func TestEncryptChunk_RoundTrip(t *testing.T) {
	key, _, _ := generateKey()
	plaintext := "aGVsbG8gd29ybGQ=" // base64("hello world")

	encrypted, err := encryptChunk(key, plaintext)
	if err != nil {
		t.Fatalf("encryptChunk: %v", err)
	}

	// Manually decrypt to verify the round-trip.
	// Structure: base64(iv [12] || ciphertext+tag)
	raw, err := base64.StdEncoding.DecodeString(encrypted)
	if err != nil {
		t.Fatalf("decoding encrypted chunk: %v", err)
	}

	block, _ := aes.NewCipher(key)
	gcm, _ := cipher.NewGCM(block)
	nonceSize := gcm.NonceSize() // 12

	if len(raw) < nonceSize {
		t.Fatalf("encrypted output too short: %d bytes", len(raw))
	}

	iv, ciphertext := raw[:nonceSize], raw[nonceSize:]
	decrypted, err := gcm.Open(nil, iv, ciphertext, nil)
	if err != nil {
		t.Fatalf("decryption failed: %v", err)
	}

	if string(decrypted) != plaintext {
		t.Errorf("round-trip mismatch: got %q, want %q", decrypted, plaintext)
	}
}

func TestEncryptChunk_IsDeterministicallyRandom(t *testing.T) {
	// Same key + same input must produce different ciphertext each call (fresh IV).
	key, _, _ := generateKey()
	data := "c29tZSBkYXRh"
	enc1, _ := encryptChunk(key, data)
	enc2, _ := encryptChunk(key, data)
	if enc1 == enc2 {
		t.Error("two encryptions of the same data produced identical output (IV reuse)")
	}
}

// ---------------------------------------------------------------------------
// EncryptText
// ---------------------------------------------------------------------------

func TestEncryptText_SecretInfoIsPlain(t *testing.T) {
	result, err := EncryptText("my secret")
	if err != nil {
		t.Fatalf("EncryptText: %v", err)
	}
	if result.SecretInfo.Type != "plain" {
		t.Errorf("expected type=plain, got %q", result.SecretInfo.Type)
	}
	if result.SecretInfo.Info != nil {
		t.Error("expected Info to be nil for plain text")
	}
}

func TestEncryptText_PayloadNotEmpty(t *testing.T) {
	result, err := EncryptText("hello")
	if err != nil {
		t.Fatalf("EncryptText: %v", err)
	}
	if result.Payload == "" {
		t.Error("expected non-empty payload")
	}
}

func TestEncryptText_PayloadHasTrailingSeparator(t *testing.T) {
	// The browser's EncryptStreamTransformer appends '$_$' after every chunk
	// including the last one. The Go implementation must match this exactly.
	result, err := EncryptText("hello world")
	if err != nil {
		t.Fatalf("EncryptText: %v", err)
	}
	if !strings.HasSuffix(result.Payload, "$_$") {
		t.Errorf("payload must end with '$_$', got: %q", result.Payload)
	}
}

func TestEncryptText_ChunksAreValidBase64(t *testing.T) {
	result, err := EncryptText("hello world")
	if err != nil {
		t.Fatalf("EncryptText: %v", err)
	}
	// Trim trailing separator before splitting so we don't get a spurious empty chunk.
	payload := strings.TrimSuffix(result.Payload, "$_$")
	for i, chunk := range strings.Split(payload, "$_$") {
		if _, err := base64.StdEncoding.DecodeString(chunk); err != nil {
			t.Errorf("chunk %d is not valid base64: %v", i, err)
		}
	}
}

func TestEncryptText_LargeInput_ProducesMultipleChunks(t *testing.T) {
	// 40 KB input must produce at least 3 chunks (16KB + 16KB + 8KB)
	big := strings.Repeat("x", 40*1024)
	result, err := EncryptText(big)
	if err != nil {
		t.Fatalf("EncryptText: %v", err)
	}
	// Trim trailing separator before counting.
	payload := strings.TrimSuffix(result.Payload, "$_$")
	chunks := strings.Split(payload, "$_$")
	if len(chunks) < 3 {
		t.Errorf("expected >=3 chunks for 40KB input, got %d", len(chunks))
	}
}

func TestEncryptText_EmptyString(t *testing.T) {
	// An empty string should either succeed with a valid (empty) encryption or
	// produce an empty payload — it must not panic or error.
	_, err := EncryptText("")
	if err != nil {
		t.Errorf("EncryptText(\"\") returned unexpected error: %v", err)
	}
}

func TestEncryptText_JWKIsValid(t *testing.T) {
	result, err := EncryptText("test")
	if err != nil {
		t.Fatalf("EncryptText: %v", err)
	}
	decoded, err := base64.RawURLEncoding.DecodeString(result.JWK.K)
	if err != nil {
		t.Fatalf("JWK k not valid base64url: %v", err)
	}
	if len(decoded) != 32 {
		t.Errorf("expected 32-byte key material, got %d", len(decoded))
	}
}

// ---------------------------------------------------------------------------
// EncryptFile
// ---------------------------------------------------------------------------

func writeTempFile(t *testing.T, name, content string) string {
	t.Helper()
	path := filepath.Join(t.TempDir(), name)
	if err := os.WriteFile(path, []byte(content), 0600); err != nil {
		t.Fatalf("writing temp file: %v", err)
	}
	return path
}

func TestEncryptFile_SecretInfoIsFile(t *testing.T) {
	path := writeTempFile(t, "secret.txt", "file content here")
	result, err := EncryptFile(path)
	if err != nil {
		t.Fatalf("EncryptFile: %v", err)
	}
	if result.SecretInfo.Type != "file" {
		t.Errorf("expected type=file, got %q", result.SecretInfo.Type)
	}
	if result.SecretInfo.Info == nil {
		t.Fatal("expected FileInfo to be set")
	}
	if result.SecretInfo.Info.Name != "secret.txt" {
		t.Errorf("expected name=secret.txt, got %q", result.SecretInfo.Info.Name)
	}
	if result.SecretInfo.Info.Type == "" {
		t.Error("expected non-empty MIME type")
	}
}

func TestEncryptFile_PayloadNotEmpty(t *testing.T) {
	path := writeTempFile(t, "data.txt", "some data")
	result, err := EncryptFile(path)
	if err != nil {
		t.Fatalf("EncryptFile: %v", err)
	}
	if result.Payload == "" {
		t.Error("expected non-empty payload")
	}
}

func TestEncryptFile_PayloadHasTrailingSeparator(t *testing.T) {
	path := writeTempFile(t, "data.txt", "some file data")
	result, err := EncryptFile(path)
	if err != nil {
		t.Fatalf("EncryptFile: %v", err)
	}
	if !strings.HasSuffix(result.Payload, "$_$") {
		t.Errorf("payload must end with '$_$', got: %q", result.Payload)
	}
}

func TestEncryptFile_ChunksAreValidBase64(t *testing.T) {
	path := writeTempFile(t, "data.txt", "some file data")
	result, err := EncryptFile(path)
	if err != nil {
		t.Fatalf("EncryptFile: %v", err)
	}
	payload := strings.TrimSuffix(result.Payload, "$_$")
	for i, chunk := range strings.Split(payload, "$_$") {
		if _, err := base64.StdEncoding.DecodeString(chunk); err != nil {
			t.Errorf("chunk %d is not valid base64: %v", i, err)
		}
	}
}

func TestEncryptFile_LargeFile_ProducesMultipleChunks(t *testing.T) {
	content := strings.Repeat("y", 40*1024)
	path := writeTempFile(t, "large.txt", content)
	result, err := EncryptFile(path)
	if err != nil {
		t.Fatalf("EncryptFile: %v", err)
	}
	payload := strings.TrimSuffix(result.Payload, "$_$")
	chunks := strings.Split(payload, "$_$")
	if len(chunks) < 3 {
		t.Errorf("expected >=3 chunks for 40KB file, got %d", len(chunks))
	}
}

func TestEncryptFile_NonExistentPath_ReturnsError(t *testing.T) {
	_, err := EncryptFile("/nonexistent/path/file.txt")
	if err == nil {
		t.Error("expected error for non-existent file, got nil")
	}
}

func TestEncryptFile_FileNamePreserved(t *testing.T) {
	path := writeTempFile(t, "myreport.pdf", "%PDF-1.4 fake pdf content")
	result, err := EncryptFile(path)
	if err != nil {
		t.Fatalf("EncryptFile: %v", err)
	}
	if result.SecretInfo.Info.Name != "myreport.pdf" {
		t.Errorf("expected name=myreport.pdf, got %q", result.SecretInfo.Info.Name)
	}
}

// ---------------------------------------------------------------------------
// BuildFragment
// ---------------------------------------------------------------------------

func TestBuildFragment_IsValidBase64(t *testing.T) {
	_, jwk, _ := generateKey()
	fragment, err := BuildFragment(jwk, SecretInfo{Type: "plain"})
	if err != nil {
		t.Fatalf("BuildFragment: %v", err)
	}
	if _, err := base64.StdEncoding.DecodeString(fragment); err != nil {
		t.Errorf("fragment is not valid base64: %v", err)
	}
}

func TestBuildFragment_DecodesWithCorrectKeys(t *testing.T) {
	_, jwk, _ := generateKey()
	si := SecretInfo{Type: "file", Info: &FileInfo{Name: "doc.pdf", Type: "application/pdf"}}
	fragment, err := BuildFragment(jwk, si)
	if err != nil {
		t.Fatalf("BuildFragment: %v", err)
	}

	raw, _ := base64.StdEncoding.DecodeString(fragment)
	var parsed map[string]json.RawMessage
	if err := json.Unmarshal(raw, &parsed); err != nil {
		t.Fatalf("fragment JSON invalid: %v", err)
	}

	if _, ok := parsed["secretKey"]; !ok {
		t.Error("fragment missing secretKey")
	}
	if _, ok := parsed["secretInfo"]; !ok {
		t.Error("fragment missing secretInfo")
	}
}

func TestBuildFragment_SecretKeyMatchesJWK(t *testing.T) {
	_, jwk, _ := generateKey()
	fragment, _ := BuildFragment(jwk, SecretInfo{Type: "plain"})
	raw, _ := base64.StdEncoding.DecodeString(fragment)

	var parsed struct {
		SecretKey  JWK        `json:"secretKey"`
		SecretInfo SecretInfo `json:"secretInfo"`
	}
	if err := json.Unmarshal(raw, &parsed); err != nil {
		t.Fatalf("unmarshalling fragment: %v", err)
	}

	if parsed.SecretKey.K != jwk.K {
		t.Errorf("secretKey.k mismatch: got %q, want %q", parsed.SecretKey.K, jwk.K)
	}
	if parsed.SecretKey.Alg != jwk.Alg {
		t.Errorf("secretKey.alg mismatch: got %q, want %q", parsed.SecretKey.Alg, jwk.Alg)
	}
	if parsed.SecretInfo.Type != "plain" {
		t.Errorf("secretInfo.type mismatch: got %q, want plain", parsed.SecretInfo.Type)
	}
}
