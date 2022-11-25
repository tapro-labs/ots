<template>
  <a :href="connectUrl" class="btn btn-outline border-base-content/20 gap-2" rel="noopener">
    <slack-icon />

    Connect to Slack
  </a>
</template>

<script lang="ts">
/**
 * External dependencies.
 */
import { defineComponent } from 'vue';

/**
 * Internal dependencies.
 */
import useConfig from '@/composables/useConfig';
import SlackIcon from '@/components/icons/SlackIcon/SlackIcon.vue';

export default defineComponent({
  name: 'ConnectViaSlack',

  components: {
    SlackIcon,
  },

  setup() {
    const { slackConnectionParams } = useConfig();
    const connectUrl = `https://slack.com/oauth/v2/authorize?${slackConnectionParams}`;

    return {
      connectUrl,
    };
  },
});
</script>
