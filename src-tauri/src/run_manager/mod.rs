//! Owns a run from start to finish.
//!
//! ps-core knows how to spawn a process and spit out events; it deliberately doesn't
//! know about the frontend, the filesystem layout, or history. this module is where all
//! that gets stitched together, and it's the *single* place a run's event stream is
//! handled — so when the elevated broker lands later, it can feed events into this same
//! path instead of growing a second copy of the logic.
//!
//! The shape: spawn the process on one task, then sit on the receiving end doing three
//! things at once — forward each event to the frontend channel, pile the output into a
//! buffer, and once the run ends write the history `.md` and delete the temp script.

use std::time::Instant;

use tauri::ipc::Channel;
use tokio::sync::mpsc;

use ps_core::runner::{self, RunEvent, RunId, RunSpec, RunStatus};

use crate::history::{self, RunReport};
use crate::state::AppState;

/// Start a run. Returns the run id right away; output arrives later over `channel`.
pub async fn start(
    state: &AppState,
    command: String,
    channel: Channel<RunEvent>,
) -> Result<RunId, String> {
    let run_id = state.next_run_id();
    let interp = runner::detect();

    // dump the typed command into its own temp .ps1. -File needs a real file on disk,
    // and going through a file keeps us clear of command-line quoting entirely.
    std::fs::create_dir_all(&state.paths.runs_dir).map_err(|e| e.to_string())?;
    let script_path = state.paths.runs_dir.join(format!("{run_id}.ps1"));
    std::fs::write(&script_path, &command).map_err(|e| e.to_string())?;

    let spec = RunSpec {
        run_id,
        program: interp.path.clone(),
        args: interp.file_invocation_args(&script_path),
        interp_label: interp.label().to_string(),
    };

    // the process itself runs on its own task and reports back over this channel.
    let (tx, rx) = mpsc::channel::<RunEvent>(256);
    tokio::spawn(runner::run(spec, tx));

    // hand everything the persistence step needs over to the consumer task. we snapshot
    // start time here so the duration covers spawn + run, which is what a user expects.
    let history_dir = state.paths.history_dir.clone();
    let interp_label = interp.label().to_string();
    let started = local_timestamp();
    let started_at = Instant::now();

    tokio::spawn(consume(
        rx,
        channel,
        command,
        script_path,
        history_dir,
        interp_label,
        started,
        started_at,
    ));

    Ok(run_id)
}

/// The middle-man loop. Drains the run's events, forwards them, remembers the output,
/// and writes the record once it's over.
#[allow(clippy::too_many_arguments)]
async fn consume(
    mut rx: mpsc::Receiver<RunEvent>,
    channel: Channel<RunEvent>,
    command: String,
    script_path: std::path::PathBuf,
    history_dir: std::path::PathBuf,
    interp_label: String,
    started: String,
    started_at: Instant,
) {
    let mut output = String::new();
    let mut exit_code = None;
    let mut status = RunStatus::Completed;

    while let Some(ev) = rx.recv().await {
        // accumulate before we forward — `send` takes the event by value.
        match &ev {
            RunEvent::Stdout { chunk, .. } | RunEvent::Stderr { chunk, .. } => {
                output.push_str(chunk);
            }
            RunEvent::Exit { code, status: st, .. } => {
                exit_code = *code;
                status = *st;
            }
            RunEvent::Started { .. } => {}
        }

        // forward to the UI. if the webview went away we just quietly stop forwarding,
        // but we keep draining so the history file still gets written in full.
        let _ = channel.send(ev);
    }

    let report = RunReport {
        command: &command,
        interp: &interp_label,
        started: &started,
        finished: &local_timestamp(),
        duration_ms: started_at.elapsed().as_millis(),
        exit_code,
        status: status_label(status),
        output: &output,
    };
    let _ = history::write_report(&history_dir, &report);

    // the canonical command lives in the .md now, so the temp script can go.
    let _ = std::fs::remove_file(&script_path);
}

fn local_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn status_label(s: RunStatus) -> &'static str {
    match s {
        RunStatus::Completed => "completed",
        RunStatus::Failed => "failed",
        RunStatus::Stopped => "stopped",
    }
}
