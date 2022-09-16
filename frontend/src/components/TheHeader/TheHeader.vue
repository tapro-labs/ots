<template>
  <div
    class="sticky top-0 z-30 flex h-16 w-full justify-center bg-opacity-90 backdrop-blur transition-all duration-100 bg-base-100 text-base-content"
  >
    <div class="bg-accent-content navbar bg-base-100">
      <a class="btn btn-ghost normal-case text-xl" href="/">
        <img alt="Logo" src="@assets/images/logo.png" />
      </a>
    </div>
  </div>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'TheHeader',
});
</script>
