<script lang="ts">
  import { onMount } from 'svelte';
  import { ApiClient } from './../api.svelte';
  import { templatesStore, TemplatesStatus, Template } from './../store.svelte';
  import Button from './../components/Button.svelte';
  import { Circle } from 'svelte-loading-spinners';

  let templateToDelete: undefined | string = undefined;
  let templateToPlay: undefined | string = undefined;

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
    templatesStore.set({ ...templatesData, status: TemplatesStatus.LOADING });
    ApiClient.leaveGame(id, (_status: number) => {
      listTemplates();
    }).catch((err: any) => {
      console.error(err);
    });
  }

  async function deleteTemplate(id) {
    templatesStore.set({ ...templatesData, status: TemplatesStatus.LOADING });
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
  <div
    class="flex justify-center items-center mt-4 rounded-lg bg-solitude dark:bg-navy w-full rounded-lg bg-solitude p-4"
  >
    <Circle size="44" color="#009ffd" />
  </div>
{:else if templatesData.templates.length < 1}
  <p class="text-center w-full">No templates found :(</p>
{:else}
  {#each templatesData.templates as template (template.id)}
    <div class="mt-4 p-4 flex flex-row justify-between items-center rounded-lg bg-solitude dark:bg-navy">
      <div class="flex flex-col justify-start items-start mr-4">
        <p class="font-bold break-all">{template.title}</p>
        <p class="mr-4 text-sm">
          {#if template.owned}
            <span>yours</span>
          {:else if template.approved}
            <span>public</span>
          {:else if template.accessCode}
            <span>invited</span>
          {/if}
          <!-- {template.fieldAmount} Words -->
          {template.playerAmount > 0 ? `| ${template.playerAmount} player${template.playerAmount > 1 ? 's' : ''}` : ''}
        </p>
      </div>
      <div class="flex flex-row justify-center items-center">
        {#if templateToPlay === template.id}
          <Button
            caption="Cancel"
            size="sm"
            variant="secondary"
            classes="ml-4"
            on:click="{() => (templateToPlay = undefined)}"
          />
          {#if template.fieldAmount >= 9}
            <Button
              caption="Play 3x3"
              size="sm"
              variant="secondary"
              classes="ml-4"
              link="{`/games/start/${template.id}/3`}"
            />
          {/if}
          {#if template.fieldAmount >= 25}
            <Button
              caption="Play 5x5"
              size="sm"
              variant="secondary"
              classes="ml-4"
              link="{`/games/start/${template.id}/5`}"
            />
          {/if}
        {:else if templateToDelete === template.id}
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
        {:else}
          {#if template.owned}
            <Button
              caption="Delete"
              size="sm"
              variant="text"
              on:click="{() => (templateToDelete = template.id)}"
              classes="mr-4"
            />
          {/if}
          {#if template.owned || template.approved || template.accessCode}
            {#if template.accessCode}
              <Button
                caption="Leave"
                size="sm"
                variant="secondary"
                on:click="{() => leaveGame(template.id)}"
                classes="mr-4"
              />
              <Button caption="Continue" size="sm" variant="secondary" link="{`/games/join/${template.accessCode}`}" />
            {:else}
              <Button caption="Play" size="sm" variant="secondary" on:click="{() => (templateToPlay = template.id)}" />
            {/if}
          {/if}
        {/if}
      </div>
    </div>
  {/each}
{/if}

<div class="mb-16"></div>
