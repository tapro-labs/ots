/**
 * Internal dependencies.
 */
import Config, { OtsConfig } from '@/utils/otsConfig';

export default function useConfig(): OtsConfig {
  return Config.getInstance();
}
