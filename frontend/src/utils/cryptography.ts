import { Base64 } from 'js-base64';
import { base64ToUint8Array, decodeText, encodeText, uint8ArrayToBase64 } from '@/utils/helpers';

export type SecretCryptograhyKey = JsonWebKey;

export const DEFAULT_IV_LENGTH = 12;
export const DEFAULT_SECRET_LENGTH = 256;
export const ENCRYPTION_ALGORITHM = 'AES-GCM';

export const createEncryptionKey = async (length: number): Promise<SecretCryptograhyKey> => {
  try {
    const secretKey = await window.crypto.subtle.generateKey({ name: ENCRYPTION_ALGORITHM, length }, true, [
      'encrypt',
      'decrypt',
    ]);

    return await window.crypto.subtle.exportKey('jwk', secretKey);
  } catch (e) {
    console.error(e);

    throw e;
  }
};

export const generateIV = () => window.crypto.getRandomValues(new Uint8Array(DEFAULT_IV_LENGTH));
export const convertToCryptoKey = (secret: SecretCryptograhyKey) =>
  window.crypto.subtle.importKey('jwk', secret, { name: ENCRYPTION_ALGORITHM }, true, ['decrypt', 'encrypt']);

export const encrypt = async (secret: SecretCryptograhyKey, data: string): Promise<string> => {
  try {
    const encoder = new TextEncoder();
    const buffer = encoder.encode(data);
    const key = await convertToCryptoKey(secret);
    const iv = generateIV();
    const encryptedData = new Uint8Array(
      await window.crypto.subtle.encrypt({ name: ENCRYPTION_ALGORITHM, iv }, key, buffer)
    );
    const ivAndEncryptedData = new Uint8Array(iv.length + encryptedData.byteLength);

    ivAndEncryptedData.set(iv);
    ivAndEncryptedData.set(encryptedData, iv.length);

    return uint8ArrayToBase64(ivAndEncryptedData);
  } catch (e) {
    console.error(e);

    throw e;
  }
};

// We assume that decrypt will always receive data in base64 format
export const decrypt = async (secret: SecretCryptograhyKey, encryptedData: string): Promise<string> => {
  try {
    const key = await convertToCryptoKey(secret);
    const ivAndEncryptedData = base64ToUint8Array(encryptedData);
    const iv = ivAndEncryptedData.slice(0, DEFAULT_IV_LENGTH);
    const encryptedBytes = ivAndEncryptedData.slice(DEFAULT_IV_LENGTH);
    const decrypted = await window.crypto.subtle.decrypt({ name: ENCRYPTION_ALGORITHM, iv }, key, encryptedBytes);

    return decodeText(decrypted);
  } catch (e) {
    console.error(e);

    throw e;
  }
};
