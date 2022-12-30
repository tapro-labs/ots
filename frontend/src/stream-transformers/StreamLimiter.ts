export default class StreamLimiter {
  public readable: ReadableStream<string>;
  public writable: WritableStream<string>;

  constructor(protected bufferSize: number) {
    let onClose: Function;
    let onChunk: (_data: string) => void;

    this.readable = new ReadableStream({
      start(controller) {
        onChunk = data => controller.enqueue(data);
        onClose = () => controller.close();
      },
    });

    let buffer = '';
    this.writable = new WritableStream({
      write: data => {
        buffer += data;

        // we wait for more data if the buffer is not filled
        if (buffer.length < this.bufferSize) {
          return;
        }

        const chunked = buffer.slice(0, this.bufferSize);
        buffer = buffer.slice(this.bufferSize); // leftovers

        onChunk(chunked);
      },
      close: () => {
        while (buffer.length) {
          const chunked = buffer.slice(0, this.bufferSize);
          buffer = buffer.slice(this.bufferSize); // leftovers

          onChunk(chunked);
        }

        onClose();
      },
    });
  }
}
