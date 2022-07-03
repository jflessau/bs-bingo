<script lang="ts">
  import './style/main.scss';
  import Home from './routes/Home.svelte';
  import Templates from './routes/Templates.svelte';
  import Play from './routes/Play.svelte';
  import CreateTemplate from './routes/CreateTemplate.svelte';
  import Nav from './components/Nav.svelte';
  import { Router, Route } from 'svelte-routing';
  import { authStore, AuthStatus } from './store.svelte';

  let authStatus: AuthStatus = AuthStatus.Init;
  authStore.subscribe((value: AuthStatus) => {
    authStatus = value;
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
  <div class="container mx-auto px-2">
    <Route path="/" component={Home} />
    <Route path="/play" component={Play} />
    <Route path="/templates" component={Templates} />
    <Route path="/create-template" component={CreateTemplate} />
  </div>
</Router>
