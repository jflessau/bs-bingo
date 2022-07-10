<script lang="ts">
  import { ApiClient } from './../api.svelte';
  import { onMount } from 'svelte';

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

  $: console.log(status);

  onMount(async () => {
    for (let n = 0; n < 9; n++) {
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

{#if status === Status.IDLE || status === Status.LOADING}
  <h2 class="text-2xl mb-4">Vorlage erstellen</h2>

  <h3 class="mt-8 mb-2">Titel</h3>
  <p class="mb-2">Gib deinem Spiel einen Titel.</p>
  <input
    type="text"
    bind:value="{title}"
    maxlength="64"
    class="p-2 bg-white border-gray border-2 focus:border-sky focus:outline-none rounded-lg"
  />

  <h3 class="mt-8 mb-2">Wörter</h3>
  <p class="mb-2">Es braucht mindestens 9 Wörter für ein 3x3 und mindestens 25 für ein 5x5 Spielfeld.</p>

  <div class="grid grid-cols-3 gap-4">
    {#each fields as { id, caption } (id)}
      <input
        type="text"
        bind:value="{caption}"
        maxlength="64"
        class="p-2 bg-white border-gray border-2 focus:border-sky focus:outline-none rounded-lg"
      />
    {/each}
  </div>

  <div
    class="w-fit mt-4 bg-sky rounded-lg px-4 py-2 select-none cursor-pointer hover:brightness-105 active:brightness-95"
    on:click="{addInput}"
  >
    <p class="text-solitude font-bold">Feld hinzufügen</p>
  </div>

  {#if fields.filter(v => v.caption.trim().length > 0).length >= 9 && title.trim().length > 0}
    <div
      class="w-fit mt-16 bg-sun rounded-lg px-4 py-2 select-none cursor-pointer hover:brightness-105 active:brightness-95"
      on:click="{createTemplate}"
    >
      <p class="text-solitude font-bold">Speichern</p>
    </div>
  {:else}
    <div class="w-fit mt-16 bg-sun rounded-lg px-4 py-2 select-none cursor-pointer saturate-0">
      <p class="text-solitude font-bold">Speichern</p>
    </div>
  {/if}
{:else if status === Status.SUCCESS}{:else}{/if}
