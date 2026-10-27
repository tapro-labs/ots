import { afterEach, describe, expect, it, vi } from 'vitest';
import { defineComponent, h } from 'vue';
import { mount, VueWrapper } from '@vue/test-utils';
import useNotifications from '@/composables/useNotifications';

const wrappers: VueWrapper[] = [];

const mountListener = (callback: Parameters<ReturnType<typeof useNotifications>['onMessageReceived']>[0]) => {
  const wrapper = mount(
    defineComponent({
      setup() {
        useNotifications().onMessageReceived(callback);

        return () => h('div');
      },
    })
  );

  wrappers.push(wrapper);

  return wrapper;
};

describe('useNotifications', () => {
  afterEach(() => {
    // notifications are app-wide, make sure listeners do not leak between tests
    wrappers.splice(0).forEach(wrapper => wrapper.unmount());
  });

  it('delivers success messages with a default duration', async () => {
    const listener = vi.fn();

    mountListener(listener);
    await useNotifications().setSuccessMessage({ message: 'Copied!' });

    expect(listener).toHaveBeenCalledWith({ message: 'Copied!', duration: 3000, error: false });
  });

  it('delivers error messages with a custom duration', async () => {
    const listener = vi.fn();

    mountListener(listener);
    await useNotifications().setErrorMessage({ message: 'Too large', duration: 10 });

    expect(listener).toHaveBeenCalledWith({ message: 'Too large', duration: 10, error: true });
  });

  it('shares messages between independent useNotifications() calls', async () => {
    const first = vi.fn();
    const second = vi.fn();

    mountListener(first);
    mountListener(second);
    await useNotifications().setSuccessMessage({ message: 'hello' });

    expect(first).toHaveBeenCalledOnce();
    expect(second).toHaveBeenCalledOnce();
  });

  it('stops delivering messages to a component after it unmounts', async () => {
    const listener = vi.fn();
    const wrapper = mountListener(listener);

    wrapper.unmount();
    await useNotifications().setSuccessMessage({ message: 'after unmount' });

    expect(listener).not.toHaveBeenCalled();
  });
});
