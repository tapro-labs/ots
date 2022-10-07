/**
 * External dependencies.
 */
import fs from 'fs';
import path from 'path';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

/**
 * Internal dependencies.
 */

declare var process: {
  env: {
    IN_DOCKER?: 'yes';
  };
};

type TSConfig = {
  compilerOptions: {
    paths: { [key: string]: string[] };
  };
};

const tsConfig: TSConfig = JSON.parse(fs.readFileSync(path.resolve('./tsconfig.json'), 'utf8'));
const alias = Object.keys(tsConfig.compilerOptions.paths).reduce((acc, aliasKey) => {
  acc[aliasKey.replace('/*', '')] = tsConfig.compilerOptions.paths[aliasKey].map(aliasPath =>
    path.resolve(aliasPath.replace('/*', ''))
  );

  return acc;
}, {});

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    port: 3000,
    host: process.env.IN_DOCKER === 'yes' ? '0.0.0.0' : '127.0.0.1',
  },
  plugins: [vue()],
  resolve: {
    alias,
  },
});
