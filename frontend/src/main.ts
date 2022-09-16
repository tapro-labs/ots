/**
 * External dependencies.
 */
import { createApp } from 'vue';
import { QueryClient, VueQueryPlugin } from 'vue-query';

/**
 * Internal dependencies.
 */
import '@scss/main.scss';

import Router from '@/Router';
import App from './App.vue';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 60 * 1000, // 1 minute
    },
  },
});

createApp(App)
  .use(VueQueryPlugin, {
    queryClient,
  })
  .use(Router)
  .mount('#app');
