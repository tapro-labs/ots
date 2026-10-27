<template>
  <button class="btn btn-primary btn-square rounded-l-none" @click="copy">
    <clipboard-icon class="h-6 w-6 text-white" />
  </button>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import copy from 'copy-to-clipboard';
import { defineComponent, toRefs } from 'vue';
import { ClipboardIcon } from '@heroicons/vue/24/outline';
import useNotifications from '@/composables/useNotifications';

/**
 * Internal dependencies.
 */

export default defineComponent({
  name: 'CopyToClipboard',

  components: {
    ClipboardIcon,
  },

  props: {
    text: {
      type: String,
      required: true,
    },

    successMessage: {
      type: String,
      default: null,
    },
  },

  setup(props) {
    const { text, successMessage } = toRefs(props);
    const { setSuccessMessage, setErrorMessage } = useNotifications();

    return {
      copy: async () => {
        const copied = await copy(text.value);

        if (!copied) {
          setErrorMessage({
            message: 'Could not copy to clipboard.',
          });

          return;
        }

        if (successMessage.value) {
          setSuccessMessage({
            message: successMessage.value,
          });
        }
      },
    };
  },
});
</script>
