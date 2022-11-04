/**
 * External dependencies.
 */
import sjcl from 'sjcl';

/**
 * Internal dependencies.
 */

export type SecretCryptograhyKey = string;

// 32 big numbers seems that are base64 encoded seems big enough for a password derived key
export const DEFAULT_SECRET_LENGTH = 32;

export const createRandomSecret = async (length: number): Promise<SecretCryptograhyKey> => {
  const waitForSjclToBeReady = () => {
    return new Promise((resolve, reject) => {
      const MAX_RETRY_TIME = 20;
      const waitUntilSjclIsResolved = (times = 0) => {
        if (times >= MAX_RETRY_TIME) {
          reject(new Error('SJCL exceeded retry time!'));
          return;
        }

        if (!sjcl.random.isReady()) {
          setTimeout(() => waitUntilSjclIsResolved(times + 1), 10);
          return;
        }

        resolve(null);
      };

      waitUntilSjclIsResolved();
    });
  };

  await waitForSjclToBeReady();

  const words = sjcl.random.randomWords(length);

  // btoa works for numbers as well
  // so we ignore here to mitigate the typescript complaint
  // @ts-ignore
  return words.map(window.btoa).join('');
};

export const encrypt = (secret: SecretCryptograhyKey, data: string): string =>
  window.btoa(sjcl.encrypt(secret, data) as any);

// We assume that decrypt will always receive data in base64 format
export const decrypt = (secret: SecretCryptograhyKey, encryptedData: string): string =>
  sjcl.decrypt(secret, window.atob(encryptedData));
