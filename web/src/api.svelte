<script lang="ts" context="module">
  import { camelizeKeys, decamelizeKeys } from 'humps';
  import axios from 'axios';

  let baseUrl: string | boolean = import.meta.env.VITE_API_BASE_URL_LOCAL;
  if (import.meta.env.MODE !== 'development') {
    baseUrl = import.meta.env.VITE_API_BASE_URL_REMOTE;
  }

  let wsBaseUrl: string | boolean = import.meta.env.VITE_API_WS_BASE_URL_LOCAL;
  if (import.meta.env.MODE !== 'development') {
    wsBaseUrl = import.meta.env.VITE_API_WS_BASE_URL_REMOTE;
  }

  export const apiUrl: string | boolean = baseUrl;
  export const apiWsUrl: string | boolean = wsBaseUrl;

  class Client {
    constructor() {
      let client: any = axios.create({
        baseURL: `${baseUrl}`,
        withCredentials: true,
      });

      client.interceptors.request.use(config => {
        const newConfig = { ...config };
        newConfig.url = `${baseUrl}${config.url}`;
        if (config.params) {
          newConfig.params = decamelizeKeys(config.params);
        }
        if (config.data) {
          newConfig.data = decamelizeKeys(config.data);
        }
        return newConfig;
      });

      client.interceptors.response.use(this.handleSuccess, this.handleError);
      client.interceptors.response.use(response => {
        if (response.data && response.headers['content-type'] === 'application/json') {
          response.data = camelizeKeys(response.data);
        }
        return response;
      });

      this.client = client;
    }

    handleSuccess(response: any) {
      return response;
    }

    handleError(error: any) {
      return Promise.reject(error);
    }

    redirectTo(document: any, path: any) {
      document.location = path;
    }

    // auth

    authSetup(callback: any) {
      return this.client.get('/auth').then(response => callback(response.status, response.data));
    }

    // templates

    listTemplates(callback: any) {
      return this.client.get('/templates').then(response => callback(response.status, response.data));
    }
  }

  export const ApiClient = new Client();
</script>
