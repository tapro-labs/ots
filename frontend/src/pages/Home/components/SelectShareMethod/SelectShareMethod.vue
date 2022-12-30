<template>
  <div class="form-control">
    <label :for="inputId" class="label">
      <span class="label-text">Select share method</span>
    </label>

    <select :id="inputId" v-model="type" class="select select-bordered w-full max-w-xs">
      <option v-for="option in options" :key="option.value" :value="option.value">{{ option.label }}</option>
    </select>
  </div>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { nanoid } from 'nanoid';
import { defineComponent, ref, watch } from 'vue';

/**
 * Internal dependencies.
 */
import { ShareMethod } from '@/enums/ShareMethod';

export default defineComponent({
  name: 'SelectShareMethod',

  emits: {
    change(_type: ShareMethod) {
      return true;
    },
  },

  setup(_, { emit }) {
    const inputId = nanoid();
    const type = ref(ShareMethod.COPY);
    const options = [
      {
        value: ShareMethod.COPY,
        label: 'Copy Link',
      },
      {
        value: ShareMethod.SLACK,
        label: 'Sending via Slack',
      },
    ];

    watch(type, newValue => emit('change', newValue));

    return {
      type,
      options,
      inputId,
    };
  },
});
</script>
