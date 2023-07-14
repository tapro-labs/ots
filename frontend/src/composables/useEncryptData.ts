/**
 * External dependencies.
 */

/**
 * Internal dependencies.
 */
import { uint8ArrayToBase64 } from '@/utils/helpers';
import Uint8StreamLimiter from '@/stream-transformers/Uint8StreamLimiter';
import EncryptStreamTransformer from '@/stream-transformers/EncryptStreamTransformer';
import GenericStreamTransformation from '@/stream-transformers/GenericStreamTransformation';
import OtsConfig from '@/utils/otsConfig';
import { SecretCryptograhyKey } from '@/utils/cryptography';

export type EncryptStreamOptions = {
  key: SecretCryptograhyKey;
  data: ReadableStream<Uint8Array>;
};

export default function useEncryptData() {
  const encryptStream = (options: EncryptStreamOptions) => {
    return options.data
      .pipeThrough(new Uint8StreamLimiter(OtsConfig.getInstance().encryptionByteSize))
      .pipeThrough(new GenericStreamTransformation(uint8ArrayToBase64))
      .pipeThrough(new EncryptStreamTransformer(options.key));
  };

  return {
    encryptStream,
  };
}
