/**
 * External dependencies.
 */

/**
 * Internal dependencies.
 */
import { SecretCryptograhyKey } from '@/utils/cryptography';
import DecryptStreamTransformer from '@/stream-transformers/DecryptStreamTransformer';

export type DecryptStreamOptions = {
  key: SecretCryptograhyKey;
  data: ReadableStream<string>;
};

export default function useDecryptData() {
  const decryptStream = (options: DecryptStreamOptions) => {
    return options.data.pipeThrough(new DecryptStreamTransformer(options.key));
  };

  return {
    decryptStream,
  };
}
