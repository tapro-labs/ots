<template>
  <div class="card w-full mx-auto bg-base-100 shadow-lg p-4">
    <h2 class="card-title mb-4 text-lg font-semibold">🎵 Review you audio!</h2>

    <audio ref="audioElement" controls class="w-full rounded-lg">
      Your browser does not support the audio element.
    </audio>
  </div>
</template>

<script lang="ts">
import { defineComponent, onMounted, useTemplateRef } from 'vue';

export default defineComponent({
  name: 'AudioPlayer',

  props: {
    blob: {
      type: Blob,
      required: true,
    },
  },

  setup(props, { emit }) {
    const audioElement = useTemplateRef<HTMLAudioElement>('audioElement');

    onMounted(() => {
      if (audioElement.value) {
        audioElement.value.src = URL.createObjectURL(props.blob);
      }
    });
    return {};
  },
});
</script>
