package crypto

import (
	"bytes"
	"crypto/aes"
	"crypto/cipher"
	"encoding/base64"
	"encoding/json"
	"errors"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"unicode/utf8"
)

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// setRand replaces the randomness source for the duration of the test.
func setRand(t *testing.T, r io.Reader) {
	t.Helper()
	orig := randReader
	randReader = r
	t.Cleanup(func() { randReader = orig })
}

// errReader always fails.
type errReader struct{ err error }

func (e errReader) Read([]byte) (int, error) { return 0, e.err }

// failAfter yields n zero bytes and then fails. It lets a test
// make key generation succeed and IV generation fail.
func failAfter(n int) io.Reader {
	return io.MultiReader(bytes.NewReader(make([]byte, n)), errReader{errors.New("entropy exhausted")})
}

// decryptPayload mirrors the frontend decryption path in Secret.vue:
//  1. split the payload on "$_$" the same way the pull() loop does
//  2. decrypt each chunk as base64(iv[12] || ciphertext || tag) (cryptography.ts:decrypt)
//  3. the decrypted bytes are a base64 string; decode it back to the raw bytes
//     (base64ToString / base64ToUint8Array)
func decryptPayload(t *testing.T, jwk JWK, payload string) []byte {
	t.Helper()
	key, err := base64.RawURLEncoding.DecodeString(jwk.K)
	if err != nil {
		t.Fatalf("decoding JWK k: %v", err)
	}
	out, err := decryptWithKey(key, payload)
	if err != nil {
		t.Fatalf("decrypting payload: %v", err)
	}
	return out
}

func decryptWithKey(key []byte, payload string) ([]byte, error) {
	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return nil, err
	}

	const sep = "$_$"
	var out []byte
	rest := payload
	for {
		idx := strings.Index(rest, sep)
		if idx == -1 {
			return nil, errors.New("payload chunk without trailing separator")
		}
		chunk := rest[:idx]
		rest = rest[idx+len(sep):]

		raw, err := base64.StdEncoding.DecodeString(chunk)
		if err != nil {
			return nil, err
		}
		if len(raw) < 12 {
			return nil, errors.New("chunk shorter than IV")
		}
		plain, err := gcm.Open(nil, raw[:12], raw[12:], nil)
		if err != nil {
			return nil, err
		}
		data, err := base64.StdEncoding.DecodeString(string(plain))
		if err != nil {
			return nil, err
		}
		out = append(out, data...)

		if rest == "" {
			return out, nil
		}
	}
}

func chunkCount(payload string) int {
	return len(strings.Split(strings.TrimSuffix(payload, "$_$"), "$_$"))
}

// ---------------------------------------------------------------------------
// Fixed vector produced by the browser algorithm
// ---------------------------------------------------------------------------

// The vector below was produced with Node's WebCrypto (globalThis.crypto.subtle),
// which follows the same steps as frontend/src/utils/cryptography.ts:encrypt and
// useEncryptData.ts (Uint8StreamLimiter -> uint8ArrayToBase64 -> encrypt):
//
//	key   = bytes 0x00..0x1f (imported as raw, exported as JWK)
//	iv    = bytes 0xa0..0xab
//	input = UTF-8 "héllo, OTS 🔐"
//	data  = TextEncoder(base64(input))
//	out   = base64(iv || AES-GCM(key, iv, data))
const (
	vectorPlaintext = "héllo, OTS 🔐"
	vectorB64Chunk  = "aMOpbGxvLCBPVFMg8J+UkA=="
	vectorChunk     = "oKGio6Slpqeoqaqrh1UzXSeMeskuJsWDUTyNuUjmckX59n9RN5hNw/eaDcB2oHycWJn2zg=="
	vectorJWK       = `{"key_ops":["encrypt","decrypt"],"ext":true,"alg":"A256GCM","kty":"oct","k":"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8"}`
)

func vectorRand() io.Reader {
	buf := make([]byte, 0, 44)
	for i := 0; i < 32; i++ {
		buf = append(buf, byte(i))
	}
	for i := 0; i < 12; i++ {
		buf = append(buf, byte(0xa0+i))
	}
	return bytes.NewReader(buf)
}

func TestEncryptText_MatchesBrowserFixedVector(t *testing.T) {
	setRand(t, vectorRand())

	result, err := EncryptText(vectorPlaintext)
	if err != nil {
		t.Fatalf("EncryptText: %v", err)
	}

	if want := vectorChunk + "$_$"; result.Payload != want {
		t.Errorf("payload mismatch\n got: %s\nwant: %s", result.Payload, want)
	}

	// The JWK must carry the same members and values as SubtleCrypto's exportKey('jwk').
	var got, want map[string]any
	gotJSON, _ := json.Marshal(result.JWK)
	if err := json.Unmarshal(gotJSON, &got); err != nil {
		t.Fatal(err)
	}
	if err := json.Unmarshal([]byte(vectorJWK), &want); err != nil {
		t.Fatal(err)
	}
	gotNorm, _ := json.Marshal(got) // map keys are sorted on marshal
	wantNorm, _ := json.Marshal(want)
	if string(gotNorm) != string(wantNorm) {
		t.Errorf("JWK mismatch\n got: %s\nwant: %s", gotNorm, wantNorm)
	}
}

func TestDecrypt_BrowserFixedVector(t *testing.T) {
	// The other direction: what the browser produced decrypts with the Go-side
	// format assumptions (12-byte IV prefix, base64 inner layer).
	var jwk JWK
	if err := json.Unmarshal([]byte(vectorJWK), &jwk); err != nil {
		t.Fatal(err)
	}
	got := decryptPayload(t, jwk, vectorChunk+"$_$")
	if string(got) != vectorPlaintext {
		t.Errorf("got %q, want %q", got, vectorPlaintext)
	}
	if base64.StdEncoding.EncodeToString([]byte(vectorPlaintext)) != vectorB64Chunk {
		t.Error("inner base64 layer does not match the browser's uint8ArrayToBase64 output")
	}
}

// ---------------------------------------------------------------------------
// Round trips through the frontend decryption path
// ---------------------------------------------------------------------------

func TestEncryptText_RoundTrip_ChunkBoundaries(t *testing.T) {
	cases := []struct {
		name   string
		size   int
		chunks int
	}{
		{"one byte", 1, 1},
		{"one below chunk", chunkSize - 1, 1},
		{"exactly one chunk", chunkSize, 1},
		{"one above chunk", chunkSize + 1, 2},
		{"exactly two chunks", 2 * chunkSize, 2},
		{"40KB", 40 * 1024, 3},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			text := strings.Repeat("a", tc.size)
			result, err := EncryptText(text)
			if err != nil {
				t.Fatalf("EncryptText: %v", err)
			}
			if n := chunkCount(result.Payload); n != tc.chunks {
				t.Errorf("expected %d chunks, got %d", tc.chunks, n)
			}
			if strings.Contains(result.Payload, "$_$$_$") {
				t.Error("payload contains an empty chunk")
			}
			if got := decryptPayload(t, result.JWK, result.Payload); string(got) != text {
				t.Errorf("round-trip mismatch (len got %d, want %d)", len(got), len(text))
			}
		})
	}
}

func TestEncryptText_RoundTrip_UnicodeAndSeparatorLikeText(t *testing.T) {
	// Text that contains the separator itself must not confuse the chunk split,
	// because only base64 (which never contains '$' or '_') is on the wire.
	text := "pässwörd $_$ 秘密 🔑\nline2\r\n\ttab"
	result, err := EncryptText(text)
	if err != nil {
		t.Fatalf("EncryptText: %v", err)
	}
	if got := decryptPayload(t, result.JWK, result.Payload); string(got) != text {
		t.Errorf("got %q, want %q", got, text)
	}
}

func TestEncryptText_ChunksSplitOnRawBytes(t *testing.T) {
	// The frontend's Uint8StreamLimiter splits on raw bytes, not on characters,
	// so a multi-byte rune can straddle two chunks. Concatenating the decoded
	// bytes must still give back valid UTF-8.
	text := strings.Repeat("a", chunkSize-1) + "€" // '€' is 3 bytes
	result, err := EncryptText(text)
	if err != nil {
		t.Fatalf("EncryptText: %v", err)
	}
	if n := chunkCount(result.Payload); n != 2 {
		t.Fatalf("expected 2 chunks, got %d", n)
	}
	got := decryptPayload(t, result.JWK, result.Payload)
	if !utf8.Valid(got) || string(got) != text {
		t.Error("multi-byte rune across a chunk boundary was not preserved")
	}
}

func TestEncryptFile_RoundTrip_BinaryLargerThanSniffBuffer(t *testing.T) {
	// The first 512 bytes are read for MIME sniffing and stitched back into the
	// stream. Verify no byte is lost or duplicated, for binary content too.
	content := make([]byte, 3*chunkSize+777)
	for i := range content {
		content[i] = byte(i * 31)
	}
	path := filepath.Join(t.TempDir(), "blob.bin")
	if err := os.WriteFile(path, content, 0600); err != nil {
		t.Fatal(err)
	}

	result, err := EncryptFile(path)
	if err != nil {
		t.Fatalf("EncryptFile: %v", err)
	}
	if got := decryptPayload(t, result.JWK, result.Payload); !bytes.Equal(got, content) {
		t.Errorf("binary round-trip mismatch (len got %d, want %d)", len(got), len(content))
	}
}

func TestEncryptFile_RoundTrip_SmallerThanSniffBuffer(t *testing.T) {
	content := "tiny"
	path := filepath.Join(t.TempDir(), "tiny.txt")
	if err := os.WriteFile(path, []byte(content), 0600); err != nil {
		t.Fatal(err)
	}
	result, err := EncryptFile(path)
	if err != nil {
		t.Fatalf("EncryptFile: %v", err)
	}
	if got := decryptPayload(t, result.JWK, result.Payload); string(got) != content {
		t.Errorf("got %q, want %q", got, content)
	}
}

// ---------------------------------------------------------------------------
// Bad input on the decryption side
// ---------------------------------------------------------------------------

func TestDecrypt_WrongKeyFails(t *testing.T) {
	result, err := EncryptText("secret")
	if err != nil {
		t.Fatal(err)
	}
	_, otherJWK, _ := generateKey()
	otherKey, _ := base64.RawURLEncoding.DecodeString(otherJWK.K)
	if _, err := decryptWithKey(otherKey, result.Payload); err == nil {
		t.Error("decryption with a different key must fail")
	}
}

func TestDecrypt_TamperedCiphertextFails(t *testing.T) {
	result, err := EncryptText("secret")
	if err != nil {
		t.Fatal(err)
	}
	key, _ := base64.RawURLEncoding.DecodeString(result.JWK.K)

	raw, _ := base64.StdEncoding.DecodeString(strings.TrimSuffix(result.Payload, "$_$"))
	raw[len(raw)-1] ^= 0x01 // flip a bit in the GCM tag
	tampered := base64.StdEncoding.EncodeToString(raw) + "$_$"
	if _, err := decryptWithKey(key, tampered); err == nil {
		t.Error("decryption of tampered ciphertext must fail")
	}

	raw, _ = base64.StdEncoding.DecodeString(strings.TrimSuffix(result.Payload, "$_$"))
	truncated := base64.StdEncoding.EncodeToString(raw[:len(raw)-4]) + "$_$"
	if _, err := decryptWithKey(key, truncated); err == nil {
		t.Error("decryption of truncated ciphertext must fail")
	}
}

func TestDecrypt_SwappedChunkOrderChangesPlaintext(t *testing.T) {
	// Chunks are encrypted independently, so this documents that order is
	// preserved on the wire: the first chunk holds the first 16 KB.
	text := strings.Repeat("A", chunkSize) + strings.Repeat("B", 10)
	result, err := EncryptText(text)
	if err != nil {
		t.Fatal(err)
	}
	parts := strings.Split(strings.TrimSuffix(result.Payload, "$_$"), "$_$")
	got := decryptPayload(t, result.JWK, parts[0]+"$_$")
	if string(got) != strings.Repeat("A", chunkSize) {
		t.Error("first chunk does not hold the first 16 KB of the input")
	}
}

// ---------------------------------------------------------------------------
// Empty input (frontend cannot decrypt a payload with no chunks)
// ---------------------------------------------------------------------------

func TestEncryptText_Empty_ReturnsErrEmptySecret(t *testing.T) {
	_, err := EncryptText("")
	if !errors.Is(err, ErrEmptySecret) {
		t.Fatalf("expected ErrEmptySecret, got %v", err)
	}
}

func TestEncryptFile_Empty_ReturnsErrEmptySecret(t *testing.T) {
	path := filepath.Join(t.TempDir(), "empty.txt")
	if err := os.WriteFile(path, nil, 0600); err != nil {
		t.Fatal(err)
	}
	_, err := EncryptFile(path)
	if !errors.Is(err, ErrEmptySecret) {
		t.Fatalf("expected ErrEmptySecret, got %v", err)
	}
}

// ---------------------------------------------------------------------------
// Error paths
// ---------------------------------------------------------------------------

func TestGenerateKey_RandFailure(t *testing.T) {
	setRand(t, errReader{errors.New("no entropy")})
	if _, _, err := generateKey(); err == nil || !strings.Contains(err.Error(), "generating key") {
		t.Fatalf("expected generating key error, got %v", err)
	}
}

func TestEncryptText_KeyRandFailure(t *testing.T) {
	setRand(t, errReader{errors.New("no entropy")})
	if _, err := EncryptText("x"); err == nil {
		t.Fatal("expected error")
	}
}

func TestEncryptText_IVRandFailure(t *testing.T) {
	setRand(t, failAfter(32)) // key succeeds, IV fails
	_, err := EncryptText("x")
	if err == nil || !strings.Contains(err.Error(), "generating IV") {
		t.Fatalf("expected generating IV error, got %v", err)
	}
}

func TestEncryptFile_KeyRandFailure(t *testing.T) {
	path := filepath.Join(t.TempDir(), "a.txt")
	os.WriteFile(path, []byte("x"), 0600)
	setRand(t, errReader{errors.New("no entropy")})
	if _, err := EncryptFile(path); err == nil {
		t.Fatal("expected error")
	}
}

func TestEncryptFile_IVRandFailure(t *testing.T) {
	path := filepath.Join(t.TempDir(), "a.txt")
	os.WriteFile(path, []byte("x"), 0600)
	setRand(t, failAfter(32))
	if _, err := EncryptFile(path); err == nil || !strings.Contains(err.Error(), "generating IV") {
		t.Fatalf("expected generating IV error, got %v", err)
	}
}

func TestEncryptChunk_InvalidKeyLength(t *testing.T) {
	if _, err := encryptChunk(make([]byte, 7), "aGk="); err == nil {
		t.Fatal("expected error for a 7-byte AES key")
	}
}

func TestEncryptStream_ReadError(t *testing.T) {
	key, _, _ := generateKey()
	r := io.MultiReader(strings.NewReader(strings.Repeat("z", chunkSize)), errReader{errors.New("disk on fire")})
	_, err := encryptStream(key, r)
	if err == nil || !strings.Contains(err.Error(), "disk on fire") {
		t.Fatalf("expected read error to propagate, got %v", err)
	}
}

func TestEncryptFile_Directory_ReturnsError(t *testing.T) {
	// os.Open succeeds on a directory, the first Read fails.
	_, err := EncryptFile(t.TempDir())
	if err == nil || !strings.Contains(err.Error(), "reading file header") {
		t.Fatalf("expected reading file header error, got %v", err)
	}
}

// ---------------------------------------------------------------------------
// MIME type detection
// ---------------------------------------------------------------------------

func TestEncryptFile_MIMEType(t *testing.T) {
	png := "\x89PNG\r\n\x1a\n" + strings.Repeat("\x00", 16)
	cases := []struct {
		name, content, want string
	}{
		{"notes.txt", "plain text", "text/plain"},
		{"data.json", `{"a":1}`, "application/json"},              // refined from text/plain by extension
		{"image.png", png, "image/png"},                           // sniffed from magic bytes
		{"image.bin", png, "image/png"},                           // sniffing beats a generic extension
		{"page.html", "<html><body>x</body></html>", "text/html"}, // charset param stripped
		{"noext", "\x00\x01\x02\x03", "application/octet-stream"},
		{"README", "just text", "text/plain"}, // no extension, charset stripped
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			path := filepath.Join(t.TempDir(), tc.name)
			if err := os.WriteFile(path, []byte(tc.content), 0600); err != nil {
				t.Fatal(err)
			}
			result, err := EncryptFile(path)
			if err != nil {
				t.Fatalf("EncryptFile: %v", err)
			}
			if got := result.SecretInfo.Info.Type; got != tc.want {
				t.Errorf("MIME type: got %q, want %q", got, tc.want)
			}
		})
	}
}

// ---------------------------------------------------------------------------
// Fragment shape (CreateSecret.vue: Base64.btoa(JSON.stringify({ secretKey, secretInfo })))
// ---------------------------------------------------------------------------

func TestBuildFragment_PlainOmitsInfo(t *testing.T) {
	setRand(t, vectorRand())
	_, jwk, err := generateKey()
	if err != nil {
		t.Fatal(err)
	}
	fragment, err := BuildFragment(jwk, SecretInfo{Type: "plain"}, "")
	if err != nil {
		t.Fatal(err)
	}
	raw, err := base64.StdEncoding.DecodeString(fragment)
	if err != nil {
		t.Fatal(err)
	}
	want := `{"secretInfo":{"type":"plain"},"secretKey":{"kty":"oct","k":"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8","alg":"A256GCM","ext":true,"key_ops":["encrypt","decrypt"]}}`
	if string(raw) != want {
		t.Errorf("fragment JSON\n got: %s\nwant: %s", raw, want)
	}
}

func TestBuildFragment_FileInfoShape(t *testing.T) {
	_, jwk, _ := generateKey()
	fragment, err := BuildFragment(jwk, SecretInfo{Type: "file", Info: &FileInfo{Name: "a b.pdf", Type: "application/pdf"}}, "")
	if err != nil {
		t.Fatal(err)
	}
	raw, _ := base64.StdEncoding.DecodeString(fragment)
	var parsed struct {
		SecretInfo map[string]any `json:"secretInfo"`
	}
	if err := json.Unmarshal(raw, &parsed); err != nil {
		t.Fatal(err)
	}
	info, ok := parsed.SecretInfo["info"].(map[string]any)
	if !ok {
		t.Fatalf("secretInfo.info missing: %s", raw)
	}
	// Secret.vue's downloadFile reads info.name and info.type.
	if info["name"] != "a b.pdf" || info["type"] != "application/pdf" {
		t.Errorf("unexpected info: %v", info)
	}
}

// ---------------------------------------------------------------------------
// Checksum (Secret.vue compares it with the sha256 of the fetched payload)
// ---------------------------------------------------------------------------

func TestChecksum_IsLowercaseHexSHA256(t *testing.T) {
	// FIPS 180-2 test vector for "abc".
	want := "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
	if got := Checksum("abc"); got != want {
		t.Errorf("Checksum(abc) = %s, want %s", got, want)
	}
}

func TestChecksum_ChangesWhenAChunkIsDropped(t *testing.T) {
	payload := "chunk1$_$chunk2$_$"
	if Checksum(payload) == Checksum("chunk1$_$") {
		t.Error("dropping the last chunk must change the checksum")
	}
	if Checksum(payload) == Checksum("chunk2$_$chunk1$_$") {
		t.Error("reordering chunks must change the checksum")
	}
}

func TestBuildFragment_IncludesChecksum(t *testing.T) {
	_, jwk, _ := generateKey()
	fragment, err := BuildFragment(jwk, SecretInfo{Type: "plain"}, "abc123")
	if err != nil {
		t.Fatal(err)
	}
	raw, _ := base64.StdEncoding.DecodeString(fragment)
	var parsed struct {
		Checksum string `json:"checksum"`
	}
	if err := json.Unmarshal(raw, &parsed); err != nil {
		t.Fatal(err)
	}
	if parsed.Checksum != "abc123" {
		t.Errorf("checksum = %q, want abc123 (fragment %s)", parsed.Checksum, raw)
	}
}
