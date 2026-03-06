// Phase 3: Windows application indexer
// crawl Start Menu, Desktop, PATH; .lnk resolution; icon extraction; background refresh

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use rusqlite::Connection;

use crate::db::{AppRecord, upsert_app};
use crate::store::Settings;

// Generic icon bundled at compile time — path relative to this source file
static GENERIC_ICON: &[u8] = include_bytes!("../icons/generic.png");

// ---- Public API (called from lib.rs) ----

/// Run a full blocking index. Called synchronously in setup() before app is ready.
/// Crawls all source dirs, upserts apps, extracts icons async, prunes stale entries.
pub fn run_full_index(db: &Arc<Mutex<Connection>>, data_dir: &Path, settings: &Settings) {
    todo!("Phase 3 Plan 04: implement full index")
}

/// Spawn background timer thread and filesystem watcher thread.
/// Called once after run_full_index() completes in setup().
/// Returns the timer reset Sender — store as managed state for reindex() command.
pub fn start_background_tasks(
    db: Arc<Mutex<Connection>>,
    data_dir: PathBuf,
    settings: &Settings,
    is_indexing: Arc<AtomicBool>,
) -> mpsc::Sender<()> {
    todo!("Phase 3 Plan 05: implement background tasks")
}

/// Tauri command: fire-and-forget manual re-index.
/// Spawns a thread, returns immediately. Frontend shows loading state.
#[tauri::command]
pub fn reindex(
    db_state: tauri::State<crate::db::DbState>,
    is_indexing: tauri::State<Arc<AtomicBool>>,
    timer_tx: tauri::State<Arc<Mutex<mpsc::Sender<()>>>>,
    data_dir: tauri::State<PathBuf>,
) {
    todo!("Phase 3 Plan 05: implement reindex command")
}

// ---- Internal functions ----

/// Get all directories to index, with their source labels.
pub(crate) fn get_index_paths(settings: &Settings) -> Vec<(PathBuf, &'static str)> {
    todo!("Phase 3 Plan 02: implement path discovery")
}

/// Walk a directory, resolve .lnk shortcuts, return AppRecords.
/// source: "start_menu" | "desktop" | "path" | "additional"
/// PATH source: .exe only, no .lnk resolution, max_depth 1.
pub(crate) fn crawl_dir(root: &Path, source: &'static str, excluded: &[String]) -> Vec<AppRecord> {
    todo!("Phase 3 Plan 02: implement crawl")
}

/// Resolve a .lnk file to its target executable path.
/// Returns None if: unresolvable, target is another .lnk, target doesn't exist, target isn't .exe.
pub(crate) fn resolve_lnk(lnk_path: &Path) -> Option<PathBuf> {
    todo!("Phase 3 Plan 02: implement LNK resolution")
}

/// Build a canonical AppRecord from an exe path.
pub(crate) fn make_app_record(exe_path: &Path, source: &'static str) -> AppRecord {
    todo!("Phase 3 Plan 02: implement make_app_record")
}

/// FNV-1a 64-bit hash of the normalized exe path → "{:016x}.png" icon filename.
pub(crate) fn icon_filename(exe_path: &str) -> String {
    todo!("Phase 3 Plan 02: implement icon_filename")
}

/// Delete all apps from DB whose id is not in discovered_ids.
pub(crate) fn prune_stale(conn: &Connection, discovered_ids: &HashSet<String>) -> rusqlite::Result<()> {
    todo!("Phase 3 Plan 02: implement prune_stale")
}

/// Copy bundled GENERIC_ICON to {data_dir}/icons/generic.png if missing.
pub(crate) fn ensure_generic_icon(data_dir: &Path) -> std::io::Result<()> {
    todo!("Phase 3 Plan 03: implement ensure_generic_icon")
}

/// Extract a 32x32 RGBA PNG from an exe file using Windows GDI.
/// Returns PNG bytes or None on any failure.
/// MUST be called from a spawned thread — GDI calls take 5-50ms per exe.
pub(crate) fn extract_icon_png(exe_path: &Path) -> Option<Vec<u8>> {
    todo!("Phase 3 Plan 03: implement icon extraction")
}

/// Try to start an index run. No-op if already indexing (AtomicBool guard).
/// Spawns a thread, sets flag true, runs index, sets flag false on completion.
pub(crate) fn try_start_index(
    is_indexing: &Arc<AtomicBool>,
    db: &Arc<Mutex<Connection>>,
    data_dir: &Path,
    settings: &Settings,
) {
    todo!("Phase 3 Plan 05: implement try_start_index")
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::fs;
    use tempfile::tempdir; // NOTE: tempfile is a dev-dependency

    // Test helpers
    fn in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::init_db_connection(&conn).unwrap();
        conn
    }

    fn temp_dir_with_exe(name: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().unwrap();
        let exe_path = dir.path().join(name);
        fs::write(&exe_path, b"MZ").unwrap(); // minimal DOS header stub
        (dir, exe_path)
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_crawl_discovers_exe() {
        let (dir, _exe) = temp_dir_with_exe("foo.exe");
        let apps = crawl_dir(dir.path(), "additional", &[]);
        assert_eq!(apps.len(), 1);
        assert!(apps[0].path.ends_with("foo.exe"));
    }

    #[test]
    #[ignore] // .lnk creation requires Windows shell APIs — tested manually in smoke test
    fn test_crawl_discovers_lnk() {
        // Will be enabled when a .lnk test fixture is available
    }

    #[test]
    #[ignore] // Requires a real .lnk file on disk
    fn test_resolve_lnk_valid() {
        // Verified in smoke test: Start Menu .lnk → exe path
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_resolve_lnk_broken() {
        let result = resolve_lnk(Path::new("C:\\nonexistent\\fake.lnk"));
        assert!(result.is_none());
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_prune_stale() {
        let conn = in_memory_db();
        // Insert two apps
        let app1 = AppRecord {
            id: "app1".to_string(), name: "App 1".to_string(),
            path: "C:\\app1.exe".to_string(), icon_path: None,
            source: "start_menu".to_string(), last_launched: None, launch_count: 0,
        };
        let app2 = AppRecord {
            id: "app2".to_string(), name: "App 2".to_string(),
            path: "C:\\app2.exe".to_string(), icon_path: None,
            source: "start_menu".to_string(), last_launched: None, launch_count: 0,
        };
        crate::db::upsert_app(&conn, &app1).unwrap();
        crate::db::upsert_app(&conn, &app2).unwrap();
        // Only app1 was discovered this index
        let mut discovered = HashSet::new();
        discovered.insert("app1".to_string());
        prune_stale(&conn, &discovered).unwrap();
        let remaining = crate::db::get_all_apps(&conn).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "app1");
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_crawl_excludes_path() {
        let dir = tempdir().unwrap();
        let excluded_sub = dir.path().join("excluded");
        fs::create_dir_all(&excluded_sub).unwrap();
        fs::write(excluded_sub.join("hidden.exe"), b"MZ").unwrap();
        fs::write(dir.path().join("visible.exe"), b"MZ").unwrap();
        let excluded = vec![excluded_sub.to_string_lossy().to_string()];
        let apps = crawl_dir(dir.path(), "additional", &excluded);
        // Only visible.exe should appear
        assert_eq!(apps.len(), 1);
        assert!(apps[0].path.contains("visible.exe"));
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_icon_filename_stable() {
        let path = "C:\\Windows\\notepad.exe";
        let a = icon_filename(path);
        let b = icon_filename(path);
        assert_eq!(a, b);
        assert!(a.ends_with(".png"));
        assert_eq!(a.len(), 20); // 16 hex chars + ".png"
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_generic_icon_bootstrap() {
        let dir = tempdir().unwrap();
        ensure_generic_icon(dir.path()).unwrap();
        let generic_path = dir.path().join("icons").join("generic.png");
        assert!(generic_path.exists());
        let size1 = fs::metadata(&generic_path).unwrap().len();
        // Second call is no-op
        ensure_generic_icon(dir.path()).unwrap();
        let size2 = fs::metadata(&generic_path).unwrap().len();
        assert_eq!(size1, size2);
    }

    #[test]
    #[ignore] // Implemented in Plan 05 — no callable stub here yet
    fn test_timer_fires() {
        use std::sync::atomic::AtomicUsize;
        use std::time::Duration;
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        // This test will be properly implemented in Plan 05
        // For now: verify start_background_tasks signature compiles
        let _ = counter_clone;
    }

    #[test]
    #[ignore] // Implemented in Plan 05 — no callable stub here yet
    fn test_timer_reset() {
        // Timer reset signal via mpsc::Sender<()>
        // Will be implemented in Plan 05
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_atomic_guard_prevents_double_index() {
        let flag = Arc::new(AtomicBool::new(false));
        let db = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        let settings = Settings::default();
        let dir = tempdir().unwrap();
        // First call should start; second concurrent call should be dropped
        try_start_index(&flag, &db, dir.path(), &settings);
        try_start_index(&flag, &db, dir.path(), &settings);
    }
}
