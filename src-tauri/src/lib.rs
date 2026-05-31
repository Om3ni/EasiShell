//! The EasiShell app shell. Wires up state + commands and starts Tauri; the actual work
//! lives in the submodules and over in `ps-core`.

mod commands;
mod db;
mod history;
mod run_manager;
mod state;

use tauri::Manager;

use state::{AppPaths, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // everything we write goes under the OS app-data dir. resolving it here (once,
            // at startup) means the rest of the code just reads paths off state and never
            // has to think about where that is.
            let base = app.path().app_data_dir()?;
            let conn = db::open(&base.join("easishell.db"))?;
            app.manage(AppState::new(
                AppPaths {
                    runs_dir: base.join("runs"),
                    history_dir: base.join("history"),
                },
                conn,
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::runs::start_run,
            commands::scripts::list_scripts,
            commands::scripts::get_script,
            commands::scripts::create_script,
            commands::scripts::update_script,
            commands::scripts::delete_script,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
