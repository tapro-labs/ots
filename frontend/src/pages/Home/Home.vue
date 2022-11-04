<template>
  <div class="h-screen">
    <the-header />

    <div class="p-6 pb-16">
      <div class="card w-full bg-base-100 shadow-xl">
        <div class="card-body">
          <template v-if="secretUrl">
            <div v-if="isSlackCreateMethod" class="flex flex-col">
              <div class="mb-4">
                <p>Secret sent successfully to Slack</p>
              </div>

              <div class="flex self-end">
                <button class="btn btn-primary" @click="secretUrl = ''">Send another secret</button>
              </div>
            </div>

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
import { computed, defineComponent, ref } from 'vue';

/**
 * Internal dependencies.
 */
import TheHeader from '@/components/TheHeader/TheHeader.vue';
import CreateSecret from '@/pages/Home/components/CreateSecret/CreateSecret.vue';
import CopySecret from '@/pages/Home/components/CopySecret/CopySecret.vue';
import { SecretCreateMethod } from '@/enums/SecretCreateMethod';

export default defineComponent({
  components: {
    CopySecret,
    CreateSecret,
    TheHeader,
  },

  setup() {
    const secretUrl = ref('');
    const createMethod = ref(SecretCreateMethod.COPY);
    const onCreated = (payload: { secretUrl: string; createMethod: SecretCreateMethod }) => {
      secretUrl.value = payload.secretUrl;
      createMethod.value = payload.createMethod;
    };
    const isSlackCreateMethod = computed(() => createMethod.value === SecretCreateMethod.SLACK);

    return {
      secretUrl,
      onCreated,
      isSlackCreateMethod,
    };
  },
});
</script>
