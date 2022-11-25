/**
 * External dependencies.
 */
import axios from 'axios';
import Config from '@/utils/otsConfig';

const otsClient = axios.create({
  baseURL: Config.getInstance().backendUrl,
});

export default otsClient;
