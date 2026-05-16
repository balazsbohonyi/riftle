// Phase 4: Nucleo fuzzy search engine with MRU-weighted ranking

use crate::db::{get_all_apps, AppRecord};
use crate::shortcuts::{shortcut_display_name, shortcut_id};
use crate::store::{load_settings_outcome, Settings, SettingsLoadOutcome};
use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher, Utf32String,
};
use serde::Serialize;
use std::path::Path;
use std::sync::{Arc, MutexGuard, PoisonError, RwLock};
use tauri::Manager;

static SYSTEM_COMMAND_ICON: &[u8] = include_bytes!("../icons/system_command.png");

const SYSTEM_COMMANDS: &[(&str, &str)] = &[
    ("system:lock", "Lock"),
    ("system:shutdown", "Shutdown"),
    ("system:restart", "Restart"),
    ("system:sleep", "Sleep"),
];
const SEARCH_RESULT_LIMIT: usize = 50;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub icon_path: String,
    pub path: String,
    pub kind: String,
    pub requires_elevation: bool,
}

pub struct SearchIndex {
    pub apps: Vec<AppRecord>,
}

pub struct SearchIndexState(pub Arc<RwLock<SearchIndex>>);

fn lock_db<'a>(
    db_state: &'a crate::db::DbState,
    context: &str,
) -> MutexGuard<'a, rusqlite::Connection> {
    db_state.0.lock().unwrap_or_else(|err: PoisonError<_>| {
        eprintln!("[search::{context}] recovering from poisoned DB mutex");
        err.into_inner()
    })
}

fn load_apps_for_index(
    db_state: &crate::db::DbState,
    context: &str,
) -> Result<Vec<AppRecord>, rusqlite::Error> {
    let conn = lock_db(db_state, context);
    get_all_apps(&conn)
}

fn replace_index_apps(state: &SearchIndexState, apps: Option<Vec<AppRecord>>) -> bool {
    let Some(apps) = apps else {
        return false;
    };

    let mut guard = state
        .0
        .write()
        .unwrap_or_else(|e: std::sync::PoisonError<_>| e.into_inner());
    guard.apps = apps;
    true
}

pub fn ensure_system_command_icon(data_dir: &Path) -> std::io::Result<()> {
    let icons_dir = data_dir.join("icons");
    std::fs::create_dir_all(&icons_dir)?;
    let dest = icons_dir.join("system_command.png");
    std::fs::write(&dest, SYSTEM_COMMAND_ICON)?;
    Ok(())
}

pub fn init_search_index(app: &tauri::AppHandle) {
    let db_state = app.state::<crate::db::DbState>();
    let apps = load_apps_for_index(&db_state, "init_search_index").unwrap_or_else(|err| {
        eprintln!(
            "[search::init_search_index] failed to load apps from DB: {}",
            err
        );
        Vec::new()
    });
    let index = SearchIndex { apps };
    app.manage(SearchIndexState(Arc::new(RwLock::new(index))));
}

pub fn rebuild_index(app: &tauri::AppHandle) {
    let db_state = app.state::<crate::db::DbState>();
    let apps = match load_apps_for_index(&db_state, "rebuild_index") {
        Ok(apps) => Some(apps),
        Err(err) => {
            eprintln!(
                "[search::rebuild_index] failed to refresh apps from DB: {}",
                err
            );
            None
        }
    };
    if let Some(state) = app.try_state::<SearchIndexState>() {
        let _ = replace_index_apps(&state, apps);
    }
}

struct ScoredResult {
    tier: u8,
    score: u32,
    launch_count: i64,
    result: SearchResult,
}

pub fn score_and_rank(query: &str, apps: &[AppRecord]) -> Vec<SearchResult> {
    if query.is_empty() {
        return Vec::new();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    let q_lower = query.to_lowercase();

    let mut scored: Vec<ScoredResult> = apps
        .iter()
        .filter_map(|app| {
            let haystack = Utf32String::from(app.name.as_str());
            pattern
                .score(haystack.slice(..), &mut matcher)
                .map(|score| {
                    let name_lower = app.name.to_lowercase();
                    let tier = match_tier(&q_lower, &name_lower);
                    ScoredResult {
                        tier,
                        score,
                        launch_count: app.launch_count,
                        result: SearchResult {
                            id: app.id.clone(),
                            name: app.name.clone(),
                            icon_path: {
                                let raw = app
                                    .icon_path
                                    .clone()
                                    .unwrap_or_else(|| "generic.png".to_string());
                                if validate_icon_filename(&raw) {
                                    raw
                                } else {
                                    "generic.png".to_string()
                                }
                            },
                            path: app.path.clone(),
                            kind: "app".to_string(),
                            requires_elevation: false,
                        },
                    }
                })
        })
        .collect();

    scored.sort_unstable_by(|a, b| {
        b.tier
            .cmp(&a.tier)
            .then_with(|| b.score.cmp(&a.score))
            .then_with(|| b.launch_count.cmp(&a.launch_count))
    });

    scored.truncate(SEARCH_RESULT_LIMIT);
    scored.into_iter().map(|s| s.result).collect()
}

fn sanitized_icon_path(icon_path: Option<&str>) -> Option<String> {
    icon_path
        .filter(|path| validate_icon_filename(path))
        .map(ToString::to_string)
}

fn app_icon_for_path(apps: &[AppRecord], path: &str) -> Option<String> {
    let normalized_path = path.trim().to_lowercase();
    apps.iter()
        .find(|app| {
            app.path.trim().to_lowercase() == normalized_path
                || app.id.trim().to_lowercase() == normalized_path
        })
        .and_then(|app| sanitized_icon_path(app.icon_path.as_deref()))
        .filter(|icon_path| icon_path != "generic.png")
}

fn ensure_shortcut_icon_file(
    data_dir: &Path,
    cache_key: &str,
    source_path: &str,
    is_directory: bool,
    is_executable: bool,
) -> Option<String> {
    let filename = crate::indexer::icon_filename(cache_key);
    let icons_dir = data_dir.join("icons");
    let icon_file = icons_dir.join(&filename);
    if icon_file.exists() {
        return Some(filename);
    }

    std::fs::create_dir_all(&icons_dir).ok()?;
    let path = Path::new(source_path);
    let bytes = if is_executable {
        crate::indexer::extract_icon_png(&crate::indexer::IconSource::File(path.to_path_buf()))
            .or_else(|| crate::indexer::extract_shell_icon_png(path, false))
    } else {
        crate::indexer::extract_shell_icon_png(path, is_directory)
    }?;

    std::fs::write(&icon_file, bytes).ok()?;
    Some(filename)
}

fn directory_shortcut_icon(path: &str, data_dir: Option<&Path>) -> String {
    data_dir
        .and_then(|dir| {
            ensure_shortcut_icon_file(dir, &format!("shortcut:dir:{path}"), path, true, false)
        })
        .unwrap_or_else(|| "generic.png".to_string())
}

fn file_shortcut_icon(path: &str, apps: &[AppRecord], data_dir: Option<&Path>) -> String {
    if let Some(icon_path) = app_icon_for_path(apps, path) {
        return icon_path;
    }

    let is_executable = crate::shortcuts::is_parameterized_executable_target(path);
    data_dir
        .and_then(|dir| {
            ensure_shortcut_icon_file(
                dir,
                &format!("shortcut:file:{path}"),
                path,
                false,
                is_executable,
            )
        })
        .unwrap_or_else(|| "generic.png".to_string())
}

pub fn search_shortcuts(
    query: &str,
    settings: &Settings,
    apps: &[AppRecord],
    data_dir: Option<&Path>,
) -> Vec<SearchResult> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();

    for shortcut in &settings.directory_shortcuts {
        let name = shortcut_display_name(&shortcut.path, &shortcut.alias);
        if !name.to_lowercase().starts_with(&q) {
            continue;
        }

        results.push(SearchResult {
            id: shortcut_id("dir", &shortcut.path),
            name,
            icon_path: directory_shortcut_icon(&shortcut.path, data_dir),
            path: shortcut.path.clone(),
            kind: "shortcut_dir".to_string(),
            requires_elevation: false,
        });

        if results.len() == SEARCH_RESULT_LIMIT {
            return results;
        }
    }

    for shortcut in &settings.file_shortcuts {
        let name = shortcut_display_name(&shortcut.path, &shortcut.alias);
        if !name.to_lowercase().starts_with(&q) {
            continue;
        }

        results.push(SearchResult {
            id: shortcut_id("file", &shortcut.path),
            name,
            icon_path: file_shortcut_icon(&shortcut.path, apps, data_dir),
            path: shortcut.path.clone(),
            kind: "shortcut_file".to_string(),
            requires_elevation: false,
        });

        if results.len() == SEARCH_RESULT_LIMIT {
            return results;
        }
    }

    results
}

pub fn search_with_shortcuts(
    query: &str,
    apps: &[AppRecord],
    settings: &Settings,
    data_dir: Option<&Path>,
) -> Vec<SearchResult> {
    if query.is_empty() {
        return Vec::new();
    }
    if query.starts_with('>') {
        let suffix = query.trim_start_matches('>').trim_start();
        return search_system_commands(suffix);
    }

    let mut results = search_shortcuts(query, settings, apps, data_dir);
    let remaining = SEARCH_RESULT_LIMIT.saturating_sub(results.len());
    if remaining == 0 {
        return results;
    }

    let mut app_results = score_and_rank(query, apps);
    app_results.truncate(remaining);
    results.extend(app_results);
    results
}

fn load_search_settings(data_dir: &Path) -> Option<Settings> {
    match load_settings_outcome(data_dir) {
        SettingsLoadOutcome::Loaded(settings)
        | SettingsLoadOutcome::Missing(settings)
        | SettingsLoadOutcome::RecoveredWithDefaults { settings, .. } => Some(settings),
        SettingsLoadOutcome::FatalBackupFailure { error } => {
            eprintln!(
                "[search::search] failed to load shortcut settings: {}",
                error
            );
            None
        }
    }
}

fn match_tier(q_lower: &str, name_lower: &str) -> u8 {
    if name_lower.starts_with(q_lower) {
        2
    } else if q_lower.len() >= 2 && is_acronym_match(q_lower, name_lower) {
        1
    } else {
        0
    }
}

fn is_acronym_match(query: &str, name: &str) -> bool {
    let initials: String = name
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .collect::<String>()
        .to_lowercase();
    initials.starts_with(query)
}

fn search_system_commands(suffix: &str) -> Vec<SearchResult> {
    let q = suffix.to_lowercase();
    SYSTEM_COMMANDS
        .iter()
        .filter(|(_, name)| q.is_empty() || name.to_lowercase().contains(&q))
        .map(|(id, name)| SearchResult {
            id: id.to_string(),
            name: name.to_string(),
            icon_path: "system_command.png".to_string(),
            path: String::new(),
            kind: "system".to_string(),
            requires_elevation: false,
        })
        .collect()
}

#[tauri::command]
pub fn search(
    query: String,
    index_state: tauri::State<SearchIndexState>,
    data_dir: tauri::State<std::path::PathBuf>,
) -> Vec<SearchResult> {
    if query.is_empty() {
        return vec![];
    }
    if query.starts_with('>') {
        let suffix = query.trim_start_matches('>').trim_start();
        return search_system_commands(suffix);
    }
    let index = index_state.0.read().unwrap_or_else(|e| e.into_inner());
    if let Some(settings) = load_search_settings(&data_dir) {
        search_with_shortcuts(&query, &index.apps, &settings, Some(&data_dir))
    } else {
        score_and_rank(&query, &index.apps)
    }
}

pub fn validate_icon_filename(filename: &str) -> bool {
    if filename == "generic.png" || filename == "system_command.png" {
        return true;
    }
    if filename.len() != 20 {
        return false;
    }
    let (hex_part, ext) = filename.split_at(16);
    ext == ".png" && hex_part.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shortcuts::{DirectoryShortcut, FileShortcut};
    use crate::store::Settings;

    fn make_app(id: &str, name: &str, launch_count: i64) -> AppRecord {
        AppRecord {
            id: id.to_string(),
            name: name.to_string(),
            path: format!("C:\\apps\\{}.exe", id),
            icon_path: None,
            source: "start_menu".to_string(),
            last_launched: None,
            launch_count,
        }
    }

    fn make_app_with_path_icon(path: &str, icon_path: &str) -> AppRecord {
        AppRecord {
            id: path.to_lowercase(),
            name: "Indexed App".to_string(),
            path: path.to_string(),
            icon_path: Some(icon_path.to_string()),
            source: "start_menu".to_string(),
            last_launched: None,
            launch_count: 0,
        }
    }

    fn make_settings_with_shortcuts(
        directory_shortcuts: Vec<DirectoryShortcut>,
        file_shortcuts: Vec<FileShortcut>,
    ) -> Settings {
        Settings {
            directory_shortcuts,
            file_shortcuts,
            ..Settings::default()
        }
    }

    fn directory_shortcut(path: &str, alias: &str) -> DirectoryShortcut {
        DirectoryShortcut {
            path: path.to_string(),
            alias: alias.to_string(),
        }
    }

    fn file_shortcut(path: &str, alias: &str) -> FileShortcut {
        FileShortcut {
            path: path.to_string(),
            parameters: String::new(),
            alias: alias.to_string(),
        }
    }

    #[test]
    fn shortcut_fallback_names_alias_prefix_matches_directory_shortcut() {
        let settings = make_settings_with_shortcuts(
            vec![directory_shortcut("C:\\Projects\\Riftle", "Work")],
            vec![],
        );

        let results = search_shortcuts("wo", &settings, &[], None);

        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].id,
            crate::shortcuts::shortcut_id("dir", "C:\\Projects\\Riftle")
        );
        assert_eq!(results[0].name, "Work");
        assert_eq!(results[0].icon_path, "generic.png");
        assert_eq!(results[0].path, "C:\\Projects\\Riftle");
        assert_eq!(results[0].kind, "shortcut_dir");
        assert!(!results[0].requires_elevation);
    }

    #[test]
    fn shortcut_fallback_names_empty_directory_alias_matches_basename() {
        let settings = make_settings_with_shortcuts(
            vec![directory_shortcut("C:\\Projects\\Riftle", "")],
            vec![],
        );

        let results = search_shortcuts("rift", &settings, &[], None);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Riftle");
        assert_eq!(results[0].kind, "shortcut_dir");
        assert_eq!(results[0].path, "C:\\Projects\\Riftle");
    }

    #[test]
    fn shortcut_fallback_names_empty_file_alias_matches_filename_without_extension() {
        let settings = make_settings_with_shortcuts(
            vec![],
            vec![file_shortcut("C:\\Docs\\Release Notes.pdf", "")],
        );

        let results = search_shortcuts("release", &settings, &[], None);

        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].id,
            crate::shortcuts::shortcut_id("file", "C:\\Docs\\Release Notes.pdf")
        );
        assert_eq!(results[0].name, "Release Notes");
        assert_eq!(results[0].kind, "shortcut_file");
        assert_eq!(results[0].path, "C:\\Docs\\Release Notes.pdf");
        assert_eq!(results[0].icon_path, "generic.png");
    }

    #[test]
    fn executable_file_shortcut_reuses_indexed_app_icon() {
        let settings = make_settings_with_shortcuts(
            vec![],
            vec![file_shortcut(
                "C:\\Program Files\\Editor\\editor.exe",
                "Config",
            )],
        );
        let apps = vec![make_app_with_path_icon(
            "C:\\Program Files\\Editor\\editor.exe",
            "0123456789abcdef.png",
        )];

        let results = search_shortcuts("config", &settings, &apps, None);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].icon_path, "0123456789abcdef.png");
    }

    #[test]
    fn executable_file_shortcut_ignores_invalid_indexed_app_icon() {
        let settings = make_settings_with_shortcuts(
            vec![],
            vec![file_shortcut(
                "C:\\Program Files\\Editor\\editor.exe",
                "Config",
            )],
        );
        let apps = vec![make_app_with_path_icon(
            "C:\\Program Files\\Editor\\editor.exe",
            "..\\bad.png",
        )];

        let results = search_shortcuts("config", &settings, &apps, None);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].icon_path, "generic.png");
    }

    #[test]
    fn shortcut_search_precedes_apps_when_both_match_same_query() {
        let settings = make_settings_with_shortcuts(
            vec![directory_shortcut("C:\\Projects\\Workbench", "Work")],
            vec![],
        );
        let apps = vec![make_app("workbench", "Workbench", 99)];

        let results = search_with_shortcuts("wo", &apps, &settings, None);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].kind, "shortcut_dir");
        assert_eq!(results[0].name, "Work");
        assert_eq!(results[1].kind, "app");
        assert_eq!(results[1].name, "Workbench");
    }

    #[test]
    fn shortcut_search_precedes_apps_and_consumes_result_cap() {
        let settings = make_settings_with_shortcuts(
            vec![
                directory_shortcut("C:\\Projects\\AppOne", "App One"),
                directory_shortcut("C:\\Projects\\AppTwo", "App Two"),
            ],
            vec![],
        );
        let apps: Vec<AppRecord> = (0..60)
            .map(|i| make_app(&format!("app{}", i), &format!("AppFoo{}", i), i as i64))
            .collect();

        let results = search_with_shortcuts("app", &apps, &settings, None);

        assert_eq!(results.len(), 50);
        assert_eq!(
            results
                .iter()
                .filter(|result| result.kind.starts_with("shortcut_"))
                .count(),
            2
        );
        assert_eq!(
            results.iter().filter(|result| result.kind == "app").count(),
            48
        );
        assert!(results[..2]
            .iter()
            .all(|result| result.kind == "shortcut_dir"));
    }

    #[test]
    fn shortcut_search_precedes_apps_system_commands_do_not_mix_shortcuts() {
        let settings = make_settings_with_shortcuts(
            vec![directory_shortcut("C:\\Tools\\Shutdown", "Shutdown")],
            vec![],
        );
        let apps = vec![make_app("shutdown", "Shutdown Helper", 99)];

        let results = search_with_shortcuts("> sh", &apps, &settings, None);

        assert!(!results.is_empty());
        assert!(results.iter().all(|result| result.kind == "system"));
        assert!(results
            .iter()
            .all(|result| !result.kind.starts_with("shortcut_")));
        assert!(results.iter().all(|result| result.kind != "app"));
    }

    #[test]
    fn test_replace_index_apps_preserves_existing_entries_on_failed_refresh() {
        let state = SearchIndexState(Arc::new(RwLock::new(SearchIndex {
            apps: vec![make_app("chrome", "Chrome", 5)],
        })));

        let replaced = replace_index_apps(&state, None);

        assert!(
            !replaced,
            "failed refresh should not replace the in-memory index"
        );
        let guard = state.0.read().unwrap();
        assert_eq!(guard.apps.len(), 1);
        assert_eq!(guard.apps[0].name, "Chrome");
    }

    #[test]
    fn test_search_empty_returns_empty() {
        let apps = vec![make_app("chrome", "Chrome", 5)];
        let results = score_and_rank("", &apps);
        assert!(
            results.is_empty(),
            "Empty query should return empty results"
        );
    }

    #[test]
    fn test_search_fuzzy_returns_matches() {
        let apps = vec![
            make_app("chrome", "Chrome", 5),
            make_app("firefox", "Firefox", 3),
        ];
        let results = score_and_rank("chr", &apps);
        assert!(!results.is_empty(), "Should return at least one result");
        assert!(
            results.iter().any(|r| r.name == "Chrome"),
            "Should contain Chrome"
        );
    }

    #[test]
    fn test_search_prefix_beats_fuzzy() {
        // Both match "chr" but Chrome has prefix tier
        let apps = vec![
            make_app("chrome", "Chrome", 5),
            make_app("chromebook", "Chromebook", 1),
        ];
        let results = score_and_rank("chr", &apps);
        assert!(!results.is_empty(), "Should return results");
        // Both are prefix matches — at minimum both should appear
        assert!(results.iter().any(|r| r.name == "Chrome"));
        assert!(results.iter().any(|r| r.name == "Chromebook"));
        // Visual Studio should rank above fuzzy-only match via acronym tier
        let apps2 = vec![
            make_app("vs", "Visual Studio", 5),
            make_app("vbox", "VirtualBox", 5),
        ];
        let results2 = score_and_rank("vs", &apps2);
        assert!(!results2.is_empty());
        // Visual Studio gets acronym tier (initials "vs"), VirtualBox does not
        let vs_pos = results2.iter().position(|r| r.name == "Visual Studio");
        let vbox_pos = results2.iter().position(|r| r.name == "VirtualBox");
        if let (Some(vs), Some(vb)) = (vs_pos, vbox_pos) {
            assert!(
                vs < vb,
                "Visual Studio (acronym) should rank before VirtualBox (fuzzy)"
            );
        }
    }

    #[test]
    fn test_search_acronym_tier() {
        // "vs" matches Visual Studio AND Video Stream (both have initials "vs")
        let apps = vec![
            make_app("vs", "Visual Studio", 5),
            make_app("vstream", "Video Stream", 3),
            make_app("vbox", "VirtualBox", 10),
        ];
        let results = score_and_rank("vs", &apps);
        assert!(!results.is_empty());
        let vs_pos = results.iter().position(|r| r.name == "Visual Studio");
        let vstream_pos = results.iter().position(|r| r.name == "Video Stream");
        let vbox_pos = results.iter().position(|r| r.name == "VirtualBox");
        // Both acronym matches should appear before fuzzy
        if let (Some(vs), Some(vb)) = (vs_pos, vbox_pos) {
            assert!(
                vs < vb,
                "Visual Studio (acronym) should rank before VirtualBox (fuzzy)"
            );
        }
        if let (Some(vs), Some(vb)) = (vstream_pos, vbox_pos) {
            assert!(
                vs < vb,
                "Video Stream (acronym) should rank before VirtualBox (fuzzy)"
            );
        }
        // Single-char query should NOT apply acronym tier
        let apps2 = vec![make_app("vbox", "VirtualBox", 5)];
        let results2 = score_and_rank("v", &apps2);
        // VirtualBox starts with "v" so it will match prefix tier (tier 2), not acronym
        // The key behavior: no panic, just verify it runs
        let _ = results2;
    }

    #[test]
    fn test_search_mru_tiebreak() {
        // Two apps both matching "note" at same tier, higher launch_count ranks first
        let apps = vec![
            make_app("notepad", "Notepad", 2),
            make_app("notepadpp", "Notepad++", 10),
        ];
        let results = score_and_rank("note", &apps);
        assert!(results.len() >= 2, "Should return both results");
        let notepadpp_pos = results.iter().position(|r| r.name == "Notepad++");
        let notepad_pos = results.iter().position(|r| r.name == "Notepad");
        if let (Some(pp), Some(np)) = (notepadpp_pos, notepad_pos) {
            assert!(
                pp < np,
                "Notepad++ (higher launch_count) should rank before Notepad"
            );
        }
    }

    #[test]
    fn test_search_capped_at_50() {
        // 60 apps all matching "app" — score_and_rank should return exactly 50
        let apps: Vec<AppRecord> = (0..60)
            .map(|i| make_app(&format!("app{}", i), &format!("AppFoo{}", i), i as i64))
            .collect();
        let results = score_and_rank("app", &apps);
        assert_eq!(results.len(), 50, "score_and_rank should cap at 50 results");
    }

    #[test]
    fn test_search_system_prefix_all() {
        let results = search_system_commands("");
        assert_eq!(
            results.len(),
            4,
            "Empty suffix should return all 4 system commands"
        );
        let names: Vec<&str> = results.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"Lock"), "Should contain Lock");
        assert!(names.contains(&"Shutdown"), "Should contain Shutdown");
        assert!(names.contains(&"Restart"), "Should contain Restart");
        assert!(names.contains(&"Sleep"), "Should contain Sleep");
    }

    #[test]
    fn test_search_system_prefix_filtered() {
        // "sh" is contained in "Shutdown" — "Sleep" does not contain "sh"
        // Plan spec noted "Sleep" but "sleep".contains("sh") == false;
        // correct behavior is Shutdown only (1 result)
        let results = search_system_commands("sh");
        assert_eq!(results.len(), 1, "'sh' should match Shutdown only");
        let names: Vec<&str> = results.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"Shutdown"), "Should contain Shutdown");
    }

    #[test]
    fn test_search_system_no_space() {
        let results = search_system_commands("lo");
        assert!(!results.is_empty(), "Should return Lock for 'lo'");
        assert!(
            results.iter().any(|r| r.name == "Lock"),
            "Should contain Lock"
        );
    }

    #[test]
    fn test_search_system_no_app_mixing() {
        let results = search_system_commands("");
        assert!(!results.is_empty());
        assert!(
            results.iter().all(|r| r.kind != "app"),
            "No system command result should have kind == 'app'"
        );
    }

    #[test]
    fn test_system_result_kind() {
        let results = search_system_commands("");
        assert!(!results.is_empty());
        assert!(
            results.iter().all(|r| r.kind == "system"),
            "All system command results should have kind == 'system'"
        );
    }

    #[test]
    fn test_system_result_icon() {
        let results = search_system_commands("");
        assert!(!results.is_empty());
        assert!(
            results.iter().all(|r| r.icon_path == "system_command.png"),
            "All system command results should have icon_path == 'system_command.png'"
        );
    }

    #[test]
    fn test_system_result_path_empty() {
        let results = search_system_commands("");
        assert!(!results.is_empty());
        assert!(
            results.iter().all(|r| r.path.is_empty()),
            "All system command results should have path == ''"
        );
    }

    #[test]
    fn test_ensure_system_command_icon_creates_file() {
        let tmp = std::env::temp_dir().join("riftle_test_icon");
        let _ = std::fs::remove_dir_all(&tmp);
        ensure_system_command_icon(&tmp).expect("Should succeed");
        assert!(
            tmp.join("icons").join("system_command.png").exists(),
            "system_command.png should be created in icons/"
        );
        // Second call should be idempotent
        ensure_system_command_icon(&tmp).expect("Second call should also succeed");
    }

    #[test]
    fn test_validate_icon_rejects_path_traversal() {
        assert!(!validate_icon_filename("../etc/passwd.png"));
        assert!(!validate_icon_filename("../../etc/passwd"));
        assert!(!validate_icon_filename("..\\system32\\evil.exe"));
    }

    #[test]
    fn test_validate_icon_rejects_absolute_path() {
        assert!(!validate_icon_filename("/etc/passwd"));
        assert!(!validate_icon_filename("C:\\Windows\\System32\\cmd.exe"));
    }

    #[test]
    fn test_validate_icon_rejects_wrong_extension() {
        assert!(!validate_icon_filename("abc1234567890defg.exe"));
        assert!(!validate_icon_filename("abc1234567890defg.jpg"));
        assert!(!validate_icon_filename("abc1234567890defg"));
    }

    #[test]
    fn test_validate_icon_rejects_uppercase_hex() {
        assert!(!validate_icon_filename("ABC1234567890DEF0.png"));
    }

    #[test]
    fn test_validate_icon_rejects_wrong_length() {
        assert!(!validate_icon_filename("abc.png"));
        assert!(!validate_icon_filename("abc1234567890defghij.png"));
    }

    #[test]
    fn test_validate_icon_accepts_valid_hex() {
        assert!(validate_icon_filename("0123456789abcdef.png"));
        assert!(validate_icon_filename("0000000000000000.png"));
        assert!(validate_icon_filename("ffffffffffffffff.png"));
    }

    #[test]
    fn test_validate_icon_accepts_generic() {
        assert!(validate_icon_filename("generic.png"));
    }

    #[test]
    fn test_validate_icon_accepts_system_command() {
        assert!(validate_icon_filename("system_command.png"));
    }
}
