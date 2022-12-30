class FileTooLargeError extends Error {
  constructor(public readonly size: number) {
    super('File too large!');
  }

  get sizeInKB() {
    return this.size / 1024;
  }

  get sizeInMB() {
    return this.sizeInKB / 1024;
  }

  get sizeInGB() {
    return this.sizeInMB / 1024;
  }
}

export default FileTooLargeError;
