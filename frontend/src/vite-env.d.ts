/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue';
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

// epic-spinners ships types but does not list them in the "." entry of its
// package.json "exports", so "Bundler" module resolution cannot find them.
declare module 'epic-spinners' {
  export * from 'epic-spinners/types';
}
