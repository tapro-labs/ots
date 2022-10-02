/**
 * External dependencies.
 */
import { computed } from 'vue';
import { useQuery } from 'vue-query';
import type { AxiosError } from 'axios';

/**
 * Internal dependencies.
 */
import otsClient from '@/utils/otsClient';
import useApiToken from '@/composables/integrations/slack/useApiToken';

export type SlackUser = {
  id: string;
  name: string;
  isBot: boolean;
  imageUrl: string;
};

export default function useFetchUsers() {
  const { apiToken, resetApiToken } = useApiToken();

  const { data, isLoading, isError } = useQuery<SlackUser[]>(
    ['slack-users', apiToken],
    async () => {
      try {
        const response = await otsClient.get('/integrations/slack/users', {
          headers: {
            Authorization: `Bearer ${apiToken.value}`,
          },
        });

        return response.data.users;
      } catch (e) {
        const error = e as AxiosError;

        if (error.response?.status === 401) {
          // if we have a 401
          // reset api token
          resetApiToken();
        }
      }
    },
    {
      enabled: computed(() => Boolean(apiToken.value)),
    }
  );

  return {
    isError,
    isLoading,
    users: computed(() => data.value || []),
  };
}
