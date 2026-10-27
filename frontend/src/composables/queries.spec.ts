import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { nextTick, ref } from 'vue';
import { flushPromises } from '@vue/test-utils';
import useCreateSecret from '@/composables/useCreateSecret';
import useFetchSecret from '@/composables/useFetchSecret';
import useDecryptData from '@/composables/useDecryptData';
import GenericStreamTransformation from '@/stream-transformers/GenericStreamTransformation';
import { createEncryptionKey, DEFAULT_SECRET_LENGTH, sha256Hex } from '@/utils/cryptography';
import { base64ToString, encodeText } from '@/utils/helpers';
import { readAll, streamOf } from '@/test-utils/streams';
import { splitEncryptedSecret } from '@/test-utils/secrets';
import { withQuery } from '@/test-utils/query';

const client = vi.hoisted(() => ({ get: vi.fn(), post: vi.fn() }));

vi.mock('@/utils/otsClient', () => ({ default: client }));

describe('useCreateSecret', () => {
  beforeEach(() => {
    client.post.mockReset();
  });

  it('uploads only the encrypted secret with its expiry and returns the new id and checksum', async () => {
    client.post.mockResolvedValue({ data: { secretId: 'secret-123' } });
    const key = await createEncryptionKey(DEFAULT_SECRET_LENGTH);
    const { result } = withQuery(() => useCreateSecret());

    const { secretId, checksum } = await result.createSecret({
      key,
      data: streamOf([encodeText('top secret')]),
      expirySeconds: 300,
    });

    expect(secretId).toBe('secret-123');
    expect(client.post).toHaveBeenCalledOnce();

    const [url, body] = client.post.mock.calls[0] as [string, { secret: string; expiry_seconds: number }];

    expect(url).toBe('/secrets');
    expect(body.expiry_seconds).toBe(300);
    expect(checksum).toBe(await sha256Hex(body.secret));
    expect(body.secret).not.toContain('top secret');
    expect(JSON.stringify(body)).not.toContain(key.k as string);

    // the uploaded payload must be decryptable with the key
    const { decryptStream } = useDecryptData();
    const decrypted = decryptStream({ key, data: streamOf(splitEncryptedSecret(body.secret)) }).pipeThrough(
      new GenericStreamTransformation(base64ToString)
    );

    expect((await readAll(decrypted)).join('')).toBe('top secret');
  });

  it('exposes the loading state while the upload is in flight', async () => {
    let resolveUpload!: (value: unknown) => void;
    client.post.mockReturnValue(new Promise(resolve => (resolveUpload = resolve)));
    const key = await createEncryptionKey(DEFAULT_SECRET_LENGTH);
    const { result } = withQuery(() => useCreateSecret());

    const pending = result.createSecret({ key, data: streamOf([encodeText('x')]), expirySeconds: 60 });

    await vi.waitFor(() => expect(client.post).toHaveBeenCalled());
    expect(result.isCreating.value).toBe(true);

    resolveUpload({ data: { secretId: 'id' } });
    await pending;
    await nextTick();
    expect(result.isCreating.value).toBe(false);
  });

  it('rejects when the upload fails', async () => {
    client.post.mockRejectedValue(new Error('Network Error'));
    const key = await createEncryptionKey(DEFAULT_SECRET_LENGTH);
    const { result } = withQuery(() => useCreateSecret());

    await expect(result.createSecret({ key, data: streamOf([encodeText('x')]), expirySeconds: 60 })).rejects.toThrow(
      'Network Error'
    );
  });
});

describe('useFetchSecret', () => {
  beforeEach(() => {
    client.get.mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('does not fetch (and is not loading) until enabled, because fetching burns the secret', async () => {
    client.get.mockResolvedValue({ data: { secret: 'encrypted' } });
    const enabled = ref(false);
    const { result } = withQuery(() => useFetchSecret({ secretId: 'abc', enabled }));

    await flushPromises();
    expect(client.get).not.toHaveBeenCalled();
    expect(result.isLoading.value).toBe(false);

    enabled.value = true;
    await vi.waitFor(() => expect(result.secret.value).toBe('encrypted'));

    expect(client.get).toHaveBeenCalledOnce();
    expect(client.get).toHaveBeenCalledWith('/secrets/abc');
    expect(result.isLoading.value).toBe(false);
  });

  it('is disabled by default', async () => {
    withQuery(() => useFetchSecret({ secretId: 'abc' }));
    await flushPromises();

    expect(client.get).not.toHaveBeenCalled();
  });

  it('reports errors, e.g. when the secret was already revealed', async () => {
    client.get.mockRejectedValue(Object.assign(new Error('Not Found'), { response: { status: 404 } }));
    const { result } = withQuery(() => useFetchSecret({ secretId: 'gone', enabled: ref(true) }));

    await vi.waitFor(() => expect(result.isError.value).toBe(true));
    expect(result.secret.value).toBeUndefined();
  });
});
