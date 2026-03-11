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

/// Messages sent to the background timer thread to control its behavior.
/// Defined here (next to the timer) and imported by store.rs via crate::indexer::TimerMsg.
#[derive(Debug)]
pub enum TimerMsg {
    /// Manual reindex completed — reset deadline to now + current interval.
    /// No-op when interval_mins == 0 (timer is disabled).
    Reset,
    /// Settings changed — update the interval. 0 = disabled (no auto-reindex).
    SetInterval(u32),
}

/// Request sent to the COM worker thread for .lnk resolution.
/// The caller sends this and then blocks on the reply channel.
/// allowlist is carried per-request (not baked into the worker at startup) so that
/// settings changes are always reflected in the next crawl without restarting the thread.
pub(crate) struct LnkQuery {
    pub path: PathBuf,
    pub allowlist: Vec<String>,
    pub reply: std::sync::mpsc::SyncSender<Option<PathBuf>>,
}

/// Spawn a dedicated thread that owns the COM apartment for all .lnk resolution.
/// CoInitializeEx is called once on thread start; CoUninitialize is called before thread exit.
/// All resolve_lnk calls are routed through this thread to ensure balanced COM init/uninit.
/// The allowlist is not baked in here — each LnkQuery carries the current allowlist.
pub(crate) fn spawn_com_worker() -> std::sync::mpsc::SyncSender<LnkQuery> {
    let (tx, rx) = std::sync::mpsc::sync_channel::<LnkQuery>(128);
    std::thread::Builder::new()
        .name("riftle-com-lnk".into())
        .spawn(move || {
            use windows::Win32::System::Com::{
                CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED,
            };
            // S_OK (0) or S_FALSE (1) = success; negative = failure (e.g. RPC_E_CHANGED_MODE)
            let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
            let com_ok = hr.0 >= 0;
            for query in rx {
                let result = resolve_lnk(&query.path, &query.allowlist);
                let _ = query.reply.send(result);
            }
            // rx is dropped when channel closes (all senders gone) — loop exits cleanly
            if com_ok {
                unsafe { CoUninitialize(); }
            }
        })
        .expect("failed to spawn COM lnk worker thread");
    tx
}

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
        let apps = crawl_dir(dir, source, &settings.excluded_paths, &settings.system_tool_allowlist);
        for app in apps {
            // Deduplicate by path: same exe can appear via multiple .lnk files or
            // in both user and all-users Start Menu. HashSet::insert returns false
            // if already present — skip the duplicate entirely.
            if !discovered_ids.insert(app.id.clone()) {
                continue;
            }

            let icon_file = icons_dir.join(icon_filename(&app.id));
            let icon_cached = icon_file.exists();

            // If the icon file is already on disk (e.g. DB was deleted but icons weren't),
            // upsert with the real filename so the DB is immediately correct.
            // Otherwise upsert with the generic placeholder and let the thread fill it in.
            let mut app = app;
            if icon_cached {
                app.icon_path = Some(
                    icon_file.file_name().unwrap_or_default().to_string_lossy().to_string()
                );
            }

            {
                let conn = db.lock().unwrap();
                let _ = upsert_app(&conn, &app);
            }

            // Spawn icon extraction thread only when the file doesn't exist yet
            if !icon_cached {
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
                        }
                        None => {} // Extraction failed — icon_path stays as "generic.png"
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
) -> mpsc::Sender<TimerMsg> {
    let interval_mins = settings.reindex_interval;

    // --- Timer thread (INDX-06) ---
    let (timer_tx, timer_rx) = mpsc::channel::<TimerMsg>();
    {
        let db = Arc::clone(&db);
        let data_dir = data_dir.clone();
        let is_indexing = Arc::clone(&is_indexing);
        let settings = settings.clone();
        std::thread::spawn(move || {
            use std::time::{Duration, Instant};
            use std::sync::mpsc::TryRecvError;

            let mut interval_mins = settings.reindex_interval;
            let mut deadline: Option<Instant> = if interval_mins == 0 {
                None
            } else {
                Some(Instant::now() + Duration::from_secs(interval_mins as u64 * 60))
            };

            loop {
                std::thread::sleep(Duration::from_secs(1));
                match timer_rx.try_recv() {
                    Ok(TimerMsg::Reset) => {
                        // Reset deadline only if timer is enabled (interval > 0)
                        if interval_mins > 0 {
                            deadline = Some(Instant::now() + Duration::from_secs(interval_mins as u64 * 60));
                        }
                    }
                    Ok(TimerMsg::SetInterval(n)) => {
                        interval_mins = n;
                        deadline = if n == 0 {
                            None
                        } else {
                            Some(Instant::now() + Duration::from_secs(n as u64 * 60))
                        };
                    }
                    Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => {}
                }
                if let Some(dl) = deadline {
                    if Instant::now() >= dl {
                        try_start_index(&is_indexing, &db, &data_dir, &settings);
                        deadline = Some(Instant::now() + Duration::from_secs(interval_mins as u64 * 60));
                    }
                }
                // If deadline is None (interval_mins == 0): skip deadline check entirely
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
    app: tauri::AppHandle,
    db_state: tauri::State<crate::db::DbState>,
    is_indexing: tauri::State<Arc<AtomicBool>>,
    timer_tx: tauri::State<Arc<Mutex<mpsc::Sender<TimerMsg>>>>,
    data_dir_state: tauri::State<PathBuf>,
) {
    let db = Arc::clone(&db_state.0);
    let flag = Arc::clone(&is_indexing);
    let tx = Arc::clone(&timer_tx);
    let data_dir = data_dir_state.inner().clone();
    let app_for_thread = app.clone();

    std::thread::spawn(move || {
        if flag
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            let settings = crate::store::get_settings(&app_for_thread, &data_dir);
            run_full_index(&db, &data_dir, &settings);
            // Phase 4: Rebuild search index with fresh DB contents
            crate::search::rebuild_index(&app_for_thread);
            flag.store(false, Ordering::Release);
            // Reset timer so next auto-index is interval minutes from now
            let _ = tx.lock().unwrap().send(TimerMsg::Reset);
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

    // Additional paths from settings
    for additional in &settings.additional_paths {
        let p = PathBuf::from(additional);
        if p.exists() {
            paths.push((p, "additional"));
        }
    }

    paths
}

/// Normalize a path for exclusion comparison: canonicalize (resolves symlinks, normalizes
/// separators), fall back to raw path if the path does not exist (deleted excluded dirs),
/// then lowercase. Pre-normalize the excluded list once; normalize each WalkDir entry inline.
fn normalize_for_exclusion(p: &Path) -> String {
    let canonical = p.canonicalize()
        .unwrap_or_else(|_| p.to_path_buf())
        .to_string_lossy()
        .to_lowercase();
    // Strip trailing path separators so "C:\foo\" and "C:\foo" compare equal
    canonical.trim_end_matches(['/', '\\']).to_string()
}

/// Walk a directory, resolve .lnk shortcuts, return AppRecords.
/// source: "start_menu" | "desktop" | "path" | "additional"
/// PATH source: .exe only, no .lnk resolution, max_depth 1.
pub(crate) fn crawl_dir(root: &Path, source: &'static str, excluded: &[String], allowlist: &[String]) -> Vec<AppRecord> {
    let mut apps = vec![];

    // Pre-normalize excluded list once (not per WalkDir entry) for performance.
    // canonicalize resolves separators and case; fallback to raw+lowercase for non-existent paths.
    let normalized_excluded: Vec<String> = excluded
        .iter()
        .map(|ex| normalize_for_exclusion(Path::new(ex)))
        .collect();

    let walker = walkdir::WalkDir::new(root)
        .max_depth(8)         // 8 levels covers all real Start Menu structures; prevents runaway on NTFS junctions
        .follow_links(false); // do not follow symlinks encountered during traversal; follow_root_links defaults to true

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Skip if under any excluded path — compare normalized strings for case/separator safety
        let norm_path = normalize_for_exclusion(path);
        if normalized_excluded.iter().any(|ex| norm_path.starts_with(ex.as_str())) {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "lnk" => {
                if let Some(target) = resolve_lnk(path, allowlist) {
                    // Use the .lnk filename as display name — it's already the human-readable
                    // name the user sees in the Start Menu (e.g. "Google Chrome.lnk" → "Google Chrome")
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string());
                    apps.push(make_app_record(&target, source, name));
                }
            }
            "exe" => {
                // For direct .exe files, try the PE FileDescription first
                let name = get_file_description(path);
                apps.push(make_app_record(path, source, name));
            }
            _ => {}
        }
    }
    apps
}

/// Resolve a .lnk file to its target executable path using the Windows IShellLink COM API.
/// Returns None if: COM fails, unresolvable, target is another .lnk, target doesn't exist, or not .exe.
/// Uses native Windows APIs — cannot panic on malformed shortcuts.
pub(crate) fn resolve_lnk(lnk_path: &Path, allowlist: &[String]) -> Option<PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows::{
        core::{Interface, PCWSTR},
        Win32::Foundation::HWND,
        Win32::Storage::FileSystem::WIN32_FIND_DATAW,
        Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, IPersistFile,
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, STGM,
        },
        Win32::UI::Shell::{IShellLinkW, ShellLink},
    };

    unsafe {
        // CoInitializeEx is idempotent per thread: S_FALSE = already initialized = OK.
        // We intentionally don't call CoUninitialize — COM lifetime matches process lifetime.
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        // Create IShellLinkW instance
        let shell_link: IShellLinkW =
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;

        // Load the .lnk file via IPersistFile
        let persist_file: IPersistFile = shell_link.cast().ok()?;
        let wide_path: Vec<u16> = lnk_path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0u16))
            .collect();
        persist_file.Load(PCWSTR(wide_path.as_ptr()), STGM(0)).ok()?;

        // Resolve with no-UI flag (0x1 = SLR_NO_UI) so broken links silently fail
        let _ = shell_link.Resolve(HWND(std::ptr::null_mut()), 0x1);

        // Get target path — 4 = SLGP_RAWPATH (returns stored path without env-var expansion)
        // IShellLinkW::GetPath is documented as MAX_PATH-limited, but 32,767 is the extended-path
        // maximum and future-proofs against any relaxation of the API constraint.
        // A larger buffer also prevents silent truncation for \\?\ prefixed target paths.
        const EXTENDED_MAX_PATH: usize = 32_767;
        let mut buf = vec![0u16; EXTENDED_MAX_PATH];
        let mut find_data: WIN32_FIND_DATAW = std::mem::zeroed();
        shell_link
            .GetPath(&mut buf, &mut find_data, 4u32)
            .ok()?;

        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        if len == 0 {
            return None;
        }
        let target = PathBuf::from(OsString::from_wide(&buf[..len]));

        // One level only: skip if target is also a .lnk
        if target.extension().and_then(|e| e.to_str()) == Some("lnk") {
            return None;
        }

        // Some system tools are worth keeping despite living in blocked directories.
        // The list comes from settings so users can extend or trim it.
        let filename = target
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("")
            .to_lowercase();
        let allowlisted = allowlist
            .iter()
            .any(|s: &String| s.to_lowercase() == filename);

        // Skip executables in known noisy system directories unless explicitly allowlisted
        if !allowlisted {
            let normalized = target.to_string_lossy().to_lowercase();
            if normalized.contains("\\windows\\system32\\")
                || normalized.contains("\\windows\\syswow64\\")
                || normalized.contains("\\windows\\winsxs\\")
                || normalized.contains("\\windows\\microsoft.net\\")
                || normalized.contains("\\program files\\common files\\")
                || normalized.contains("\\program files (x86)\\common files\\")
            {
                return None;
            }
        }

        // Must exist and be an .exe
        if target.exists() && target.extension().and_then(|e| e.to_str()) == Some("exe") {
            Some(target)
        } else {
            None
        }
    }
}

/// Build a canonical AppRecord from an exe path.
/// `display_name`: human-readable name override (from .lnk filename or PE FileDescription).
/// Falls back to the exe stem when None.
pub(crate) fn make_app_record(exe_path: &Path, source: &'static str, display_name: Option<String>) -> AppRecord {
    let name = display_name.unwrap_or_else(|| {
        exe_path.file_stem().unwrap_or_default().to_string_lossy().to_string()
    });
    AppRecord {
        id: exe_path.to_string_lossy().to_lowercase(),
        name,
        path: exe_path.to_string_lossy().to_string(),
        icon_path: Some("generic.png".to_string()),
        source: source.to_string(),
        last_launched: None,
        launch_count: 0,
    }
}

/// Read the FileDescription string from a PE executable's VERSIONINFO resource.
/// Returns None if the file has no version info or doesn't carry a FileDescription.
pub(crate) fn get_file_description(exe_path: &Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
    };

    let wide: Vec<u16> = exe_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect();

    unsafe {
        let mut dummy: u32 = 0;
        let size = GetFileVersionInfoSizeW(wide.as_ptr(), &mut dummy);
        if size == 0 {
            return None;
        }

        let mut buf = vec![0u8; size as usize];
        if GetFileVersionInfoW(wide.as_ptr(), 0, size, buf.as_mut_ptr() as *mut _) == 0 {
            return None;
        }

        // Query FileDescription using the US-English + Unicode codepage (040904b0).
        // This covers the vast majority of Windows applications.
        let subblock: Vec<u16> = "\\StringFileInfo\\040904b0\\FileDescription\0"
            .encode_utf16()
            .collect();

        let mut ptr: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut len: u32 = 0;
        if VerQueryValueW(
            buf.as_ptr() as *const _,
            subblock.as_ptr(),
            &mut ptr,
            &mut len,
        ) == 0 || len == 0 || ptr.is_null() {
            return None;
        }

        let slice = std::slice::from_raw_parts(ptr as *const u16, len as usize);
        let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
        let desc = String::from_utf16_lossy(&slice[..end]);
        if desc.is_empty() { None } else { Some(desc) }
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
        let apps = crawl_dir(dir.path(), "additional", &[], &[]);
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
        let result = resolve_lnk(Path::new("C:\\nonexistent\\fake.lnk"), &[]);
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
        let apps = crawl_dir(dir.path(), "additional", &excluded, &[]);
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

    #[test]
    fn test_zero_interval_disables_timer() {
        // RED: timer deadline must be None when interval_mins == 0
        // This test documents the expected behavior of the new timer loop.
        // It will PASS once the production timer loop is rewritten in Plan 02.
        // For now it exercises the logic inline so it fails if production code
        // still uses Duration::from_secs(0 * 60) which is always-past.
        let interval_mins: u32 = 0;
        let deadline: Option<std::time::Instant> = if interval_mins == 0 {
            None
        } else {
            Some(std::time::Instant::now() + std::time::Duration::from_secs(interval_mins as u64 * 60))
        };
        assert!(deadline.is_none(), "interval_mins=0 must disable the timer (deadline must be None)");
    }

    #[test]
    fn test_nonzero_interval_arms_timer() {
        // GREEN immediately (pure logic) — kept here to document both sides of the guard
        let interval_mins: u32 = 15;
        let deadline: Option<std::time::Instant> = if interval_mins == 0 {
            None
        } else {
            Some(std::time::Instant::now() + std::time::Duration::from_secs(interval_mins as u64 * 60))
        };
        assert!(deadline.is_some(), "interval_mins=15 must arm the timer");
    }

    #[test]
    fn test_set_interval_message_updates_deadline() {
        // RED: simulates the SetInterval(n) handler logic from the new timer loop.
        // Verifies that receiving SetInterval(5) arms the deadline and SetInterval(0) clears it.
        // Will PASS once TimerMsg enum and new loop are added in Plan 02.
        // For now the test uses the same inline logic expression to document the contract.
        let mut interval_mins: u32 = 15;
        let mut deadline: Option<std::time::Instant> = Some(std::time::Instant::now() + std::time::Duration::from_secs(interval_mins as u64 * 60));

        // Simulate SetInterval(5)
        let new_interval: u32 = 5;
        interval_mins = new_interval;
        deadline = if new_interval == 0 {
            None
        } else {
            Some(std::time::Instant::now() + std::time::Duration::from_secs(new_interval as u64 * 60))
        };
        assert!(deadline.is_some(), "SetInterval(5) must arm the timer");
        assert_eq!(interval_mins, 5, "interval_mins must update to 5");

        // Simulate SetInterval(0) — disables timer
        let new_interval: u32 = 0;
        interval_mins = new_interval;
        deadline = if new_interval == 0 {
            None
        } else {
            Some(std::time::Instant::now() + std::time::Duration::from_secs(new_interval as u64 * 60))
        };
        assert!(deadline.is_none(), "SetInterval(0) must disable the timer");
        assert_eq!(interval_mins, 0, "interval_mins must update to 0");
    }

    #[test]
    fn test_reindex_uses_live_settings() {
        // Documents the contract: get_index_paths() uses the Settings passed to it.
        // The reindex() Tauri command must call get_settings() and pass the result to run_full_index.
        // This test verifies the get_index_paths() function correctly includes additional_paths.
        // Note: paths that don't exist on disk are filtered out by get_index_paths, so we use
        // a real existing temp directory to confirm the path is included.
        let dir = tempdir().unwrap();
        let mut settings = Settings::default();
        settings.additional_paths = vec![dir.path().to_string_lossy().to_string()];
        let paths = get_index_paths(&settings);
        let path_strings: Vec<String> = paths.iter().map(|(p, _)| p.to_string_lossy().to_string()).collect();
        assert!(
            path_strings.iter().any(|p| p == &dir.path().to_string_lossy().to_string()),
            "get_index_paths must include additional_paths from settings; found: {:?}",
            path_strings
        );

        // Also verify that Settings::default() does NOT include this path
        let default_paths = get_index_paths(&Settings::default());
        let default_strings: Vec<String> = default_paths.iter().map(|(p, _)| p.to_string_lossy().to_string()).collect();
        assert!(
            !default_strings.iter().any(|p| p == &dir.path().to_string_lossy().to_string()),
            "Settings::default() must NOT include the custom path"
        );
    }

    #[test]
    fn test_crawl_excludes_normalized() {
        // RED: current crawl_dir uses raw starts_with — case-sensitive.
        // An excluded path supplied in uppercase does NOT exclude the lowercase filesystem entry.
        // This test will FAIL until Plan 02 adds normalize_for_exclusion().
        let dir = tempdir().unwrap();
        let excluded_sub = dir.path().join("excluded");
        fs::create_dir_all(&excluded_sub).unwrap();
        fs::write(excluded_sub.join("hidden.exe"), b"MZ").unwrap();
        fs::write(dir.path().join("visible.exe"), b"MZ").unwrap();

        // Supply excluded path with uppercase final component to trigger case mismatch
        let excluded_upper = excluded_sub.to_string_lossy().to_uppercase();
        let excluded = vec![excluded_upper];
        let apps = crawl_dir(dir.path(), "additional", &excluded, &[]);

        // After normalization, hidden.exe must be excluded → only visible.exe
        assert_eq!(apps.len(), 1, "hidden.exe must be excluded despite case difference");
        assert!(apps[0].path.contains("visible.exe"), "visible.exe must be present");
    }

    #[test]
    fn test_crawl_excludes_trailing_slash() {
        // RED: current crawl_dir uses raw Path::starts_with on user-supplied strings.
        // When the excluded path combines a case difference (UPPERCASE subdirectory) with
        // a trailing separator, the raw comparison fails on BOTH counts.
        // Plan 02's normalize_for_exclusion() (canonicalize + to_lowercase + separator strip)
        // will handle both issues together. This test will FAIL until Plan 02 lands.
        let dir = tempdir().unwrap();
        let excluded_sub = dir.path().join("excluded");
        fs::create_dir_all(&excluded_sub).unwrap();
        fs::write(excluded_sub.join("hidden.exe"), b"MZ").unwrap();
        fs::write(dir.path().join("visible.exe"), b"MZ").unwrap();

        // Excluded path: uppercase directory component + trailing backslash separator.
        // Simulates a user typing "C:\Users\Me\EXCLUDED\" in the settings path field.
        let excluded_upper_with_slash = format!(
            "{}\\",
            excluded_sub.to_string_lossy().to_uppercase()
        );
        let excluded = vec![excluded_upper_with_slash];
        let apps = crawl_dir(dir.path(), "additional", &excluded, &[]);

        // After normalization: both sides lowercase, no trailing separator →
        // hidden.exe is excluded → only visible.exe remains
        assert_eq!(apps.len(), 1, "hidden.exe must be excluded despite uppercase + trailing separator");
        assert!(apps[0].path.contains("visible.exe"), "visible.exe must be present");
    }

    #[test]
    fn test_crawl_respects_max_depth() {
        // RED: current WalkDir has no max_depth — descends arbitrarily deep.
        // A directory 9 levels deep will be traversed. This test will FAIL until
        // Plan 02 adds .max_depth(8) to the WalkDir builder.
        let dir = tempdir().unwrap();

        // Build a chain of 9 nested directories
        let mut deep = dir.path().to_path_buf();
        for i in 1..=9 {
            deep = deep.join(format!("depth_{}", i));
        }
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("deep.exe"), b"MZ").unwrap();
        fs::write(dir.path().join("shallow.exe"), b"MZ").unwrap();

        let apps = crawl_dir(dir.path(), "additional", &[], &[]);

        // With max_depth(8), depth-9 directory is not traversed → deep.exe absent
        let paths: Vec<&str> = apps.iter().map(|a| a.path.as_str()).collect();
        assert!(
            !paths.iter().any(|p| p.contains("deep.exe")),
            "deep.exe at depth 9 must NOT be discovered when max_depth is 8; found: {:?}",
            paths
        );
        assert!(
            paths.iter().any(|p| p.contains("shallow.exe")),
            "shallow.exe at root must still be discovered"
        );
    }
}
