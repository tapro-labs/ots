/**
 * Builds a readable stream that emits the given chunks in order and then closes.
 */
export const streamOf = <T>(chunks: T[]): ReadableStream<T> => {
  const queue = [...chunks];

  return new ReadableStream<T>({
    pull(controller) {
      if (!queue.length) {
        controller.close();

        return;
      }

      controller.enqueue(queue.shift() as T);
    },
  });
};

/**
 * Reads a stream to the end and returns every chunk it produced.
 */
export const readAll = async <T>(stream: ReadableStream<T>): Promise<T[]> => {
  const reader = stream.getReader();
  const chunks: T[] = [];

  for (;;) {
    const { done, value } = await reader.read();

    if (done) {
      return chunks;
    }

    chunks.push(value);
  }
};

/**
 * Runs chunks through a transformer (readable/writable pair) and returns the output chunks.
 */
export const transform = <I, O>(
  chunks: I[],
  transformer: { readable: ReadableStream<O>; writable: WritableStream<I> }
): Promise<O[]> => readAll(streamOf(chunks).pipeThrough(transformer));

export const concatBytes = (chunks: Uint8Array[]): Uint8Array => {
  const result = new Uint8Array(chunks.reduce((total, chunk) => total + chunk.length, 0));
  let offset = 0;

  chunks.forEach(chunk => {
    result.set(chunk, offset);
    offset += chunk.length;
  });

  return result;
};

export const randomBytes = (length: number): Uint8Array => {
  const bytes = new Uint8Array(length);

  // getRandomValues accepts at most 65536 bytes per call
  for (let offset = 0; offset < length; offset += 65536) {
    window.crypto.getRandomValues(bytes.subarray(offset, Math.min(offset + 65536, length)));
  }

  return bytes;
};
