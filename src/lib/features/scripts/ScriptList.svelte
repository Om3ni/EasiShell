<script lang="ts">
  import { onMount } from "svelte";
  import * as scripts from "../../stores/scripts.svelte";

  let query = $state("");

  onMount(() => {
    scripts.refresh();
  });

  // simple name/tags contains-match. good enough until the library gets big enough to
  // want something smarter.
  const filtered = $derived(
    scripts.store.items.filter((s) => {
      const q = query.toLowerCase();
      return s.name.toLowerCase().includes(q) || s.tags.toLowerCase().includes(q);
    }),
  );

  function confirmDelete(id: number, name: string) {
    if (confirm(`Delete "${name}"? This cant be undone.`)) {
      scripts.remove(id);
    }
  }
</script>

<div class="list">
  <div class="top">
    <input placeholder="Search scripts..." bind:value={query} />
    <button class="new" onclick={() => scripts.select(null)} title="New script">+</button>
  </div>

  <ul>
    {#each filtered as s (s.id)}
      <li class:active={s.id === scripts.store.selectedId}>
        <button class="item" onclick={() => scripts.select(s.id)}>
          <span class="name">{s.name}</span>
          {#if s.tags}<span class="tags">{s.tags}</span>{/if}
        </button>
        <button class="del" title="Delete" onclick={() => confirmDelete(s.id, s.name)}>×</button>
      </li>
    {:else}
      <li class="empty">No scripts yet. Write one and hit Save.</li>
    {/each}
  </ul>
</div>

<style>
  .list {
    display: flex;
    flex-direction: column;
    height: 100%;
    font-family: system-ui, sans-serif;
    font-size: 0.9rem;
  }
  .top {
    display: flex;
    gap: 0.4rem;
    padding: 0.5rem;
    border-bottom: 1px solid #333;
  }
  .top input {
    flex: 1;
    padding: 0.35rem 0.5rem;
    min-width: 0;
  }
  .new {
    width: 2rem;
    cursor: pointer;
    font-size: 1.1rem;
    line-height: 1;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow: auto;
    flex: 1;
  }
  li {
    display: flex;
    align-items: center;
  }
  li.active {
    background: rgba(120, 160, 255, 0.18);
  }
  .item {
    flex: 1;
    text-align: left;
    background: none;
    border: none;
    padding: 0.5rem 0.6rem;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    color: inherit;
    min-width: 0;
  }
  .name {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tags {
    font-size: 0.75rem;
    opacity: 0.6;
  }
  .del {
    background: none;
    border: none;
    color: inherit;
    opacity: 0.4;
    cursor: pointer;
    font-size: 1.1rem;
    padding: 0 0.6rem;
  }
  .del:hover {
    opacity: 1;
    color: #ff7b72;
  }
  .empty {
    padding: 1rem 0.6rem;
    opacity: 0.5;
    font-size: 0.85rem;
  }
</style>
