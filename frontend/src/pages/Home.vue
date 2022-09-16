<template>
  <div class="h-screen">
    <the-header />

    <div class="p-6 pb-16">
      <div class="card w-full bg-base-100 shadow-xl">
        <div class="card-body">
          <div class="prose max-w-full">
            <template v-if="secretId">
              <h3>Secret created! Copy the link below and send it to your buddy!</h3>

              <div>
                <input :value="secretLink" autofocus class="input w-full" disabled />
              </div>

              <p>
                Please remember not to go to this URL yourself as that would destroy the secret. Just pass it to someone
                else!
              </p>
            </template>

            <template v-else>
              <h3 class="card-title">Create a secret below</h3>

              <div class="form-control mt-2 mb-4">
                <textarea
                  v-model="secret"
                  :class="{ 'textarea-error': hasError }"
                  autofocus
                  class="secret-textarea textarea w-full textarea-bordered"
                  placeholder="Secret here"
                />

                <label v-if="hasError" class="label">
                  <span class="text-error label-text-alt">Secret must not be empty!</span>
                </label>
              </div>

              <div class="card-actions justify-end">
                <button :class="{ loading: isCreating }" class="btn btn-primary" @click="onCreateSecret">
                  Create Secret
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
import { computed, defineComponent, ref, watch } from 'vue';

/**
 * Internal dependencies.
 */
import TheHeader from '@/components/TheHeader/TheHeader.vue';
import useCreateSecret from '@/composables/useCreateSecret';

export default defineComponent({
  components: {
    TheHeader,
  },

  setup() {
    const hasError = ref(false);
    const secret = ref('');
    const secretId = ref('');
    const secretLink = computed(() => window.location.origin + '/secret/' + secretId.value);
    const { createSecret, isCreating } = useCreateSecret();
    const onCreateSecret = async () => {
      if (!secret.value) {
        hasError.value = true;

        return;
      }

      if (isCreating.value) {
        return;
      }

      secretId.value = await createSecret({ secret: secret.value });

      // reset secret
      secret.value = '';
    };

    watch(secret, newValue => {
      if (!newValue) {
        hasError.value = true;
      }
    });

    return {
      secret,
      hasError,
      secretId,
      secretLink,
      isCreating,
      onCreateSecret,
    };
  },
});
</script>

<style lang="scss" scoped>
.secret-textarea {
  min-height: 12rem;
}
</style>
