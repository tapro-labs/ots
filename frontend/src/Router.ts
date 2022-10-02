/**
 * External dependencies.
 */
import { createRouter, createWebHistory } from 'vue-router';

/**
 * Internal dependencies.
 */
const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('@/pages/Home/Home.vue'),
  },

  {
    path: '/secret/:secretId',
    component: () => import('@/pages/Secret.vue'),
  },

  {
    path: '/:matchedPath(.*)*',
    component: () => import('@/pages/Home/Home.vue'),
  },
];

const router = createRouter({
  routes,
  history: createWebHistory(),
});

export default router;
