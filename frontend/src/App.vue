<template>
  <notification-popup />

  <router-view />
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { defineComponent } from 'vue';

/**
 * Internal dependencies.
 */
import NotificationPopup from '@/components/NotificationsPopup/NotificationsPopup.vue';

export default defineComponent({
  components: { NotificationPopup },
});
</script>
