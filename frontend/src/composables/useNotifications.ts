/**
 * External dependencies.
 */
import { onUnmounted } from 'vue-demi';
import { getCurrentInstance } from 'vue';

/**
 * Internal dependencies.
 */
import useObserver from './useObserver';

export type Notification = {
  message: string;
  duration: number;
  error: boolean;
};

const { trigger, subscribe, unsubscribe } = useObserver<Notification>();

type NotificationCallback = Parameters<typeof subscribe>[0];

export default function useNotifications() {
  const localNotificationCallbacks: NotificationCallback[] = [];
  const currentComponent = getCurrentInstance();
  const setSuccessMessage = ({ message, duration = 3000 }: Omit<Notification, 'error'>) => {
    return trigger({
      message,
      duration,
      error: false,
    });
  };
  const setErrorMessage = ({ message, duration = 3000 }: Omit<Notification, 'error'>) => {
    return trigger({
      message,
      duration,
      error: true,
    });
  };

  const onMessageReceived = (cb: NotificationCallback) => {
    subscribe(cb);
    localNotificationCallbacks.push(cb);
  };

  if (!currentComponent) {
    onUnmounted(() => {
      localNotificationCallbacks.forEach(unsubscribe);
      localNotificationCallbacks.length = 0;
    });
  }

  return {
    onMessageReceived,
    setErrorMessage,
    setSuccessMessage,
  };
}
