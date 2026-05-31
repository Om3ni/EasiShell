//! Opening the database the one true way.
//!
//! Everything that needs the db goes through here so the pragmas and migrations are
//! always applied the same way — no "oh this connection didnt have WAL on" surprises.

use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let conn = Connection::open(path)?;

    // WAL lets a read (listing scripts, the history view) happen without blocking on a
    // write thats in flight. journal_mode hands back the new mode as a row, so it goes
    // through query_row rather than pragma_update.
    conn.query_row("PRAGMA journal_mode=WAL", [], |_| Ok(()))?;
    // instead of failing the moment the db is momentarily locked, wait a little.
    conn.busy_timeout(Duration::from_secs(5))?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    super::migrations::migrate(&conn)?;
    Ok(conn)
}
