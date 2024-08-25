import { Base64 } from 'js-base64';

export const uint8ArrayToBase64 = (bytes: Uint8Array): string => {
  return Base64.fromUint8Array(bytes);
};

/**
 * Assumes that the base64 encoded an unit8 array
 */
export const base64ToUint8Array = (base64String: string): Uint8Array => {
  return Base64.toUint8Array(base64String);
};

export const base64ToString = (base64String: string): string => {
  const bytes = base64ToUint8Array(base64String);

  return new TextDecoder().decode(bytes);
};
