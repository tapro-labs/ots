import { describe, expect, it } from 'vitest';
import StreamLimiter from '@/stream-transformers/StreamLimiter';
import Uint8StreamLimiter from '@/stream-transformers/Uint8StreamLimiter';
import GenericStreamTransformation from '@/stream-transformers/GenericStreamTransformation';
import { concatBytes, randomBytes, readAll, transform } from '@/test-utils/streams';

const bytes = (...values: number[]) => new Uint8Array(values);
const asArrays = (chunks: Uint8Array[]) => chunks.map(chunk => Array.from(chunk));

describe('Uint8StreamLimiter', () => {
  it('re-chunks input into fixed size chunks with the remainder last', async () => {
    const output = await transform([bytes(1, 2), bytes(3, 4, 5), bytes(6, 7)], new Uint8StreamLimiter(3));

    expect(asArrays(output)).toEqual([[1, 2, 3], [4, 5, 6], [7]]);
  });

  it('splits a single chunk larger than the buffer size', async () => {
    const output = await transform([bytes(1, 2, 3, 4, 5, 6, 7)], new Uint8StreamLimiter(2));

    expect(asArrays(output)).toEqual([[1, 2], [3, 4], [5, 6], [7]]);
  });

  it('does not emit an empty trailing chunk when input is an exact multiple', async () => {
    const output = await transform([bytes(1, 2, 3, 4)], new Uint8StreamLimiter(2));

    expect(asArrays(output)).toEqual([
      [1, 2],
      [3, 4],
    ]);
  });

  it('emits nothing for an empty stream or empty chunks', async () => {
    expect(await transform([], new Uint8StreamLimiter(4))).toEqual([]);
    expect(await transform([bytes(), bytes()], new Uint8StreamLimiter(4))).toEqual([]);
  });

  it('preserves every byte for large inputs', async () => {
    const input = randomBytes(100_000);
    const output = await transform(
      [input.slice(0, 1), input.slice(1, 40_000), input.slice(40_000)],
      new Uint8StreamLimiter(16 * 1024)
    );

    expect(output.slice(0, -1).every(chunk => chunk.length === 16 * 1024)).toBe(true);
    expect(concatBytes(output)).toEqual(input);
  });

  it('errors its output when the source stream errors instead of hanging', async () => {
    const failingSource = new ReadableStream<Uint8Array>({
      pull(controller) {
        controller.error(new Error('read failed'));
      },
    });

    await expect(readAll(failingSource.pipeThrough(new Uint8StreamLimiter(4)))).rejects.toThrow('read failed');
  });
});

describe('StreamLimiter', () => {
  it('re-chunks strings and flushes leftovers on close, preserving order', async () => {
    const output = await transform(['ab', 'cdefgh', 'ij'], new StreamLimiter(3));

    expect(output).toEqual(['abc', 'def', 'ghi', 'j']);
  });

  it('emits nothing for an empty stream', async () => {
    expect(await transform([], new StreamLimiter(3))).toEqual([]);
  });
});

describe('GenericStreamTransformation', () => {
  it('maps every chunk through the transform function', async () => {
    const output = await transform([1, 2, 3], new GenericStreamTransformation((value: number) => `#${value * 2}`));

    expect(output).toEqual(['#2', '#4', '#6']);
  });

  it('propagates errors thrown by the transform function', async () => {
    const failing = new GenericStreamTransformation<string, string>(() => {
      throw new Error('boom');
    });

    await expect(transform(['x'], failing)).rejects.toThrow('boom');
  });
});
