// Riftle Launcher — Tauri v2 application entry point
// Phase 1: plugin registration scaffold; command handlers added in later phases

use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
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

            // Phase 3: Indexer — synchronous first index then background refresh
            #[cfg(desktop)]
            {
                let db_arc = Arc::clone(&app.state::<crate::db::DbState>().0);
                let is_indexing = Arc::new(AtomicBool::new(false));

                // Run full index synchronously (window is hidden — startup latency OK)
                crate::indexer::run_full_index(&db_arc, &data_dir, &settings);

                // Phase 4: Ensure system command icon exists
                if let Err(e) = crate::search::ensure_system_command_icon(&data_dir) {
                    eprintln!("[search] failed to write system_command icon: {}", e);
                }

                // Phase 4: Build nucleo search index from freshly-indexed DB
                crate::search::init_search_index(app.handle());

                // Store data_dir as managed state for reindex() command
                app.manage(data_dir.clone());

                // Store is_indexing flag as managed state
                app.manage(Arc::clone(&is_indexing));

                // Start background timer + watcher; get timer reset Sender
                let timer_tx = crate::indexer::start_background_tasks(
                    Arc::clone(&db_arc),
                    data_dir.clone(),
                    &settings,
                    Arc::clone(&is_indexing),
                );

                // Store timer Sender as managed state (reindex() command resets the timer via this)
                app.manage(Arc::new(Mutex::new(timer_tx)));
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::indexer::reindex,
            crate::search::search,
            crate::store::get_settings_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
