<script lang="ts">
  import './style/main.scss';
  import Home from './routes/Home.svelte';
  import Game from './routes/Game.svelte';
  import CreateTemplate from './routes/CreateTemplate.svelte';
  import Snackbar from './components/Snackbar.svelte';
  import { Router, Route } from 'svelte-routing';
  import { ApiClient } from './api.svelte';
  import { AuthStatus, authStore } from './store.svelte';
  import { onMount } from 'svelte';
  import { Circle } from 'svelte-loading-spinners';

  $: authData = $authStore;

  onMount(async () => {
    authStore.set(AuthStatus.LOADING);

    await ApiClient.authSetup((status: number) => {
      if (status === 200) {
        authStore.set(AuthStatus.SUCCESS);
      } else {
        console.error('Auth setup failed with status:', status);
        authStore.set(AuthStatus.ERROR);
      }
    }).catch((err: any) => {
      console.error(err);
      authStore.set(AuthStatus.ERROR);
    });
  });
</script>

<Snackbar />

<Router>
  {#if authData === AuthStatus.SUCCESS}
    <div class="container mx-auto px-2">
      <Route path="/" component="{Home}" />
      <Route path="/games/start/:gameTemplateId/:gridSize" component="{Game}" />
      <Route path="/games/join/:accessCode" component="{Game}" />
      <Route path="/create-template" component="{CreateTemplate}" />
    </div>
  {:else if authData === AuthStatus.IDLE || authData === AuthStatus.LOADING}
    <div class="flex justify-center items-center mt-16"><Circle size="60" color="#009ffd" /></div>
  {:else}
    <div class="flex justify-center items-center mt-16">
      <p class="text-center">An error occurred :(<br />Please refresh the page.</p>
    </div>
  {/if}
</Router>

<div class="w-full mt-32 mb-8">
  <p class="text-sm text-center mt-32 mb-8">
    Found a bug?<br />
    <a class="underline" href="https://github.com/jflessau/bs-bingo/issues">Open an issue</a>
    or <a class="underline" href="https://github.com/jflessau/bs-bingo">fork </a> me on GitHub.
  </p>
  <p class="text-center text-sm">
    <a class="" href="https://jflessau.com/info/legal-notice/">Legal Notice</a> |
    <a class="" href="https://jflessau.com/info/privacy-policy/">Privacy Policy</a>
  </p>
</div>
