<script lang="ts">
  // app shell. for now it's just the runner — the sidebar/library/tools come in later
  // phases and will slot in around this.
  import RunPanel from "./lib/features/runner/RunPanel.svelte";
</script>

<RunPanel />

<style>
  :global(html, body, #app) {
    height: 100%;
  }
  :global(body) {
    margin: 0;
  }
</style>
