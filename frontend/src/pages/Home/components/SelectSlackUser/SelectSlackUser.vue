<template>
  <div class="form-control flex justify-end">
    <label v-if="!isLoading && apiToken" :for="inputId" class="label">
      <span class="label-text">Select who should receive secret </span>
    </label>

    <connect-via-slack v-if="!apiToken" />

    <template v-else>
      <half-circle-spinner v-if="isLoading" :size="32" class="mx-auto" color="#000" />

      <tapro-select v-else :id="inputId" v-model="selectedUser" :options="usersForSelect" />
    </template>
  </div>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import omit from 'lodash.omit';
import { nanoid } from 'nanoid';
import { HalfCircleSpinner } from 'epic-spinners';
import { computed, defineComponent, ref, watch } from 'vue';

/**
 * Internal dependencies.
 */
import useApiToken from '@/composables/integrations/slack/useApiToken';
import useFetchUsers, { SlackUser } from '@/composables/integrations/slack/useFetchUsers';
import ConnectViaSlack from '@/components/integrations/ConnectViaSlack/ConnectViaSlack.vue';
import TaproSelect from '@/components/Select/Select.vue';

export default defineComponent({
  name: 'SelectSlackUser',

  components: {
    TaproSelect,
    ConnectViaSlack,
    HalfCircleSpinner,
  },

  emits: {
    change(_user: SlackUser) {
      return true;
    },
  },

  setup(_, { emit }) {
    const inputId = nanoid();
    const { apiToken } = useApiToken();
    const { users, isLoading } = useFetchUsers();
    const selectedUser = ref<SlackUser | null>(null);
    const nonBotUsers = computed(() =>
      users.value.filter(user => user.name !== 'Slackbot' && !user.isBot && !user.deleted)
    );
    const usersForSelect = computed(() =>
      nonBotUsers.value.map(user => ({
        text: user.name,
        value: user.id,
        ...user,
      }))
    );

    watch(selectedUser, newValue => {
      if (!newValue) {
        return;
      }

      emit('change', omit(newValue, ['text', 'value']));
    });

    watch(
      usersForSelect,
      (newValue, oldValue) => {
        // if we do not have new users, then do nothing
        // if we had users before, then do nothing
        if (!newValue.length || oldValue?.length) {
          return;
        }

        selectedUser.value = newValue[0];
      },
      { immediate: true }
    );

    return {
      inputId,
      selectedUser,
      apiToken,
      isLoading,
      nonBotUsers,
      usersForSelect,
    };
  },
});
</script>
