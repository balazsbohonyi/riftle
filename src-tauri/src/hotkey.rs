// Phase 9: Global hotkey registration and toggle logic

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Registers the given hotkey string as a global shortcut that toggles launcher visibility.
///
/// When the hotkey is pressed:
/// - If the launcher is visible → hide it immediately.
/// - If the launcher is hidden → center, show, focus, and emit "launcher-show" so Vue
///   replays the appear animation, clears the query, and focuses the input.
///
/// Errors are non-fatal: logged via eprintln! and the app continues without the shortcut.
pub fn register(app: &AppHandle, hotkey_str: &str) {
    let win: tauri::WebviewWindow = match app.get_webview_window("launcher") {
        Some(w) => w,
        None => {
            eprintln!("[hotkey] launcher window not found");
            return;
        }
    };

    let win_clone = win.clone();

    if let Err(e) = app.global_shortcut().on_shortcut(hotkey_str, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            if win_clone.is_visible().unwrap_or(false) {
                let _ = win_clone.hide();
            } else {
                let _ = win_clone.center();
                let _ = win_clone.show();
                let _ = win_clone.set_focus();
                let _ = win_clone.emit("launcher-show", ());
            }
        }
    }) {
        eprintln!("[hotkey] failed to register '{}': {}", hotkey_str, e);
    }
}

/// Tauri command: deregisters the currently configured hotkey and registers a new one,
/// then persists the change to settings.json.
///
/// Called by the Phase 8 Settings UI when the user rebinds the shortcut.
#[tauri::command]
pub fn update_hotkey(
    app: tauri::AppHandle,
    hotkey: String,
    data_dir: tauri::State<std::path::PathBuf>,
) -> Result<(), String> {
    let mut settings = crate::store::get_settings(&app, &data_dir);

    // Unregister old shortcut — non-fatal if it wasn't registered
    app.global_shortcut()
        .unregister(settings.hotkey.as_str())
        .unwrap_or_else(|e| eprintln!("[hotkey] unregister failed: {}", e));

    // Register new shortcut with toggle handler
    crate::hotkey::register(&app, &hotkey);

    // Persist new hotkey to settings.json
    settings.hotkey = hotkey;
    crate::store::set_settings(&app, &data_dir, &settings);

    Ok(())
}
