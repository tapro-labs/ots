import EncryptStreamTransformer from '@/stream-transformers/EncryptStreamTransformer';

/**
 * Splits a stored secret (encrypted chunks joined by the separator) back into its chunks,
 * the same way the Secret page does.
 */
export const splitEncryptedSecret = (secret: string): string[] =>
  secret.split(EncryptStreamTransformer.SEPARATOR).filter(chunk => chunk.length > 0);
