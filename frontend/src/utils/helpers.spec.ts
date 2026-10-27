import { describe, expect, it } from 'vitest';
import { base64ToString, base64ToUint8Array, decodeText, encodeText, uint8ArrayToBase64 } from '@/utils/helpers';

describe('helpers', () => {
  it('encodes bytes to standard base64', () => {
    expect(uint8ArrayToBase64(new Uint8Array([104, 105]))).toBe('aGk=');
    expect(uint8ArrayToBase64(new Uint8Array([]))).toBe('');
  });

  it('round-trips every byte value through base64', () => {
    const bytes = new Uint8Array(256).map((_, index) => index);

    expect(Array.from(base64ToUint8Array(uint8ArrayToBase64(bytes)))).toEqual(Array.from(bytes));
  });

  it('decodes base64 to a UTF-8 string, including multi-byte characters', () => {
    const text = 'Пароль: 🔐 ünïcödé';

    expect(base64ToString(uint8ArrayToBase64(encodeText(text)))).toBe(text);
  });

  it('encodes and decodes text symmetrically from Uint8Array and ArrayBuffer', () => {
    const encoded = encodeText('secret €');

    expect(Array.from(encoded)).toEqual([115, 101, 99, 114, 101, 116, 32, 226, 130, 172]);
    expect(decodeText(encoded)).toBe('secret €');
    expect(decodeText(new Uint8Array(encoded).buffer)).toBe('secret €');
  });
});
