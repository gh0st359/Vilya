<script lang="ts">
  import { events } from "../store";
  let list:any[] = [];
  const unsub = events.subscribe(v => list = v);
  $: list;
</script>
<div class="p">
  <h3>Events</h3>
  <ul>
    {#each list as e}
      <li>{e.title}</li>
    {/each}
  </ul>
</div>
<style>
.p { padding: 10px; font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif; }
h3 { margin: 0 0 8px; }
ul { list-style: none; padding: 0; margin: 0; }
li { padding: 4px 0; border-bottom: 1px solid #222; }
</style>
