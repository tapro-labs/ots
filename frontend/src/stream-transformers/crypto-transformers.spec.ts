import { beforeEach, describe, expect, it, vi } from 'vitest';
import EncryptStreamTransformer from '@/stream-transformers/EncryptStreamTransformer';
import DecryptStreamTransformer from '@/stream-transformers/DecryptStreamTransformer';
import { createEncryptionKey, decrypt, DEFAULT_SECRET_LENGTH } from '@/utils/cryptography';
import { transform } from '@/test-utils/streams';

describe('Encrypt/Decrypt stream transformers', () => {
  let key: JsonWebKey;

  beforeEach(async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {});
    key = await createEncryptionKey(DEFAULT_SECRET_LENGTH);
  });

  it('encrypts each chunk separately and terminates it with the separator', async () => {
    const output = await transform(['first', 'second'], new EncryptStreamTransformer(key));

    expect(output).toHaveLength(2);

    for (const [index, chunk] of output.entries()) {
      expect(chunk.endsWith(EncryptStreamTransformer.SEPARATOR)).toBe(true);
      // the separator must never occur inside base64 ciphertext, otherwise splitting would break
      expect(chunk.indexOf(EncryptStreamTransformer.SEPARATOR)).toBe(
        chunk.length - EncryptStreamTransformer.SEPARATOR.length
      );
      expect(await decrypt(key, chunk.slice(0, -EncryptStreamTransformer.SEPARATOR.length))).toBe(
        ['first', 'second'][index]
      );
    }
  });

  it('decrypts a stream of encrypted chunks back to the original chunks', async () => {
    const encrypted = await transform(['a', 'bb', 'ccc'], new EncryptStreamTransformer(key));
    const withoutSeparators = encrypted.map(chunk => chunk.slice(0, -EncryptStreamTransformer.SEPARATOR.length));

    expect(await transform(withoutSeparators, new DecryptStreamTransformer(key))).toEqual(['a', 'bb', 'ccc']);
  });

  it('errors the stream when a chunk was encrypted with another key', async () => {
    const otherKey = await createEncryptionKey(DEFAULT_SECRET_LENGTH);
    const [encrypted] = await transform(['secret'], new EncryptStreamTransformer(otherKey));

    await expect(
      transform([encrypted.slice(0, -EncryptStreamTransformer.SEPARATOR.length)], new DecryptStreamTransformer(key))
    ).rejects.toThrow();
  });
});
