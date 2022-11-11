/**
 * External dependencies.
 */
import { onUnmounted } from 'vue-demi';
import { getCurrentInstance } from 'vue';

/**
 * Internal dependencies.
 */

export type Subscriber<T, R> = (data: T) => R;

export default function useObserver<T, R = void | Promise<void>>() {
  type InnerSubscriber = Subscriber<T, R>;

  let subscribers: InnerSubscriber[] = [];
  const currentComponent = getCurrentInstance();
  const unsubscribe = (subscriber: InnerSubscriber) => (subscribers = subscribers.filter(sub => sub !== subscriber));
  const subscribe = (subscriber: InnerSubscriber) => {
    if (subscribers.find(sub => sub === subscriber)) {
      return;
    }

    subscribers = [...subscribers, subscriber];
  };
  const trigger = async (data: T) => {
    const values = [];

    for (const subscriber of subscribers) {
      const returnValueOfSubscriber = subscriber(data);

      if (returnValueOfSubscriber instanceof Promise) {
        values.push(await returnValueOfSubscriber);
      } else {
        values.push(returnValueOfSubscriber);
      }
    }

    return values;
  };

  if (currentComponent) {
    onUnmounted(() => {
      subscribers = [];
    });
  }

  return {
    trigger,
    subscribe,
    unsubscribe,
  };
}
