import { defineComponent, h } from 'vue';
import { mount } from '@vue/test-utils';
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query';

export const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

export const queryPlugin = () =>
  [VueQueryPlugin, { queryClient: createTestQueryClient() }] as [typeof VueQueryPlugin, object];

/**
 * Runs a composable inside a component that has vue-query installed.
 */
export const withQuery = <T>(composable: () => T) => {
  let result!: T;
  const wrapper = mount(
    defineComponent({
      setup() {
        result = composable();

        return () => h('div');
      },
    }),
    { global: { plugins: [queryPlugin()] } }
  );

  return { result, wrapper };
};
