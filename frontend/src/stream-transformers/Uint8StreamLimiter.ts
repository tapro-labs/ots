export default class Uint8StreamLimiter {
  public readable: ReadableStream<Uint8Array>;
  public writable: WritableStream<Uint8Array>;

  constructor(protected bufferSize: number) {
    let onClose: Function;
    let onError: (_reason: unknown) => void;
    let onChunk: (_data: Uint8Array) => void;

    this.readable = new ReadableStream({
      start(controller) {
        onChunk = data => controller.enqueue(data);
        onClose = () => controller.close();
        // without this an error on the writable side would leave readers waiting forever
        onError = reason => controller.error(reason);
      },
    });

    let buffer: number[] = [];
    const emptyBufferOut = () => {
      while (buffer.length) {
        const chunked = buffer.slice(0, this.bufferSize);
        buffer = buffer.slice(this.bufferSize); // leftovers

        onChunk(new Uint8Array(chunked));
      }
    };

    this.writable = new WritableStream({
      write: data => {
        const iterations = Math.ceil(Math.max(data.length / this.bufferSize, 1));

        for (let i = 0; i < iterations; i++) {
          buffer.push(...data.slice(i * this.bufferSize, (i + 1) * this.bufferSize));
        }

        while (buffer.length >= this.bufferSize) {
          const chunked = buffer.slice(0, this.bufferSize);
          buffer = buffer.slice(this.bufferSize); // leftovers

          onChunk(new Uint8Array(chunked));
        }
      },
      abort: reason => onError(reason),
      close: () => {
        emptyBufferOut();
        onClose();
      },
    });
  }
}
