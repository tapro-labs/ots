/**
 * External dependencies.
 */
import axios from 'axios';

const otsClient = axios.create({
  baseURL: import.meta.env.VITE_OTS_URL,
});

export default otsClient;
