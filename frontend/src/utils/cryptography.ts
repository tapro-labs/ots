/**
 * External dependencies.
 */
import sjcl from 'sjcl';

/**
 * Internal dependencies.
 */

export type SecretCryptograhyKey = JsonWebKey;

export const DEFAULT_SECRET_LENGTH = 256;

export const createRandomSecret = async (length: number): Promise<SecretCryptograhyKey> => {
  try {
    const secretKey = await window.crypto.subtle.generateKey({ name: 'AES-GCM', length }, true, ['encrypt', 'decrypt']);

    console.log(secretKey);
    return await window.crypto.subtle.exportKey('jwk', secretKey);
  } catch (e) {
    console.error(e);

    throw e;
  }
};

const ct = new TextEncoder().encode('�#�|J\x1E�7td\x16j');

export const encrypt = async (secret: SecretCryptograhyKey, data: string): Promise<string> => {
  try {
    const algorithm = 'AES-GCM';
    const encoder = new TextEncoder();
    const dataBuffer = encoder.encode(data);
    const key = await window.crypto.subtle.importKey('jwk', secret, { name: algorithm }, true, ['encrypt']);
    const encryptedData = await window.crypto.subtle.encrypt({ name: algorithm, iv: ct }, key, dataBuffer);

    const encryptedDataArray = Array.from(new Uint8Array(encryptedData));

    // Convert the encrypted data to a base64 string
    const base64EncryptedData = btoa(encryptedDataArray.map(byte => String.fromCharCode(byte)).join(''));

    return base64EncryptedData;
  } catch (e) {
    console.error(e);

    throw e;
  }
};

// We assume that decrypt will always receive data in base64 format
export const decrypt = async (secret: SecretCryptograhyKey, encryptedData: string): Promise<string> => {
  try {
    const algorithm = 'AES-GCM';
    const decoder = new TextDecoder();
    const key = await window.crypto.subtle.importKey('jwk', secret, { name: algorithm }, true, ['encrypt', 'decrypt']);

    const encryptedDataArray = atob(encryptedData)
      .split('')
      .map(char => char.charCodeAt(0));

    const decryptedDataBuffer = await window.crypto.subtle.decrypt(
      { name: algorithm, iv: ct },
      key,
      new Uint8Array(encryptedDataArray)
    );

    const decryptedData = decoder.decode(decryptedDataBuffer);

    return decryptedData;
  } catch (e) {
    console.error(e);
  }
};
