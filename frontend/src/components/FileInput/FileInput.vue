<template>
  <div class="relative">
    <div class="absolute top-2 right-4">
      <div class="tooltip" data-tip="Upload a file">
        <label :for="inputId" class="cursor-pointer hover:opacity-60">
          <upload-icon class="h-6 w-6 text-black" />

          <input :id="inputId" class="hidden" name="secret_file" type="file" @change="onFileUploaded" />
        </label>
      </div>
    </div>
    <slot />
  </div>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { nanoid } from 'nanoid';
import { computed, defineComponent, toRefs } from 'vue';

/**
 * Internal dependencies.
 */
import { ArrowUpTrayIcon as UploadIcon } from '@heroicons/vue/24/outline';
import FileTooLargeError from '@/exceptions/FileTooLargeError';

export default defineComponent({
  name: 'Upload',

  components: {
    UploadIcon,
  },

  props: {
    maxSize: {
      type: String,
      default: '1MB',
    },
  },

  emits: {
    file(_file: File) {
      return true;
    },

    error(_error: FileTooLargeError) {
      return true;
    },
  },

  setup(props, { emit }) {
    const { maxSize } = toRefs(props);
    const inputId = `input-upload-${nanoid()}`;
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
    const onFileUploaded = (event: Event) => {
      const file = (event.target as any).files[0] as File;

      if (file.size > fileMaxSizeInBytes.value) {
        emit('error', new FileTooLargeError(file.size));

        return;
      }

      emit('file', file);
    };

    return {
      inputId,
      onFileUploaded,
    };
  },
});
</script>
