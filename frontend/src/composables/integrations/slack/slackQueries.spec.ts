import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import useApiToken from '@/composables/integrations/slack/useApiToken';
import useFetchUsers from '@/composables/integrations/slack/useFetchUsers';
import useSendMessage from '@/composables/integrations/slack/useSendMessage';
import { withQuery } from '@/test-utils/query';

const client = vi.hoisted(() => ({ get: vi.fn(), post: vi.fn() }));

vi.mock('@/utils/otsClient', () => ({ default: client }));
const users = [{ id: 'U1', name: 'Ada', isBot: false, deleted: false, imageUrl: 'https://img/ada.png' }];

describe('slack integration queries', () => {
  beforeEach(() => {
    client.get.mockReset();
    client.post.mockReset();
    useApiToken().resetApiToken();
  });

  describe('useFetchUsers', () => {
    it('does not call the api and returns no users without a token', async () => {
      const { result } = withQuery(() => useFetchUsers());

      await flushPromises();

      expect(client.get).not.toHaveBeenCalled();
      expect(result.users.value).toEqual([]);
      expect(result.isLoading.value).toBe(false);
    });

    it('fetches users with the token as bearer auth', async () => {
      useApiToken().setApiToken('xoxp-1');
      client.get.mockResolvedValue({ data: { users } });
      const { result } = withQuery(() => useFetchUsers());

      await vi.waitFor(() => expect(result.users.value).toEqual(users));

      expect(client.get).toHaveBeenCalledWith('/integrations/slack/users', {
        headers: { Authorization: 'Bearer xoxp-1' },
      });
    });

    it('drops the stored token when slack rejects it with 401', async () => {
      useApiToken().setApiToken('expired');
      client.get.mockRejectedValue(Object.assign(new Error('Unauthorized'), { response: { status: 401 } }));
      const { result } = withQuery(() => useFetchUsers());

      await vi.waitFor(() => expect(useApiToken().apiToken.value).toBeNull());
      expect(result.users.value).toEqual([]);
    });

    it('keeps the token on other errors', async () => {
      useApiToken().setApiToken('valid');
      client.get.mockRejectedValue(Object.assign(new Error('Server Error'), { response: { status: 500 } }));
      withQuery(() => useFetchUsers());

      await vi.waitFor(() => expect(client.get).toHaveBeenCalled());
      await flushPromises();

      expect(useApiToken().apiToken.value).toBe('valid');
    });
  });

  describe('useSendMessage', () => {
    it('posts the message to the channel with the token', async () => {
      useApiToken().setApiToken('xoxp-2');
      client.post.mockResolvedValue({ data: {} });
      const { result } = withQuery(() => useSendMessage());

      await result.sendMessage({ channelId: 'U1', message: 'https://ots/secret/1#key' });

      expect(client.post).toHaveBeenCalledWith(
        '/integrations/slack/send-message',
        { channelId: 'U1', message: 'https://ots/secret/1#key' },
        { headers: { Authorization: 'Bearer xoxp-2' } }
      );
    });
  });
});
