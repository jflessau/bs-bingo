<script lang="ts">
  import { ApiClient } from './../api.svelte';
  import { onMount } from 'svelte';
  import { Circle } from 'svelte-loading-spinners';
  import { Link } from 'svelte-routing';
  import Button from './../components/Button.svelte';

  enum Status {
    IDLE,
    LOADING,
    SUCCESS,
    ERROR,
  }

  interface Field {
    id: string;
    caption: string;
  }

  let status: Status = Status.IDLE;
  let title: string = '';
  let fields: Array<Field> = [];

  onMount(async () => {
    for (let n = 0; n < 25; n++) {
      fields.push({
        id: n + '',
        caption: '',
      });
      fields = [...fields];
    }
  });

  function addInput() {
    fields.push({
      id: Math.floor(Math.random() * 1000000) + 1 + '',
      caption: '',
    });
    fields = [...fields];
  }

  async function createTemplate() {
    status = Status.LOADING;
    await ApiClient.createTemplate(
      { title, fields: fields.map(v => v.caption).filter(v => v.trim().length > 0) },
      (_status: number) => {
        status = Status.SUCCESS;
      },
    ).catch((err: any) => {
      console.error(err);
      status = Status.ERROR;
    });
  }
</script>

{#if status === Status.IDLE}
  <div class="nav mb-16 mt-4 w-fit p-4 bg-solitude rounded-lg">
    <Link to="/" class="font-bold px-2 py-4">{'Home'}</Link>
  </div>

  <h2 class="text-2xl mb-4">Create Template</h2>

  <h3 class="mt-8 mb-2">Title</h3>
  <input
    type="text"
    bind:value="{title}"
    maxlength="64"
    class="p-2 bg-white border-gray border-2 focus:border-sky focus:outline-none rounded-lg"
  />

  <h3 class="mt-8 mb-2">Words</h3>
  <p class="mb-2">You need at least 25 words.</p>

  <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
    {#each fields as { id, caption } (id)}
      <input
        type="text"
        bind:value="{caption}"
        maxlength="64"
        class="p-2 bg-white border-gray border-2 focus:border-sky focus:outline-none rounded-lg"
      />
    {/each}
  </div>

  <Button on:click="{addInput}" caption="Add Word" variant="secondary" classes="mt-4" />

  {#if fields.filter(v => v.caption.trim().length > 0).length >= 25 && title.trim().length > 0}
    <Button caption="Save" classes="mt-16 mb-8" on:click="{createTemplate}" />
  {:else}
    <p class="text-xs mt-16">You need to fill at least 25 fields.</p>
    <Button caption="Save" disabled classes="mt-4 mb-8" />
  {/if}
{:else if status === Status.SUCCESS}
  <p class="text-center mb-8 mt-16">Template saved!</p>

  <div class="flex justify-center items-center mb-16">
    <Button link="/" caption="Continue" />
  </div>
{:else if status === Status.LOADING}
  <div class="flex justify-center items-center mt-16"><Circle size="60" color="#009ffd" /></div>
{:else}<p class="text-center">An error occurred.<br />Please refresh the page.</p>{/if}
