<script lang="ts">
  interface Input {
    id: string;
    text: string;
  }

  let inputs: Array<Input> = [
    {
      id: Math.floor(Math.random() * 1000000) + 1 + '',
      text: '',
    },
    {
      id: Math.floor(Math.random() * 1000000) + 1 + '',
      text: '',
    },
    {
      id: Math.floor(Math.random() * 1000000) + 1 + '',
      text: '',
    },
    {
      id: Math.floor(Math.random() * 1000000) + 1 + '',
      text: '',
    },
    {
      id: Math.floor(Math.random() * 1000000) + 1 + '',
      text: '',
    },
    {
      id: Math.floor(Math.random() * 1000000) + 1 + '',
      text: '',
    },
    {
      id: Math.floor(Math.random() * 1000000) + 1 + '',
      text: '',
    },
    {
      id: Math.floor(Math.random() * 1000000) + 1 + '',
      text: '',
    },
    {
      id: Math.floor(Math.random() * 1000000) + 1 + '',
      text: '',
    },
  ];
  let templateTitle: string = '';
  let publishTemplate: boolean = false;

  function addInput() {
    console.log('add');
    let newInput: Input = {
      id: Math.floor(Math.random() * 1000000) + 1 + '',
      text: '',
    };

    inputs = [...inputs];
    inputs.push(newInput);
  }
</script>

<h2 class="text-2xl mb-4">Vorlage erstellen</h2>

<h3 class="mt-8 mb-2">Titel</h3>
<input
  type="text"
  bind:value="{templateTitle}"
  maxlength="64"
  class="p-2 bg-white border-gray border-2 focus:border-sky focus:outline-none rounded-lg"
/>

<h3 class="mt-8 mb-2">Wörter</h3>
<p class="mb-2">Es braucht mindestens 25 Wörter für ein 5x5 Spielfeld.</p>

<div class="grid grid-cols-3 gap-4">
  {#each inputs as { id, text } (id)}
    <input
      type="text"
      bind:value="{text}"
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

<h2 class="mt-8 mb-2">Speichern & Starten</h2>
<div class="flex flex-row justify-start items-center">
  <input type="checkbox" name="publish-template" bind:value="{publishTemplate}" />
  <label for="publish-template" class="ml-2">Vorlage veröffentlichen, sodass alle sie sehen und nutzen können.</label>
</div>

<div
  class="w-fit mt-4 bg-sun rounded-lg px-4 py-2 select-none cursor-pointer hover:brightness-105 active:brightness-95"
  on:click="{addInput}"
>
  <p class="text-solitude font-bold">Spiel Starten</p>
</div>
