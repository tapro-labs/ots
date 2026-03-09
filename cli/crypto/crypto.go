package crypto

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"mime"
	"net/http"
	"os"
	"path/filepath"
	"strings"
)

const chunkSize = 16 * 1024 // 16 KB — matches the frontend's Uint8StreamLimiter

// JWK is the JSON Web Key representation of the AES-256-GCM encryption key.
// The shape matches exactly what the browser's SubtleCrypto exportKey('jwk') produces
// so the frontend can decrypt using the key embedded in the URL fragment.
type JWK struct {
	Kty    string   `json:"kty"`
	K      string   `json:"k"` // base64url (no padding) of the raw 32-byte key material
	Alg    string   `json:"alg"`
	Ext    bool     `json:"ext"`
	KeyOps []string `json:"key_ops"`
}

// FileInfo holds the name and MIME type of an encrypted file secret.
type FileInfo struct {
	Name string `json:"name"`
	Type string `json:"type"`
}

// SecretInfo is the metadata embedded in the URL fragment alongside the encryption key.
type SecretInfo struct {
	Type string    `json:"type"`           // "plain" or "file"
	Info *FileInfo `json:"info,omitempty"` // only set for file secrets
}

// EncryptResult is the output of a successful encryption operation.
type EncryptResult struct {
	// Payload is the full "$_$"-joined string of base64(iv||ciphertext) chunks,
	// ready to be sent as the "secret" field in the POST /secrets request body.
	Payload    string
	JWK        JWK
	SecretInfo SecretInfo
}

// generateKey creates a new random 32-byte AES-256-GCM key and returns both the
// raw key bytes (for use with crypto/aes) and the JWK representation (for the URL fragment).
func generateKey() ([]byte, JWK, error) {
	keyBytes := make([]byte, 32)
	if _, err := rand.Read(keyBytes); err != nil {
		return nil, JWK{}, fmt.Errorf("generating key: %w", err)
	}

	jwk := JWK{
		Kty:    "oct",
		K:      base64.RawURLEncoding.EncodeToString(keyBytes), // base64url, no padding — matches SubtleCrypto JWK export
		Alg:    "A256GCM",
		Ext:    true,
		KeyOps: []string{"encrypt", "decrypt"},
	}

	return keyBytes, jwk, nil
}

// encryptChunk encrypts a single base64-encoded chunk using AES-256-GCM.
//
// The pipeline mirrors the browser's EncryptStreamTransformer + cryptography.ts:encrypt exactly:
//  1. The input `b64Data` is a base64 string of a raw ≤16 KB byte slice (mirrors GenericStreamTransformation)
//  2. A fresh 12-byte IV is generated for every chunk
//  3. gcm.Seal(iv, iv, plaintext, nil) produces iv || ciphertext || tag (16-byte GCM tag)
//     — identical to SubtleCrypto AES-GCM which also appends a 16-byte authentication tag
//  4. The result is base64 (StdEncoding) encoded — matching uint8ArrayToBase64 in the browser
func encryptChunk(key []byte, b64Data string) (string, error) {
	block, err := aes.NewCipher(key)
	if err != nil {
		return "", fmt.Errorf("creating cipher: %w", err)
	}

	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return "", fmt.Errorf("creating GCM: %w", err)
	}

	iv := make([]byte, gcm.NonceSize()) // 12 bytes
	if _, err := rand.Read(iv); err != nil {
		return "", fmt.Errorf("generating IV: %w", err)
	}

	// gcm.Seal appends ciphertext+tag to dst (iv), giving: iv || ciphertext || tag
	sealed := gcm.Seal(iv, iv, []byte(b64Data), nil)
	return base64.StdEncoding.EncodeToString(sealed), nil
}

// encryptStream reads from r in chunkSize blocks, encrypts each chunk, and returns the
// "$_$"-joined result string. This is the shared core of EncryptText and EncryptFile.
func encryptStream(key []byte, r io.Reader) (string, error) {
	buf := make([]byte, chunkSize)
	var chunks []string

	for {
		n, err := io.ReadFull(r, buf)
		if n > 0 {
			b64Chunk := base64.StdEncoding.EncodeToString(buf[:n])
			encrypted, encErr := encryptChunk(key, b64Chunk)
			if encErr != nil {
				return "", encErr
			}
			chunks = append(chunks, encrypted)
		}

		if err == io.EOF || err == io.ErrUnexpectedEOF {
			break
		}
		if err != nil {
			return "", fmt.Errorf("reading data: %w", err)
		}
	}

	// The browser's EncryptStreamTransformer appends '$_$' after every chunk
	// unconditionally (including the last one). Secret.vue's stream reader relies
	// on this trailing separator to detect chunk boundaries, so we must match it.
	return strings.Join(chunks, "$_$") + "$_$", nil
}

// EncryptText encrypts a plaintext string secret.
func EncryptText(text string) (EncryptResult, error) {
	key, jwk, err := generateKey()
	if err != nil {
		return EncryptResult{}, err
	}

	payload, err := encryptStream(key, strings.NewReader(text))
	if err != nil {
		return EncryptResult{}, err
	}

	return EncryptResult{
		Payload:    payload,
		JWK:        jwk,
		SecretInfo: SecretInfo{Type: "plain"},
	}, nil
}

// EncryptFile encrypts a file at the given path, streaming it in 16 KB chunks.
// The first 512 bytes are used for MIME type detection before streaming begins.
func EncryptFile(path string) (EncryptResult, error) {
	key, jwk, err := generateKey()
	if err != nil {
		return EncryptResult{}, err
	}

	f, err := os.Open(path)
	if err != nil {
		return EncryptResult{}, fmt.Errorf("opening file: %w", err)
	}
	defer f.Close()

	// Read the first 512 bytes for MIME detection, then stitch them back
	// into the stream using io.MultiReader so no data is skipped.
	sniff := make([]byte, 512)
	n, err := f.Read(sniff)
	if err != nil && err != io.EOF {
		return EncryptResult{}, fmt.Errorf("reading file header: %w", err)
	}
	sniff = sniff[:n]

	mimeType := http.DetectContentType(sniff)
	// DetectContentType may return a generic type; refine via file extension if so.
	if mimeType == "application/octet-stream" || mimeType == "text/plain; charset=utf-8" {
		if ext := filepath.Ext(path); ext != "" {
			if byExt := mime.TypeByExtension(ext); byExt != "" {
				mimeType = byExt
			}
		}
	}
	// Strip charset params for cleanliness (e.g. "text/plain; charset=utf-8" → "text/plain")
	if idx := strings.Index(mimeType, ";"); idx != -1 {
		mimeType = strings.TrimSpace(mimeType[:idx])
	}

	// Reconstruct the full stream: sniffed bytes + rest of file
	stream := io.MultiReader(strings.NewReader(string(sniff)), f)

	payload, err := encryptStream(key, stream)
	if err != nil {
		return EncryptResult{}, err
	}

	return EncryptResult{
		Payload: payload,
		JWK:     jwk,
		SecretInfo: SecretInfo{
			Type: "file",
			Info: &FileInfo{
				Name: filepath.Base(path),
				Type: mimeType,
			},
		},
	}, nil
}

// BuildFragment encodes the JWK and SecretInfo into the base64 string that goes
// in the URL fragment (#). The shape matches CreateSecret.vue exactly:
//
//	Base64.btoa(JSON.stringify({ secretKey, secretInfo }))
func BuildFragment(jwk JWK, secretInfo SecretInfo) (string, error) {
	payload := map[string]any{
		"secretKey":  jwk,
		"secretInfo": secretInfo,
	}

	jsonBytes, err := json.Marshal(payload)
	if err != nil {
		return "", fmt.Errorf("marshalling fragment: %w", err)
	}

	return base64.StdEncoding.EncodeToString(jsonBytes), nil
}
