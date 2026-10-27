import { beforeEach, describe, expect, it, vi } from 'vitest';
import useEncryptData from '@/composables/useEncryptData';
import useDecryptData from '@/composables/useDecryptData';
import OtsConfig from '@/utils/otsConfig';
import EncryptStreamTransformer from '@/stream-transformers/EncryptStreamTransformer';
import GenericStreamTransformation from '@/stream-transformers/GenericStreamTransformation';
import { createEncryptionKey, DEFAULT_SECRET_LENGTH } from '@/utils/cryptography';
import { base64ToString, base64ToUint8Array, encodeText } from '@/utils/helpers';
import { concatBytes, randomBytes, readAll, streamOf } from '@/test-utils/streams';
import { splitEncryptedSecret } from '@/test-utils/secrets';

const encryptToString = async (key: JsonWebKey, chunks: Uint8Array[]) => {
  const { encryptStream } = useEncryptData();

  return (await readAll(encryptStream({ key, data: streamOf(chunks) }))).join('');
};

const decryptToBytes = async (key: JsonWebKey, secret: string) => {
  const { decryptStream } = useDecryptData();
  const decrypted = decryptStream({ key, data: streamOf(splitEncryptedSecret(secret)) });

  return concatBytes(await readAll(decrypted.pipeThrough(new GenericStreamTransformation(base64ToUint8Array))));
};

describe('useEncryptData / useDecryptData', () => {
  let key: JsonWebKey;

  beforeEach(async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {});
    key = await createEncryptionKey(DEFAULT_SECRET_LENGTH);
  });

  it('round-trips a plain text secret', async () => {
    const text = 'hunter2 – with ünicode 🔑';
    const secret = await encryptToString(key, [encodeText(text)]);
    const { decryptStream } = useDecryptData();
    const decrypted = decryptStream({ key, data: streamOf(splitEncryptedSecret(secret)) });
    const output = await readAll(decrypted.pipeThrough(new GenericStreamTransformation(base64ToString)));

    expect(secret).not.toContain('hunter2');
    expect(output.join('')).toBe(text);
  });

  it('encrypts in chunks of the configured byte size and round-trips binary data', async () => {
    const chunkSize = OtsConfig.getInstance().encryptionByteSize;
    const input = randomBytes(chunkSize * 2 + 123);
    // feed it in odd-sized pieces to make sure re-chunking is not dependent on input boundaries
    const secret = await encryptToString(key, [
      input.slice(0, 7),
      input.slice(7, chunkSize + 50),
      input.slice(chunkSize + 50),
    ]);

    expect(splitEncryptedSecret(secret)).toHaveLength(3);
    expect(secret.endsWith(EncryptStreamTransformer.SEPARATOR)).toBe(true);
    expect(await decryptToBytes(key, secret)).toEqual(input);
  });

  it('produces an empty secret for empty input', async () => {
    expect(await encryptToString(key, [])).toBe('');
  });

  it('rejects instead of hanging when decrypting with the wrong key', async () => {
    const secret = await encryptToString(key, [encodeText('secret')]);
    const wrongKey = await createEncryptionKey(DEFAULT_SECRET_LENGTH);

    await expect(decryptToBytes(wrongKey, secret)).rejects.toThrow();
  });
});
