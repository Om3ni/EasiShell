//! The run event stream — basically the contract between the runner and whoever's
//! watching it.
//!
//! These serde types are the one source of truth for run output. the Tauri layer
//! just forwards them over an ipc::Channel to the frontend, and down the line we
//! generate the matching TypeScript from these with ts-rs so the two sides cant
//! drift apart.

use serde::Serialize;
use ts_rs::TS;

/// Id for a single run. plain u32 alias — kept narrow on purpose so it crosses the
/// IPC boundary as a normal JS number (a u64 would come out the other side as bigint,
/// which is just friction here). if we ever want UUIDs for concurrent runs, this is
/// the one line that changes.
pub type RunId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/ipc/bindings/")]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Completed,
    Failed,
    /// user hit Stop and we killed it — so the exit code here doesnt mean anything.
    Stopped,
}

/// One ordered event in a run's life. Important bit: output goes out as raw **chunks**,
/// not lines. a terminal wants the `\r`s, the half-lines and the ANSI left alone, and a
/// prompt printed without a trailing newline still has to reach the user — splitting on
/// newlines would break all of that.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/ipc/bindings/")]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RunEvent {
    Started {
        run_id: RunId,
        interp: String,
        interp_version: String,
        pid: u32,
    },
    Stdout { run_id: RunId, chunk: String },
    Stderr { run_id: RunId, chunk: String },
    /// `code` is None when the process was killed rather than exiting on its own.
    Exit {
        run_id: RunId,
        code: Option<i32>,
        status: RunStatus,
    },
}
