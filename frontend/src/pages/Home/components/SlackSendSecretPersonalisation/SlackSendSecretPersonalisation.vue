<template>
  <div class="flex flex-col">
    <template v-if="!hasPersonalized">
      <div class="mb-4">
        <p><strong>(Optional) Send a personalized message to the user</strong></p>
      </div>

      <div class="form-control mb-4">
        <textarea
          v-model="slackPersonalizedMessage"
          :class="{ 'textarea-error': errorMessage }"
          autofocus
          class="big-textarea textarea w-full textarea-bordered droppable-indicator"
          placeholder="Personalized message here"
        />

        <label v-if="errorMessage" class="label">
          <span class="text-error label-text-alt">{{ errorMessage }}</span>
        </label>
      </div>

      <div class="brn-group flex self-end">
        <button :disabled="isSending" class="btn btn-secondary mr-2" @click="proceed">Proceed without a message</button>

        <button
          :class="{ loading: isSending }"
          :disabled="isSending"
          class="btn btn-primary"
          @click="sendSlackMessage(true)"
        >
          Send
        </button>
      </div>
    </template>

    <template v-else>
      <div class="mb-4">
        <p>Secret sent successfully to Slack</p>
      </div>

      <div class="flex self-end">
        <button class="btn btn-primary" @click="$emit('back')">Send another secret</button>
      </div>
    </template>
  </div>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import type { PropType } from 'vue';
import { defineComponent, ref, toRefs, watch } from 'vue';

/**
 * Internal dependencies.
 */
import useSendMessage from '@/composables/integrations/slack/useSendMessage';
import { SlackCreateMethodData } from '@/types/CreateMethodData';
import useNotifications from '@/composables/useNotifications';

export default defineComponent({
  name: 'SlackSendSecretPersonalisation',

  props: {
    secretUrl: {
      type: String,
      required: true,
    },

    data: {
      type: Object as PropType<SlackCreateMethodData>,
      required: true,
    },
  },

  emits: {
    back() {
      return true;
    },
  },

  setup(props) {
    const { data, secretUrl } = toRefs(props);
    const errorMessage = ref('');
    const hasPersonalized = ref(false);
    const slackPersonalizedMessage = ref('');
    const { isSending, sendMessage } = useSendMessage();
    const { setErrorMessage } = useNotifications();
    const initPersonalizedMessage = () => {
      slackPersonalizedMessage.value = `Secret: ${secretUrl.value}`;
    };
    const proceed = async () => {
      initPersonalizedMessage();
      await sendSlackMessage(false);
    };
    const validate = () => {
      if (!slackPersonalizedMessage.value.includes(secretUrl.value)) {
        errorMessage.value = 'Message does not contain secret url!';

        return false;
      }

      return true;
    };
    const sendSlackMessage = async (shouldValidate = true) => {
      try {
        errorMessage.value = '';

        if (shouldValidate && !validate()) {
          return;
        }

        const message = slackPersonalizedMessage.value;

        await sendMessage({
          message,
          channelId: data.value.channelId,
        });

        hasPersonalized.value = true;
      } catch (e) {
        setErrorMessage({ message: 'Failed to send message to slack!' });
        throw e;
      }
    };

    watch(secretUrl, initPersonalizedMessage, { immediate: true });

    return {
      proceed,
      isSending,
      errorMessage,
      hasPersonalized,
      sendSlackMessage,
      slackPersonalizedMessage,
    };
  },
});
</script>
