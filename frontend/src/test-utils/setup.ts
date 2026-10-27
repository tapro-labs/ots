/**
 * Global test setup.
 *
 * jsdom does not implement the Web Crypto API, so expose Node's implementation
 * on `window` (the app code calls `window.crypto.subtle`).
 */
/// <reference types="node" />
import { webcrypto } from 'node:crypto';

if (!window.crypto?.subtle) {
  Object.defineProperty(window, 'crypto', { value: webcrypto, configurable: true });
}
