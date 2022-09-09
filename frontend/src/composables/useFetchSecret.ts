/**
 * External dependencies.
 */
import { ref, Ref } from 'vue';
import { useQuery } from 'vue-query';

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

  const { data, isLoading, isError } = useQuery(
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
    isLoading,
    secret: data,
  };
}
