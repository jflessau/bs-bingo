<script lang="ts">
  import Button from './../components/Button.svelte';
  import { Circle } from 'svelte-loading-spinners';
  import { Confetti } from 'svelte-confetti';
  import { createNotification, toggleNotifications } from './../components/Sackbar.svelte';
  import { onMount } from 'svelte';
  import { apiWsUrl, ApiClient } from './../api.svelte';
  import { notificationsStore } from './../store.svelte';

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
    ERROR,
  }

  export let gameTemplateId: string | undefined = undefined;
  export let gridSize: number | undefined = undefined;
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
  let showConfetti: undefined | Date = undefined;
  let newUsername: string | undefined = undefined;

  $: notificationsData = $notificationsStore;

  const confettiDuration = 2000;
  const interval = () =>
    showConfetti && new Date().getTime() - showConfetti.getTime() > confettiDuration ? (showConfetti = undefined) : {};

  let clear: ReturnType<typeof setInterval>;
  $: {
    clearInterval(clear);
    clear = setInterval(interval, confettiDuration);
  }

  $: gridCols = fields ? fields.length : 0;

  onMount(async () => {
    if (gameTemplateId) {
      status = GameStatus.LOADING;
      await ApiClient.startGame(gameTemplateId, gridSize, (_status: number, data: GameUpdate) => {
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
    console.info('start websocket');
    websocket = new WebSocket(`${apiWsUrl}/game/${id}`);

    websocket.addEventListener('message', event => {
      let data: any | undefined = event.data ? JSON.parse(event.data) : undefined;

      if (data.fields) {
        let fieldsUpdate: Array<Array<Field>> = data.fields;
        fields = fieldsUpdate;
      } else if (data.players) {
        let playersUpdate: Player[] = data.players;

        // confetti if player got bingo

        let pastMe = players.find((v: Player) => v.isMe === true);
        let futureMe = playersUpdate.find((v: Player) => v.isMe === true);
        if (pastMe && futureMe && pastMe.bingos < futureMe.bingos) {
          showConfetti = new Date();
        }

        // notification if new opponent joins

        let playersJoined: number = 0;
        for (let newPlayer of playersUpdate) {
          let known: boolean = false;
          for (let player of players) {
            if (newPlayer.userId === player.userId) {
              known = true;
            }
          }
          if (!known && !newPlayer.isMe) {
            playersJoined++;
          }
        }
        if (playersJoined > 0) {
          createNotification(`${playersJoined} player${playersJoined > 1 ? 's' : ''} joined!`);
        }

        // notification if opponent got bingo

        for (let player of players) {
          for (let newPlayer of playersUpdate) {
            if (player.userId === newPlayer.userId && !newPlayer.isMe && newPlayer.bingos > player.bingos) {
              createNotification(
                `${newPlayer.username} now has ${newPlayer.bingos} bingo${newPlayer.bingos > 1 ? 's' : ''}!`,
              );
            }
          }
        }

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

{#if status === GameStatus.ERROR}
  <div class="flex justify-center items-center mt-16">
    <p class="text-center">An error occurred :(<br />Please refresh the page.</p>
  </div>
{:else if status === GameStatus.INIT || status === GameStatus.LOADING}
  <div class="flex justify-center items-center mt-16"><Circle size="60" color="#009ffd" /></div>
{:else if status === GameStatus.OPEN}
  <div class="mt-4 mb-16 flex flex-col items-start sm:flex-row sm:items-center justify-between">
    <Button caption="Home" variant="secondary" link="{`/`}" classes="mb-8 sm:mb-0" />

    {#if newUsername === undefined}
      {#if username}
        <div class="w-full flex flex-row items-center justify-end">
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
      <div class="w-full flex flex-row items-center justify-end">
        <input type="text" bind:value="{newUsername}" maxlength="16" class="mr-2 px-2 py-1 text-sm rounded-lg" />
        <Button
          caption="Save username"
          variant="primary"
          size="sm"
          on:click="{updateUsername}"
          disabled="{!newUsername || (newUsername && newUsername.length < 1)}"
        />
      </div>
    {/if}
  </div>

  <div class="grid gap-2 grid-cols-{gridCols}">
    {#each fields as row, i}
      {#each row as { id, text, checked, bingo } (id)}
        <div
          on:click="{() => toggleField(id)}"
          class="flex justify-center items-center rounded-lg p-2 select-none cursor-pointer {checked
            ? bingo
              ? 'bg-sun dark:bg-sun'
              : 'bg-sky dark:bg-sun'
            : 'bg-solitude dark:bg-navy'}"
        >
          <p
            class="text-xxs sm:text-base text-center {checked ? 'font-bold text-white' : ''}"
            style="word-break: break-word;"
          >
            {text}
          </p>
        </div>
      {/each}
    {/each}
  </div>

  {#if showConfetti}
    <div class="flex flex-row justify-center">
      <Confetti amount="50" xSpread="0.15" fallDistance="200px" duration="{confettiDuration}" />
    </div>
  {/if}

  <div class="rounded-lg mt-16 mb-4">
    <h2 class="font-bold text-lg mb-4">Players</h2>
    {#each players as { username, hits, bingos, isMe }, i}
      <div
        class="w-full mt-2 flex flex-row justify-start items-center rounded-lg border-4 p-2 {isMe
          ? 'border-sky'
          : 'border-solitude dark:border-navy'}"
      >
        <div class="grid grid-cols-{gridCols} overflow-hidden rounded">
          {#each hits as hit, i}
            <div class="w-3 h-3 {hit ? 'bg-sun' : 'bg-solitude dark:bg-navy'}"></div>
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
      <Button
        caption="Copy to Clipboard"
        variant="secondary"
        size="sm"
        on:click="{() => {
          navigator.clipboard.writeText(`${url}/games/join/${code}`).then(
            function () {
              createNotification('Copied!');
            },
            function (_) {
              createNotification('Failed to copy :(');
            },
          );
        }}"
        classes="mt-4"
      />
    {/if}

    <h2 class="font-bold text-lg mb-4 mt-16">Settings</h2>
    <label>
      <input type="checkbox" checked="{notificationsData.enabled}" on:change="{toggleNotifications}" />
      Receive in-game notifications about your opponents.
    </label>
  </div>
{/if}

<!-- grid-cols-x is used conditionally and tailwind won't include these classes otherwise -->
<div class="grid-cols-1 grid-cols-2 grid-cols-3 grid-cols-4 grid-cols-5 grid-cols-6"></div>
