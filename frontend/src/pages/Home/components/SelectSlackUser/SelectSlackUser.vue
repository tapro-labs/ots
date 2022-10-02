<template>
  <div class="form-control">
    <label v-if="apiToken" :for="inputId" class="label">
      <!-- Dirty hack to fix layout -->
      <span v-if="isLoading" class="label-text">&nbsp;</span>
      <span v-else class="label-text">Select who should receive secret </span>
    </label>

    <connect-via-slack v-if="!apiToken" />

    <template v-else>
      <half-circle-spinner v-if="isLoading" :size="32" class="mx-auto" color="#000" />

      <select v-else :id="inputId" class="select select-bordered w-full max-w-xs" @change="onSelect">
        <option v-for="user in nonBotUsers" :key="user.id" :value="user.id">{{ user.name }}</option>
      </select>
    </template>
  </div>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { nanoid } from 'nanoid';
import { HalfCircleSpinner } from 'epic-spinners';
import { computed, defineComponent, watch } from 'vue';

/**
 * Internal dependencies.
 */
import useApiToken from '@/composables/integrations/slack/useApiToken';
import useFetchUsers, { SlackUser } from '@/composables/integrations/slack/useFetchUsers';
import ConnectViaSlack from '@/components/integrations/ConnectViaSlack/ConnectViaSlack.vue';

export default defineComponent({
  name: 'SelectSlackUser',

  components: {
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
    const nonBotUsers = computed(() => users.value.filter(user => !user.isBot));

    const onSelect = (event: Event) => {
      const id: string = (event?.target as HTMLSelectElement).value;

      emitChangeEvent(id);
    };

    const emitChangeEvent = (id: string) => {
      emit('change', users.value.find(user => user.id === id) as SlackUser);
    };

    watch(
      users,
      (newValue, oldValue) => {
        // if we do not have new users, then do nothing
        // if we had users before, then do nothing
        if (!newValue.length || oldValue?.length) {
          return;
        }

        // else if we had nothing emit change event to the first user
        emitChangeEvent(newValue[0].id);
      },
      { immediate: true }
    );

    return {
      inputId,
      onSelect,
      apiToken,
      isLoading,
      nonBotUsers,
    };
  },
});
</script>
