import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { mount, VueWrapper } from '@vue/test-utils';
import { Base64 } from 'js-base64';
import Secret from '@/pages/Secret.vue';
import AudioPlayer from '@/pages/Home/components/AudioPlayer/AudioPlayer.vue';
import useEncryptData from '@/composables/useEncryptData';
import { createEncryptionKey, DEFAULT_SECRET_LENGTH, sha256Hex } from '@/utils/cryptography';
import { encodeText } from '@/utils/helpers';
import { SecretInfo } from '@/types/SecretInfo';
import { randomBytes, readAll, streamOf } from '@/test-utils/streams';
import { queryPlugin } from '@/test-utils/query';

const route = vi.hoisted(() => ({ params: { secretId: 'secret-1' }, hash: '' }));

vi.mock('vue-router', () => ({ useRoute: () => route }));
const client = vi.hoisted(() => ({ get: vi.fn(), post: vi.fn() }));

vi.mock('@/utils/otsClient', () => ({ default: client }));

const encryptSecret = async (key: JsonWebKey, bytes: Uint8Array) => {
  const { encryptStream } = useEncryptData();

  return (await readAll(encryptStream({ key, data: streamOf([bytes]) }))).join('');
};

const buildHash = (secretKey: JsonWebKey, secretInfo: SecretInfo, checksum?: string) =>
  '#' + Base64.btoa(JSON.stringify({ secretKey, secretInfo, checksum }));

const readBlob = (blob: Blob) =>
  new Promise<Uint8Array>((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = () => resolve(new Uint8Array(reader.result as ArrayBuffer));
    reader.onerror = () => reject(reader.error);
    reader.readAsArrayBuffer(blob);
  });

const wrappers: VueWrapper[] = [];

const mountSecret = () => {
  const wrapper = mount(Secret, {
    global: {
      plugins: [queryPlugin()],
      stubs: { TheHeader: true, AudioPlayer: true },
      components: { RouterLink: { template: '<a><slot /></a>' } },
    },
  });

  wrappers.push(wrapper);

  return wrapper;
};

const mountAndReveal = async () => {
  const wrapper = mountSecret();

  await wrapper.get('button').trigger('click');

  return wrapper;
};

describe('Secret page', () => {
  let key: JsonWebKey;

  beforeEach(async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {});
    client.get.mockReset();
    key = await createEncryptionKey(DEFAULT_SECRET_LENGTH);
  });

  afterEach(() => {
    wrappers.splice(0).forEach(wrapper => wrapper.unmount());
  });

  it('only fetches the secret after the user asks to reveal it', async () => {
    route.hash = buildHash(key, { type: 'plain' });
    client.get.mockResolvedValue({ data: { secret: await encryptSecret(key, encodeText('hi')) } });

    const wrapper = mountSecret();

    expect(wrapper.text()).toContain('Click the button to reveal secret');
    expect(client.get).not.toHaveBeenCalled();

    await wrapper.get('button').trigger('click');
    await vi.waitFor(() => expect(client.get).toHaveBeenCalledWith('/secrets/secret-1'));
  });

  it('decrypts and shows a plain text secret', async () => {
    const text = 'correct horse battery staple ✓';

    route.hash = buildHash(key, { type: 'plain' });
    client.get.mockResolvedValue({ data: { secret: await encryptSecret(key, encodeText(text)) } });

    const wrapper = await mountAndReveal();

    await vi.waitFor(() =>
      expect((wrapper.find('textarea').element as HTMLTextAreaElement | undefined)?.value).toBe(text)
    );
  });

  it('decrypts multi-chunk secrets', async () => {
    const text = 'x'.repeat(40 * 1024) + 'end';

    route.hash = buildHash(key, { type: 'plain' });
    client.get.mockResolvedValue({ data: { secret: await encryptSecret(key, encodeText(text)) } });

    const wrapper = await mountAndReveal();

    await vi.waitFor(() =>
      expect((wrapper.find('textarea').element as HTMLTextAreaElement | undefined)?.value).toBe(text)
    );
  });

  it('decrypts a secret whose checksum matches', async () => {
    const text = 'x'.repeat(40 * 1024) + 'end';
    const secret = await encryptSecret(key, encodeText(text));

    route.hash = buildHash(key, { type: 'plain' }, await sha256Hex(secret));
    client.get.mockResolvedValue({ data: { secret } });

    const wrapper = await mountAndReveal();

    await vi.waitFor(() =>
      expect((wrapper.find('textarea').element as HTMLTextAreaElement | undefined)?.value).toBe(text)
    );
  });

  it('refuses a secret with a dropped chunk when the checksum does not match', async () => {
    // each chunk is valid AES-GCM on its own, only the checksum shows that the last one is gone
    const secret = await encryptSecret(key, encodeText('x'.repeat(40 * 1024) + 'end'));
    const truncated = secret.split('$_$').slice(0, -2).join('$_$') + '$_$';

    route.hash = buildHash(key, { type: 'plain' }, await sha256Hex(secret));
    client.get.mockResolvedValue({ data: { secret: truncated } });

    const wrapper = await mountAndReveal();

    await vi.waitFor(() => expect(wrapper.text()).toContain('The secret was changed after it was created!'));
    expect(wrapper.find('textarea').exists()).toBe(false);
  });

  it('shows an error when the secret was already revealed', async () => {
    route.hash = buildHash(key, { type: 'plain' });
    client.get.mockRejectedValue(Object.assign(new Error('Not Found'), { response: { status: 404 } }));

    const wrapper = await mountAndReveal();

    await vi.waitFor(() => expect(wrapper.text()).toContain('The secret has already been revealed!'));
    expect(wrapper.text()).toContain('Create a new secret');
  });

  it('shows an error instead of an empty secret when the key in the url is wrong', async () => {
    const wrongKey = await createEncryptionKey(DEFAULT_SECRET_LENGTH);

    route.hash = buildHash(wrongKey, { type: 'plain' });
    client.get.mockResolvedValue({ data: { secret: await encryptSecret(key, encodeText('secret')) } });

    const wrapper = await mountAndReveal();

    await vi.waitFor(() => expect(wrapper.text()).toContain('Cannot decrypt your secret!'));
    expect(wrapper.find('textarea').exists()).toBe(false);
  });

  it('shows an error when the url hash is missing or garbage', async () => {
    route.hash = '#not-base64-json';
    client.get.mockResolvedValue({ data: { secret: await encryptSecret(key, encodeText('secret')) } });

    const wrapper = await mountAndReveal();

    await vi.waitFor(() => expect(wrapper.text()).toContain('Cannot decrypt your secret!'));
  });

  it('hands the exact decrypted audio bytes to the player', async () => {
    const audio = randomBytes(20 * 1024);

    // this is the shape CreateSecret produces (the AudioSecretInfo type does not model it, hence the cast)
    route.hash = buildHash(key, {
      type: 'audio',
      info: { name: 'audio.webm', type: 'audio/webm' },
    } as unknown as SecretInfo);
    client.get.mockResolvedValue({ data: { secret: await encryptSecret(key, audio) } });

    const wrapper = await mountAndReveal();

    await vi.waitFor(() => expect(wrapper.findComponent(AudioPlayer).exists()).toBe(true));

    const blob = wrapper.findComponent(AudioPlayer).props('blob') as Blob;

    expect(blob.type).toBe('audio/webm');
    expect(await readBlob(blob)).toEqual(audio);
  });

  it('downloads a decrypted file under its original name', async () => {
    const content = randomBytes(1000);
    const createObjectURL = vi.fn((_blob: Blob) => 'blob:secret-file');
    const revokeObjectURL = vi.fn();
    const clicks: { href: string; download: string | null }[] = [];

    Object.assign(window.URL, { createObjectURL, revokeObjectURL });
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(function (this: HTMLAnchorElement) {
      clicks.push({ href: this.href, download: this.getAttribute('download') });
    });

    route.hash = buildHash(key, { type: 'file', info: { name: 'report.pdf', type: 'application/pdf' } });
    client.get.mockResolvedValue({ data: { secret: await encryptSecret(key, content) } });

    await mountAndReveal();

    await vi.waitFor(() => expect(clicks).toEqual([{ href: 'blob:secret-file', download: 'report.pdf' }]));

    const blob = createObjectURL.mock.calls[0][0];

    expect(blob.type).toBe('application/pdf');
    expect(await readBlob(blob)).toEqual(content);
    expect(revokeObjectURL).toHaveBeenCalledWith('blob:secret-file');
    expect(document.querySelector('a[download]')).toBeNull();
  });
});
