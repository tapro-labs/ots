<template>
  <model-select v-model="value" class="tapro-select" :options="options" />
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { ModelSelect } from 'vue-search-select';
import { defineComponent } from 'vue';
import { useVModel } from '@vueuse/core';

/**
 * Internal dependencies.
 */

export default defineComponent({
  name: 'TaproSelect',

  components: {
    ModelSelect,
  },

  props: {
    modelValue: {
      type: [String, Number, Object, Boolean],
      default: null,
    },

    options: {
      type: Array,
      required: true,
    },
  },

  setup(props, { emit }) {
    const value = useVModel(props, 'modelValue', emit);

    return {
      value,
    };
  },
});
</script>
