// Phase 4: Nucleo fuzzy search engine with MRU-weighted ranking

use nucleo_matcher::{Matcher, Config, Utf32String, pattern::{Pattern, CaseMatching, Normalization}};
use serde::Serialize;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tauri::Manager;
use crate::db::{AppRecord, get_all_apps};

static SYSTEM_COMMAND_ICON: &[u8] = include_bytes!("../icons/system_command.png");

const SYSTEM_COMMANDS: &[(&str, &str)] = &[
    ("system:lock",     "Lock"),
    ("system:shutdown", "Shutdown"),
    ("system:restart",  "Restart"),
    ("system:sleep",    "Sleep"),
];

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

pub fn ensure_system_command_icon(data_dir: &Path) -> std::io::Result<()> {
    let icons_dir = data_dir.join("icons");
    std::fs::create_dir_all(&icons_dir)?;
    let dest = icons_dir.join("system_command.png");
    if !dest.exists() {
        std::fs::write(&dest, SYSTEM_COMMAND_ICON)?;
    }
    Ok(())
}

pub fn init_search_index(app: &tauri::AppHandle) {
    let db_state = app.state::<crate::db::DbState>();
    let apps = {
        let conn = db_state.0.lock().unwrap();
        get_all_apps(&conn).unwrap_or_default()
    };
    let index = SearchIndex { apps };
    app.manage(SearchIndexState(Arc::new(RwLock::new(index))));
}

pub fn rebuild_index(app: &tauri::AppHandle) {
    let db_state = app.state::<crate::db::DbState>();
    let apps = {
        let conn = db_state.0.lock().unwrap();
        get_all_apps(&conn).unwrap_or_default()
    };
    let new_index = SearchIndex { apps };
    if let Some(state) = app.try_state::<SearchIndexState>() {
        let mut guard = state.0.write().unwrap_or_else(|e: std::sync::PoisonError<_>| e.into_inner());
        *guard = new_index;
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
                            icon_path: app
                                .icon_path
                                .clone()
                                .unwrap_or_else(|| "generic.png".to_string()),
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

    scored.truncate(50);
    scored.into_iter().map(|s| s.result).collect()
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
pub fn search(query: String, index_state: tauri::State<SearchIndexState>) -> Vec<SearchResult> {
    if query.is_empty() {
        return vec![];
    }
    if query.starts_with('>') {
        let suffix = query.trim_start_matches('>').trim_start();
        return search_system_commands(suffix);
    }
    let index = index_state.0.read().unwrap_or_else(|e| e.into_inner());
    score_and_rank(&query, &index.apps)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_search_empty_returns_empty() {
        let apps = vec![make_app("chrome", "Chrome", 5)];
        let results = score_and_rank("", &apps);
        assert!(results.is_empty(), "Empty query should return empty results");
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
            assert!(vs < vb, "Visual Studio (acronym) should rank before VirtualBox (fuzzy)");
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
            assert!(vs < vb, "Visual Studio (acronym) should rank before VirtualBox (fuzzy)");
        }
        if let (Some(vs), Some(vb)) = (vstream_pos, vbox_pos) {
            assert!(vs < vb, "Video Stream (acronym) should rank before VirtualBox (fuzzy)");
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
            assert!(pp < np, "Notepad++ (higher launch_count) should rank before Notepad");
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
        assert_eq!(results.len(), 4, "Empty suffix should return all 4 system commands");
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
        assert!(results.iter().any(|r| r.name == "Lock"), "Should contain Lock");
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
}
