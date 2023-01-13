<template>
  <slot
    :activate-upload-file-prompt="activateUploadFilePrompt"
    :drag-active="dragActive"
    :drag-over="dragOver"
    :error-message="errorMessage"
    :input-id="inputId"
  />

  <input
    :id="inputId"
    ref="input"
    :accept="inputAccepted"
    :name="inputId"
    class="hidden"
    type="file"
    @change="handleFile"
  />
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { nanoid } from 'nanoid';
import { templateRef } from '@vueuse/core';
import { computed, defineComponent, getCurrentInstance, onBeforeUnmount, onMounted, Ref, ref } from 'vue';

/**
 * Internal dependencies.
 */
import Droppable from '@/utils/droppable';

const getFileFromInputAndDropZone = (event: Event, dropZoneFiles: File[]): File | null => {
  const target = event.target as HTMLInputElement;

  if (!target?.files?.length && !dropZoneFiles?.length) {
    return null;
  }

  return dropZoneFiles?.[0] || target.files?.[0];
};

export default defineComponent({
  name: 'Dropzone',

  props: {
    inputAccepted: {
      type: String,
      default: '*',
    },

    validation: {
      type: Function,
      default: () => {},
    },

    useCapture: {
      type: [String, Boolean],
      default: false,
      validator: (data: string) => ['user', false].includes(data),
    },
  },

  emits: {
    error(err: Error) {
      return !!err;
    },
    success(file: File) {
      return true;
    },
  },

  setup(props, { emit, slots }) {
    const inputId = 'input-' + nanoid();
    const input = templateRef<HTMLInputElement>('input');
    const dragActive = ref(false);
    const dragOver = ref(false);
    const error: Ref<Error | null> = ref(null);
    const instance = getCurrentInstance();
    let droppable: Droppable | null = null;

    if (!slots?.default) {
      throw new Error('This is a renderless component please wrap this component inside elements or other components.');
    }

    const handleFile = (event: Event, files: File[] = []) => {
      error.value = null;
      const actualFile = getFileFromInputAndDropZone(event, files);

      if (!actualFile) {
        error.value = new Error('No file was provided!');
        emit('error', error.value);

        return;
      }

      error.value = props.validation(actualFile);

      if (error.value) {
        emit('error', error.value);

        return;
      }

      emit('success', actualFile);
    };
    const errorMessage = computed(() => error?.value?.message || '');

    const activateUploadFilePrompt = () => input.value?.click();
    const reset = () => {
      if (!input.value) {
        return;
      }

      input.value.value = '';
    };

    onMounted(() => {
      const elementWeAreWrapping = instance?.vnode.el?.nextElementSibling;

      droppable = new Droppable({
        container: elementWeAreWrapping,
        onDrop: handleFile,
      });

      droppable.addEventListener('dragEnter', () => (dragOver.value = true));
      droppable.addEventListener('dragLeave', () => (dragOver.value = false));
      droppable.addEventListener('dragActive', () => (dragActive.value = true));
      droppable.addEventListener('dragActiveLeave', () => (dragActive.value = false));
    });

    onBeforeUnmount(() => {
      droppable?.destroy();
      droppable = null;
    });

    return {
      reset,
      inputId,
      dragOver,
      dragActive,
      handleFile,
      errorMessage,
      activateUploadFilePrompt,
    };
  },
});
</script>
