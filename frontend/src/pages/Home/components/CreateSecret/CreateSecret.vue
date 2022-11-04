<template>
  <div class="prose max-w-full">
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

    <div class="card-actions flex justify-between">
      <div class="flex justify-center">
        <select-create-method @change="createMethod = $event" />

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
import useCreateSecret from '@/composables/useCreateSecret';
import SelectCreateMethod from '@/pages/Home/components/SelectCreateMethod/SelectCreateMethod.vue';
import SelectSlackUser from '@/pages/Home/components/SelectSlackUser/SelectSlackUser.vue';
import useApiToken from '@/composables/integrations/slack/useApiToken';
import useSendMessage from '@/composables/integrations/slack/useSendMessage';
import { SlackUser } from '@/composables/integrations/slack/useFetchUsers';
import { SecretCreateMethod } from '@/enums/SecretCreateMethod';
import { createRandomSecret, DEFAULT_SECRET_LENGTH, encrypt } from '@/utils/cryptography';

export default defineComponent({
  name: 'CreateSecret',

  components: {
    SelectSlackUser,
    SelectCreateMethod,
  },

  emits: {
    created(_payload: { secretUrl: string; createMethod: SecretCreateMethod }) {
      return true;
    },
  },

  setup(_, { emit }) {
    const { apiToken } = useApiToken();
    const createMethod = ref(SecretCreateMethod.COPY);
    const selectedSlackUser: Ref<SlackUser | null> = ref(null);
    const hasError = ref(false);
    const secret = ref('');
    const isSlackCreateMethod = computed(() => createMethod.value === SecretCreateMethod.SLACK);
    const { createSecret, isCreating } = useCreateSecret();
    const { isSending, sendMessage } = useSendMessage();
    const onCreateSecret = async () => {
      if (!secret.value) {
        hasError.value = true;

        return;
      }

      if (isCreating.value || isSending.value) {
        return;
      }

      const secretKey = await createRandomSecret(DEFAULT_SECRET_LENGTH);
      const secretId = await createSecret({ secret: encrypt(secretKey, secret.value) });
      const cryptograhyDetails = window.btoa(JSON.stringify({ secretKey, secretType: 'plain' }));
      const secretUrl = window.location.origin + '/secret/' + secretId + '#' + cryptograhyDetails;

      if (isSlackCreateMethod.value && selectedSlackUser?.value?.id) {
        await sendMessage({
          channelId: selectedSlackUser.value.id,
          message: `Secret: ${secretUrl}`,
        });
      }

      // reset secret
      secret.value = '';

      emit('created', {
        secretUrl,
        createMethod: createMethod.value,
      });
    };
    const isButtonDisabled = computed(() => {
      if (createMethod.value !== 'slack') {
        return false;
      }

      return !apiToken.value;
    });

    const onSlackUserSelected = (user: SlackUser) => {
      selectedSlackUser.value = { ...user };
    };

    watch(secret, newValue => {
      if (!newValue) {
        hasError.value = true;
      }
    });

    return {
      secret,
      createMethod,
      hasError,
      onCreateSecret,
      isButtonDisabled,
      selectedSlackUser,
      onSlackUserSelected,
      isLoading: computed(() => isCreating.value || isSending.value),
      isSlackCreateMethod: computed(() => createMethod.value === SecretCreateMethod.SLACK),
    };
  },
});
</script>

<style lang="scss" scoped>
.secret-textarea {
  min-height: 12rem;
}
</style>
