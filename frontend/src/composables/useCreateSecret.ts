/**
 * External dependencies.
 */
import { useMutation } from 'vue-query';

/**
 * Internal dependencies.
 */
import otsClient from '@/utils/otsClient';
import { SecretId } from '@/types/SecretTypes';

export type CreateSecretOptions = {
  secret: string;
};

export default function useCreateSecret() {
  const { mutateAsync, isLoading: isCreating } = useMutation('create-secret', async (data: CreateSecretOptions) => {
    const response = await otsClient.post('/secret', {
      secret: data.secret,
    });

    return response.data.secretId as SecretId;
  });

  return {
    isCreating,
    createSecret: mutateAsync,
  };
}
