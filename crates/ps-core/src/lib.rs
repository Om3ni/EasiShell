//! ps-core — the shared PowerShell execution core for PowerShell Studio.
//!
//! This crate owns the hard, reusable parts so they are written exactly once and
//! called from both sides of the privilege boundary:
//!
//! - `runner`   — spawn an interpreter, stream stdout/stderr, accept stdin, kill the tree
//! - `tools`    — built-in guided actions + shared path/danger guardrails
//! - `paramspec`— the parameter-spec model + a `.ps1` `param()` parser
//! - `types`    — serde structs that form the app's IPC contract (exported to TS via ts-rs)
//!
//! Consumers: the Tauri app (`src-tauri`, direct/in-process backend) and the
//! elevated broker (`crates/psbroker`, added in Phase 4). Modules are added as
//! each vertical slice needs them — the runner lands first.
