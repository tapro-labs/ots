<template>
  <div class="h-screen">
    <the-header />

    <div class="p-6 pb-16">
      <div class="card w-full bg-base-100 shadow-xl">
        <div class="card-body">
          <div class="prose max-w-full">
            <h3>Your secret is below</h3>

            <template v-if="!isLoading && isError">
              <p class="text-error">The secret has already been revealed!</p>

              <router-link is="button" :to="{ name: 'Home' }" class="btn btn-primary">
                Create a new secret
              </router-link>
            </template>

            <template v-else-if="showSecret && !isLoading">
              <div class="form-control mb-4">
                <textarea :value="decryptedSecret" class="textarea reveal-secret-textarea textarea-disabled" disabled />

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
import { computed, ComputedRef, defineComponent, ref } from 'vue';

/**
 * Internal dependencies.
 */
import { SecretId } from '@/types/SecretTypes';
import useFetchSecret from '@/composables/useFetchSecret';
import TheHeader from '@/components/TheHeader/TheHeader.vue';
import { decrypt, SecretCryptograhyKey } from '@/utils/cryptography';

export default defineComponent({
  name: 'Secret',

  components: {
    TheHeader,
  },

  setup() {
    const route = useRoute();
    const showSecret = ref(false);
    const secretId = route.params.secretId as SecretId;
    const { secret, isError, isLoading } = useFetchSecret({
      secretId,
      enabled: showSecret,
    });
    const cryptographyDetails: ComputedRef<{ secretKey: SecretCryptograhyKey; secretType: string }> = computed(() => {
      try {
        return JSON.parse(window.atob(route.hash.replace('#', '')));
      } catch (a) {
        return { secretKey: '', secretType: '' };
      }
    });
    const decryptedSecret = computed(() => {
      if (!secret.value) {
        return undefined;
      }

      return decrypt(cryptographyDetails.value.secretKey, secret.value);
    });

    return {
      isError,
      secretId,
      isLoading,
      showSecret,
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
