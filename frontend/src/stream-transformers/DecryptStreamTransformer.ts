import { decrypt, SecretCryptograhyKey } from '@/utils/cryptography';

export default class DecryptStreamTransformer {
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
      write: async data => onChunk(await decrypt(key, data)),
      close() {
        onClose();
      },
    });
  }
}
