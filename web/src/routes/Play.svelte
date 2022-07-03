<script lang="ts">
  import { onMount } from 'svelte';

  interface Field {
    text: string;
    checked: boolean;
    bingo: boolean;
  }

  let field: Array<Array<Field>> = [
    [
      { text: 'test', checked: false, bingo: false },
      { text: 'test', checked: false, bingo: false },
      { text: 'test', checked: false, bingo: false },
      { text: 'test', checked: false, bingo: false },
      { text: 'test', checked: true, bingo: true },
    ],
    [
      { text: 'test', checked: false, bingo: false },
      { text: 'test', checked: true, bingo: false },
      { text: 'tests sad fasdf asdf', checked: true, bingo: false },
      { text: 'test', checked: true, bingo: true },
      { text: 'test', checked: false, bingo: false },
    ],
    [
      { text: 'test', checked: false, bingo: false },
      { text: 'test asdfasdf asf', checked: false, bingo: false },
      { text: 'test', checked: true, bingo: true },
      { text: 'tesasfdasdfasdfasdft', checked: false, bingo: false },
      { text: 'test', checked: false, bingo: false },
    ],
    [
      { text: 'test', checked: false, bingo: false },
      { text: 'test', checked: true, bingo: true },
      { text: 'tesadfasfdasdst', checked: false, bingo: false },
      { text: 'test', checked: false, bingo: false },
      { text: 'test', checked: false, bingo: false },
    ],
    [
      { text: 'test', checked: true, bingo: true },
      { text: 'test', checked: false, bingo: false },
      { text: 'test', checked: false, bingo: false },
      { text: 'test', checked: false, bingo: false },
      { text: 'test', checked: false, bingo: false },
    ],
  ];

  interface Player {
    id: string;
    name: string;
    bingos: number;
    hits: Array<number>;
  }

  let players: Array<Player> = [
    {
      id: '0',
      name: 'steevestoat',
      bingos: 1,
      hits: [0, 2, 6, 12, 18, 22, 24],
    },
    {
      id: '1',
      name: 'scholarsailing',
      bingos: 1,
      hits: [0, 1, 2, 3, 4, 12, 22],
    },
    {
      id: '2',
      name: 'nobodytacos',
      bingos: 0,
      hits: [0, 16, 18, 4, 5],
    },
    {
      id: '3',
      name: 'architectgreedy',
      bingos: 0,
      hits: [4, 8, 12],
    },
  ];

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
  ];

  onMount(() => {
    addInput();
  });

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

<div class="grid gap-2 grid-cols-5 grid-rows-5">
  {#each field as row, i}
    {#each row as { text, checked, bingo }, i}
      <div
        class="flex justify-center items-center rounded-lg p-1 select-none cursor-pointer {checked
          ? bingo
            ? 'bg-sun'
            : 'bg-sky'
          : 'bg-solitude'}"
      >
        <p class="break-all text-xxs sm:text-base text-center {checked ? 'text-solitude font-bold' : 'text-navy'}">
          {text}
        </p>
      </div>
    {/each}
  {/each}
</div>
<div class="bg-solitude rounded-lg p-4 mt-16 mb-4">
  <div class="flex justify-between items-center mb-4">
    <h2 class="font-bold text-lg">Spieler</h2>
    <div class="flex flex-col items-end">
      <p class="text-sm">Party-Code</p>
      <p class="text-sm font-bold">6H9DH29J</p>
    </div>
  </div>
  {#each players as { id, name, hits, bingos }, i}
    <div class="w-full mt-2 flex flex-row justify-start items-center rounded bg-sky p-2">
      <div class="grid grid-cols-5 overflow-hidden rounded">
        {#each Array(25) as n, i}
          <div class="w-3 h-3 {hits.includes(i) ? 'bg-sun' : 'bg-solitude'}"></div>
        {/each}
      </div>
      <div class="flex flex-col ml-8">
        <p class="text-sm text-solitude font-bold">#{i + 1} {name}</p>
        <p class="text-sm text-solitude">
          {bingos} Bingo{bingos !== 1 ? 's' : ''}, {hits.length}Hit{hits.length !== 1 ? 's' : ''}
        </p>
      </div>
    </div>
  {/each}
</div>
