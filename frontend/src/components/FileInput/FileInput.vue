<template>
  <dropzone :validation="fileValidation" @error="$emit('error', $event)" @success="$emit('success', $event)">
    <template #default="{ inputId }">
      <div class="relative" v-bind="$attrs">
        <div class="absolute flex gap-4 top-2 right-4">
          <slot name="icons" />

          <div class="tooltip" data-tip="Drag or Upload a file">
            <label :for="inputId" class="cursor-pointer hover:opacity-60">
              <upload-icon class="h-6 w-6 text-black" />
            </label>
          </div>
        </div>

        <slot />
      </div>
    </template>
  </dropzone>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { computed, defineComponent, toRefs } from 'vue';
import { ArrowUpTrayIcon as UploadIcon } from '@heroicons/vue/24/outline';

/**
 * Internal dependencies.
 */
import Dropzone from '@/components/Dropzone/Dropzone.vue';
import FileTooLargeError from '@/exceptions/FileTooLargeError';

// e.g. "40MB", "512kb", "1.5GB" or a plain number of bytes
const SIZE_PATTERN = /^(\d+(?:\.\d+)?)\s*(B|KB|MB|GB)?$/i;
const UNIT_MULTIPLIERS: Record<string, number> = {
  B: 1,
  KB: 1024,
  MB: 1024 * 1024,
  GB: 1024 * 1024 * 1024,
};

export default defineComponent({
  name: 'FileInput',

  components: {
    Dropzone,
    UploadIcon,
  },

  inheritAttrs: false,

  props: {
    maxSize: {
      type: String,
      default: '1MB',
      validator: (value: string) => SIZE_PATTERN.test(value.trim()),
    },
  },

  emits: {
    success(_file: File) {
      return true;
    },

    error(_error: Error) {
      return true;
    },
  },

  setup(props, { emit }) {
    const { maxSize } = toRefs(props);
    const fileMaxSizeInBytes = computed(() => {
      const [, size, unit = 'B'] = maxSize.value.trim().match(SIZE_PATTERN) || [];

      return parseFloat(size) * UNIT_MULTIPLIERS[unit.toUpperCase()];
    });
    const fileValidation = (file: File) => {
      if (file.size > fileMaxSizeInBytes.value) {
        return new FileTooLargeError(file.size);
      }
    };

    return {
      fileValidation,
    };
  },
});
</script>
