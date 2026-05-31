<script lang="ts">
  // Ties the editor, the params, and the runner together. Holds the editable copy of
  // whatever script is selected (or a blank one), tracks whether it's been changed, and
  // owns saving. The child components stay dumb; this is the only piece that talks to
  // the store.
  import ScriptEditor from "./scripts/ScriptEditor.svelte";
  import ParamForm from "../params/ParamForm.svelte";
  import ParamBuilder from "../params/ParamBuilder.svelte";
  import RunPanel from "./runner/RunPanel.svelte";
  import * as scripts from "../stores/scripts.svelte";
  import type { ParamSpec, RunArg, Script } from "../ipc/api";

  let name = $state("");
  let tags = $state("");
  let body = $state("");
  let params = $state<ParamSpec[]>([]);
  let args = $state<RunArg[]>([]);
  let paramMode = $state<"fill" | "edit">("fill");

  const selected = $derived(
    scripts.store.items.find((s) => s.id === scripts.store.selectedId) ?? null,
  );

  // params_json is stored as text; be forgiving if it's ever malformed rather than
  // blowing up the whole editor over one bad row.
  function parseParams(s: Script | null): ParamSpec[] {
    if (!s) return [];
    try {
      const v = JSON.parse(s.params_json);
      return Array.isArray(v) ? v : [];
    } catch {
      return [];
    }
  }

  // load the selected script into the editable fields when the selection changes. typing
  // afterwards doesnt re-run this, so edits stick until you switch scripts.
  $effect(() => {
    const s = selected;
    name = s?.name ?? "";
    tags = s?.tags ?? "";
    body = s?.body ?? "";
    params = parseParams(s);
  });

  // round-trip the saved params through parse so the comparison is apples-to-apples with
  // our in-memory copy (key order etc. matches).
  const savedParams = $derived(JSON.stringify(parseParams(selected)));

  const dirty = $derived(
    name !== (selected?.name ?? "") ||
      tags !== (selected?.tags ?? "") ||
      body !== (selected?.body ?? "") ||
      JSON.stringify(params) !== savedParams,
  );

  async function save() {
    if (!name.trim()) return;
    await scripts.save(scripts.store.selectedId, {
      name: name.trim(),
      description: "",
      tags: tags.trim(),
      body,
      params_json: JSON.stringify(params),
      danger: "safe",
    });
  }
</script>

<div class="workspace">
  <ScriptEditor bind:name bind:tags bind:body {dirty} onsave={save} />

  <section class="params">
    <div class="head">
      <span class="title">Parameters</span>
      <div class="modes">
        <button class:active={paramMode === "fill"} onclick={() => (paramMode = "fill")}>
          Fill
        </button>
        <button class:active={paramMode === "edit"} onclick={() => (paramMode = "edit")}>
          Edit
        </button>
      </div>
    </div>

    {#if paramMode === "edit"}
      <ParamBuilder bind:specs={params} />
    {:else if params.length > 0}
      <ParamForm specs={params} bind:args />
    {:else}
      <p class="hint">No parameters. Switch to <b>Edit</b> to add some.</p>
    {/if}
  </section>

  <RunPanel {body} {args} />
</div>

<style>
  .workspace {
    display: flex;
    flex-direction: column;
    height: 100%;
    font-family: system-ui, sans-serif;
  }
  .params {
    border-bottom: 1px solid #333;
    max-height: 14rem;
    overflow: auto;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.4rem 0.8rem;
    background: rgba(255, 255, 255, 0.03);
  }
  .title {
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    opacity: 0.6;
  }
  .modes button {
    padding: 0.2rem 0.7rem;
    cursor: pointer;
    border: 1px solid #444;
    background: none;
    color: inherit;
  }
  .modes button.active {
    background: rgba(120, 160, 255, 0.25);
  }
  .hint {
    padding: 0.6rem 0.8rem;
    margin: 0;
    font-size: 0.85rem;
    opacity: 0.6;
  }
</style>
