/**
 * External dependencies.
 */
import { computed } from 'vue';
import { useQuery } from '@tanstack/vue-query';
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
  deleted: boolean;
  imageUrl: string;
};

export default function useFetchUsers() {
  const { apiToken, resetApiToken } = useApiToken();

  const { data, isLoading, isError, fetchStatus } = useQuery<SlackUser[]>(
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
    users: computed(() => data.value || []),
    // Query has changed how isLoading works 🤦‍
    // @see https://github.com/TanStack/query/issues/3584 (reported issue)
    // @see https://github.com/TanStack/query/issues/3975#issuecomment-1245101647 (v5 fix)
    isLoading: computed(() => isLoading.value && fetchStatus.value !== 'idle'),
  };
}
