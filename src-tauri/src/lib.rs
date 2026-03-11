// Riftle Launcher — Tauri v2 application entry point
// Phase 1: plugin registration scaffold; command handlers added in later phases

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

/// Tracks whether the settings window has been centered this session.
/// First open: center then show. Subsequent opens: show at current position.
struct SettingsCentered(AtomicBool);
struct SettingsCloseBehavior(AtomicBool);

fn show_launcher_window(app: &tauri::AppHandle) {
    let Some(win) = app.get_webview_window("launcher") else {
        eprintln!("[tray] launcher window not found");
        return;
    };

    let _ = win.show();
    let _ = win.set_focus();
    let _ = win.emit("launcher-show", ());
}

fn toggle_launcher_window(app: &tauri::AppHandle) {
    let Some(win) = app.get_webview_window("launcher") else {
        eprintln!("[tray] launcher window not found");
        return;
    };

    if win.is_visible().unwrap_or(false) {
        let _ = win.hide();
    } else {
        show_launcher_window(app);
    }
}

fn show_settings_window(app: &tauri::AppHandle) -> Result<(), String> {
    let centered = app.state::<SettingsCentered>();
    let win = app
        .get_webview_window("settings")
        .ok_or_else(|| "settings window not found".to_string())?;
    if !centered.0.swap(true, Ordering::Relaxed) {
        win.center().map_err(|e| e.to_string())?;
    }
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

fn set_restore_launcher_on_settings_close(app: &tauri::AppHandle, restore: bool) {
    let state = app.state::<SettingsCloseBehavior>();
    state.0.store(restore, Ordering::Relaxed);
}

mod db;           // Phase 2: SQLite database layer
mod store;        // Phase 2: Settings persistence via tauri-plugin-store
mod paths;        // Phase 2: Portable-aware data directory resolution
mod hotkey;       // Phase 9: Global hotkey registration
mod indexer;      // Phase 3: Windows application indexer
mod search;       // Phase 4: Nucleo fuzzy search engine
mod commands;     // Phase 6: Launch commands (launch, launch_elevated)
mod system_commands; // Phase 6: System commands (lock, shutdown, restart, sleep)

#[tauri::command]
fn open_settings_window(
    app: tauri::AppHandle,
) -> Result<(), String> {
    set_restore_launcher_on_settings_close(&app, true);
    show_settings_window(&app)
}

#[tauri::command]
fn consume_restore_launcher_on_settings_close(app: tauri::AppHandle) -> bool {
    let state = app.state::<SettingsCloseBehavior>();
    state.0.swap(true, Ordering::Relaxed)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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

                // Spawn COM worker thread — owns the COM apartment for all .lnk resolution.
                // Must be created before run_full_index and start_background_tasks.
                let com_tx = crate::indexer::spawn_com_worker();

                // Run full index synchronously (window is hidden — startup latency OK)
                crate::indexer::run_full_index(&db_arc, &data_dir, &settings, &com_tx);

                // Phase 4: Ensure system command icon exists
                if let Err(e) = crate::search::ensure_system_command_icon(&data_dir) {
                    eprintln!("[search] failed to write system_command icon: {}", e);
                }

                // Phase 4: Build nucleo search index from freshly-indexed DB
                crate::search::init_search_index(app.handle());

                // Phase 9: Register global hotkey (toggle launcher visibility)
                // register() returns the actually-registered hotkey (may fall back to Alt+Space).
                // Persist the fallback so next startup doesn't try the broken key again.
                let actual_hotkey = crate::hotkey::register(app.handle(), &settings.hotkey);
                if actual_hotkey != settings.hotkey {
                    let mut updated = settings.clone();
                    updated.hotkey = actual_hotkey;
                    crate::store::set_settings(app.handle(), &data_dir, &updated);
                }

                // Store data_dir as managed state for reindex() command
                app.manage(data_dir.clone());

                // Store is_indexing flag as managed state
                app.manage(Arc::clone(&is_indexing));

                // Store COM worker SyncSender as managed state for reindex() command
                app.manage(Arc::new(com_tx.clone()));

                // Start background timer + watcher; get timer reset Sender
                let timer_tx = crate::indexer::start_background_tasks(
                    Arc::clone(&db_arc),
                    data_dir.clone(),
                    &settings,
                    Arc::clone(&is_indexing),
                    com_tx,
                );

                // Store timer Sender as managed state (reindex() command resets the timer via this)
                app.manage(Arc::new(Mutex::new(timer_tx)));
            }

            // Settings window: centered on first open, position remembered within session
            app.manage(SettingsCentered(AtomicBool::new(false)));
            app.manage(SettingsCloseBehavior(AtomicBool::new(true)));

            // Phase 09.1: Native system tray icon + native context menu.
            #[cfg(desktop)]
            {
                let settings_item = MenuItem::with_id(app, "settings", "Settings", true, Option::<&str>::None)?;
                let quit_item = MenuItem::with_id(app, "quit", "Quit Launcher", true, Option::<&str>::None)?;
                let tray_menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

                let pending_left_click = Arc::new(Mutex::new(None::<Instant>));
                let pending_left_click_for_handler = Arc::clone(&pending_left_click);
                let suppress_left_click_until = Arc::new(Mutex::new(None::<Instant>));
                let suppress_left_click_until_for_handler = Arc::clone(&suppress_left_click_until);
                let app_handle = app.handle().clone();
                let icon = app.default_window_icon().cloned();

                if let Some(icon) = icon {
                    if let Err(e) = TrayIconBuilder::with_id("main-tray")
                        .icon(icon)
                        .tooltip("Riftle Launcher")
                        .menu(&tray_menu)
                        .show_menu_on_left_click(false)
                        .on_menu_event(move |app, event| {
                            let id = event.id.as_ref();
                            if id == "settings" {
                                let launcher_visible = app
                                    .get_webview_window("launcher")
                                    .and_then(|win| win.is_visible().ok())
                                    .unwrap_or(false);
                                set_restore_launcher_on_settings_close(app, launcher_visible);
                                if let Err(e) = show_settings_window(app) {
                                    eprintln!("[tray] failed to open settings: {}", e);
                                }
                            } else if id == "quit" {
                                crate::commands::quit_app(app.clone());
                            }
                        })
                        .on_tray_icon_event(move |_tray, event| match event {
                            TrayIconEvent::DoubleClick { button, .. } if button == MouseButton::Left => {
                                *pending_left_click_for_handler.lock().unwrap() = None;
                                *suppress_left_click_until_for_handler.lock().unwrap() =
                                    Some(Instant::now() + Duration::from_millis(350));
                                show_launcher_window(&app_handle);
                            }
                            TrayIconEvent::Click { button, button_state, .. }
                                if button == MouseButton::Left && button_state == MouseButtonState::Up =>
                            {
                                let stamp = Instant::now();
                                {
                                    let mut suppress = suppress_left_click_until_for_handler.lock().unwrap();
                                    if let Some(until) = *suppress {
                                        if stamp <= until {
                                            return;
                                        }
                                        *suppress = None;
                                    }
                                }
                                *pending_left_click_for_handler.lock().unwrap() = Some(stamp);

                                let pending = Arc::clone(&pending_left_click_for_handler);
                                let app_for_toggle = app_handle.clone();
                                std::thread::spawn(move || {
                                    std::thread::sleep(Duration::from_millis(280));
                                    let mut guard = pending.lock().unwrap();
                                    if guard.map(|s| s == stamp).unwrap_or(false) {
                                        *guard = None;
                                        drop(guard);
                                        toggle_launcher_window(&app_for_toggle);
                                    }
                                });
                            }
                            _ => {}
                        })
                        .build(app)
                    {
                        eprintln!("[tray] failed to create tray icon: {}", e);
                    }
                } else {
                    eprintln!("[tray] failed to create tray icon: default window icon missing");
                }
            }

            // Make launcher window fully invisible to DWM: no border, no rounding, no shadow.
            // The window is a transparent canvas — CSS handles all visuals (border-radius, shadow).
            // Window is intentionally larger than launcher content to give room for CSS shadow.
            #[cfg(target_os = "windows")]
            if let Some(launcher) = app.get_webview_window("launcher") {
                use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE};
                use windows::Win32::Foundation::HWND;
                const DWMWA_COLOR_NONE: u32 = 0xFFFFFFFE; // no accent border
                const DWMWCP_DONOTROUND: u32 = 1;          // no DWM rounding — CSS owns border-radius
                let hwnd = HWND(launcher.hwnd().unwrap().0 as *mut std::ffi::c_void);
                unsafe {
                    let _ = DwmSetWindowAttribute(
                        hwnd,
                        DWMWA_BORDER_COLOR,
                        &DWMWA_COLOR_NONE as *const u32 as *const _,
                        std::mem::size_of::<u32>() as u32,
                    );
                    let _ = DwmSetWindowAttribute(
                        hwnd,
                        DWMWA_WINDOW_CORNER_PREFERENCE,
                        &DWMWCP_DONOTROUND as *const u32 as *const _,
                        std::mem::size_of::<u32>() as u32,
                    );
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::indexer::reindex,
            crate::search::search,
            crate::store::get_settings_cmd,
            crate::store::set_settings_cmd,
            crate::commands::launch,
            crate::commands::launch_elevated,
            crate::commands::get_icon_bytes,
            crate::system_commands::run_system_command,
            crate::hotkey::update_hotkey,
            crate::commands::quit_app,   // Phase 7: context menu quit action
            open_settings_window,        // Phase 8: open settings window
            consume_restore_launcher_on_settings_close,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
