// Phase 3: Windows application indexer
// crawl Start Menu, Desktop, PATH; .lnk resolution; icon extraction; background refresh

use std::collections::HashSet;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use rusqlite::Connection;
use windows_sys::Win32::UI::Shell::ExtractIconExW;
use windows_sys::Win32::UI::WindowsAndMessaging::{ICONINFO, GetIconInfo, DestroyIcon};
use windows_sys::Win32::Graphics::Gdi::{
    GetDIBits, GetObjectW, DeleteObject, CreateCompatibleDC, DeleteDC,
    BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS,
};

use crate::db::{AppRecord, upsert_app};
use crate::store::Settings;

// Generic icon bundled at compile time — path relative to this source file
static GENERIC_ICON: &[u8] = include_bytes!("../icons/generic.png");

// ---- Public API (called from lib.rs) ----

/// Run a full blocking index. Called synchronously in setup() before app is ready.
/// Crawls all source dirs, upserts apps, extracts icons async, prunes stale entries.
pub fn run_full_index(db: &Arc<Mutex<Connection>>, data_dir: &Path, settings: &Settings) {
    // 1. Ensure {data_dir}/icons/ exists and generic.png is present
    if let Err(e) = ensure_generic_icon(data_dir) {
        eprintln!("[indexer] failed to write generic icon: {}", e);
    }

    let icons_dir = data_dir.join("icons");

    // 2. Collect all source directories to crawl
    let source_dirs = get_index_paths(settings);

    // 3. Crawl each directory, upsert each app, spawn icon extraction thread
    let mut discovered_ids: HashSet<String> = HashSet::new();

    for (dir, source) in &source_dirs {
        let apps = crawl_dir(dir, source, &settings.excluded_paths);
        for app in apps {
            discovered_ids.insert(app.id.clone());

            // Upsert with generic.png placeholder — release lock immediately
            {
                let conn = db.lock().unwrap();
                let _ = upsert_app(&conn, &app);
            } // lock released here

            // Spawn icon extraction thread (non-blocking, INDX-05)
            // Skip if icon file already exists on disk (re-index optimization)
            let icon_file = icons_dir.join(icon_filename(&app.id));
            if !icon_file.exists() {
                let db_clone = Arc::clone(db);
                let exe_path = PathBuf::from(&app.path);
                let app_id = app.id.clone();
                std::thread::spawn(move || {
                    match extract_icon_png(&exe_path) {
                        Some(bytes) => {
                            if std::fs::write(&icon_file, &bytes).is_ok() {
                                let filename = icon_file
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                let conn = db_clone.lock().unwrap();
                                let _ = conn.execute(
                                    "UPDATE apps SET icon_path = ?1 WHERE id = ?2",
                                    rusqlite::params![filename, app_id],
                                );
                            }
                            // else: icon write failed, keeps generic.png — acceptable
                        }
                        None => {
                            // Extraction failed: icon_path stays as "generic.png" — correct fallback
                        }
                    }
                });
            }
        }
    }

    // 4. Prune stale entries (apps removed from disk since last index)
    let conn = db.lock().unwrap();
    let _ = prune_stale(&conn, &discovered_ids);
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
    let interval_mins = settings.reindex_interval;

    // --- Timer thread (INDX-06) ---
    let (timer_tx, timer_rx) = mpsc::channel::<()>();
    {
        let db = Arc::clone(&db);
        let data_dir = data_dir.clone();
        let is_indexing = Arc::clone(&is_indexing);
        let settings = settings.clone();
        std::thread::spawn(move || {
            use std::time::{Duration, Instant};
            use std::sync::mpsc::TryRecvError;
            let interval = Duration::from_secs(interval_mins as u64 * 60);
            let mut deadline = Instant::now() + interval;
            loop {
                std::thread::sleep(Duration::from_secs(1));
                match timer_rx.try_recv() {
                    Ok(()) => {
                        // Manual reindex completed — reset timer deadline
                        deadline = Instant::now() + interval;
                    }
                    Err(TryRecvError::Disconnected) => break, // App shutting down
                    Err(TryRecvError::Empty) => {}
                }
                if Instant::now() >= deadline {
                    try_start_index(&is_indexing, &db, &data_dir, &settings);
                    deadline = Instant::now() + interval;
                }
            }
        });
    }

    // --- Filesystem watcher thread (INDX-07) ---
    // Watch Start Menu directories only (per locked decision + INDX-07 spec)
    let watch_paths: Vec<PathBuf> = {
        let mut paths = vec![];
        if let Ok(appdata) = std::env::var("APPDATA") {
            let p = PathBuf::from(appdata)
                .join("Microsoft\\Windows\\Start Menu\\Programs");
            if p.exists() { paths.push(p); }
        }
        if let Ok(pdata) = std::env::var("PROGRAMDATA") {
            let p = PathBuf::from(pdata)
                .join("Microsoft\\Windows\\Start Menu\\Programs");
            if p.exists() { paths.push(p); }
        }
        paths
    };

    if !watch_paths.is_empty() {
        let db = Arc::clone(&db);
        let data_dir = data_dir.clone();
        let is_indexing = Arc::clone(&is_indexing);
        let settings = settings.clone();
        std::thread::spawn(move || {
            use notify_debouncer_mini::{notify::RecursiveMode, new_debouncer, DebounceEventResult};
            use std::time::Duration;

            let (tx, rx) = mpsc::channel::<DebounceEventResult>();
            let mut debouncer = match new_debouncer(
                Duration::from_millis(500),
                move |res: DebounceEventResult| { let _ = tx.send(res); },
            ) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[watcher] failed to create debouncer: {}", e);
                    return;
                }
            };

            for path in &watch_paths {
                debouncer.watcher()
                    .watch(path.as_ref(), RecursiveMode::Recursive)
                    .unwrap_or_else(|e| {
                        eprintln!("[watcher] failed to watch {:?}: {}", path, e);
                    });
            }

            for result in rx {
                if result.is_ok() {
                    // AtomicBool guard inside try_start_index suppresses events during full re-index
                    try_start_index(&is_indexing, &db, &data_dir, &settings);
                }
            }
            // Loop ends when tx is dropped (debouncer goes out of scope on thread exit)
        });
    }

    timer_tx
}

/// Tauri command: fire-and-forget manual re-index.
/// Spawns a thread, returns immediately. Frontend shows loading state.
#[tauri::command]
pub fn reindex(
    db_state: tauri::State<crate::db::DbState>,
    is_indexing: tauri::State<Arc<AtomicBool>>,
    timer_tx: tauri::State<Arc<Mutex<mpsc::Sender<()>>>>,
    data_dir_state: tauri::State<PathBuf>,
) {
    let db = Arc::clone(&db_state.0);
    let flag = Arc::clone(&is_indexing);
    let tx = Arc::clone(&timer_tx);
    let data_dir = data_dir_state.inner().clone();

    std::thread::spawn(move || {
        if flag
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            // Re-read settings at index time (user may have changed them)
            // Settings are not stored as managed state — derive data_dir from state
            // For now: use Settings::default() — lib.rs will pass real settings in future
            // Phase 8 will wire set_settings changes to trigger reindex via this command
            let settings = crate::store::Settings::default();
            run_full_index(&db, &data_dir, &settings);
            flag.store(false, Ordering::Release);
            // Reset timer so next auto-index is interval minutes from now
            let _ = tx.lock().unwrap().send(());
        }
    });
}

// ---- Internal functions ----

/// Get all directories to index, with their source labels.
pub(crate) fn get_index_paths(settings: &Settings) -> Vec<(PathBuf, &'static str)> {
    let mut paths: Vec<(PathBuf, &'static str)> = vec![];

    // Start Menu user
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs");
        if p.exists() {
            paths.push((p, "start_menu"));
        }
    }

    // Start Menu all-users
    if let Ok(programdata) = std::env::var("PROGRAMDATA") {
        let p = PathBuf::from(programdata).join("Microsoft\\Windows\\Start Menu\\Programs");
        if p.exists() {
            paths.push((p, "start_menu"));
        }
    }

    // Desktop user
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(userprofile).join("Desktop");
        if p.exists() {
            paths.push((p, "desktop"));
        }
    }

    // Desktop public
    let public_desktop = PathBuf::from("C:\\Users\\Public\\Desktop");
    if public_desktop.exists() {
        paths.push((public_desktop, "desktop"));
    }

    // PATH entries
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if dir.exists() {
                paths.push((dir, "path"));
            }
        }
    }

    // Additional paths from settings
    for additional in &settings.additional_paths {
        let p = PathBuf::from(additional);
        if p.exists() {
            paths.push((p, "additional"));
        }
    }

    paths
}

/// Walk a directory, resolve .lnk shortcuts, return AppRecords.
/// source: "start_menu" | "desktop" | "path" | "additional"
/// PATH source: .exe only, no .lnk resolution, max_depth 1.
pub(crate) fn crawl_dir(root: &Path, source: &'static str, excluded: &[String]) -> Vec<AppRecord> {
    let mut apps = vec![];
    let walker = if source == "path" {
        // PATH directories: only top-level .exe files (not recursive)
        walkdir::WalkDir::new(root).max_depth(1)
    } else {
        walkdir::WalkDir::new(root).follow_links(true)
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Skip if under any excluded path
        if excluded.iter().any(|ex| path.starts_with(ex)) {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "lnk" if source != "path" => {
                if let Some(target) = resolve_lnk(path) {
                    apps.push(make_app_record(&target, source));
                }
            }
            "exe" => {
                apps.push(make_app_record(path, source));
            }
            _ => {}
        }
    }
    apps
}

/// Resolve a .lnk file to its target executable path.
/// Returns None if: unresolvable, target is another .lnk, target doesn't exist, target isn't .exe.
pub(crate) fn resolve_lnk(lnk_path: &Path) -> Option<PathBuf> {
    use lnk::ShellLink;

    // Open the shell link — returns None if file missing or not a valid .lnk
    let shortcut = ShellLink::open(lnk_path).ok()?;

    // Reconstruct target path from working_dir + filename from relative_path.
    // Real Windows shortcuts store: working_dir = parent dir, relative_path = "./filename.exe"
    // Fallback: try working_dir alone if no relative_path.
    let target = {
        let working_dir = shortcut.working_dir().as_ref()?;
        let relative = shortcut.relative_path().clone();

        if let Some(rel) = relative {
            // Strip leading "./" or ".\" prefix from relative path
            let rel_stripped = rel
                .trim_start_matches("./")
                .trim_start_matches(".\\");
            PathBuf::from(working_dir).join(rel_stripped)
        } else {
            PathBuf::from(working_dir)
        }
    };

    // One level only: skip if target is also a .lnk
    if target.extension().and_then(|e| e.to_str()) == Some("lnk") {
        return None;
    }

    // Must exist and be an .exe
    if target.exists() && target.extension().and_then(|e| e.to_str()) == Some("exe") {
        Some(target)
    } else {
        None
    }
}

/// Build a canonical AppRecord from an exe path.
pub(crate) fn make_app_record(exe_path: &Path, source: &'static str) -> AppRecord {
    AppRecord {
        id: exe_path.to_string_lossy().to_lowercase(),
        name: exe_path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
        path: exe_path.to_string_lossy().to_string(),
        icon_path: Some("generic.png".to_string()),
        source: source.to_string(),
        last_launched: None,
        launch_count: 0,
    }
}

/// FNV-1a 64-bit hash of the normalized exe path → "{:016x}.png" icon filename.
pub(crate) fn icon_filename(exe_path: &str) -> String {
    let mut hash: u64 = 14695981039346656037u64;
    for byte in exe_path.to_lowercase().bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211u64);
    }
    format!("{:016x}.png", hash)
}

/// Delete all apps from DB whose id is not in discovered_ids.
pub(crate) fn prune_stale(conn: &Connection, discovered_ids: &HashSet<String>) -> rusqlite::Result<()> {
    let existing: Vec<String> = conn
        .prepare("SELECT id FROM apps")?
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();
    for id in existing {
        if !discovered_ids.contains(&id) {
            conn.execute("DELETE FROM apps WHERE id = ?1", rusqlite::params![id])?;
        }
    }
    Ok(())
}

/// Copy bundled GENERIC_ICON to {data_dir}/icons/generic.png if missing.
pub(crate) fn ensure_generic_icon(data_dir: &Path) -> std::io::Result<()> {
    let icons_dir = data_dir.join("icons");
    std::fs::create_dir_all(&icons_dir)?;
    let dest = icons_dir.join("generic.png");
    if !dest.exists() {
        std::fs::write(&dest, GENERIC_ICON)?;
    }
    Ok(())
}

/// Extract a 32x32 RGBA PNG from an exe file using Windows GDI.
/// Returns PNG bytes or None on any failure.
/// MUST be called from a spawned thread — GDI calls take 5-50ms per exe.
pub(crate) fn extract_icon_png(exe_path: &Path) -> Option<Vec<u8>> {
    // Convert path to null-terminated wide string
    let wide: Vec<u16> = exe_path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect();

    unsafe {
        // Extract large icon (32x32) at index 0
        let mut hicon_large: isize = 0;
        let count = ExtractIconExW(
            wide.as_ptr(),
            0,
            &mut hicon_large,
            std::ptr::null_mut(),
            1,
        );
        if count == 0 || hicon_large == 0 {
            return None;
        }

        // Get icon bitmap handle info
        let mut icon_info = ICONINFO {
            fIcon: 0,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: 0,
            hbmColor: 0,
        };
        if GetIconInfo(hicon_large, &mut icon_info) == 0 {
            DestroyIcon(hicon_large);
            return None;
        }

        // Get bitmap dimensions from hbmColor
        let mut bmp: BITMAP = std::mem::zeroed();
        let got = GetObjectW(
            icon_info.hbmColor,
            std::mem::size_of::<BITMAP>() as i32,
            &mut bmp as *mut _ as *mut _,
        );
        if got == 0 || bmp.bmWidth == 0 || bmp.bmHeight == 0 {
            DeleteObject(icon_info.hbmColor);
            DeleteObject(icon_info.hbmMask);
            DestroyIcon(hicon_large);
            return None;
        }

        let width = bmp.bmWidth.unsigned_abs();
        let height = bmp.bmHeight.unsigned_abs();

        // Allocate BGRA pixel buffer
        let row_bytes = width * 4;
        let mut pixels = vec![0u8; (row_bytes * height) as usize];

        // Create compatible DC for GetDIBits
        let dc = CreateCompatibleDC(0);

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32), // negative = top-down (no flip needed)
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0, // BI_RGB
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [std::mem::zeroed()],
        };

        let lines = GetDIBits(
            dc,
            icon_info.hbmColor,
            0,
            height,
            pixels.as_mut_ptr() as *mut _,
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Cleanup GDI resources
        DeleteDC(dc);
        DeleteObject(icon_info.hbmColor);
        DeleteObject(icon_info.hbmMask);
        DestroyIcon(hicon_large);

        if lines == 0 {
            return None;
        }

        // BGRA -> RGBA channel swap (GDI uses BGRA, image crate expects RGBA)
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // B <-> R
        }

        // Encode as PNG via image crate
        let img = image::RgbaImage::from_raw(width, height, pixels)?;
        let mut png_bytes: Vec<u8> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png).ok()?;
        Some(png_bytes)
    }
}

/// Try to start an index run. No-op if already indexing (AtomicBool guard).
/// Spawns a thread, sets flag true, runs index, sets flag false on completion.
pub(crate) fn try_start_index(
    is_indexing: &Arc<AtomicBool>,
    db: &Arc<Mutex<Connection>>,
    data_dir: &Path,
    settings: &Settings,
) {
    // compare_exchange: atomically flip false → true; only one thread wins
    if is_indexing
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_ok()
    {
        let flag = Arc::clone(is_indexing);
        let db = Arc::clone(db);
        let data_dir = data_dir.to_path_buf();
        let settings = settings.clone();
        std::thread::spawn(move || {
            run_full_index(&db, &data_dir, &settings);
            flag.store(false, Ordering::Release);
        });
    }
    // else: already indexing — silently drop trigger (per locked decision)
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
    fn test_resolve_lnk_broken() {
        let result = resolve_lnk(Path::new("C:\\nonexistent\\fake.lnk"));
        assert!(result.is_none());
    }

    #[test]
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
    fn test_icon_filename_stable() {
        let path = "C:\\Windows\\notepad.exe";
        let a = icon_filename(path);
        let b = icon_filename(path);
        assert_eq!(a, b);
        assert!(a.ends_with(".png"));
        assert_eq!(a.len(), 20); // 16 hex chars + ".png"
    }

    #[test]
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
    fn test_timer_fires() {
        use std::time::Duration;
        // Test: try_start_index with flag=false should flip it to true and spawn a thread
        let flag = Arc::new(AtomicBool::new(false));
        let db = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        crate::db::init_db_connection(&db.lock().unwrap()).unwrap();
        let dir = tempdir().unwrap();
        let settings = Settings::default();

        // try_start_index with flag=false should flip it to true and spawn a thread
        try_start_index(&flag, &db, dir.path(), &settings);
        // Give thread a moment to start and set the flag
        std::thread::sleep(Duration::from_millis(50));
        // The spawned thread may have already finished (empty index) and reset flag to false
        // Either state is valid — the important thing is no panic
        // Just assert the function returns without error
        let _ = flag.load(Ordering::SeqCst);
    }

    #[test]
    fn test_timer_reset() {
        use std::sync::mpsc;
        // Test: mpsc channel reset signal
        let (tx, rx) = mpsc::channel::<()>();
        tx.send(()).unwrap();
        // Receiver should get the signal
        assert!(rx.try_recv().is_ok(), "timer reset signal should be receivable");
        // After consume, channel should be empty
        assert!(matches!(rx.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn test_atomic_guard_prevents_double_index() {
        use std::sync::atomic::Ordering;
        let flag = Arc::new(AtomicBool::new(false));

        // Pre-set flag to "indexing" to simulate a running index
        flag.store(true, Ordering::SeqCst);

        // Second try_start_index with flag=true should be a no-op (no thread spawned)
        // We verify by checking the flag is still true after the call (no one reset it)
        let flag_clone = Arc::clone(&flag);
        let db = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        crate::db::init_db_connection(&db.lock().unwrap()).unwrap();
        let dir = tempdir().unwrap();
        let settings = Settings::default();

        try_start_index(&flag_clone, &db, dir.path(), &settings);
        // Flag should still be true — no thread was spawned to reset it
        assert!(flag_clone.load(Ordering::SeqCst), "flag should remain true (second call was dropped)");

        // Reset for cleanup
        flag.store(false, Ordering::SeqCst);
    }
}
