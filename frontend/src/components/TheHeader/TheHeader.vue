<template>
  <div
    class="sticky top-0 z-30 flex h-16 w-full justify-center bg-opacity-90 backdrop-blur transition-all duration-100 bg-base-100 text-base-content"
  >
    <div class="flex justify-between bg-accent-content navbar">
      <a class="btn btn-ghost normal-case text-xl" href="/">
        <img alt="Logo" src="@assets/images/logo.png" />
      </a>

      <a
        class="mr-4 text-white transition-opacity hover:opacity-60"
        href="https://github.com/tapro-labs/ots"
        target="_blank"
      >
        <github-icon />
      </a>
    </div>
  </div>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { defineComponent } from 'vue';
import GithubIcon from '@/components/icons/GithubIcon/GithubIcon.vue';

export default defineComponent({
  name: 'TheHeader',
  components: { GithubIcon },
});
</script>
