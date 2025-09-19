import axios from 'axios';
import { API_BASE_URL } from '../config';

export const api = axios.create({ 
  baseURL: API_BASE_URL, 
  timeout: 15000 
});

export function setAuthToken(jwt?: string) {
  if (jwt) {
    api.defaults.headers.common.Authorization = `Bearer ${jwt}`;
  } else {
    delete api.defaults.headers.common.Authorization;
  }
}
