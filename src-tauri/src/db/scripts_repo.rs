//! Reading and writing rows in the `scripts` table.
//!
//! This is the only place that knows the table's columns — commands and the rest of the
//! app deal in `Script`/`ScriptInput`, never raw SQL. params_json stays an opaque string
//! at this layer; parsing it into real param specs is the param-builder slice's job.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A stored script, exactly as it lives in the table.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/ipc/bindings/")]
pub struct Script {
    // i64 to match SQLite's rowid, but tell ts-rs to emit `number`: it crosses IPC as a
    // JSON number anyway, and real id values never get anywhere near 2^53.
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub description: String,
    pub tags: String,
    pub body: String,
    pub params_json: String,
    pub danger: String,
    pub created_at: String,
    pub updated_at: String,
}

/// What the caller hands us to create or edit one. No id or timestamps — those are ours
/// to assign, not the frontend's.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/ipc/bindings/")]
pub struct ScriptInput {
    pub name: String,
    pub description: String,
    pub tags: String,
    pub body: String,
    pub params_json: String,
    pub danger: String,
}

const COLUMNS: &str =
    "id, name, description, tags, body, params_json, danger, created_at, updated_at";

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Script>> {
    // most-recently-touched first — thats the order a library view wants by default.
    let sql = format!("SELECT {COLUMNS} FROM scripts ORDER BY updated_at DESC, id DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_script)?;
    rows.collect()
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<Script>> {
    let sql = format!("SELECT {COLUMNS} FROM scripts WHERE id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query_map([id], row_to_script)?;
    match rows.next() {
        Some(r) => Ok(Some(r?)),
        None => Ok(None),
    }
}

pub fn create(conn: &Connection, input: &ScriptInput) -> rusqlite::Result<Script> {
    let ts = now();
    conn.execute(
        "INSERT INTO scripts (name, description, tags, body, params_json, danger, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![input.name, input.description, input.tags, input.body, input.params_json, input.danger, ts],
    )?;
    // a fresh row always exists right after insert, so the unwrap is honest here.
    let id = conn.last_insert_rowid();
    get(conn, id).map(|s| s.expect("row we just inserted"))
}

pub fn update(conn: &Connection, id: i64, input: &ScriptInput) -> rusqlite::Result<Option<Script>> {
    let ts = now();
    let changed = conn.execute(
        "UPDATE scripts
            SET name=?2, description=?3, tags=?4, body=?5, params_json=?6, danger=?7, updated_at=?8
          WHERE id=?1",
        params![id, input.name, input.description, input.tags, input.body, input.params_json, input.danger, ts],
    )?;
    if changed == 0 {
        return Ok(None); // no such id
    }
    get(conn, id)
}

pub fn delete(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let removed = conn.execute("DELETE FROM scripts WHERE id = ?1", [id])?;
    Ok(removed > 0)
}

fn row_to_script(row: &rusqlite::Row) -> rusqlite::Result<Script> {
    Ok(Script {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        tags: row.get(3)?,
        body: row.get(4)?,
        params_json: row.get(5)?,
        danger: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::migrate(&conn).unwrap();
        conn
    }

    fn sample() -> ScriptInput {
        ScriptInput {
            name: "Hello".into(),
            description: "says hi".into(),
            tags: "util".into(),
            body: "Write-Output hi".into(),
            params_json: "[]".into(),
            danger: "safe".into(),
        }
    }

    #[test]
    fn full_crud_round_trip() {
        let conn = fresh_db();
        assert!(list(&conn).unwrap().is_empty(), "should start empty");

        let created = create(&conn, &sample()).unwrap();
        assert!(created.id > 0);
        assert_eq!(created.name, "Hello");
        // created_at and updated_at get the same stamp on insert
        assert_eq!(created.created_at, created.updated_at);

        assert_eq!(list(&conn).unwrap().len(), 1);
        assert_eq!(get(&conn, created.id).unwrap().unwrap().body, "Write-Output hi");

        let edited = ScriptInput { name: "Hello (renamed)".into(), ..sample() };
        let updated = update(&conn, created.id, &edited).unwrap().unwrap();
        assert_eq!(updated.name, "Hello (renamed)");

        // editing a missing id is a clean None, not an error
        assert!(update(&conn, 9999, &edited).unwrap().is_none());

        assert!(delete(&conn, created.id).unwrap());
        assert!(!delete(&conn, created.id).unwrap(), "second delete finds nothing");
        assert!(list(&conn).unwrap().is_empty());
    }
}
