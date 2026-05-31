// Shared state for the script library. Lives in a .svelte.ts module so the runes work
// outside a component — the sidebar and the workspace both read the same `store`, and a
// save in one shows up in the other without any event wiring.

import {
  listScripts,
  createScript,
  updateScript,
  deleteScript,
  type Script,
  type ScriptInput,
} from "../ipc/api";

export const store = $state<{
  items: Script[];
  selectedId: number | null;
}>({
  items: [],
  selectedId: null,
});

export async function refresh(): Promise<void> {
  store.items = await listScripts();
}

// null selection == the "new, unsaved" state — the editor shows blank fields.
export function select(id: number | null): void {
  store.selectedId = id;
}

// create when there's no id yet, otherwise update. either way we refresh and keep the
// just-saved script selected so the editor doesnt jump away from what you were doing.
export async function save(id: number | null, input: ScriptInput): Promise<void> {
  const saved = id === null ? await createScript(input) : await updateScript(id, input);
  await refresh();
  if (saved) store.selectedId = saved.id;
}

export async function remove(id: number): Promise<void> {
  await deleteScript(id);
  if (store.selectedId === id) store.selectedId = null;
  await refresh();
}
