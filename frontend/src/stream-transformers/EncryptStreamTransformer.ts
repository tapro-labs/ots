import { encrypt } from '@/utils/cryptography';

export default class EncryptStreamTransformer {
  public static readonly SEPARATOR = '$_$';
  public readable: ReadableStream<string>;
  public writable: WritableStream<string>;

  constructor(key: string) {
    let onClose: Function;
    let onChunk: (_data: string) => void;

    this.readable = new ReadableStream({
      start(controller) {
        onChunk = data => controller.enqueue(data);
        onClose = () => controller.close();
      },
    });

    this.writable = new WritableStream({
      write: data => onChunk(encrypt(key, data) + EncryptStreamTransformer.SEPARATOR),
      close() {
        onClose();
      },
    });
  }
}
