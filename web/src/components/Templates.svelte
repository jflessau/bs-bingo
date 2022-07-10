<script lang="ts">
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

{#each templatesData.templates as { title, fieldAmount, owned, id } (id)}
  <div class="mt-4 flex flex-row justify-between items-center rounded-lg bg-solitude p-4">
    <div class="flex flex-col justify-start items-start">
      <p class="font-bold">{title} <span class="text-navy text-xs">(öffentlich)</span></p>
      <p class="mr-4 text-sm">{owned ? 'privat' : 'öffentlich'}, {fieldAmount} Wörter</p>
    </div>
    <div
      class=" ml-4 w-fit bg-sun rounded px-2 py-1 select-none cursor-pointer hover:brightness-105 active:brightness-95"
    >
      <p class="text-solitude font-bold text-sm">Spielen</p>
    </div>
  </div>
{/each}
