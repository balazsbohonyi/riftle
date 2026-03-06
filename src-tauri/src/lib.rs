// Riftle Launcher — Tauri v2 application entry point
// Phase 1: plugin registration scaffold; command handlers added in later phases

use std::sync::{Arc, Mutex};
use tauri::Manager;

mod db;           // Phase 2: SQLite database layer
mod store;        // Phase 2: Settings persistence via tauri-plugin-store
mod paths;        // Phase 2: Portable-aware data directory resolution
mod hotkey;       // Phase 9: Global hotkey registration
mod indexer;      // Phase 3: Windows application indexer
mod search;       // Phase 4: Nucleo fuzzy search engine
mod commands;     // Phase 6: Launch commands (launch, launch_elevated)
mod system_commands; // Phase 6: System commands (lock, shutdown, restart, sleep)

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new().build(),
                )?;
                app.handle().plugin(tauri_plugin_autostart::init(
                    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                    None,
                ))?;
            }

            // Phase 2: Resolve data directory (portable or installed mode)
            let data_dir = crate::paths::data_dir(app.handle());

            // Phase 2: Initialize SQLite database and register as managed state
            let db_path = data_dir.join("launcher.db");
            let conn = crate::db::init_db(&db_path)
                .expect("failed to initialize database");
            app.manage(crate::db::DbState(Arc::new(Mutex::new(conn))));

            // Phase 2: Initialize settings store — loads existing settings or writes defaults on first run
            let settings = crate::store::get_settings(app.handle(), &data_dir);
            crate::store::set_settings(app.handle(), &data_dir, &settings);
            // get_settings returns defaults if no file exists; set_settings persists them.
            // This guarantees settings.json exists after setup (DATA-04).
            // Phase 8 will expose settings to the frontend via Tauri commands.
            let _ = settings;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
