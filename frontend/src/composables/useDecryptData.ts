/**
 * External dependencies.
 */

import DecryptStreamTransformer from '@/stream-transformers/DecryptStreamTransformer';

/**
 * Internal dependencies.
 */

export type DecryptStreamOptions = {
  key: string;
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
