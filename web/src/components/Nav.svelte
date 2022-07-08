<script lang="ts">
  import { Link } from 'svelte-routing';
</script>

<div class="w-full bg-sky mb-16 nav">
  <div class="container mx-auto">
    <div class="flex justify-between items-center">
      <Link to="/" class="text-solitude font-bold px-2 py-4">Home</Link>
      <Link to="/play" class="text-solitude font-bold px-2 py-4">Spielen</Link>
      <Link to="/templates" class="text-solitude font-bold px-2 py-4">Vorlagen</Link>
      <Link to="/create-template" class="text-solitude font-bold px-2 py-4">Vorlage Erstellen</Link>
    </div>
  </div>
</div>
