<script lang="ts">
  import './style/main.scss';
  import Home from './routes/Home.svelte';
  import Templates from './routes/Templates.svelte';
  import Game from './routes/Game.svelte';
  import CreateTemplate from './routes/CreateTemplate.svelte';
  import Nav from './components/Nav.svelte';
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

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
  <link
    href="https://fonts.googleapis.com/css2?family=Source+Code+Pro:wght@200;400;700&display=swap"
    rel="stylesheet"
  />
  <title>BS Bingo</title>
</svelte:head>

<Router>
  <Nav />
  {#if authData === AuthStatus.SUCCESS}
    <div class="container mx-auto px-2">
      <Route path="/" component="{Home}" />
      <Route path="/games/start/:gameTemplateId" component="{Game}" />
      <Route path="/games/join/:accessCode" component="{Game}" />
      <Route path="/templates" component="{Templates}" />
      <Route path="/create-template" component="{CreateTemplate}" />
    </div>
  {:else if authData === AuthStatus.IDLE || authData === AuthStatus.LOADING}
    <div class="flex justify-center items-center"><Circle size="60" color="#009ffd" /></div>
  {:else}
    <div class="flex justify-center items-center">
      <p class="text-center">An error occurred.<br />Please refresh the page.</p>
    </div>
  {/if}
</Router>
