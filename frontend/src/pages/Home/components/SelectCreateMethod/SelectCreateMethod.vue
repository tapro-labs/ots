<template>
  <div class="form-control">
    <label :for="inputId" class="label">
      <span class="label-text">Select create secret method</span>
    </label>

    <select :id="inputId" v-model="type" class="select select-bordered w-full max-w-xs">
      <option v-for="option in options" :value="option.value">{{ option.label }}</option>
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
import { SecretCreateMethod } from '@/enums/SecretCreateMethod';

export default defineComponent({
  name: 'SelectCreateMethod',

  emits: {
    change(_type: SecretCreateMethod) {
      return true;
    },
  },

  setup(_, { emit }) {
    const inputId = nanoid();
    const type = ref(SecretCreateMethod.COPY);
    const options = [
      {
        value: SecretCreateMethod.COPY,
        label: 'Copy Link',
      },
      {
        value: SecretCreateMethod.SLACK,
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
