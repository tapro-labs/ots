/**
 * External dependencies.
 */
import { ref, Ref } from 'vue';
import { useQuery } from '@tanstack/vue-query';

/**
 * Internal dependencies.
 */
import otsClient from '@/utils/otsClient';
import { SecretData, SecretId } from '@/types/SecretTypes';

export type FetchSecretOptions = {
  secretId: SecretId;
  enabled?: Ref<boolean>;
};

export default function useFetchSecret({ secretId, enabled }: FetchSecretOptions) {
  enabled = enabled ?? ref(false);

  // In v5, isLoading is only true while the first fetch runs (isPending && isFetching),
  // so a disabled query does not report that it is loading.
  const { data, isLoading, isError } = useQuery({
    queryKey: ['secret', secretId],
    queryFn: async () => {
      const response = await otsClient.get(`/secrets/${secretId}`);

      return response.data.secret as SecretData;
    },
    enabled,
  });

  return {
    isError,
    secret: data,
    isLoading,
  };
}
