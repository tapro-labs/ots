/**
 * External dependencies.
 */
import { createApp } from 'vue';
import Notifications from '@kyvg/vue3-notification';
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query';

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

      // prevents from open secret to be refetched
      refetchInterval: false,
      refetchIntervalInBackground: false,
      refetchOnWindowFocus: false,
    },
  },
});

createApp(App)
  .use(VueQueryPlugin, {
    queryClient,
  })
  .use(Notifications)
  .use(Router)
  .mount('#app');
