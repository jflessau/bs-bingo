<script lang="ts" context="module">
  import { writable } from 'svelte/store';
  import type { Writable } from 'svelte/store';

  // auth
  export enum AuthStatus {
    IDLE,
    LOADING,
    SUCCESS,
    ERROR,
  }

  export const authStore: Writable<AuthStatus> = writable(AuthStatus.IDLE);

  // templates

  export enum TemplatesStatus {
    IDLE,
    LOADING,
    SUCCESS,
    ERROR,
  }

  export interface Template {
    id: string;
    title: string;
    word_amount: number;
  }

  export interface Templates {
    status: TemplatesStatus;
    templates: Array<Template>;
  }

  export const templatesStore: Writable<Templates> = writable({ status: TemplatesStatus.IDLE, templates: [] });
</script>
