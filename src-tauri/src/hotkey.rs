// Phase 9: Global hotkey registration and toggle logic

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Registers the given hotkey string as a global shortcut that toggles launcher visibility.
/// Returns the hotkey that was actually registered (may be the default fallback).
///
/// When the hotkey is pressed:
/// - If the launcher is visible → hide it immediately.
/// - If the launcher is hidden → center, show, focus, and emit "launcher-show" so Vue
///   replays the appear animation, clears the query, and focuses the input.
pub fn register(app: &AppHandle, hotkey_str: &str) -> String {
    let win: tauri::WebviewWindow = match app.get_webview_window("launcher") {
        Some(w) => w,
        None => {
            eprintln!("[hotkey] launcher window not found");
            return hotkey_str.to_string();
        }
    };

    let win_clone = win.clone();

    let result = app.global_shortcut().on_shortcut(hotkey_str, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            if win_clone.is_visible().unwrap_or(false) {
                let _ = win_clone.hide();
            } else {
                let _ = win_clone.show();
                let _ = win_clone.set_focus();
                let _ = win_clone.emit("launcher-show", ());
            }
        }
    });

    match result {
        Ok(_) => hotkey_str.to_string(),
        Err(e) => {
            eprintln!("[hotkey] failed to register '{}': {}", hotkey_str, e);
            const DEFAULT: &str = "Alt+Space";
            if hotkey_str != DEFAULT {
                eprintln!("[hotkey] falling back to '{}'", DEFAULT);
                register(app, DEFAULT)
            } else {
                hotkey_str.to_string()
            }
        }
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

    // Register new shortcut — returns the hotkey that was actually registered
    // (may be the default fallback if the requested key is OS-reserved)
    let actual = crate::hotkey::register(&app, &hotkey);

    // Persist whatever was actually registered so startup uses a working hotkey
    settings.hotkey = actual.clone();
    crate::store::set_settings(&app, &data_dir, &settings);

    if actual != hotkey {
        return Err(format!("'{}' could not be registered; fell back to '{}'", hotkey, actual));
    }
    Ok(())
}
