<template>
  <lottie :options="options" class="lottie-animation-icon" v-bind="{ width, height }" />
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import Lottie from 'vue-lottie/src/lottie.vue';
import { computed, defineComponent } from 'vue';

/**
 * Internal dependencies.
 */
import animationData from './checkmark.json';

export default defineComponent({
  name: 'Checkmark',

  components: {
    Lottie,
  },

  props: {
    width: {
      type: Number,
      default: 32,
    },
    height: {
      type: Number,
      default: 32,
    },

    loop: {
      type: Boolean,
      default: false,
    },
  },

  setup(props) {
    const options = computed(() => ({
      loop: props.loop,
      animationData,
    }));

    return {
      options,
    };
  },
});
</script>
