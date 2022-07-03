<script lang="ts">
  interface Template {
    id: string;
    title: string;
    field_amount: number;
  }

  let templates: Array<Template> = [
    {
      id: '0',
      title: 'Lorem ipsum',
      field_amount: 9,
    },
    {
      id: '1',
      title: 'Dolor sit amet',
      field_amount: 32,
    },
    {
      id: '2',
      title: 'consetetur sadipscing elitr sed diam',
      field_amount: 25,
    },
    {
      id: '3',
      title: 'Takimata sanctus est Lorem ipsum dolor sit amet',
      field_amount: 28,
    },
    {
      id: '4',
      title: 'Sanctus Est Lorem Ipsum Dolor Sit Amet',
      field_amount: 64,
    },
  ];
</script>

{#each templates as { title, field_amount, id } (id)}
  <div class="mt-4 flex flex-row justify-between items-center rounded-lg bg-solitude p-4">
    <div class="flex flex-col justify-start items-start">
      <p class="font-bold">{title}</p>
      <p class="mr-4 text-sm">{field_amount} Wörter</p>
    </div>
    <div
      class=" ml-4 w-fit bg-sun rounded px-2 py-1 select-none cursor-pointer hover:brightness-105 active:brightness-95"
    >
      <p class="text-solitude font-bold text-sm">Spielen</p>
    </div>
  </div>
{/each}
