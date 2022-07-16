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

  // game

  export interface Field {
    id: string;
    text: string;
    position: number;
    checked: boolean;
    bingo: boolean;
  }

  export interface Player {
    iduserId: string;
    username: string;
    bingos: number;
    hits: Array<bool>;
  }

  export interface GameUpdate {
    id: string;
    open: boolean;
    accessCode: string;
  }

  export interface StartGame {
    gameTemplateId: string;
  }

  export interface JoinGame {
    accessCode: string;
  }

  export interface FieldUpdate {
    id: string;
    checked: boolean;
  }
</script>
