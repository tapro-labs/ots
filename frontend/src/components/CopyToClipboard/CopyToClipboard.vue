<template>
  <button class="btn btn-square rounded-l-none" @click="copy">
    <clipboard-icon class="h-6 w-6 text-white" />
  </button>
</template>

<script>
/**
 * External dependencies.
 */
import { toRefs } from 'vue';
import copy from 'copy-to-clipboard';
import { ClipboardIcon } from '@heroicons/vue/24/outline';
import useNotifications from '@/composables/useNotifications';

/**
 * Internal dependencies.
 */

export default {
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
    const { setSuccessMessage } = useNotifications();

    return {
      copy: () => {
        copy(text.value);

        if (successMessage.value) {
          setSuccessMessage({
            message: successMessage.value,
          });
        }
      },
    };
  },
};
</script>
