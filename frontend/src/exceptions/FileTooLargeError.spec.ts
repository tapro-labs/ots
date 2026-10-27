import { describe, expect, it } from 'vitest';
import FileTooLargeError from '@/exceptions/FileTooLargeError';

describe('FileTooLargeError', () => {
  it('is an Error carrying the offending size', () => {
    const error = new FileTooLargeError(2048);

    expect(error).toBeInstanceOf(Error);
    expect(error.message).toBe('File too large!');
    expect(error.size).toBe(2048);
  });

  it('converts the size to binary units', () => {
    const error = new FileTooLargeError(3 * 1024 * 1024 * 1024);

    expect(error.sizeInKB).toBe(3 * 1024 * 1024);
    expect(error.sizeInMB).toBe(3 * 1024);
    expect(error.sizeInGB).toBe(3);
    expect(new FileTooLargeError(512).sizeInKB).toBe(0.5);
  });
});
