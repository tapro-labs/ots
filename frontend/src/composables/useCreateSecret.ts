/**
 * External dependencies.
 */
import { useMutation } from '@tanstack/vue-query';

/**
 * Internal dependencies.
 */
import otsClient from '@/utils/otsClient';
import { SecretId } from '@/types/SecretTypes';
import useEncryptData, { EncryptStreamOptions } from '@/composables/useEncryptData';

export type CreateSecretOptions = EncryptStreamOptions & {
  expirySeconds: number;
};

export default function useCreateSecret() {
  const { encryptStream } = useEncryptData();
  const { mutateAsync, isLoading: isCreating } = useMutation(async (data: CreateSecretOptions) => {
    const stream = encryptStream(data);

    let secretData = '';
    const reader = stream.getReader();
    let read: any;

    do {
      read = await reader.read();

      if (read.value) {
        secretData += read.value;
      }
    } while (!read.done);

    const response = await otsClient.post('/secrets', {
      secret: secretData,
      expiry_seconds: data.expirySeconds,
    });

    return response.data.secretId as SecretId;
  });

  return {
    isCreating,
    createSecret: mutateAsync,
  };
}
