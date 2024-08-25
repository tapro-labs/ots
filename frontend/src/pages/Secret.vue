<template>
  <div class="h-screen">
    <the-header />

    <div class="p-6 pb-16">
      <div class="card w-full bg-base-100 shadow-xl">
        <div class="card-body">
          <div class="prose max-w-full">
            <h3>Your secret is below</h3>

            <template v-if="!isLoading && errorMessage">
              <p class="text-error">{{ errorMessage }}</p>

              <router-link is="button" :to="{ name: 'Home' }" class="btn btn-primary">
                Create a new secret
              </router-link>
            </template>

            <template v-else-if="showSecret && !isLoading">
              <div class="form-control mb-4">
                <textarea
                  v-if="!isFile"
                  :value="decryptedSecret"
                  class="textarea reveal-secret-textarea textarea-disabled"
                  disabled
                />

                <label class="label">
                  <span class="text-info font-bold label-text-alt">
                    Attention: You're only seeing this once. As soon as you reload the page the secret will be gone so
                    maybe copy it now
                  </span>
                </label>
              </div>
            </template>

            <template v-else>
              <p>Click the button to reveal secret</p>

              <p v-if="isFile && isLoading">
                It seems someone has sent you a file 😁. Please be patient while the file is being fetched!
              </p>

              <div class="card-actions justify-end">
                <button :class="{ loading: isLoading }" class="btn btn-primary" @click="showSecret = true">
                  Reveal Secret
                </button>
              </div>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { useRoute } from 'vue-router';
import { computed, ComputedRef, defineComponent, ref, watch } from 'vue';

/**
 * Internal dependencies.
 */
import { Base64 } from 'js-base64';
import { SecretId } from '@/types/SecretTypes';
import useFetchSecret from '@/composables/useFetchSecret';
import TheHeader from '@/components/TheHeader/TheHeader.vue';
import { SecretCryptograhyKey } from '@/utils/cryptography';
import useDecryptData from '@/composables/useDecryptData';
import EncryptStreamTransformer from '@/stream-transformers/EncryptStreamTransformer';
import GenericStreamTransformation from '@/stream-transformers/GenericStreamTransformation';
import { base64ToString, base64ToUint8Array } from '@/utils/helpers';

export default defineComponent({
  name: 'Secret',

  components: {
    TheHeader,
  },

  setup() {
    const route = useRoute();
    const { decryptStream } = useDecryptData();
    const showSecret = ref(false);
    const secretId = route.params.secretId as SecretId;
    const errorMessage = ref('');
    const { secret, isError, isLoading } = useFetchSecret({
      secretId,
      enabled: showSecret,
    });
    const decryptedSecret = ref('');
    const cryptographyDetails: ComputedRef<{
      secretKey: SecretCryptograhyKey;
      secretInfo: { type: string; info?: any };
    }> = computed(() => {
      try {
        return JSON.parse(Base64.atob(route.hash.replace('#', '')));
      } catch (a) {
        return { secretKey: '', secretInfo: { type: 'plain' } };
      }
    });
    const downloadFile = async (stream: ReadableStream, info: { type: string; name: string }) => {
      const chunks = [];
      const reader = stream.pipeThrough(new GenericStreamTransformation(base64ToUint8Array)).getReader();
      let read: any;

      do {
        read = await reader.read();

        if (read.value) {
          chunks.push(read.value);
        }
      } while (!read.done);

      const blob = new Blob(chunks, { type: info.type });
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.setAttribute('download', info.name);
      document.body.appendChild(link);
      link.click();
      link.remove();
      window.URL.revokeObjectURL(url);
    };

    watch(isError, value => {
      if (value) {
        errorMessage.value = 'The secret has already been revealed!';
      }
    });
    watch(secret, async secretValue => {
      if (!secretValue) {
        return;
      }

      const secretStream = new ReadableStream({
        pull(controller) {
          const separatorIndex = (secretValue as string).indexOf(EncryptStreamTransformer.SEPARATOR);
          const value = (secretValue as string).slice(0, separatorIndex);
          secretValue = (secretValue as string).slice(separatorIndex + EncryptStreamTransformer.SEPARATOR.length);

          controller.enqueue(value);

          if (!secretValue.length) {
            controller.close();
          }
        },
      });

      try {
        const stream = decryptStream({
          key: cryptographyDetails.value.secretKey,
          data: secretStream,
        });

        if (cryptographyDetails.value.secretInfo.type === 'plain') {
          let secretData = '';
          const reader = stream.pipeThrough(new GenericStreamTransformation(base64ToString)).getReader();
          let read: any;

          do {
            read = await reader.read();

            if (read.value) {
              secretData += read.value;
            }
          } while (!read.done);

          decryptedSecret.value = secretData;
        } else if (cryptographyDetails.value.secretInfo.type === 'file') {
          await downloadFile(stream, cryptographyDetails.value.secretInfo.info);
        }
      } catch (e) {
        errorMessage.value = 'Cannot decrypt your secret! Please create a new secret and copy the full URL.';
      }
    });
    const isFile = computed(() => cryptographyDetails.value?.secretInfo?.type === 'file');

    return {
      isFile,
      secretId,
      isLoading,
      showSecret,
      errorMessage,
      decryptedSecret,
    };
  },
});
</script>

<style lang="scss" scoped>
.reveal-secret-textarea {
  min-height: 12rem;
}
</style>
