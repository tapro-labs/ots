<template>
  <div class="p-6 flex flex-col gap-5 justify-center items-center">
    <h2 class="text-xl font-bold my-0">Audio Recorder</h2>

    <div class="flex items-center gap-4">
      <button class="btn rounded-full w-16 h-16" :class="{ 'btn-error': isRecording }" @click="toggleRecording">
        <component
          :is="isRecording ? 'stop-circle-icon' : 'play-icon'"
          class="h-12 w-12 text-black"
          :class="{ 'text-white': isRecording }"
        />
      </button>

      <div>
        <p class="text-lg font-medium">
          {{ isRecording ? 'Recording...' : 'Idle' }}
        </p>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import { PlayIcon, StopCircleIcon } from '@heroicons/vue/24/outline';
import useNotifications from '@/composables/useNotifications';

export default defineComponent({
  name: 'RecordAudio',

  components: {
    PlayIcon,
    StopCircleIcon,
  },

  emits: {
    onStream(_: Blob) {
      return true;
    },
  },

  setup(_, { emit }) {
    let mediaRecorder: MediaRecorder | null = null;
    const isRecording = ref(false);
    const { setErrorMessage } = useNotifications();
    const toggleRecording = async () => {
      if (isRecording.value) {
        mediaRecorder?.stop();
        isRecording.value = false;

        return;
      }

      try {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        mediaRecorder = new MediaRecorder(stream);
        const audioChunks: any[] = [];

        mediaRecorder.ondataavailable = event => {
          if (event.data.size > 0) {
            audioChunks.push(event.data);
          }
        };

        mediaRecorder.onstop = () => {
          stream?.getTracks()?.forEach(track => track.stop());

          const blob = new Blob(audioChunks, { type: 'audio/webm' });
          emit('onStream', blob);
        };

        mediaRecorder.start();

        isRecording.value = true;
      } catch (e: any) {
        await setErrorMessage({
          message: e.message || 'Failed to record!',
        });
      }
    };

    return {
      isRecording,
      toggleRecording,
    };
  },
});
</script>
