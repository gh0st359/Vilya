<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import './app.css'; import './reset.css'; import './theme.css';
  import Sidebar from "./lib/components/Sidebar.svelte";
  import Analytics from "./lib/components/Analytics.svelte";
  import "./lib/map";
  import { events } from "./lib/store";
  async function load() {
    const res = await invoke("search_events", { q: null, since: null, until: null }) as any[];
    events.set(res);
  }
  load();
</script>
<div class="root"><div class="sidebar"><Sidebar/></div><div class="main"><div id="mapContainer" class="map"></div><Analytics/></div></div>
<style>
.root { display:grid; grid-template-columns: 360px 1fr; height:100vh; }
.sidebar { border-right: 1px solid #222; overflow:auto; }
.main { position:relative; }
.map { position:absolute; inset:0; }
html, body, #app { margin:0; height:100%; }
</style>
