//! ps-core — the shared PowerShell execution core for EasiShell.
//!
//! The whole point of this crate is to write the hard, reusable stuff exactly once and
//! call it from both sides of the privilege line (the normal in-process backend and the
//! elevated broker), so we're never maintaining two copies of the tricky bits:
//!
//! - `runner`    — launch an interpreter, stream stdout/stderr, take stdin, kill the tree
//! - `tools`     — the built-in guided actions, plus the shared path/danger guardrails
//! - `paramspec` — the parameter-spec model and a parser for a script's `param()` block
//! - `types`     — the serde structs that make up the app's IPC contract (TS via ts-rs)
//!
//! Used by the Tauri app over in `src-tauri` and, from Phase 4, the elevated broker in
//! `crates/psbroker`. modules show up here as each slice actually needs them rather than
//! all at once — runner came first since it's the riskiest piece.

pub mod runner;
