export const uint8ArrayToBase64 = (bytes: Uint8Array): string => {
  let binary = '';
  const len = bytes.byteLength;

  for (let i = 0; i < len; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return window.btoa(binary);
};

/**
 * Assumes that the base64 encoded an unit8 array
 */
export const base64ToUint8Array = (base64String: string): Uint8Array => {
  const data = window.atob(base64String);
  const buffer = new Uint8Array(data.length);

  for (let i = 0; i < data.length; i++) {
    buffer.fill(data.charCodeAt(i), i, i + 1);
  }

  return buffer;
};

export const mergeUint8Array = (bytesArray: Uint8Array[]): Uint8Array => {
  // Get the total length of all arrays.
  let length = 0;
  bytesArray.forEach(item => {
    length += item.length;
  });

  // Create a new array with total length and merge all source arrays.
  let mergedArray = new Uint8Array(length);
  let offset = 0;
  bytesArray.forEach(item => {
    mergedArray.set(item, offset);
    offset += item.length;
  });

  return mergedArray;
};
