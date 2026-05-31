//! Tauri commands for the script library. Each one just grabs the db lock and hands off
//! to the repo — the lock is held only for the length of a single quick query, never
//! across an await, so it's never a contention problem.

use tauri::State;

use crate::db::scripts_repo::{self, Script, ScriptInput};
use crate::state::AppState;

// little helper so every command isnt repeating the same lock-and-stringify dance.
fn with_db<T>(
    state: &AppState,
    f: impl FnOnce(&rusqlite::Connection) -> rusqlite::Result<T>,
) -> Result<T, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    f(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_scripts(state: State<'_, AppState>) -> Result<Vec<Script>, String> {
    with_db(&state, scripts_repo::list)
}

#[tauri::command]
pub fn get_script(id: i64, state: State<'_, AppState>) -> Result<Option<Script>, String> {
    with_db(&state, |c| scripts_repo::get(c, id))
}

#[tauri::command]
pub fn create_script(input: ScriptInput, state: State<'_, AppState>) -> Result<Script, String> {
    with_db(&state, |c| scripts_repo::create(c, &input))
}

#[tauri::command]
pub fn update_script(
    id: i64,
    input: ScriptInput,
    state: State<'_, AppState>,
) -> Result<Option<Script>, String> {
    with_db(&state, |c| scripts_repo::update(c, id, &input))
}

#[tauri::command]
pub fn delete_script(id: i64, state: State<'_, AppState>) -> Result<bool, String> {
    with_db(&state, |c| scripts_repo::delete(c, id))
}
