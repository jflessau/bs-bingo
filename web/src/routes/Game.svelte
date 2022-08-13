<script lang="ts">
  import Button from './../components/Button.svelte';

  import { Link } from 'svelte-routing';
  import { onMount } from 'svelte';
  import { apiWsUrl, ApiClient } from './../api.svelte';

  interface GameUpdate {
    id: string;
    open: boolean;
    accessCode: string;
    fields: Field[][];
    players: Player[];
    username: string;
  }

  interface Field {
    id: string;
    text: string;
    position: number;
    checked: boolean;
    bingo: boolean;
  }

  interface Player {
    userId: string;
    username: string;
    bingos: number;
    hits: Array<boolean>;
    isMe: boolean;
  }

  enum GameStatus {
    INIT,
    LOADING,
    OPEN,
    CLOSED,
    ERROR,
  }

  export let gameTemplateId: string | undefined = undefined;
  export let accessCode: string | undefined = undefined;

  let url: string | boolean = import.meta.env.VITE_BASE_URL_LOCAL;
  if (import.meta.env.MODE !== 'development') {
    url = import.meta.env.VITE_BASE_URL_REMOTE;
  }

  let id: string | undefined = undefined;
  let status: GameStatus = GameStatus.INIT;
  let username: string | undefined = undefined;
  let code: string | undefined = undefined;
  let players: Array<Player> = [];
  let fields: Array<Array<Field>> = [];
  let websocket: any | undefined = undefined;

  let newUsername: string | undefined = undefined;

  onMount(async () => {
    if (gameTemplateId) {
      await ApiClient.startGame(gameTemplateId, (_status: number, data: GameUpdate) => {
        code = data.accessCode;
        fields = data.fields;
        players = data.players;
        status = GameStatus.OPEN;
        id = data.id;
        username = data.username;
      }).catch((err: any) => {
        console.error(err);
        status = GameStatus.ERROR;
      });

      startWebSocket();
    } else if (accessCode) {
      await ApiClient.joinGame(accessCode, (_status: number, data: GameUpdate) => {
        code = data.accessCode;
        fields = data.fields;
        players = data.players;
        status = GameStatus.OPEN;
        id = data.id;
        username = data.username;
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
    console.log('start websocket');
    websocket = new WebSocket(`${apiWsUrl}/game/${id}`);

    websocket.addEventListener('message', event => {
      let data: any | undefined = event.data ? JSON.parse(event.data) : undefined;

      if (data.fields) {
        let fieldsUpdate: Array<Array<Field>> = data.fields;
        fields = fieldsUpdate;
      } else if (data.players) {
        let playersUpdate: Array<Player> = data.players;
        players = playersUpdate;
      }
    });
  }

  function assertWebsocket() {
    if (websocket) {
      if (websocket.readyState > 1) {
        startWebSocket();
      }
    } else {
      startWebSocket();
    }
  }

  async function toggleField(id) {
    assertWebsocket();

    await ApiClient.updateField(id, (_status: number, data: any) => {}).catch((err: any) => {
      console.error(err);
      status = GameStatus.ERROR;
    });
  }

  async function updateUsername() {
    assertWebsocket();

    await ApiClient.updateUsername(id, newUsername, (_status: number, data: any) => {
      username = newUsername;
      newUsername = undefined;
    }).catch((err: any) => {
      console.error(err);
      status = GameStatus.ERROR;
    });
  }
</script>

{#if status === GameStatus.OPEN}
  <div class="mt-4 mb-16 flex flex-col items-start md:flex-row md:items-center justify-between">
    <div class="nav w-fit p-4 bg-solitude rounded-lg mb-8 md:mb-0">
      <Link to="/" class="font-bold px-2 py-4">{'Home'}</Link>
    </div>

    {#if newUsername === undefined}
      {#if username}
        <div class="w-full flex flex-row items-center justify-between sm:justify-end">
          <p class="mr-4"><span class="font-bold">{username}</span></p>
          <Button
            caption="Edit Username"
            variant="secondary"
            size="sm"
            on:click="{() => {
              newUsername = '';
            }}"
          />
        </div>
      {/if}
    {:else}
      <div class="flex flex-row items-center">
        <input
          type="text"
          bind:value="{newUsername}"
          maxlength="16"
          class="mr-2 px-2 py-1 text-sm bg-white border-gray border-2 focus:border-sky focus:outline-none rounded-lg"
        />
        <Button
          caption="Save username"
          size="sm"
          on:click="{updateUsername}"
          disabled="{!newUsername || (newUsername && newUsername.length < 1)}"
        />
      </div>
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

  <div class="rounded-lg mt-16 mb-4">
    <h2 class="font-bold text-lg mb-4">Players</h2>
    {#each players as { username, hits, bingos, isMe }, i}
      <div
        class="w-full mt-2 flex flex-row justify-start items-center rounded border-2 p-2 {isMe
          ? 'border-sky'
          : 'border-solitude'}"
      >
        <div class="grid grid-cols-5 overflow-hidden rounded">
          {#each hits as hit, i}
            <div class="w-3 h-3 {hit ? 'bg-sun' : 'bg-solitude'}"></div>
          {/each}
        </div>
        <div class="flex flex-col ml-8">
          <p class="text-sm font-bold">{`${i + 1} ${username} ${isMe ? '(me)' : ''}`}</p>
          <p class="text-sm">
            {bingos} Bingo{bingos !== 1 ? 's' : ''}, {hits.filter(v => v).length} Hit{hits.length !== 1 ? 's' : ''}
          </p>
        </div>
      </div>
    {/each}

    {#if code}
      <h2 class="font-bold text-lg mb-4 mt-16">Invite Link</h2>
      <p class="text-sm font-bold">
        <a class="text-sky underline break-all" href="{`${url}/games/join/${code}`}">{`${url}/games/join/${code}`}</a>
      </p>
    {/if}
  </div>
{/if}
