<template>
  <div class="h-screen">
    <the-header />

    <div class="p-6 pb-16">
      <div class="card w-full bg-base-100 shadow-xl">
        <div class="card-body">
          <template v-if="secretUrl">
            <slack-send-secret-personalisation
              v-if="isSlackCreateMethod && createMethodData"
              :data="createMethodData"
              :secret-url="secretUrl"
              @back="reset"
            />

            <copy-secret v-else :secret-url="secretUrl" />
          </template>

          <create-secret v-else @created="onCreated" />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import type { Ref } from 'vue';
import { computed, defineComponent, ref } from 'vue';

/**
 * Internal dependencies.
 */
import { ShareMethod } from '@/enums/ShareMethod';
import TheHeader from '@/components/TheHeader/TheHeader.vue';
import CopySecret from '@/pages/Home/components/CopySecret/CopySecret.vue';
import CreateSecret, { CreatedEventPayload } from '@/pages/Home/components/CreateSecret/CreateSecret.vue';
import SlackSendSecretPersonalisation from '@/pages/Home/components/SlackSendSecretPersonalisation/SlackSendSecretPersonalisation.vue';

export default defineComponent({
  components: {
    SlackSendSecretPersonalisation,
    CopySecret,
    CreateSecret,
    TheHeader,
  },

  setup() {
    const secretUrl = ref('');
    const createMethodData: Ref<CreatedEventPayload['createMethodData']> = ref(undefined);
    const createMethod = ref(ShareMethod.COPY);
    const onCreated = (payload: CreatedEventPayload) => {
      secretUrl.value = payload.secretUrl;
      createMethod.value = payload.createMethod;
      createMethodData.value = payload.createMethodData;
    };
    const isSlackCreateMethod = computed(() => createMethod.value === ShareMethod.SLACK);
    const reset = () => {
      secretUrl.value = '';
      createMethodData.value = undefined;
    };

    return {
      reset,
      secretUrl,
      onCreated,
      createMethodData,
      isSlackCreateMethod,
    };
  },
});
</script>
