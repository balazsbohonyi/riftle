// Phase 9: Global hotkey registration and toggle logic

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

fn hotkey_capture_active(app: &AppHandle) -> bool {
    app.try_state::<crate::HotkeyCaptureActive>()
        .map(|state| state.0.load(std::sync::atomic::Ordering::Relaxed))
        .unwrap_or(false)
}

fn format_hotkey_register_error(hotkey: &str, error: impl std::fmt::Display) -> String {
    let raw = error.to_string();
    if raw.contains("already registered") {
        return format!("'{}' could not be registered because it is already in use", hotkey);
    }

    format!("'{}' could not be registered: {}", hotkey, raw)
}

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
            if hotkey_capture_active(_app) {
                return;
            }
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

/// Tauri command: registers the new hotkey first; if that succeeds, unregisters the old one.
/// This guarantees there is no window where the user has no active hotkey.
///
/// On new-key registration failure: returns Err immediately — old hotkey remains active.
/// On old-key unregister failure after successful new registration: logs, does NOT roll back.
///
/// Called by the Phase 8 Settings UI when the user rebinds the shortcut.
#[tauri::command]
pub fn update_hotkey(
    app: tauri::AppHandle,
    hotkey: String,
    data_dir: tauri::State<std::path::PathBuf>,
) -> Result<(), String> {
    let mut settings = crate::store::get_settings(&app, &data_dir);
    let old_hotkey = settings.hotkey.clone();

    // Step 1: Get the launcher window (required to build the handler closure)
    let win = app
        .get_webview_window("launcher")
        .ok_or_else(|| "launcher window not found".to_string())?;

    let win_clone = win.clone();

    // Step 2: Register the new hotkey FIRST — same handler body as register()
    app.global_shortcut()
        .on_shortcut(hotkey.as_str(), move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                if hotkey_capture_active(_app) {
                    return;
                }
                if win_clone.is_visible().unwrap_or(false) {
                    let _ = win_clone.hide();
                } else {
                    let _ = win_clone.show();
                    let _ = win_clone.set_focus();
                    let _ = win_clone.emit("launcher-show", ());
                }
            }
        })
        .map_err(|e| format_hotkey_register_error(&hotkey, e))?;

    // Step 3: New key registered — now safely unregister old. Non-fatal on failure.
    app.global_shortcut()
        .unregister(old_hotkey.as_str())
        .unwrap_or_else(|e| eprintln!("[hotkey] unregister old failed: {}", e));

    // Step 4: Persist the newly registered hotkey
    settings.hotkey = hotkey;
    crate::store::set_settings(&app, &data_dir, &settings);

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_update_hotkey_err_format() {
        // Documents the error string format callers (Settings.vue) should expect
        let hotkey = "Ctrl+F13";
        let raw_err = "already registered";
        let formatted = super::format_hotkey_register_error(hotkey, raw_err);
        assert!(formatted.contains("could not be registered"));
        assert!(formatted.contains(hotkey));
    }

    #[test]
    fn test_update_hotkey_err_format_sanitizes_registered_message() {
        let formatted = super::format_hotkey_register_error(
            "Alt+Space",
            "HotKey already registered: HotKey { mods: Modifiers(ALT), key: Space, id: 65598 }",
        );
        assert_eq!(
            formatted,
            "'Alt+Space' could not be registered because it is already in use"
        );
    }
}
