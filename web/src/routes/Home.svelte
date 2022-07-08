<script lang="ts">
  import Templates from './../components/Templates.svelte';
  import { Link } from 'svelte-routing';
  import { onMount } from 'svelte';
  import { ApiClient } from './../api.svelte';
  import { templatesStore, TemplatesStatus, Template } from './../store.svelte';

  $: templatesData = $templatesStore;

  onMount(async () => {
    templatesStore.set({ ...templatesData, status: TemplatesStatus.LOADING });

    ApiClient.listTemplates((_status: number, data: Array<Template>) => {
      templatesStore.set({ ...templatesData, templates: data, status: TemplatesStatus.SUCCESS });
    }).catch((err: any) => {
      console.error(err);
      templatesStore.set({ ...templatesData, status: TemplatesStatus.ERROR });
    });
  });
</script>

<h2 class="text-2xl mb-4 text-center">Bullshit Bingo</h2>
<p class="text-center mb-8">Starte das Spiel indem du eine der Vorlagen auswählst oder selbst eine erstellst.</p>

<div class="flex justify-center items-center mb-16">
  <Link to="/create-template">
    <div
      class="w-fit mt-4 bg-sun rounded-lg px-4 py-2 select-none cursor-pointer hover:brightness-105 active:brightness-95"
    >
      <p class="text-solitude font-bold">Eigene Vorlage erstellen</p>
    </div>
  </Link>
</div>

<Templates />
