// Typed front door to the Rust side. One small wrapper per command so the rest of the
// app never touches `invoke` strings or Channel plumbing directly.

import { invoke, Channel } from "@tauri-apps/api/core";
import type { RunEvent } from "./bindings/RunEvent";

export type { RunEvent };

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
