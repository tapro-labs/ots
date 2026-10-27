import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { nextTick } from 'vue';

const STORAGE_KEY = 'slack_api_token';

// the token lives in module scope, so load a fresh copy of the module per test
const loadUseApiToken = async () => (await import('@/composables/integrations/slack/useApiToken')).default;

describe('useApiToken', () => {
  beforeEach(() => {
    vi.resetModules();
    localStorage.clear();
    window.history.replaceState(null, '', '/');
  });

  afterEach(() => {
    window.history.replaceState(null, '', '/');
  });

  it('has no token by default', async () => {
    const useApiToken = await loadUseApiToken();

    expect(useApiToken().apiToken.value).toBeNull();
  });

  it('picks the token up from the url and persists it', async () => {
    window.history.replaceState(null, '', `/?${STORAGE_KEY}=xoxp-from-url`);
    const useApiToken = await loadUseApiToken();

    expect(useApiToken().apiToken.value).toBe('xoxp-from-url');
    await nextTick();
    expect(localStorage.getItem(STORAGE_KEY)).toBe('xoxp-from-url');
  });

  it('restores a token stored in localStorage', async () => {
    localStorage.setItem(STORAGE_KEY, 'stored-token');
    const useApiToken = await loadUseApiToken();

    expect(useApiToken().apiToken.value).toBe('stored-token');
  });

  it('shares the token between callers and can set and reset it', async () => {
    const useApiToken = await loadUseApiToken();
    const first = useApiToken();
    const second = useApiToken();

    first.setApiToken('new-token');
    expect(second.apiToken.value).toBe('new-token');

    second.resetApiToken();
    expect(first.apiToken.value).toBeNull();
    await nextTick();
    expect(localStorage.getItem(STORAGE_KEY)).toBeNull();
  });
});
