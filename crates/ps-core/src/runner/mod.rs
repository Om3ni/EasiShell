//! The process runner: interpreter resolution, spawning, and output streaming.
//!
//! Sub-slices land here as the build progresses (stdin, job-object tree-kill,
//! the temp-`.ps1` + encoding preamble for the real runner). This first cut
//! proves spawn → stream → exit.

pub mod events;
pub mod interp;
mod spawn;

pub use events::{RunEvent, RunId, RunStatus};
pub use interp::{detect, Interpreter, InterpKind};
pub use spawn::{run, RunSpec};
