<script lang="ts">
  import { onMount } from 'svelte';
  import { ApiClient } from './../api.svelte';
  import { templatesStore, TemplatesStatus, Template } from './../store.svelte';
  import Button from './../components/Button.svelte';
  import { Circle } from 'svelte-loading-spinners';

  let templateToDelete: undefined | string = undefined;

  $: templatesData = $templatesStore;

  onMount(async () => {
    listTemplates();
  });

  async function listTemplates() {
    templatesStore.set({ ...templatesData, status: TemplatesStatus.LOADING });

    await ApiClient.listTemplates((_status: number, data: Array<Template>) => {
      templatesStore.set({ ...templatesData, templates: data, status: TemplatesStatus.SUCCESS });
    }).catch((err: any) => {
      console.error(err);
      templatesStore.set({ ...templatesData, status: TemplatesStatus.ERROR });
    });
  }

  async function leaveGame(id) {
    ApiClient.leaveGame(id, (_status: number) => {
      listTemplates();
    }).catch((err: any) => {
      console.error(err);
    });
  }

  async function deleteTemplate(id) {
    ApiClient.deleteTemplate(id, (_status: number) => {
      listTemplates();
      templateToDelete = undefined;
    }).catch((err: any) => {
      console.error(err);
    });
  }
</script>

<div class="flex flex-row justify-between items-center mb-8">
  <h2 class="text-2xl text-center">Templates</h2>
  <Button caption="Create Template" size="sm" link="/create-template" />
</div>

{#if templatesData.status === TemplatesStatus.LOADING}
  <div class="flex justify-center items-center mt-4 rounded-lg bg-solitude w-full rounded-lg bg-solitude p-4">
    <Circle size="44" color="#009ffd" />
  </div>
{:else if templatesData.templates.length < 1}
  <p class="text-center w-full">No templates found :(</p>
{:else}
  {#each templatesData.templates as { title, fieldAmount, owned, id, resumable } (id)}
    <div class="mt-4 flex flex-row justify-between items-center rounded-lg bg-solitude p-4">
      <div class="flex flex-col justify-start items-start">
        <p class="font-bold">{title}</p>
        <p class="mr-4 text-sm">
          {#if owned}
            <span>private</span>,
          {:else}
            <span>public</span>,
          {/if}
          {fieldAmount} Words
        </p>
      </div>
      <div class="flex flex-row justify-center items-center">
        {#if templateToDelete}
          <Button
            caption="Don't Delete Template"
            size="sm"
            variant="secondary"
            on:click="{() => (templateToDelete = undefined)}"
            classes="mr-4"
          />
          <Button
            caption="Delete Template"
            size="sm"
            variant="primary"
            on:click="{() => deleteTemplate(templateToDelete)}"
          />
        {:else if resumable}
          {#if owned}
            <Button
              caption="Delete"
              size="sm"
              variant="text"
              on:click="{() => (templateToDelete = id)}"
              classes="mr-4"
            />
          {/if}
          <Button caption="Leave" size="sm" variant="secondary" on:click="{() => leaveGame(id)}" classes="mr-4" />
          <Button caption="Continue" size="sm" variant="secondary" link="{`/games/start/${id}`}" />
        {:else}{#if owned}
            <Button
              caption="Delete"
              size="sm"
              variant="text"
              on:click="{() => (templateToDelete = id)}"
              classes="mr-4"
            />
          {/if}
          <Button caption="Start" size="sm" variant="secondary" link="{`/games/start/${id}`}" />
        {/if}
      </div>
    </div>
  {/each}
{/if}

<div class="mb-16"></div>
