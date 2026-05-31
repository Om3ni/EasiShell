//! SQLite persistence. `open` is the entry point; repos own their own tables.

mod connection;
pub(crate) mod migrations;
pub mod scripts_repo;

pub use connection::open;
