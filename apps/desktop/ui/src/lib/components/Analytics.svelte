<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  let daily:any[] = [];
  let byClass:any[] = [];
  onMount(async ()=>{
    daily = await invoke("analytics_daily") as any[];
    byClass = await invoke("analytics_by_class") as any[];
  });
</script>
<div class="p">
  <h3>Analytics</h3>
  <div class="grid">
    <div>
      <h4>Events per day (30d)</h4>
      <pre>{JSON.stringify(daily, null, 2)}</pre>
    </div>
    <div>
      <h4>By class (7d)</h4>
      <pre>{JSON.stringify(byClass, null, 2)}</pre>
    </div>
  </div>
</div>
<style>
.p { position:absolute; bottom:0; left:0; right:0; max-height:40%; overflow:auto; background:#0b0b0b99; color:#ddd; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
.grid { display:grid; grid-template-columns: 1fr 1fr; gap: 12px; padding: 10px; }
h3,h4 { margin: 8px 0; }
pre { background:#111; padding:6px; border:1px solid #222; }
</style>
