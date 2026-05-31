//! App-wide state we hand to Tauri to manage.
//!
//! Nothing fancy yet — a counter for handing out run ids and the two folders we write
//! into. it'll grow (db handle, the live run registry, the broker handle) as later
//! slices need it, but keeping it small now means there's less to reason about.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use rusqlite::Connection;

use ps_core::runner::RunId;

/// Where on disk we keep things. `runs_dir` holds the throwaway `.ps1` we hand to
/// PowerShell; `history_dir` holds the per-run `.md` logs that are the real record.
pub struct AppPaths {
    pub runs_dir: PathBuf,
    pub history_dir: PathBuf,
}

pub struct AppState {
    next_run_id: AtomicU32,
    pub paths: AppPaths,
    /// One serialized writer connection behind a Mutex. rusqlite's Connection isn't Sync,
    /// and a single writer keeps us clear of SQLITE_BUSY fights — DB ops are tiny and
    /// quick, so the lock is never held long enough to matter.
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn new(paths: AppPaths, db: Connection) -> Self {
        Self {
            // start at 1 so a run id of 0 always means "unset" if we ever see one
            next_run_id: AtomicU32::new(1),
            paths,
            db: Mutex::new(db),
        }
    }

    /// Hand out the next run id. Relaxed is fine — we only need uniqueness, not any
    /// ordering relationship with other memory.
    pub fn next_run_id(&self) -> RunId {
        self.next_run_id.fetch_add(1, Ordering::Relaxed)
    }
}
