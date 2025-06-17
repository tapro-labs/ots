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

export type AudioSecretInfo = FileInfo & { type: 'audio' };

export type SecretInfo = PlainSecretInfo | FileSecretInfo | AudioSecretInfo;
