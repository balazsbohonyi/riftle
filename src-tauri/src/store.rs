// Phase 2: Settings persistence via tauri-plugin-store
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use serde_json::json;

// ---- Settings struct ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_opacity")]
    pub opacity: f64,

    #[serde(default)]
    pub show_path: bool,       // false by default

    #[serde(default)]
    pub autostart: bool,       // false by default

    #[serde(default)]
    pub additional_paths: Vec<String>,  // [] by default

    #[serde(default)]
    pub excluded_paths: Vec<String>,    // [] by default

    #[serde(default = "default_reindex_interval")]
    pub reindex_interval: u32,

    /// Filenames (lowercase) that are allowed through the system-directory filter.
    /// Lets useful Windows tools (notepad, regedit, …) appear even though they live
    /// in blocked paths like System32. Users can extend or trim this list.
    #[serde(default = "default_system_tool_allowlist")]
    pub system_tool_allowlist: Vec<String>,
}

fn default_hotkey() -> String { "Alt+Space".to_string() }
fn default_theme() -> String { "system".to_string() }
fn default_opacity() -> f64 { 1.0 }
fn default_reindex_interval() -> u32 { 15 }
fn default_system_tool_allowlist() -> Vec<String> {
    [
        // Text / media
        "notepad.exe", "wordpad.exe", "write.exe", "mspaint.exe",
        "wmplayer.exe",
        // Calculators / utilities
        "calc.exe", "charmap.exe", "snippingtool.exe",
        // Shell
        "cmd.exe", "powershell.exe",
        // Remote / network
        "mstsc.exe",
        // System admin
        "regedit.exe", "taskmgr.exe", "msconfig.exe", "msinfo32.exe",
        "resmon.exe", "perfmon.exe", "eventvwr.exe", "compmgmt.exe",
        "dfrgui.exe", "cleanmgr.exe", "optionalfeatures.exe",
        // Accessibility
        "magnify.exe", "osk.exe", "narrator.exe",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey(),
            theme: default_theme(),
            opacity: default_opacity(),
            show_path: false,
            autostart: false,
            additional_paths: vec![],
            excluded_paths: vec![],
            reindex_interval: default_reindex_interval(),
            system_tool_allowlist: default_system_tool_allowlist(),
        }
    }
}

// ---- Store functions ----
//
// IMPORTANT: app.store() is called with an absolute PathBuf.
// Per RESEARCH.md: Tauri's resolve_store_path uses BaseDirectory::AppData.
// An absolute PathBuf is expected to bypass this via PathBuf::join() semantics,
// but this is a LOW-confidence finding (source-inspected, not runtime-verified).
//
// If settings.json appears at %APPDATA%\riftle-launcher\settings.json
// even in portable mode, replace app.store() with direct serde_json file I/O:
//   let json_bytes = std::fs::read(store_path)?;
//   serde_json::from_slice(&json_bytes).unwrap_or_default()
// The smoke test in lib.rs setup will reveal this if absolute path is not respected.

/// Returns current settings from settings.json, or Settings::default() if not found/malformed.
/// Silent reset on malformed JSON per CONTEXT.md decision.
/// store_path must be the absolute path to settings.json (data_dir.join("settings.json")).
pub fn get_settings(app: &AppHandle, data_dir: &Path) -> Settings {
    let store_path = data_dir.join("settings.json");
    match app.store(store_path) {
        Ok(store) => {
            match store.get("settings") {
                Some(val) => serde_json::from_value(val).unwrap_or_default(),
                None => Settings::default(),
            }
        }
        Err(_) => Settings::default(),
    }
}

/// Persists the full Settings struct to settings.json.
/// Accepts the complete struct — no partial patch (per CONTEXT.md decision).
/// store_path must be the absolute path to settings.json (data_dir.join("settings.json")).
pub fn set_settings(app: &AppHandle, data_dir: &Path, settings: &Settings) {
    let store_path = data_dir.join("settings.json");
    if let Ok(store) = app.store(store_path) {
        store.set("settings", json!(settings));
        if let Err(e) = store.save() {
            eprintln!("[store] failed to persist settings: {}", e);
        }
    } else {
        eprintln!("[store] failed to open settings store");
    }
}

// ---- Unit tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_defaults() {
        let s = Settings::default();
        assert_eq!(s.hotkey, "Alt+Space");
        assert_eq!(s.theme, "system");
        assert!((s.opacity - 1.0).abs() < f64::EPSILON);
        assert!(!s.show_path);
        assert!(!s.autostart);
        assert!(s.additional_paths.is_empty());
        assert!(s.excluded_paths.is_empty());
        assert_eq!(s.reindex_interval, 15);
        assert_eq!(s.animation, "slide");
    }

    #[test]
    fn test_serde_round_trip() {
        let original = Settings::default();
        let json = serde_json::to_value(&original).unwrap();
        let deserialized: Settings = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.hotkey, original.hotkey);
        assert_eq!(deserialized.theme, original.theme);
        assert!((deserialized.opacity - original.opacity).abs() < f64::EPSILON);
        assert_eq!(deserialized.show_path, original.show_path);
        assert_eq!(deserialized.autostart, original.autostart);
        assert_eq!(deserialized.reindex_interval, original.reindex_interval);
        assert_eq!(deserialized.animation, original.animation);
    }

    #[test]
    fn test_partial_json_fills_defaults() {
        // JSON with only hotkey — all other fields should get their serde defaults
        let partial = r#"{"hotkey": "Ctrl+Alt+Space"}"#;
        let s: Settings = serde_json::from_str(partial).unwrap();
        assert_eq!(s.hotkey, "Ctrl+Alt+Space");
        assert_eq!(s.theme, "system");          // from serde default
        assert_eq!(s.reindex_interval, 15);     // from serde default
        assert!(!s.show_path);                  // bool default
        assert_eq!(s.animation, "slide");       // from serde default
    }

    #[test]
    fn test_malformed_json_falls_back_to_default() {
        let malformed = r#"not valid json at all {{{"#;
        let result: Result<Settings, _> = serde_json::from_str(malformed);
        let s = result.unwrap_or_default();
        // Should get defaults, not panic
        assert_eq!(s.hotkey, "Alt+Space");
        assert_eq!(s.reindex_interval, 15);
    }
}
