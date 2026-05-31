<script lang="ts">
  // app shell: library down the left, the edit+run workspace filling the rest. tools,
  // settings and the elevation toggle slot in around this in later phases.
  import ScriptList from "./lib/features/scripts/ScriptList.svelte";
  import Workspace from "./lib/features/Workspace.svelte";
</script>

<div class="app">
  <aside><ScriptList /></aside>
  <main><Workspace /></main>
</div>

<style>
  :global(html, body, #app) {
    height: 100%;
  }
  :global(body) {
    margin: 0;
  }
  .app {
    display: grid;
    grid-template-columns: 260px 1fr;
    height: 100%;
  }
  aside {
    border-right: 1px solid #333;
    min-height: 0;
  }
  main {
    min-height: 0;
  }
</style>
