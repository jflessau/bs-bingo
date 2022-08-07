<script lang="ts">
  import Button from './../components/Button.svelte';
  import { onMount } from 'svelte';
  import { apiWsUrl, ApiClient } from './../api.svelte';
  import type { Player, Field } from './../store.svelte';

  enum GameStatus {
    INIT,
    LOADING,
    OPEN,
    CLOSED,
    ERROR,
  }

  export let gameTemplateId: string | undefined = undefined;
  export let accessCode: string | undefined = undefined;

  let id: string | undefined = undefined;
  let status: GameStatus = GameStatus.INIT;
  let code: string | undefined = undefined;
  let players: Array<Player> = [];
  let fields: Array<Array<Field>> = [];
  let websocket: any | undefined = undefined;

  let newUsername: string | undefined = undefined;

  onMount(async () => {
    if (gameTemplateId) {
      await ApiClient.startGame(gameTemplateId, (_status: number, data: any) => {
        code = data.accessCode;
        fields = data.fields;
        players = data.players;
        status = GameStatus.OPEN;
        id = data.id;
      }).catch((err: any) => {
        console.error(err);
        status = GameStatus.ERROR;
      });

      startWebSocket();
    } else if (accessCode) {
      await ApiClient.joinGame(accessCode, (_status: number, data: any) => {
        code = data.accessCode;
        fields = data.fields;
        players = data.players;
        status = GameStatus.OPEN;
        id = data.id;
      }).catch((err: any) => {
        console.error(err);
        status = GameStatus.ERROR;
      });

      startWebSocket();
    } else {
      console.error('no game id or accessCode found');
      status = GameStatus.ERROR;
    }
  });

  function startWebSocket() {
    websocket = new WebSocket(`${apiWsUrl}/game/${id}`);

    websocket.addEventListener('message', event => {
      let data: any | undefined = event.data ? JSON.parse(event.data) : undefined;

      if (data.fieldsUpdate) {
        let fieldsUpdate: Array<Array<Field>> = data.fieldsUpdate;
        fields = fieldsUpdate;
      } else if (data.playersUpdate) {
        let playersUpdate: Array<Player> = data.playersUpdate;
        players = playersUpdate;
      }
    });
  }

  async function toggleField(id) {
    await ApiClient.updateField(id, (_status: number, data: any) => {
      console.log(data);
    }).catch((err: any) => {
      console.error(err);
      status = GameStatus.ERROR;
    });
  }

  async function updateUsername() {
    await ApiClient.updateUsername(id, newUsername, (_status: number, data: any) => {
      console.log(data);
      newUsername = undefined;
    }).catch((err: any) => {
      console.error(err);
      status = GameStatus.ERROR;
    });
  }
</script>

{#if status === GameStatus.OPEN}
  <div class="flex flex-row justify-end items-center mb-2">
    {#if newUsername === undefined}
      <Button
        caption="Edit Username"
        size="sm"
        on:click="{() => {
          newUsername = '';
        }}"
      />
    {:else}
      <input
        type="text"
        bind:value="{newUsername}"
        maxlength="22"
        class="mr-2 px-2 py-1 text-sm bg-white border-gray border-2 focus:border-sky focus:outline-none rounded-lg"
      />
      <Button caption="Save" size="sm" on:click="{updateUsername}" />
    {/if}
  </div>
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
    {#each players as { username, hits, bingos }, i}
      <div class="w-full mt-2 flex flex-row justify-start items-center rounded bg-sky p-2">
        <div class="grid grid-cols-5 overflow-hidden rounded">
          {#each hits as hit, i}
            <div class="w-3 h-3 {hit ? 'bg-sun' : 'bg-solitude'}"></div>
          {/each}
        </div>
        <div class="flex flex-col ml-8">
          <p class="text-sm text-solitude font-bold">#{i + 1} {username}</p>
          <p class="text-sm text-solitude">
            {bingos} Bingo{bingos !== 1 ? 's' : ''}, {hits.filter(v => v).length} Hit{hits.length !== 1 ? 's' : ''}
          </p>
        </div>
      </div>
    {/each}
  </div>
{/if}
