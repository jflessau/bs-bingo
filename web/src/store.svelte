<script lang="ts" context="module">
  import { writable } from 'svelte/store';
  import type { Writable } from 'svelte/store';

  // auth

  export enum AuthStatus {
    Init,
    Loading,
    Authenticated,
    NotAuthenticated,
    Error,
  }

  export const authStore: Writable<AuthStatus> = writable(AuthStatus.NotAuthenticated);
</script>
