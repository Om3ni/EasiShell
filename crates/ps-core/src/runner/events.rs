//! The run event stream — the contract between the runner and any consumer.
//!
//! These serde types are the single source of truth for run output. The Tauri
//! layer forwards them over an `ipc::Channel` to the frontend; later they are
//! exported to TypeScript via ts-rs so the contract cannot drift.

use serde::Serialize;

/// Monotonic identifier for a single run. Aliased so a future switch to UUIDs
/// (e.g. for concurrent tabbed runs) is a one-line change.
pub type RunId = u64;

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    /// Process exited with code 0.
    Completed,
    /// Process exited with a non-zero code.
    Failed,
    /// Killed by the user (Stop) — exit code is not meaningful.
    Stopped,
}

/// One ordered event in a run's lifecycle. Output is emitted as raw **chunks**
/// (not lines): a terminal consumer wants `\r`, partial lines, and ANSI intact,
/// and a prompt written without a trailing newline must still reach the user.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RunEvent {
    /// The process started; carries identifying metadata.
    Started {
        run_id: RunId,
        interp: String,
        interp_version: String,
        pid: u32,
    },
    /// A chunk of standard output.
    Stdout { run_id: RunId, chunk: String },
    /// A chunk of standard error.
    Stderr { run_id: RunId, chunk: String },
    /// The process ended. `code` is `None` when killed (see `RunStatus::Stopped`).
    Exit {
        run_id: RunId,
        code: Option<i32>,
        status: RunStatus,
    },
}
