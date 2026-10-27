/// <reference types="vitest/config" />
/**
 * External dependencies.
 */
import dns from 'dns';
import fs from 'fs';
import path from 'path';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

/**
 * Internal dependencies.
 */

/**
 * Allows us to display localhost as the server when starting dev server
 * instead of 127.0.0.1
 * @link: https://vitejs.dev/config/server-options.html#server-host
 */
dns.setDefaultResultOrder('verbatim');

type TSConfig = {
  compilerOptions: {
    paths: { [key: string]: string[] };
  };
};

const tsConfig: TSConfig = JSON.parse(fs.readFileSync(path.resolve('./tsconfig.json'), 'utf8'));
// Vite needs one string per alias, so use the first path of each tsconfig entry.
const alias = Object.keys(tsConfig.compilerOptions.paths).reduce<Record<string, string>>((acc, aliasKey) => {
  acc[aliasKey.replace('/*', '')] = path.resolve(tsConfig.compilerOptions.paths[aliasKey][0].replace('/*', ''));

  return acc;
}, {});

const getHttpsOptions = () => {
  try {
    return {
      key: fs.readFileSync(path.resolve('../server_ssl/default.key')),
      cert: fs.readFileSync(path.resolve('../server_ssl/default.crt')),
    };
  } catch (e) {
    console.warn(
      'Private key and certificate were not found. These are required for hot-module replacement. See server-ssl/README.md'
    );

    return undefined;
  }
};

const inADockerContainer = fs.existsSync(path.resolve('/.dockerenv'));

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    open: !inADockerContainer,
    port: 3000,
    host: inADockerContainer ? '0.0.0.0' : 'localhost',
    https: getHttpsOptions(),
  },
  plugins: [vue()],
  resolve: {
    alias: {
      ...alias,
    },
  },
  test: {
    environment: 'jsdom',
    include: ['src/**/*.spec.ts'],
    setupFiles: ['src/test-utils/setup.ts'],
    restoreMocks: true,
  },
});
