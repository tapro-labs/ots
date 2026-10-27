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

  // In v5, isLoading is only true while the first fetch runs (isPending && isFetching),
  // so a disabled query does not report that it is loading.
  const { data, isLoading, isError } = useQuery<SlackUser[]>({
    queryKey: ['slack-users', apiToken],
    queryFn: async () => {
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
    enabled: computed(() => Boolean(apiToken.value)),
  });

  return {
    isError,
    users: computed(() => data.value || []),
    isLoading,
  };
}
