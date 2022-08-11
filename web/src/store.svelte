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
    fieldAmount: number;
    owned: boolean;
    resumable: boolean;
  }

  export interface Templates {
    status: TemplatesStatus;
    templates: Array<Template>;
  }

  export const templatesStore: Writable<Templates> = writable({ status: TemplatesStatus.IDLE, templates: [] });

  // create template

  export enum CreateTemplateStatus {
    IDLE,
    LOADING,
    SUCCESS,
    ERROR,
  }

  export interface CreateTemplateField {
    id: string;
    caption: string;
  }

  export interface CreateTemplateState {
    status: CreateTemplateStatus;
    template: { title: string; fields: Array<CreateTemplateField> };
  }

  export const createTemplateStore: Writable<CreateTemplateState> = writable({
    status: CreateTemplateStatus.IDLE,
    template: { title: '', fields: [] },
  });
</script>
