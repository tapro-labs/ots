/**
 * External dependencies.
 */
import { useMutation } from '@tanstack/vue-query';

/**
 * Internal dependencies.
 */
import otsClient from '@/utils/otsClient';
import useApiToken from '@/composables/integrations/slack/useApiToken';

export type SendMessageData = {
  channelId: string;
  message: string;
};

export default function useSendMessage() {
  const { apiToken } = useApiToken();

  const { isLoading, mutateAsync } = useMutation((data: SendMessageData) =>
    otsClient.post('/integrations/slack/send-message', data, {
      headers: {
        Authorization: `Bearer ${apiToken.value}`,
      },
    })
  );

  return {
    isSending: isLoading,
    sendMessage: mutateAsync,
  };
}
