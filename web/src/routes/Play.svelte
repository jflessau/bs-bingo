<script lang="ts">
  import { onMount } from 'svelte';
  import { apiWsUrl } from './../api.svelte';
  import type { Player, Field, GameUpdate, StartGame, JoinGame } from './../store.svelte';

  enum GameStatus {
    INIT,
    LOADING,
    OPEN,
    CLOSED,
    ERROR,
  }

  interface StartGame {
    gameTemplateId: string;
  }

  interface JoinGame {
    accessCode: string;
  }

  interface FieldUpdate {
    id: string;
    checked: boolean;
  }

  interface GameUpdate {
    id: string;
    open: boolean;
    accessCode: string;
  }

  export let id: string | undefined = undefined;
  export let accessCode: string | undefined = undefined;

  let status: GameStatus = GameStatus.INIT;
  let code: string | undefined = undefined;
  let players: Array<Player> = [];
  let fields: Array<Array<Field>> = [];
  let websocket: any | undefined = undefined;

  onMount(async () => {
    websocket = new WebSocket(`${apiWsUrl}/game`);
    websocket.addEventListener('open', event => {
      if (id) {
        const startGame: StartGame = {
          gameTemplateId: id,
        };
        websocket.send(JSON.stringify({ startGame }));
      } else if (accessCode) {
        let joinGame: JoinGame = { accessCode };
        websocket.send(JSON.stringify({ joinGame }));
      } else {
        status = GameStatus.ERROR;
      }
    });

    websocket.addEventListener('message', event => {
      let data: any | undefined = event.data ? JSON.parse(event.data) : undefined;
      if (data.gameUpdate) {
        let gameUpdate: GameUpdate = data.gameUpdate;
        if (gameUpdate.open) {
          status = GameStatus.OPEN;
        } else {
          status = GameStatus.CLOSED;
        }
        code = gameUpdate.accessCode;
      } else if (data.fieldsUpdate) {
        let fieldsUpdate: Array<Array<Field>> = data.fieldsUpdate;
        fields = fieldsUpdate;
      }
    });
  });

  function toggleField(id) {
    let checked: boolean = false;
    fields.forEach(v =>
      v.forEach(v => {
        if (v.id === id) checked = !v.checked;
      }),
    );

    let fieldUpdate: FieldUpdate = {
      id: id,
      checked,
    };
    websocket.send(JSON.stringify({ fieldUpdate }));
  }
</script>

{#if status === GameStatus.OPEN}
  <div class="grid gap-2 grid-cols-5 grid-rows-5">
    {#each fields as row, i}
      {#each row as { id, text, checked, bingo } (id)}
        <div
          on:click="{() => toggleField(id)}"
          class="flex justify-center items-center rounded-lg p-2 select-none cursor-pointer {checked
            ? bingo
              ? 'bg-sun'
              : 'bg-sky'
            : 'bg-solitude'}"
        >
          <p
            class="text-xxs sm:text-base text-center {checked ? 'text-solitude font-bold' : 'text-navy'}"
            style="word-break: break-word;"
          >
            {text}
          </p>
        </div>
      {/each}
    {/each}
  </div>

  <div class="bg-solitude rounded-lg p-4 mt-16 mb-4">
    <div class="flex justify-between items-center mb-4">
      <h2 class="font-bold text-lg">Spieler</h2>
      {#if code}
        <div class="flex flex-col items-end">
          <p class="text-sm">Party-Code</p>
          <p class="text-sm font-bold">{code}</p>
        </div>
      {/if}
    </div>
    <!-- {#each players as { id, name, hits, bingos }, i}
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
  {/each} -->
  </div>
{/if}
