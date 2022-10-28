/**
 * External dependencies.
 */
import { computed, ref, Ref } from 'vue';
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

  const { data, isLoading, fetchStatus, isError } = useQuery(
    ['secret', secretId],
    async () => {
      const response = await otsClient.get(`/secrets/${secretId}`);

      return response.data.secret as SecretData;
    },
    {
      enabled,
    }
  );

  return {
    isError,
    secret: data,
    // Query has changed how isLoading works 🤦‍
    // @see https://github.com/TanStack/query/issues/3584 (reported issue)
    // @see https://github.com/TanStack/query/issues/3975#issuecomment-1245101647 (v5 fix)
    isLoading: computed(() => isLoading.value && fetchStatus.value !== 'idle'),
  };
}
