/**
 * Internal dependencies.
 */
import { FileInfo } from '@/types/FileInfo';

export type PlainSecretInfo = {
  type: 'plain';
};

export type FileSecretInfo = {
  type: 'file';
  info: FileInfo;
};

export type SecretInfo = PlainSecretInfo | FileSecretInfo;
