/**
 * External dependencies.
 */
import { useMutation } from '@tanstack/vue-query';

/**
 * Internal dependencies.
 */
import otsClient from '@/utils/otsClient';
import { SecretId } from '@/types/SecretTypes';

export type CreateSecretOptions = {
  secret: string;
};

export default function useCreateSecret() {
  const { mutateAsync, isLoading: isCreating } = useMutation(async (data: CreateSecretOptions) => {
    const response = await otsClient.post('/secrets', {
      secret: data.secret,
    });

    return response.data.secretId as SecretId;
  });

  return {
    isCreating,
    createSecret: mutateAsync,
  };
}
