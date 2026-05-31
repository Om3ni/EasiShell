//! The process runner — figuring out the interpreter, launching it, and streaming
//! what it prints.
//!
//! More lands here as we go (stdin, the job-object tree-kill, the temp `.ps1` plus
//! encoding preamble the real runner needs). this first cut just proves the spine:
//! spawn → stream → exit.

pub mod events;
pub mod interp;
mod spawn;

pub use events::{RunEvent, RunId, RunStatus};
pub use interp::{detect, Interpreter, InterpKind};
pub use spawn::{run, RunSpec};
