<script lang="ts" context="module">
  import { get } from 'svelte/store';
  import { NotificationLevel, notificationsStore } from '../store.svelte';

  export function createNotification(text: string) {
    let notificationsData = get(notificationsStore);
    if (notificationsData.enabled) {
      let notifications = [...notificationsData.notifications];

      if (notifications.length < 3) {
        notifications.push({
          id: Math.floor(Math.random() * 1000000) + '',
          createdAt: new Date(),
          agePercent: 0,
          level: NotificationLevel.INFO,
          text,
        });
      }

      notificationsStore.set({ ...notificationsData, notifications: notifications });
    }
  }

  export function toggleNotifications() {
    let notificationsData = get(notificationsStore);
    notificationsStore.set({ ...notificationsData, enabled: !notificationsData.enabled });
  }
</script>

<script lang="ts">
  import { onDestroy } from 'svelte';
  import { fly } from 'svelte/transition';

  const maxNotificationAge = 5000;

  $: notificationsData = $notificationsStore;

  const updateNotifications = () => {
    let notifications = [...notificationsData.notifications];
    notifications = notifications.filter(v => new Date().getTime() - v.createdAt.getTime() < maxNotificationAge);
    notifications = notifications.map(v => {
      v.agePercent = ((new Date().getTime() - v.createdAt.getTime()) / maxNotificationAge) * 100;
      return v;
    });
    notificationsStore.set({ ...notificationsData, notifications: notifications });
  };

  let interval: ReturnType<typeof setInterval>;
  $: {
    interval = setInterval(updateNotifications, 220);
  }

  onDestroy(() => {
    clearInterval(interval);
  });
</script>

<div class="pr-2 sm:pr-4 fixed bottom-0 right-0" style="width: 300px;">
  {#each notificationsData.notifications as notification (notification.id)}
    <div
      class="w-full mb-2 sm:mb-4 max-w-xs flex flex-row overflow-hidden rounded-lg bg-white border border-sky drop-shadow-2xl dark:drop-shadow-none"
      in:fly
      out:fly
    >
      <div class="w-full flex flex-col">
        <p class="w-full px-4 py-2 text-raisin font-bold text-xs">{notification.text}</p>
        <div class="w-full bg-white" style="height: 2px;">
          <div
            class="bg-sky"
            style="height: 2px; width: {notification.agePercent}%; transition: width 0.2s linear;"
          ></div>
        </div>
      </div>
      <div
        class="w-12 flex bg-sky flex-col justify-center items-center cursor-pointer select-none"
        on:click="{() => {
          let notifications = [...notificationsData.notifications];
          notifications = notifications.filter(v => v.id !== notification.id);
          notificationsStore.set({ ...notificationsData, notifications: notifications });
        }}"
      >
        <p class="w-12 text-white text-center font-bold text-lg">X</p>
      </div>
    </div>
  {/each}
</div>
