/**
 * External dependencies.
 */
import sjcl from 'sjcl';
import { generate } from 'generate-password-browser';

/**
 * Internal dependencies.
 */

export type SecretCryptograhyKey = string;

export const DEFAULT_SECRET_LENGTH = 64;

export const createRandomSecret = async (length: number): Promise<SecretCryptograhyKey> => generate({ length });

export const encrypt = (secret: SecretCryptograhyKey, data: string): string =>
  window.btoa(sjcl.encrypt(secret, data) as any);

// We assume that decrypt will always receive data in base64 format
export const decrypt = (secret: SecretCryptograhyKey, encryptedData: string): string =>
  sjcl.decrypt(secret, window.atob(encryptedData));
