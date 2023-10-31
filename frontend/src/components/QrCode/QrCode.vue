<template>
  <button v-bind="$attrs" class="btn btn-primary btn-square rounded-l-none" @click="openModal">
    <qr-code-icon class="h-6 w-6 text-white" />
  </button>

  <dialog ref="modal" class="modal" @click="closeModal">
    <div class="modal-box" @click.prevent.stop>
      <h3 class="text-center font-bold text-lg">Scan QR Code to get url</h3>

      <div class="flex justify-center">
        <qr-code-box :size="300" :value="text" />
      </div>

      <div class="modal-action flex justify-center">
        <button class="btn" @click="closeModal">Close</button>
      </div>
    </div>
  </dialog>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import QrCodeBox from 'qrcode.vue';
import { templateRef } from '@vueuse/core';
import { defineComponent } from 'vue';
import { QrCodeIcon } from '@heroicons/vue/24/outline';

/**
 * Internal dependencies.
 */

export default defineComponent({
  name: 'QrCode',

  components: {
    QrCodeBox,
    QrCodeIcon,
  },

  inheritAttrs: false,

  props: {
    text: {
      type: String,
      required: true,
    },
  },

  setup() {
    const qrModal = templateRef<HTMLDialogElement | null>('modal');
    const openModal = () => qrModal.value?.showModal();
    const closeModal = () => qrModal.value?.close();

    return {
      openModal,
      closeModal,
    };
  },
});
</script>
