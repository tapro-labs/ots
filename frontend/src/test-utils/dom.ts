/**
 * jsdom has no DragEvent/DataTransfer, so emulate the parts the app uses.
 */
export const dragEvent = (type: string, files?: File[]): Event => {
  const event = new Event(type, { bubbles: true, cancelable: true });

  if (files) {
    Object.defineProperty(event, 'dataTransfer', { value: { files } });
  }

  return event;
};

export const fileOfSize = (size: number, name = 'file.bin', type = 'application/octet-stream'): File => {
  const file = new File(['x'], name, { type });

  Object.defineProperty(file, 'size', { value: size });

  return file;
};

/**
 * `input.files` is read-only in jsdom, override it to simulate a user picking files.
 */
export const setInputFiles = (input: HTMLInputElement, files: File[]) => {
  Object.defineProperty(input, 'files', { value: files, configurable: true });
};
