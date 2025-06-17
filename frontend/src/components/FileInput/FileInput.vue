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
      let size = parseInt(maxSize.value.slice(0, -2));
      const type = maxSize.value.slice(-2);

      if (type === 'GB') {
        size *= 1024 * 1024 * 1024;
      } else if (type === 'MB') {
        size *= 1024 * 1024;
      } else if (type === 'KB') {
        size *= 1024;
      }

      return size;
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
