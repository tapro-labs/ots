import { afterEach, describe, expect, it, vi } from 'vitest';
import Config from '@/utils/otsConfig';
import useConfig from '@/composables/useConfig';

describe('otsConfig', () => {
  afterEach(() => {
    vi.unstubAllEnvs();
  });

  it('is a singleton shared with useConfig()', () => {
    expect(Config.getInstance()).toBe(Config.getInstance());
    expect(useConfig()).toBe(Config.getInstance());
  });

  it.each([
    ['slack', true],
    ['audio,SLACK', true],
    ['audio', false],
    ['', false],
  ])('detects the slack feature from VITE_ENABLED_FEATURES=%j', (features, expected) => {
    vi.stubEnv('VITE_ENABLED_FEATURES', features);

    expect(Config.getInstance().isSlackFeatureEnabled).toBe(expected);
  });

  it('reads the backend url and slack connection params from the environment', () => {
    vi.stubEnv('VITE_OTS_URL', 'https://ots.example.com/api');
    vi.stubEnv('VITE_SLACK_CONNECT_QUERY_PARAMS', 'client_id=1');

    expect(Config.getInstance().backendUrl).toBe('https://ots.example.com/api');
    expect(Config.getInstance().slackConnectionParams).toBe('client_id=1');
  });

  it('encrypts 16KB at a time', () => {
    expect(Config.getInstance().encryptionByteSize).toBe(16 * 1024);
  });
});
