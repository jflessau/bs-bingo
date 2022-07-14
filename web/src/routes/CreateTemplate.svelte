<script lang="ts">
  import { ApiClient } from './../api.svelte';
  import { onMount } from 'svelte';
  import { Circle } from 'svelte-loading-spinners';
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

{#if status === Status.IDLE}
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

  <Button on:click="{addInput}" caption="Feld Hinzufügen" variant="secondary" classes="mt-4" />

  {#if fields.filter(v => v.caption.trim().length > 0).length >= 9 && title.trim().length > 0}
    <Button caption="Speichern" classes="mt-16" on:click="{createTemplate}" />
  {:else}
    <Button caption="Speichern" disabled classes="mt-16" />
  {/if}
{:else if status === Status.SUCCESS}
  <p class="text-center mb-8">Vorlage gespeichert!</p>

  <div class="flex justify-center items-center mb-16">
    <Button link="/templates" caption="Zu meinen Vorlagen" />
  </div>
{:else if status === Status.LOADING}
  <div class="flex justify-center items-center mt-4"><Circle size="60" color="#009ffd" /></div>
{:else}<p>An Error occurred.</p>{/if}
