//! Schema migrations, kept deliberately boring.
//!
//! `user_version` is just an integer SQLite stores in the db file, so we use it as a
//! "how many migrations has this database seen" counter. each entry below is one forward
//! step; on startup we run only the ones this db hasnt run yet. add a new schema change
//! by appending a string — never editing an old one, or existing databases would skip it.

use rusqlite::Connection;

const MIGRATIONS: &[&str] = &[
    // v1 — the scripts library. params_json holds the param spec as JSON; danger drives
    // the confirm dialog later (safe / caution / destructive).
    "CREATE TABLE scripts (
        id           INTEGER PRIMARY KEY AUTOINCREMENT,
        name         TEXT NOT NULL,
        description  TEXT NOT NULL DEFAULT '',
        tags         TEXT NOT NULL DEFAULT '',
        body         TEXT NOT NULL,
        params_json  TEXT NOT NULL DEFAULT '[]',
        danger       TEXT NOT NULL DEFAULT 'safe',
        created_at   TEXT NOT NULL,
        updated_at   TEXT NOT NULL
    );",
];

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;

    for (i, sql) in MIGRATIONS.iter().enumerate() {
        let version = (i + 1) as i64;
        if current < version {
            conn.execute_batch(sql)?;
            // user_version cant be a bound parameter. the value is our own loop index,
            // not anything a user can touch, so the format! is safe here.
            conn.execute_batch(&format!("PRAGMA user_version = {version};"))?;
        }
    }
    Ok(())
}
