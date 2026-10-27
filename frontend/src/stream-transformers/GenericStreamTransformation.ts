export default class GenericStreamTransformation<R, W> {
  public readable: ReadableStream<R>;
  public writable: WritableStream<W>;

  constructor(transformFunction: (_base64String: W) => R) {
    let onClose: Function;
    let onError: (_reason: unknown) => void;
    let onChunk: (_data: R) => void;

    this.readable = new ReadableStream({
      start(controller) {
        onChunk = data => controller.enqueue(data);
        onClose = () => controller.close();
        // without this an error on the writable side would leave readers waiting forever
        onError = reason => controller.error(reason);
      },
    });

    this.writable = new WritableStream({
      write: data => {
        try {
          onChunk(transformFunction(data));
        } catch (e) {
          onError(e);

          throw e;
        }
      },
      abort: reason => onError(reason),
      close() {
        onClose();
      },
    });
  }
}
