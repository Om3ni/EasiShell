<script lang="ts">
  // Pure editing surface. It owns no state of its own — name/tags/body are bound from
  // the parent (the workspace), and saving is the parent's job too. keeps this dumb and
  // reusable.
  import CodeEditor from "../../ui/CodeEditor.svelte";

  interface Props {
    name: string;
    tags: string;
    body: string;
    dirty: boolean;
    onsave: () => void;
  }
  let {
    name = $bindable(),
    tags = $bindable(),
    body = $bindable(),
    dirty,
    onsave,
  }: Props = $props();
</script>

<div class="editor">
  <div class="meta">
    <input class="name" placeholder="Script name" bind:value={name} />
    <input class="tags" placeholder="tags, comma separated" bind:value={tags} />
    <button onclick={onsave} disabled={!name.trim() || !dirty}>
      {dirty ? "Save" : "Saved"}
    </button>
  </div>
  <div class="code">
    <CodeEditor bind:value={body} />
  </div>
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    border-bottom: 1px solid #333;
  }
  .meta {
    display: flex;
    gap: 0.5rem;
    padding: 0.5rem;
    align-items: center;
  }
  .name {
    flex: 2;
    padding: 0.35rem 0.5rem;
    min-width: 0;
  }
  .tags {
    flex: 1;
    padding: 0.35rem 0.5rem;
    min-width: 0;
  }
  button {
    padding: 0.35rem 1rem;
    cursor: pointer;
  }
  button:disabled {
    cursor: default;
    opacity: 0.5;
  }
  .code {
    height: 240px;
    border-top: 1px solid #333;
    resize: vertical;
    overflow: hidden;
  }
</style>
