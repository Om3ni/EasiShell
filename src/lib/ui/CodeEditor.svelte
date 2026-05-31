<script lang="ts">
  // CodeMirror 6 wrapped up as a plain two-way-bound editor. PowerShell highlighting
  // comes from the legacy stream mode (theres no native CM6 grammar for it, but the
  // legacy one is perfectly good). lives in ui/ because anything that needs a code box —
  // scripts now, tool snippets later — should reuse this rather than re-wiring CM.
  import { onMount, onDestroy } from "svelte";
  import { EditorView, basicSetup } from "codemirror";
  import { StreamLanguage } from "@codemirror/language";
  import { powerShell } from "@codemirror/legacy-modes/mode/powershell";
  import { oneDark } from "@codemirror/theme-one-dark";

  let { value = $bindable("") }: { value?: string } = $props();

  let host: HTMLDivElement;
  let view: EditorView | undefined;

  onMount(() => {
    view = new EditorView({
      doc: value,
      parent: host,
      extensions: [
        basicSetup,
        StreamLanguage.define(powerShell),
        oneDark,
        // mirror edits back out to the bound value. guard against echoing a change that
        // came from us in the first place, otherwise typing fights the $effect below.
        EditorView.updateListener.of((u) => {
          if (u.docChanged) {
            const next = u.state.doc.toString();
            if (next !== value) value = next;
          }
        }),
      ],
    });
  });

  // push external changes (picking a different script) into the editor — but only when
  // the text genuinely differs, so we never stomp on what the user is mid-typing.
  $effect(() => {
    const incoming = value;
    if (view && incoming !== view.state.doc.toString()) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: incoming },
      });
    }
  });

  onDestroy(() => view?.destroy());
</script>

<div class="code-editor" bind:this={host}></div>

<style>
  .code-editor {
    height: 100%;
    overflow: hidden;
  }
  .code-editor :global(.cm-editor) {
    height: 100%;
  }
  .code-editor :global(.cm-scroller) {
    overflow: auto;
  }
</style>
