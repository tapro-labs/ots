<template>
  <div class="prose max-w-full">
    <h3>Secret created! Copy the link below and send it to your buddy!</h3>

    <div class="flex">
      <input :value="secretUrl" autofocus class="input w-full rounded-r-none" disabled />
      <copy-to-clipboard :text="secretUrl" success-message="Successfully copied link to clipboard!" />
    </div>

    <p>
      Please remember not to go to this URL yourself as that would destroy the secret. Just pass it to someone else!
    </p>
  </div>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { defineComponent, PropType } from 'vue';

/**
 * Internal dependencies.
 */
import { SecretId } from '@/types/SecretTypes';
import CopyToClipboard from '@/components/CopyToClipboard/CopyToClipboard.vue';

export default defineComponent({
  name: 'CopySecret',
  components: { CopyToClipboard },
  props: {
    secretUrl: {
      type: String as PropType<SecretId>,
      required: true,
    },
  },
});
</script>
