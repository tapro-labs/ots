import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  createEncryptionKey,
  decrypt,
  DEFAULT_IV_LENGTH,
  DEFAULT_SECRET_LENGTH,
  encrypt,
  ENCRYPTION_ALGORITHM,
  sha256Hex,
} from '@/utils/cryptography';
import { base64ToUint8Array, uint8ArrayToBase64 } from '@/utils/helpers';

// AES-GCM appends a 16 byte authentication tag to the ciphertext
const GCM_TAG_LENGTH = 16;

describe('cryptography', () => {
  let key: JsonWebKey;

  beforeEach(async () => {
    // errors are logged before being rethrown, keep test output clean
    vi.spyOn(console, 'error').mockImplementation(() => {});
    key = await createEncryptionKey(DEFAULT_SECRET_LENGTH);
  });

  it('creates an exportable AES-GCM JWK of the requested length', async () => {
    expect(key).toMatchObject({ kty: 'oct', alg: 'A256GCM', ext: true });
    expect(key.key_ops).toEqual(expect.arrayContaining(['encrypt', 'decrypt']));
    // 256 bits => 32 bytes => 43 base64url characters
    expect(key.k).toHaveLength(43);

    const shortKey = await createEncryptionKey(128);

    expect(shortKey.alg).toBe('A128GCM');
  });

  it('rejects invalid key lengths', async () => {
    await expect(createEncryptionKey(100)).rejects.toThrow();
  });

  it('generates a unique key every time', async () => {
    const otherKey = await createEncryptionKey(DEFAULT_SECRET_LENGTH);

    expect(otherKey.k).not.toBe(key.k);
  });

  it('decrypts what it encrypted', async () => {
    const plain = 'my very secret password 🙈';

    expect(await decrypt(key, await encrypt(key, plain))).toBe(plain);
    expect(await decrypt(key, await encrypt(key, ''))).toBe('');
  });

  it('prefixes the ciphertext with a random IV so equal inputs produce different outputs', async () => {
    const first = await encrypt(key, 'same');
    const second = await encrypt(key, 'same');
    const bytes = base64ToUint8Array(first);

    expect(first).not.toBe(second);
    expect(bytes.length).toBe(DEFAULT_IV_LENGTH + 'same'.length + GCM_TAG_LENGTH);
    expect(Array.from(bytes.slice(0, DEFAULT_IV_LENGTH))).not.toEqual(
      Array.from(base64ToUint8Array(second).slice(0, DEFAULT_IV_LENGTH))
    );
  });

  it('produces output that can be decrypted with plain Web Crypto (iv || ciphertext)', async () => {
    const bytes = base64ToUint8Array(await encrypt(key, 'interop'));
    const cryptoKey = await window.crypto.subtle.importKey('jwk', key, { name: ENCRYPTION_ALGORITHM }, false, [
      'decrypt',
    ]);
    const decrypted = await window.crypto.subtle.decrypt(
      { name: ENCRYPTION_ALGORITHM, iv: bytes.slice(0, DEFAULT_IV_LENGTH) },
      cryptoKey,
      bytes.slice(DEFAULT_IV_LENGTH)
    );

    expect(new TextDecoder().decode(decrypted)).toBe('interop');
  });

  it('fails to decrypt with a different key', async () => {
    const otherKey = await createEncryptionKey(DEFAULT_SECRET_LENGTH);

    await expect(decrypt(otherKey, await encrypt(key, 'secret'))).rejects.toThrow();
  });

  it('fails to decrypt tampered ciphertext', async () => {
    const bytes = base64ToUint8Array(await encrypt(key, 'secret'));

    bytes[bytes.length - 1] ^= 1;

    await expect(decrypt(key, uint8ArrayToBase64(bytes))).rejects.toThrow();
  });

  it('fails to decrypt truncated data', async () => {
    const encrypted = await encrypt(key, 'secret');

    await expect(decrypt(key, encrypted.slice(0, 8))).rejects.toThrow();
  });
});

describe('sha256Hex', () => {
  it('returns the lowercase hex sha256 of the text', async () => {
    // FIPS 180-2 test vector, the Go CLI checks the same one
    expect(await sha256Hex('abc')).toBe('ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
  });

  it('changes when chunks are dropped or reordered', async () => {
    const payload = 'chunk1$_$chunk2$_$';

    expect(await sha256Hex('chunk1$_$')).not.toBe(await sha256Hex(payload));
    expect(await sha256Hex('chunk2$_$chunk1$_$')).not.toBe(await sha256Hex(payload));
  });
});
