<script lang="ts">
  // Ties the editor and the runner together. Holds the editable copy of whatever script
  // is selected (or a blank one), tracks whether it's been changed, and owns saving.
  // The editor and runner stay dumb; this is the only piece that talks to the store.
  import ScriptEditor from "./scripts/ScriptEditor.svelte";
  import RunPanel from "./runner/RunPanel.svelte";
  import * as scripts from "../stores/scripts.svelte";

  let name = $state("");
  let tags = $state("");
  let body = $state("");

  const selected = $derived(
    scripts.store.items.find((s) => s.id === scripts.store.selectedId) ?? null,
  );

  // load the selected script into the editable fields when the selection changes. typing
  // afterwards doesnt re-run this (the fields arent what `selected` depends on), so your
  // edits stick until you actually switch to another script.
  $effect(() => {
    const s = selected;
    name = s?.name ?? "";
    tags = s?.tags ?? "";
    body = s?.body ?? "";
  });

  // have the fields drifted from whats saved? drives the Save button's enabled state.
  const dirty = $derived(
    name !== (selected?.name ?? "") ||
      tags !== (selected?.tags ?? "") ||
      body !== (selected?.body ?? ""),
  );

  async function save() {
    if (!name.trim()) return;
    await scripts.save(scripts.store.selectedId, {
      name: name.trim(),
      description: "",
      tags: tags.trim(),
      body,
      params_json: "[]", // real param specs land in the param-builder slice
      danger: "safe",
    });
  }
</script>

<div class="workspace">
  <ScriptEditor bind:name bind:tags bind:body {dirty} onsave={save} />
  <RunPanel {body} />
</div>

<style>
  .workspace {
    display: flex;
    flex-direction: column;
    height: 100%;
    font-family: system-ui, sans-serif;
  }
</style>
