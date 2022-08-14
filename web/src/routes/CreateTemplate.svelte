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
    caption: string;
  }

  let status: Status = Status.IDLE;
  let title: string = '';
  let fields: Array<Field> = [];
  let focusLastInput = false;

  onMount(async () => {
    for (let n = 0; n < 25; n++) {
      fields.push({
        caption: '',
      });
      fields = [...fields];
    }
  });

  function addInput() {
    fields.push({
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

  function handleKeypress(event) {
    event.keyCode === 13 ? (focusLastInput = true) : (focusLastInput = false);
    if (event.keyCode === 13 || event.keyCode === 9) addInput();
  }

  function initInput(el) {
    focusLastInput && el.focus();
  }
</script>

{#if status === Status.IDLE}
  <Button caption="Home" variant="secondary" link="{`/`}" classes="mb-16 mt-4" />

  <h2 class="text-2xl mb-4">Create Template</h2>

  <h3 class="mt-8 mb-2">Title</h3>
  <input type="text" bind:value="{title}" maxlength="64" class="p-2 rounded-lg" />

  <h3 class="mt-8 mb-2">Words</h3>
  <p class="mb-2">You need at least 25 words.</p>

  <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
    {#each fields as { caption }, i}
      <input
        type="text"
        bind:value="{caption}"
        maxlength="64"
        use:initInput
        on:keydown="{event => (i === fields.length - 1 ? handleKeypress(event) : {})}"
        class="p-2 rounded-lg"
      />
    {/each}
  </div>

  <Button on:click="{addInput}" caption="Add One More" variant="secondary" classes="mt-4" />

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
{:else}<p class="text-center">An error occurred :(<br />Please refresh the page.</p>{/if}
