export default class GenericStreamTransformation<R, W> {
  public readable: ReadableStream<R>;
  public writable: WritableStream<W>;

  constructor(transformFunction: (_base64String: W) => R) {
    let onClose: Function;
    let onChunk: (_data: R) => void;

    this.readable = new ReadableStream({
      start(controller) {
        onChunk = data => controller.enqueue(data);
        onClose = () => controller.close();
      },
    });

    this.writable = new WritableStream({
      write: data => onChunk(transformFunction(data)),
      close() {
        onClose();
      },
    });
  }
}
