import { describe, expect, it, vi } from 'vitest';
import { defineComponent, h } from 'vue';
import { mount } from '@vue/test-utils';
import useObserver from '@/composables/useObserver';

describe('useObserver', () => {
  it('calls subscribers in order and collects their (awaited) return values', async () => {
    const { subscribe, trigger } = useObserver<number, number | Promise<number>>();

    subscribe(value => value + 1);
    subscribe(async value => value * 10);

    expect(await trigger(2)).toEqual([3, 20]);
  });

  it('ignores duplicate subscriptions of the same callback', async () => {
    const { subscribe, trigger } = useObserver<string>();
    const subscriber = vi.fn();

    subscribe(subscriber);
    subscribe(subscriber);
    await trigger('hi');

    expect(subscriber).toHaveBeenCalledTimes(1);
    expect(subscriber).toHaveBeenCalledWith('hi');
  });

  it('stops calling unsubscribed callbacks', async () => {
    const { subscribe, unsubscribe, trigger } = useObserver<string>();
    const kept = vi.fn();
    const removed = vi.fn();

    subscribe(kept);
    subscribe(removed);
    unsubscribe(removed);
    await trigger('x');

    expect(kept).toHaveBeenCalledOnce();
    expect(removed).not.toHaveBeenCalled();
  });

  it('keeps observers independent from each other', async () => {
    const first = useObserver<string>();
    const second = useObserver<string>();
    const subscriber = vi.fn();

    first.subscribe(subscriber);
    await second.trigger('x');

    expect(subscriber).not.toHaveBeenCalled();
  });

  it('drops all subscribers when the owning component unmounts', async () => {
    let observer!: ReturnType<typeof useObserver<string>>;
    const subscriber = vi.fn();
    const wrapper = mount(
      defineComponent({
        setup() {
          observer = useObserver<string>();
          observer.subscribe(subscriber);

          return () => h('div');
        },
      })
    );

    wrapper.unmount();
    await observer.trigger('after unmount');

    expect(subscriber).not.toHaveBeenCalled();
  });
});
