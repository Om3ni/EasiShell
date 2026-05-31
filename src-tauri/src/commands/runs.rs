//! Tauri commands for running things. Thin on purpose — they unwrap the managed state
//! and hand straight off to run_manager, which is where the actual work lives.

use tauri::ipc::Channel;
use tauri::State;

use ps_core::runner::{RunEvent, RunId};

use crate::run_manager::{self, RunArg};
use crate::state::AppState;

/// Kick off a run of `command` with the given resolved `args`. Output streams back over
/// `on_event`; the return value is just the run id so the frontend can tag what it's
/// watching.
#[tauri::command]
pub async fn start_run(
    command: String,
    args: Vec<RunArg>,
    on_event: Channel<RunEvent>,
    state: State<'_, AppState>,
) -> Result<RunId, String> {
    run_manager::start(state.inner(), command, args, on_event).await
}
