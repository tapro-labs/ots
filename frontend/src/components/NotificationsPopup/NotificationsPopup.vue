<template>
  <notifications group="notification-popup" position="top right">
    <template #body="{ close, item }">
      <div :class="['notifier', item.type]" @click="close">
        <div class="notifier__icon">
          <checkmark v-if="item.type === 'success'" :height="28" :width="28" />

          <cross v-if="item.type === 'error'" :height="46" :width="46" />
        </div>

        <div class="notifier__content">
          <div class="notifier__entry">
            {{ item.text }}
          </div>
        </div>
      </div>
    </template>
  </notifications>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { defineComponent } from 'vue';
import { notify } from '@kyvg/vue3-notification';

/**
 * Internal dependencies.
 */
import useNotifications from '@/composables/useNotifications';
import Cross from '@/lottie-animated-components/Cross/Cross.vue';
import Checkmark from '@/lottie-animated-components/Checkmark/Checkmark.vue';

export default defineComponent({
  name: 'NotificationPopup',

  components: {
    Cross,
    Checkmark,
  },

  setup() {
    const { onMessageReceived } = useNotifications();

    onMessageReceived(({ message, duration, error }) => {
      notify({
        group: 'notification-popup',
        type: error ? 'error' : 'success',
        text: message,
        duration: duration,
      });
    });
  },
});
</script>
