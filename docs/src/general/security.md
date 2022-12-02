# Security

- The security comes from creating a unique password and deriving a secret key with [PBKDF2](https://en.wikipedia.org/wiki/PBKDF2). AES cipher in [CCM](https://en.wikipedia.org/wiki/CCM_mode) mode is used to encrypt and decrypt the data.
- The server is never aware of the key and all the encryption/decryption happens in the user's browser.
- The encrypted data is sent and stored on the server for one day or until someone reads the secret

### Resources

- [AES](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard)
- [PBKDF2](https://en.wikipedia.org/wiki/PBKDF2)
- [AES CCM](https://en.wikipedia.org/wiki/CCM_mode)
