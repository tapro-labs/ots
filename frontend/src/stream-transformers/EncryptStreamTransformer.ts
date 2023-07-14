import { encrypt, SecretCryptograhyKey } from '@/utils/cryptography';

export default class EncryptStreamTransformer {
  public static readonly SEPARATOR = '$_$';
  public readable: ReadableStream<string>;
  public writable: WritableStream<string>;

  constructor(key: SecretCryptograhyKey) {
    let onClose: Function;
    let onChunk: (_data: string) => void;

    this.readable = new ReadableStream({
      start(controller) {
        onChunk = data => controller.enqueue(data);
        onClose = () => controller.close();
      },
    });

    this.writable = new WritableStream({
      write: async data => {
        const encrypted = (await encrypt(key, data)) + EncryptStreamTransformer.SEPARATOR;
        onChunk(encrypted);
      },
      close() {
        onClose();
      },
    });
  }
}
