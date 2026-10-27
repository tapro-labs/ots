import { encrypt, SecretCryptograhyKey } from '@/utils/cryptography';

export default class EncryptStreamTransformer {
  public static readonly SEPARATOR = '$_$';
  public readable: ReadableStream<string>;
  public writable: WritableStream<string>;

  constructor(key: SecretCryptograhyKey) {
    let onClose: Function;
    let onError: (_reason: unknown) => void;
    let onChunk: (_data: string) => void;

    this.readable = new ReadableStream({
      start(controller) {
        onChunk = data => controller.enqueue(data);
        onClose = () => controller.close();
        // without this an error on the writable side would leave readers waiting forever
        onError = reason => controller.error(reason);
      },
    });

    this.writable = new WritableStream({
      write: async data => {
        try {
          const encrypted = (await encrypt(key, data)) + EncryptStreamTransformer.SEPARATOR;
          onChunk(encrypted);
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
