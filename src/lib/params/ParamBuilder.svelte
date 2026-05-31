<script lang="ts">
  // The "define a script's parameters without touching JSON" surface. Edits the spec
  // array in place; the parent persists it into the script's params_json on save.
  import type { ParamSpec, ParamKind } from "../ipc/api";

  let { specs = $bindable([]) }: { specs?: ParamSpec[] } = $props();

  const kinds: ParamKind[] = [
    "text",
    "number",
    "dropdown",
    "checkbox",
    "path",
    "multiline",
  ];

  function add() {
    specs = [
      ...specs,
      { name: "", label: "", kind: "text", default: "", options: [], required: false },
    ];
  }

  function remove(i: number) {
    specs = specs.filter((_, idx) => idx !== i);
  }

  // dropdown options are edited as a plain comma list and split back into the array.
  function setOptions(i: number, text: string) {
    specs[i].options = text
      .split(",")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
  }
</script>

<div class="builder">
  {#each specs as s, i (i)}
    <div class="prow">
      <input class="nm" placeholder="Name" bind:value={s.name} />
      <input class="lb" placeholder="Label" bind:value={s.label} />
      <select bind:value={s.kind}>
        {#each kinds as k}<option value={k}>{k}</option>{/each}
      </select>
      <input class="df" placeholder="default" bind:value={s.default} />
      {#if s.kind === "dropdown"}
        <input
          class="op"
          placeholder="opt1, opt2"
          value={s.options.join(", ")}
          oninput={(e) => setOptions(i, e.currentTarget.value)}
        />
      {/if}
      <label class="req"><input type="checkbox" bind:checked={s.required} /> req</label>
      <button class="del" title="Remove" onclick={() => remove(i)}>×</button>
    </div>
  {/each}
  <button class="add" onclick={add}>+ Add parameter</button>
</div>

<style>
  .builder {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.5rem 0.8rem;
    font-size: 0.82rem;
  }
  .prow {
    display: flex;
    gap: 0.35rem;
    align-items: center;
    flex-wrap: wrap;
  }
  .nm,
  .lb,
  .df,
  .op {
    padding: 0.25rem 0.4rem;
    min-width: 0;
  }
  .nm {
    width: 7rem;
  }
  .lb {
    width: 8rem;
  }
  .df,
  .op {
    width: 7rem;
  }
  .req {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    opacity: 0.8;
  }
  .del {
    background: none;
    border: none;
    color: inherit;
    opacity: 0.5;
    cursor: pointer;
    font-size: 1.1rem;
  }
  .del:hover {
    opacity: 1;
    color: #ff7b72;
  }
  .add {
    align-self: start;
    cursor: pointer;
    padding: 0.25rem 0.7rem;
  }
</style>
