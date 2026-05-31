//! Per-run markdown logs — the durable record of what ran and what came out.

mod naming;
mod writer;

pub use writer::{write_report, RunReport};
