/**
 * External dependencies.
 */
import { computed } from 'vue';
import { useLocalStorage } from '@vueuse/core';

/**
 * Internal dependencies.
 */

export type ApiToken = string;

const apiTokenKey = 'slack_api_token';
const apiToken = useLocalStorage<string | null>(apiTokenKey, null, { listenToStorageChanges: true });

export default function useApiToken() {
  const resetApiToken = () => (apiToken.value = null);
  const setApiToken = (token: ApiToken) => {
    apiToken.value = token;
  };
  const setTokenFromUrlParam = () => {
    const url = new URL(window.location.href);
    const token = url.searchParams.get(apiTokenKey);

    if (token) {
      apiToken.value = token;
    }
  };

  setTokenFromUrlParam();

  return {
    setApiToken,
    resetApiToken,
    apiToken: computed(() => apiToken.value),
  };
}
