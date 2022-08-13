<script lang="ts">
  import { Link } from 'svelte-routing';
  import { createEventDispatcher } from 'svelte';

  export let variant: string = 'primary';
  export let size: 'sm' | 'md' = 'md';
  export let caption: string;
  export let classes: string = '';
  export let link: string | undefined = undefined;
  export let disabled: boolean | undefined = undefined;

  const dispatch = createEventDispatcher();

  function click() {
    dispatch('click');
  }
</script>

{#if link}
  <Link to="{link}">
    <div
      on:click="{click}"
      class="w-fit {variant === 'primary' ? 'bg-sun dark:bg-navy' : variant === 'text' ? 'bg-gray' : 'bg-sky'} {size ===
      'sm'
        ? 'px-2 py-1 rounded'
        : 'px-4 py-2 rounded-lg'} select-none cursor-pointer hover:brightness-105 active:brightness-95 {disabled &&
        'saturate-0'} {classes}"
    >
      <p class="text-solitude font-bold {size === 'sm' && 'text-sm'} {variant === 'text' ? 'text-raisin' : ''}">
        {caption}
      </p>
    </div>
  </Link>
{:else}
  <div
    on:click="{click}"
    class="w-fit {variant === 'primary' ? 'bg-sun dark:bg-navy' : variant === 'text' ? 'bg-gray' : 'bg-sky'} {size ===
    'sm'
      ? 'px-2 py-1 rounded'
      : 'px-4 py-2 rounded-lg'} select-none cursor-pointer hover:brightness-105 active:brightness-95 {disabled &&
      'saturate-0'} {classes}"
  >
    <p class="text-solitude font-bold {size === 'sm' && 'text-sm'} {variant === 'text' ? 'text-raisin' : ''}">
      {caption}
    </p>
  </div>
{/if}
