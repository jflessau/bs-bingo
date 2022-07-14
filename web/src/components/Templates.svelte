<script lang="ts">
  import { onMount } from 'svelte';
  import { ApiClient } from './../api.svelte';
  import { templatesStore, TemplatesStatus, Template } from './../store.svelte';
  import Button from './../components/Button.svelte';
  import { Circle } from 'svelte-loading-spinners';

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

<div class="flex flex-row justify-between items-center mb-8">
  <h2 class="text-2xl text-center">Vorlagen</h2>
  <Button caption="Vorlage Erstellen" size="sm" link="/create-template" />
</div>

{#if templatesData.status === TemplatesStatus.LOADING}
  <div class="flex justify-center items-center mt-4 rounded-lg bg-solitude w-full rounded-lg bg-solitude p-4">
    <Circle size="44" color="#009ffd" />
  </div>
{:else}
  {#each templatesData.templates as { title, fieldAmount, owned, id } (id)}
    <div class="mt-4 flex flex-row justify-between items-center rounded-lg bg-solitude p-4">
      <div class="flex flex-col justify-start items-start">
        <p class="font-bold">{title}</p>
        <p class="mr-4 text-sm">
          {#if owned}
            <span>privat</span>,
          {:else}
            <span>öffentlich</span>,
          {/if}
          {fieldAmount} Wörter
        </p>
      </div>
      <Button caption="Spielen" size="sm" variant="secondary" />
    </div>
  {/each}
{/if}

<div class="mb-16"></div>
