<template>
  <div class="prose max-w-full">
    <h3 class="card-title">Create a secret below</h3>

    <record-audio v-if="showRecording" @on-stream="onAudioStream" />

    <file-input v-else class="mb-4" max-size="40MB" @error="onFileError" @success="onFile">
      <template #icons>
        <div class="cursor-pointer" @click="recordAudio">
          <microphone-icon class="h-6 w-6 text-black" />
        </div>
      </template>

      <div v-if="!fileInfo" class="form-control">
        <textarea
          v-model="secret"
          :class="{ 'textarea-error': hasError }"
          autofocus
          class="big-textarea textarea w-full textarea-bordered droppable-indicator"
          placeholder="Secret here"
        />

        <label v-if="hasError" class="label">
          <span class="text-error label-text-alt">Secret must not be empty!</span>
        </label>
      </div>

      <div v-else class="form-control droppable-indicator">
        You have uploaded a file: <strong>{{ fileInfo.name }}</strong>

        <audio-player v-if="isAudioRecording && audioStream" :blob="audioStream" />
      </div>
    </file-input>

    <div class="card-actions flex justify-between">
      <div v-if="isSlackFeatureEnabled" class="flex justify-center">
        <select-share-method @change="createMethod = $event" />

        <select-slack-user v-if="isSlackCreateMethod" class="ml-4" @change="onSlackUserSelected" />
      </div>

      <button
        :class="{ loading: isLoading }"
        :disabled="isButtonDisabled"
        class="btn btn-primary"
        @click="onCreateSecret"
      >
        Create Secret
      </button>
    </div>
  </div>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { computed, defineComponent, Ref, ref, watch } from 'vue';

/**
 * Internal dependencies.
 */
import { Base64 } from 'js-base64';
import { MicrophoneIcon } from '@heroicons/vue/24/outline';
import useCreateSecret from '@/composables/useCreateSecret';
import SelectSlackUser from '@/pages/Home/components/SelectSlackUser/SelectSlackUser.vue';
import useApiToken from '@/composables/integrations/slack/useApiToken';
import useSendMessage from '@/composables/integrations/slack/useSendMessage';
import { SlackUser } from '@/composables/integrations/slack/useFetchUsers';
import { ShareMethod } from '@/enums/ShareMethod';
import { createEncryptionKey, DEFAULT_SECRET_LENGTH } from '@/utils/cryptography';
import useConfig from '@/composables/useConfig';
import SelectShareMethod from '@/pages/Home/components/SelectShareMethod/SelectShareMethod.vue';
import FileInput from '@/components/FileInput/FileInput.vue';
import useNotifications from '@/composables/useNotifications';
import { SecretInfo } from '@/types/SecretInfo';
import { FileInfo } from '@/types/FileInfo';
import { CreateMethodData } from '@/types/CreateMethodData';
import RecordAudio from '@/pages/Home/components/RecordAudio/RecordAudio.vue';
import AudioPlayer from '@/pages/Home/components/AudioPlayer/AudioPlayer.vue';

export type CreatedEventPayload = { secretUrl: string; createMethod: ShareMethod; createMethodData?: CreateMethodData };

export default defineComponent({
  name: 'CreateSecret',

  components: {
    AudioPlayer,
    RecordAudio,
    FileInput,
    MicrophoneIcon,
    SelectSlackUser,
    SelectShareMethod,
  },

  emits: {
    created(_payload: CreatedEventPayload) {
      return true;
    },
  },

  setup(_, { emit }) {
    let stream: ReadableStream<Uint8Array> | null = null;
    // temporary variable to at least show a spinner while encrypting large files
    const isLoading = ref(false);
    const { isSlackFeatureEnabled } = useConfig();
    const { apiToken } = useApiToken();
    const createMethod = ref(ShareMethod.COPY);
    const selectedSlackUser: Ref<SlackUser | null> = ref(null);
    const hasError = ref(false);
    const secret = ref('');
    const createSecretState = ref('INITIAL');
    const fileInfo: Ref<FileInfo | null> = ref(null);
    const isSlackCreateMethod = computed(() => createMethod.value === ShareMethod.SLACK);
    const { createSecret, isCreating } = useCreateSecret();
    const { isSending, sendMessage } = useSendMessage();
    const { setErrorMessage } = useNotifications();
    const showRecording = computed(() => createSecretState.value === 'RECORDING');
    const isAudioRecording = computed(() => fileInfo.value?.type === 'audio/webm');
    const audioStream = ref<Blob | null>(null);
    const onCreateSecret = async () => {
      if (!stream) {
        hasError.value = true;

        return;
      }

      if (isCreating.value || isSending.value || isLoading.value) {
        return;
      }

      try {
        isLoading.value = true;

        let secretInfo: SecretInfo = {
          type: 'plain',
        };

        if (fileInfo.value) {
          // TODO: fix type check
          secretInfo = {
            type: fileInfo.value.type === 'audio/webm' ? 'audio' : 'file',
            info: fileInfo.value,
          } as any;
        }

        const secretKey = await createEncryptionKey(DEFAULT_SECRET_LENGTH);
        const secretId = await createSecret({ data: stream, key: secretKey });
        const cryptograhyDetails = Base64.btoa(JSON.stringify({ secretKey, secretInfo }));
        const secretUrl = window.location.origin + '/secret/' + secretId + '#' + cryptograhyDetails;

        // reset secret
        secret.value = '';

        if (isSlackCreateMethod.value && selectedSlackUser?.value?.id) {
          emit('created', {
            secretUrl,
            createMethod: createMethod.value,
            createMethodData: {
              channelId: selectedSlackUser.value.id,
            },
          });

          return;
        }

        emit('created', {
          secretUrl,
          createMethod: createMethod.value,
        });
      } catch {
        isLoading.value = false;
      }
    };
    const isButtonDisabled = computed(() => {
      if (showRecording.value) {
        return true;
      }

      if (createMethod.value !== 'slack') {
        return false;
      }

      return !apiToken.value;
    });

    const onSlackUserSelected = (user: SlackUser) => {
      selectedSlackUser.value = { ...user };
    };

    const recordAudio = () => {
      createSecretState.value = 'RECORDING';
    };

    const onFile = async (file: File) => {
      stream = file.stream();
      fileInfo.value = {
        name: file.name,
        type: file.type,
      };
    };
    const onAudioStream = (s: Blob) => {
      audioStream.value = s;
      stream = s.stream();
      fileInfo.value = {
        name: 'audio.webm',
        type: 'audio/webm',
      };
      createSecretState.value = 'INITIAL';
    };
    const onFileError = () => {
      setErrorMessage({ message: 'File is larger than 40MB' });
    };

    watch(secret, newValue => {
      if (!newValue) {
        hasError.value = true;

        return;
      }

      fileInfo.value = null;
      stream = new ReadableStream({
        pull(controller) {
          const textEncoder = new TextEncoder();

          controller.enqueue(textEncoder.encode(newValue));
          controller.close();
        },
      });
    });

    return {
      secret,
      createMethod,
      hasError,
      onFile,
      fileInfo,
      showRecording,
      recordAudio,
      onFileError,
      onCreateSecret,
      isButtonDisabled,
      selectedSlackUser,
      onSlackUserSelected,
      isSlackFeatureEnabled,
      onAudioStream,
      isAudioRecording,
      audioStream,
      isLoading: computed(() => isCreating.value || isSending.value || isLoading.value),
      isSlackCreateMethod: computed(() => createMethod.value === ShareMethod.SLACK),
    };
  },
});
</script>
