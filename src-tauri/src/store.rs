// Phase 2: Settings persistence via tauri-plugin-store
use crate::warnings::BackendWarning;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

// ---- Settings struct ----

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,

    #[serde(default = "default_theme")]
    pub theme: String,


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

    #[serde(default = "default_animation")]
    pub animation: String,

    /// Filenames (lowercase) that are allowed through the system-directory filter.
    /// Lets useful Windows tools (notepad, regedit, …) appear even though they live
    /// in blocked paths like System32. Users can extend or trim this list.
    #[serde(default = "default_system_tool_allowlist")]
    pub system_tool_allowlist: Vec<String>,
}

fn default_hotkey() -> String { "Ctrl+Alt+Space".to_string() }
fn default_theme() -> String { "system".to_string() }

fn default_reindex_interval() -> u32 { 15 }
fn default_animation() -> String { "slide".to_string() }
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

            show_path: false,
            autostart: false,
            additional_paths: vec![],
            excluded_paths: vec![],
            reindex_interval: default_reindex_interval(),
            animation: default_animation(),
            system_tool_allowlist: default_system_tool_allowlist(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SettingsLoadOutcome {
    Loaded(Settings),
    Missing(Settings),
    RecoveredWithDefaults {
        settings: Settings,
        warning: BackendWarning,
    },
    FatalBackupFailure {
        error: String,
    },
}

fn settings_path(data_dir: &Path) -> PathBuf {
    data_dir.join("settings.json")
}

fn backup_path(store_path: &Path) -> PathBuf {
    store_path.with_file_name("settings.json.bak")
}

fn load_settings_from_file(store_path: &Path) -> Result<Settings, String> {
    let raw = fs::read_to_string(store_path)
        .map_err(|err| format!("failed to read {}: {}", store_path.display(), err))?;
    let root: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse {}: {}", store_path.display(), err))?;
    let settings = root
        .get("settings")
        .cloned()
        .ok_or_else(|| format!("missing settings payload in {}", store_path.display()))?;

    serde_json::from_value(settings)
        .map_err(|err| format!("failed to deserialize settings from {}: {}", store_path.display(), err))
}

fn backup_file_with_overwrite(source_path: &Path, backup_path: &Path) -> std::io::Result<()> {
    if backup_path.exists() {
        if backup_path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("backup path {} is a directory", backup_path.display()),
            ));
        }
        fs::remove_file(backup_path)?;
    }

    fs::copy(source_path, backup_path)?;
    Ok(())
}

pub fn load_settings_outcome(data_dir: &Path) -> SettingsLoadOutcome {
    let store_path = settings_path(data_dir);
    let defaults = Settings::default();

    if !store_path.exists() {
        return SettingsLoadOutcome::Missing(defaults);
    }

    match load_settings_from_file(&store_path) {
        Ok(settings) => SettingsLoadOutcome::Loaded(settings),
        Err(load_error) => {
            let backup_path = backup_path(&store_path);
            if let Err(backup_error) = backup_file_with_overwrite(&store_path, &backup_path) {
                return SettingsLoadOutcome::FatalBackupFailure {
                    error: format!(
                        "failed to create {} before recovering {}: {} ({})",
                        backup_path.display(),
                        store_path.display(),
                        backup_error,
                        load_error
                    ),
                };
            }

            SettingsLoadOutcome::RecoveredWithDefaults {
                settings: defaults,
                warning: BackendWarning {
                    kind: "settings-reset".to_string(),
                    title: "Settings were reset".to_string(),
                    message:
                        "Riftle could not read your existing settings and restored defaults."
                            .to_string(),
                    backup_path: Some(backup_path.to_string_lossy().into_owned()),
                },
            }
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

/// Returns current settings for non-startup call sites.
/// Startup code should use load_settings_outcome() so it can branch on recovery state.
/// store_path must be the absolute path to settings.json (data_dir.join("settings.json")).
pub fn get_settings(_app: &AppHandle, data_dir: &Path) -> Settings {
    match load_settings_outcome(data_dir) {
        SettingsLoadOutcome::Loaded(settings)
        | SettingsLoadOutcome::Missing(settings)
        | SettingsLoadOutcome::RecoveredWithDefaults { settings, .. } => settings,
        SettingsLoadOutcome::FatalBackupFailure { error } => {
            eprintln!("[store] {}", error);
            Settings::default()
        }
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

// ---- Tauri commands ----

#[tauri::command]
pub fn get_settings_cmd(
    _app: tauri::AppHandle,
    data_dir: tauri::State<std::path::PathBuf>,
) -> serde_json::Value {
    let settings = match load_settings_outcome(&data_dir) {
        SettingsLoadOutcome::Loaded(settings)
        | SettingsLoadOutcome::Missing(settings)
        | SettingsLoadOutcome::RecoveredWithDefaults { settings, .. } => settings,
        SettingsLoadOutcome::FatalBackupFailure { error } => {
            eprintln!("[store] {}", error);
            Settings::default()
        }
    };
    let is_portable = data_dir.ends_with("data") &&
        data_dir.parent().map(|p| p.join("riftle-launcher.portable").exists()).unwrap_or(false);
    let build_profile = if cfg!(debug_assertions) { "debug" } else { "release" };
    let can_autostart = !cfg!(debug_assertions) && !is_portable;
    serde_json::json!({
        "hotkey": settings.hotkey,
        "theme": settings.theme,

        "show_path": settings.show_path,
        "autostart": settings.autostart,
        "additional_paths": settings.additional_paths,
        "excluded_paths": settings.excluded_paths,
        "reindex_interval": settings.reindex_interval,
        "animation": settings.animation,
        "system_tool_allowlist": settings.system_tool_allowlist,
        "data_dir": data_dir.to_string_lossy(),
        "is_portable": is_portable,
        "build_profile": build_profile,
        "can_autostart": can_autostart,
    })
}

#[tauri::command]
pub fn set_settings_cmd(
    app: tauri::AppHandle,
    data_dir: tauri::State<std::path::PathBuf>,
    settings: Settings,
) -> Result<(), String> {
    set_settings(&app, &data_dir, &settings);
    // Notify the background timer thread of the new interval.
    // Uses try_state (not State parameter) so this command is safe in all build targets,
    // even if the timer was not started (e.g. mobile builds or tests).
    // Pattern matches existing app.try_state usage in search.rs.
    use std::sync::{Arc, Mutex};
    use tauri::Manager;
    if let Some(timer_state) = app.try_state::<Arc<Mutex<std::sync::mpsc::Sender<crate::indexer::TimerMsg>>>>() {
        if let Ok(tx) = timer_state.lock() {
            let _ = tx.send(crate::indexer::TimerMsg::SetInterval(settings.reindex_interval));
        }
    }
    Ok(())
}

// ---- Unit tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("riftle-store-{label}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup_temp_dir(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    fn write_raw_settings(dir: &Path, contents: &str) {
        fs::write(settings_path(dir), contents).unwrap();
    }

    #[test]
    fn test_settings_defaults() {
        let s = Settings::default();
        assert_eq!(s.hotkey, "Ctrl+Alt+Space");
        assert_eq!(s.theme, "system");

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
        assert_eq!(s.hotkey, "Ctrl+Alt+Space");
        assert_eq!(s.reindex_interval, 15);
    }

    #[test]
    fn test_system_tool_allowlist_survives_serde_round_trip() {
        // Verify Settings serde round-trip preserves the allowlist field.
        // Protects against accidentally dropping the field during JSON serialization.
        let mut s = Settings::default();
        s.system_tool_allowlist = vec!["notepad.exe".to_string(), "calc.exe".to_string()];
        let json = serde_json::to_value(&s).unwrap();
        let restored: Settings = serde_json::from_value(json).unwrap();
        assert_eq!(
            restored.system_tool_allowlist,
            s.system_tool_allowlist,
            "system_tool_allowlist must survive serde round-trip"
        );
    }

    #[test]
    fn test_get_settings_cmd_json_includes_allowlist_field() {
        // RED: documents the required shape of the JSON returned by get_settings_cmd.
        // get_settings_cmd currently omits system_tool_allowlist from its json!() block,
        // causing a silent overwrite when the frontend round-trips settings.
        // This test mirrors the json!() expression from get_settings_cmd WITH the fix applied.
        // It will PASS once the production code is updated in Plan 03.
        let s = Settings::default();
        // This is the json!() expression AS IT SHOULD BE after the fix (includes allowlist):
        let json = serde_json::json!({
            "hotkey": s.hotkey,
            "theme": s.theme,
            "show_path": s.show_path,
            "autostart": s.autostart,
            "additional_paths": s.additional_paths,
            "excluded_paths": s.excluded_paths,
            "reindex_interval": s.reindex_interval,
            "animation": s.animation,
            "system_tool_allowlist": s.system_tool_allowlist,
            "data_dir": "C:\\test",
            "is_portable": false,
            "build_profile": "debug",
            "can_autostart": false,
        });
        assert!(
            json.get("system_tool_allowlist").is_some(),
            "get_settings_cmd JSON must include system_tool_allowlist field"
        );
        let allowlist = json["system_tool_allowlist"].as_array().unwrap();
        assert!(
            !allowlist.is_empty(),
            "default system_tool_allowlist must not be empty"
        );
    }

    #[test]
    fn settings_missing_file_yields_defaults_without_warning() {
        let dir = unique_temp_dir("missing");
        let outcome = load_settings_outcome(&dir);

        match outcome {
            SettingsLoadOutcome::Missing(settings) => {
                assert_eq!(settings.hotkey, "Ctrl+Alt+Space");
                assert!(!backup_path(&settings_path(&dir)).exists());
            }
            other => panic!("expected missing outcome, got {:?}", other),
        }

        cleanup_temp_dir(&dir);
    }

    #[test]
    fn settings_malformed_file_creates_backup_before_recovery() {
        let dir = unique_temp_dir("malformed");
        let malformed = r#"not valid json"#;
        write_raw_settings(&dir, malformed);

        let outcome = load_settings_outcome(&dir);
        let backup = backup_path(&settings_path(&dir));

        match outcome {
            SettingsLoadOutcome::RecoveredWithDefaults { settings, warning } => {
                assert_eq!(settings, Settings::default());
                assert_eq!(warning.kind, "settings-reset");
                assert!(warning.message.contains("restored defaults"));
                assert!(!warning.message.contains("settings.json.bak"));
                assert!(!warning.message.contains(backup.to_string_lossy().as_ref()));
                assert_eq!(warning.backup_path.as_deref(), Some(backup.to_string_lossy().as_ref()));
            }
            other => panic!("expected recovery outcome, got {:?}", other),
        }

        assert_eq!(fs::read_to_string(backup).unwrap(), malformed);
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn settings_existing_backup_is_overwritten_on_recovery() {
        let dir = unique_temp_dir("overwrite");
        write_raw_settings(&dir, "{ definitely broken");
        fs::write(backup_path(&settings_path(&dir)), "old backup").unwrap();

        let outcome = load_settings_outcome(&dir);
        assert!(matches!(
            outcome,
            SettingsLoadOutcome::RecoveredWithDefaults { .. }
        ));

        assert_eq!(
            fs::read_to_string(backup_path(&settings_path(&dir))).unwrap(),
            "{ definitely broken"
        );
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn backup_failure_preserves_original_settings_file() {
        let dir = unique_temp_dir("backup-failure");
        let original = "{ definitely broken";
        write_raw_settings(&dir, original);
        fs::create_dir_all(backup_path(&settings_path(&dir))).unwrap();

        let outcome = load_settings_outcome(&dir);

        match outcome {
            SettingsLoadOutcome::FatalBackupFailure { error } => {
                assert!(error.contains("settings.json.bak"));
            }
            other => panic!("expected fatal backup failure, got {:?}", other),
        }

        assert_eq!(fs::read_to_string(settings_path(&dir)).unwrap(), original);
        cleanup_temp_dir(&dir);
    }
}
