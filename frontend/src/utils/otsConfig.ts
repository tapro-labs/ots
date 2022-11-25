export interface OtsConfig {
  backendUrl: string;
  isSlackFeatureEnabled: boolean;
  slackConnectionParams: string;
}

class Config implements OtsConfig {
  private static instance: Config;

  private constructor() {}

  get isSlackFeatureEnabled() {
    return /slack/i.test(this.enabledFeatures);
  }

  get enabledFeatures() {
    return import.meta.env.VITE_ENABLED_FEATURES || '';
  }

  get slackConnectionParams() {
    return import.meta.env.VITE_SLACK_CONNECT_QUERY_PARAMS || '';
  }

  get backendUrl() {
    return import.meta.env.VITE_OTS_URL;
  }

  static getInstance() {
    if (!Config.instance) {
      Config.instance = new Config();
    }

    return Config.instance;
  }
}

export default Config;
