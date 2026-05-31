<script lang="ts">
  // Renders the inputs for a script's params and hands back the resolved `-Name value`
  // args. This is the ONE form used everywhere params show up — user scripts now, the
  // built-in tools later — so it deliberately knows nothing about scripts or tools.
  import type { ParamSpec, RunArg } from "../ipc/api";

  let { specs, args = $bindable([]) }: { specs: ParamSpec[]; args?: RunArg[] } =
    $props();

  // current value per param, keyed by name. always strings; a checkbox is "true"/"false".
  let values = $state<Record<string, string>>({});

  // reset to defaults whenever the set of params changes (i.e. you picked another script).
  // editing a value afterwards doesnt re-run this, so your inputs stick.
  $effect(() => {
    const next: Record<string, string> = {};
    for (const s of specs) {
      next[s.name] = s.default ?? (s.kind === "checkbox" ? "false" : "");
    }
    values = next;
  });

  // recompute the args any time a value (or the specs) change. empty optionals and
  // unticked switches are simply left out, so the script's own param() defaults win.
  $effect(() => {
    const out: RunArg[] = [];
    for (const s of specs) {
      const v = values[s.name] ?? "";
      if (s.kind === "checkbox") {
        if (v === "true") out.push({ name: s.name, value: null });
      } else if (v !== "") {
        out.push({ name: s.name, value: v });
      }
    }
    args = out;
  });

  function setBool(name: string, on: boolean) {
    values[name] = on ? "true" : "false";
  }
</script>

{#if specs.length > 0}
  <div class="form">
    {#each specs as s (s.name)}
      <label class="row">
        <span class="lbl">{s.label || s.name}{#if s.required}<em>*</em>{/if}</span>

        {#if s.kind === "checkbox"}
          <input
            type="checkbox"
            checked={values[s.name] === "true"}
            onchange={(e) => setBool(s.name, e.currentTarget.checked)}
          />
        {:else if s.kind === "dropdown"}
          <select bind:value={values[s.name]}>
            {#each s.options as opt}
              <option value={opt}>{opt}</option>
            {/each}
          </select>
        {:else if s.kind === "multiline"}
          <textarea rows="2" bind:value={values[s.name]}></textarea>
        {:else}
          <!-- text / number / path all use a text box; PowerShell coerces by param type -->
          <input
            type="text"
            inputmode={s.kind === "number" ? "numeric" : undefined}
            placeholder={s.kind === "path" ? "path..." : ""}
            bind:value={values[s.name]}
          />
        {/if}
      </label>
    {/each}
  </div>
{/if}

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.5rem 0.8rem;
  }
  .row {
    display: grid;
    grid-template-columns: 9rem 1fr;
    align-items: center;
    gap: 0.6rem;
    font-size: 0.85rem;
  }
  .lbl em {
    color: #ff7b72;
    font-style: normal;
    margin-left: 0.15rem;
  }
  input[type="text"],
  select,
  textarea {
    padding: 0.3rem 0.45rem;
    min-width: 0;
  }
  input[type="checkbox"] {
    justify-self: start;
  }
</style>
