// Typed front door to the Rust side. One small wrapper per command so the rest of the
// app never touches `invoke` strings or Channel plumbing directly.

import { invoke, Channel } from "@tauri-apps/api/core";
import type { RunEvent } from "./bindings/RunEvent";
import type { Script } from "./bindings/Script";
import type { ScriptInput } from "./bindings/ScriptInput";

export type { RunEvent, Script, ScriptInput };

// --- running -------------------------------------------------------------------

// Start a run. `onEvent` fires for every event as it streams in (started / stdout /
// stderr / exit); the promise resolves with the run id as soon as the backend has
// things moving, well before the run actually finishes.
export async function startRun(
  command: string,
  onEvent: (ev: RunEvent) => void,
): Promise<number> {
  const channel = new Channel<RunEvent>();
  channel.onmessage = onEvent;
  return await invoke<number>("start_run", { command, onEvent: channel });
}

// --- the script library --------------------------------------------------------

export async function listScripts(): Promise<Script[]> {
  return await invoke<Script[]>("list_scripts");
}

export async function getScript(id: number): Promise<Script | null> {
  return await invoke<Script | null>("get_script", { id });
}

export async function createScript(input: ScriptInput): Promise<Script> {
  return await invoke<Script>("create_script", { input });
}

// returns null if the id no longer exists (e.g. deleted in another window)
export async function updateScript(id: number, input: ScriptInput): Promise<Script | null> {
  return await invoke<Script | null>("update_script", { id, input });
}

export async function deleteScript(id: number): Promise<boolean> {
  return await invoke<boolean>("delete_script", { id });
}
