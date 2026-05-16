// Phase 3: Windows application indexer
// crawl Start Menu, Desktop, PATH; .lnk resolution; icon extraction; background refresh
// Added UWP app discovery via shell:AppsFolder

use rusqlite::Connection;
use std::collections::HashSet;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

// windows crate for COM and Shell APIs
use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{HWND, SIZE};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER, STGM,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{
    IEnumShellItems, IShellItem, IShellItemImageFactory, SHCreateItemFromParsingName,
    SHGetKnownFolderItem, ShellLink, BHID_EnumItems, BHID_PropertyStore, FOLDERID_AppsFolder,
    IShellLinkW, KF_FLAG_DEFAULT, SIGDN_NORMALDISPLAY, SIIGBF_RESIZETOFIT,
};
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;

// windows-sys for GDI and lightweight shell info (maintaining existing patterns)
use windows_sys::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
    BITMAPINFOHEADER, DIB_RGB_COLORS,
};
use windows_sys::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL};
use windows_sys::Win32::UI::Shell::{
    ExtractIconExW, SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON,
    SHGFI_USEFILEATTRIBUTES,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

use crate::db::{upsert_app, AppRecord};
use crate::store::Settings;

// Generic icon bundled at compile time — path relative to this source file
static GENERIC_ICON: &[u8] = include_bytes!("../icons/generic.png");

/// Messages sent to the background timer thread to control its behavior.
#[derive(Debug)]
pub enum TimerMsg {
    /// Manual reindex completed — reset deadline to now + current interval.
    Reset,
    /// Settings changed — update the interval. 0 = disabled (no auto-reindex).
    SetInterval(u32),
}

/// Defines where an icon should be extracted from.
#[derive(Clone, Debug)]
pub enum IconSource {
    /// Standard file path (exe, lnk, ico).
    File(PathBuf),
    /// UWP Application User Model ID (AUMID).
    Uwp(String),
}

/// Request sent to the COM worker thread for .lnk resolution.
pub(crate) struct LnkQuery {
    pub path: PathBuf,
    pub allowlist: Vec<String>,
    pub reply: std::sync::mpsc::SyncSender<Option<PathBuf>>,
}

/// Spawn a dedicated thread that owns the COM apartment for all .lnk resolution.
pub(crate) fn spawn_com_worker() -> std::sync::mpsc::SyncSender<LnkQuery> {
    let (tx, rx) = std::sync::mpsc::sync_channel::<LnkQuery>(128);
    std::thread::Builder::new()
        .name("riftle-com-lnk".into())
        .spawn(move || {
            let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
            let com_ok = hr.0 >= 0;
            for query in rx {
                let result = resolve_lnk(&query.path, &query.allowlist);
                let _ = query.reply.send(result);
            }
            if com_ok {
                unsafe {
                    CoUninitialize();
                }
            }
        })
        .expect("failed to spawn COM lnk worker thread");
    tx
}

// ---- Public API (called from lib.rs) ----

/// Run a full blocking index.
pub fn run_full_index(
    db: &Arc<Mutex<Connection>>,
    data_dir: &Path,
    settings: &Settings,
    com_tx: &std::sync::mpsc::SyncSender<LnkQuery>,
) {
    // Ensure COM is initialized for the current thread (needed for UWP discovery)
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };

    if let Err(e) = ensure_generic_icon(data_dir) {
        eprintln!("[indexer] failed to write generic icon: {}", e);
    }

    let icons_dir = data_dir.join("icons");
    let source_dirs = get_index_paths(settings);

    let mut discovered_ids: HashSet<String> = HashSet::new();
    let mut seen_start_menu_name_exe: HashSet<(String, String)> = HashSet::new();
    let mut seen_names: HashSet<String> = HashSet::new();

    let icon_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .thread_name(|i| format!("riftle-icon-{}", i))
        .build()
        .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().build().unwrap());

    // 1. High-priority: Standard sources (Start Menu, Desktop, Additional)
    // These are processed first to ensure specialized shortcuts (like Steam/Epic games)
    // claim their names and custom icons before UWP fallback.
    for (dir, source) in &source_dirs {
        let apps = crawl_dir(
            dir,
            source,
            &settings.excluded_paths,
            &settings.system_tool_allowlist,
            com_tx,
        );
        for (app, icon_source) in apps {
            process_index_entry(
                app,
                icon_source,
                source,
                &mut discovered_ids,
                &mut seen_start_menu_name_exe,
                &mut seen_names,
                &icons_dir,
                db,
                &icon_pool,
            );
        }
    }

    // 2. Fallback: UWP Apps (enumerate shell:AppsFolder)
    // Only adds apps that weren't already found in standard sources.
    let uwp_apps = crawl_apps_folder();
    for (app, icon_source) in uwp_apps {
        process_index_entry(
            app,
            icon_source,
            "uwp",
            &mut discovered_ids,
            &mut seen_start_menu_name_exe,
            &mut seen_names,
            &icons_dir,
            db,
            &icon_pool,
        );
    }

    // 3. Prune stale entries
    let conn = db.lock().unwrap();
    let _ = prune_stale(&conn, &discovered_ids);
}

/// Helper to process a discovered app, handle deduplication, and schedule icon extraction.
fn process_index_entry(
    app: AppRecord,
    icon_source: IconSource,
    source: &'static str,
    discovered_ids: &mut HashSet<String>,
    seen_start_menu_name_exe: &mut HashSet<(String, String)>,
    seen_names: &mut HashSet<String>,
    icons_dir: &Path,
    db: &Arc<Mutex<Connection>>,
    icon_pool: &rayon::ThreadPool,
) {
    if discovered_ids.contains(&app.id) {
        return;
    }

    // Start Menu: collapse the same app present in both user and all-users dirs.
    if source == "start_menu" && app.path.to_lowercase().ends_with(".lnk") {
        let icon_key = match &icon_source {
            IconSource::File(p) => p.to_string_lossy().to_lowercase(),
            IconSource::Uwp(a) => a.to_lowercase(),
        };
        let key = (app.name.to_lowercase(), icon_key);
        if !seen_start_menu_name_exe.insert(key) {
            return;
        }
    }

    // Deduplicate by name: skip if the same name was already indexed from an
    // earlier, higher-priority source.
    if seen_names.contains(&app.name.to_lowercase()) {
        return;
    }

    discovered_ids.insert(app.id.clone());
    seen_names.insert(app.name.to_lowercase());

    let icon_file = icons_dir.join(icon_filename(&app.id));
    let icon_cached = icon_file.exists();

    let mut app = app;
    if icon_cached {
        app.icon_path = Some(
            icon_file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );
    }

    {
        let conn = db.lock().unwrap();
        let _ = upsert_app(&conn, &app);
    }

    if !icon_cached {
        let db_clone = Arc::clone(db);
        let app_id = app.id.clone();
        let icon_file_clone = icon_file.clone();
        icon_pool.spawn(move || {
            if let Some(bytes) = extract_icon_png(&icon_source) {
                if std::fs::write(&icon_file_clone, &bytes).is_ok() {
                    let filename = icon_file_clone
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
        });
    }
}

/// Spawn background timer thread and filesystem watcher thread.
pub fn start_background_tasks(
    db: Arc<Mutex<Connection>>,
    app: tauri::AppHandle,
    app_count: i64,
    data_dir: PathBuf,
    settings: &Settings,
    is_indexing: Arc<AtomicBool>,
    com_tx: std::sync::mpsc::SyncSender<LnkQuery>,
) -> mpsc::Sender<TimerMsg> {
    let (timer_tx, timer_rx) = mpsc::channel::<TimerMsg>();

    {
        let app_deferred = app.clone();
        let db_deferred = Arc::clone(&db);
        let data_dir_deferred = data_dir.clone();
        let settings_deferred = settings.clone();
        let is_indexing_deferred = Arc::clone(&is_indexing);
        let com_tx_deferred = com_tx.clone();
        let timer_tx_for_deferred = timer_tx.clone();
        std::thread::spawn(move || {
            if app_count > 0 {
                std::thread::sleep(std::time::Duration::from_secs(30));
            }
            try_start_index(
                &app_deferred,
                &is_indexing_deferred,
                &db_deferred,
                &data_dir_deferred,
                &settings_deferred,
                com_tx_deferred,
            );
            let _ = timer_tx_for_deferred.send(TimerMsg::Reset);
        });
    }

    {
        let db = Arc::clone(&db);
        let data_dir = data_dir.clone();
        let is_indexing = Arc::clone(&is_indexing);
        let settings = settings.clone();
        let com_tx_timer = com_tx.clone();
        let app = app.clone();
        std::thread::spawn(move || {
            use std::sync::mpsc::TryRecvError;
            use std::time::{Duration, Instant};

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
                        if interval_mins > 0 {
                            deadline = Some(
                                Instant::now() + Duration::from_secs(interval_mins as u64 * 60),
                            );
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
                        try_start_index(
                            &app,
                            &is_indexing,
                            &db,
                            &data_dir,
                            &settings,
                            com_tx_timer.clone(),
                        );
                        deadline =
                            Some(Instant::now() + Duration::from_secs(interval_mins as u64 * 60));
                    }
                }
            }
        });
    }

    let watch_paths: Vec<PathBuf> = {
        let mut paths = vec![];
        if let Ok(appdata) = std::env::var("APPDATA") {
            let p = PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs");
            if p.exists() {
                paths.push(p);
            }
        }
        if let Ok(pdata) = std::env::var("PROGRAMDATA") {
            let p = PathBuf::from(pdata).join("Microsoft\\Windows\\Start Menu\\Programs");
            if p.exists() {
                paths.push(p);
            }
        }
        paths
    };

    if !watch_paths.is_empty() {
        let db = Arc::clone(&db);
        let data_dir = data_dir.clone();
        let is_indexing = Arc::clone(&is_indexing);
        let settings = settings.clone();
        let app = app.clone();
        std::thread::spawn(move || {
            use notify_debouncer_mini::{
                new_debouncer, notify::RecursiveMode, DebounceEventResult,
            };
            use std::time::Duration;

            let (tx, rx) = mpsc::channel::<DebounceEventResult>();
            let mut debouncer = match new_debouncer(
                Duration::from_millis(500),
                move |res: DebounceEventResult| {
                    let _ = tx.send(res);
                },
            ) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[watcher] failed to create debouncer: {}", e);
                    return;
                }
            };

            for path in &watch_paths {
                let _ = debouncer.watcher().watch(path.as_ref(), RecursiveMode::Recursive);
            }

            for result in rx {
                if result.is_ok() {
                    try_start_index(&app, &is_indexing, &db, &data_dir, &settings, com_tx.clone());
                }
            }
        });
    }

    timer_tx
}

/// Tauri command: fire-and-forget manual re-index.
#[tauri::command]
pub fn reindex(
    app: tauri::AppHandle,
    db_state: tauri::State<crate::db::DbState>,
    is_indexing: tauri::State<Arc<AtomicBool>>,
    timer_tx: tauri::State<Arc<Mutex<mpsc::Sender<TimerMsg>>>>,
    data_dir_state: tauri::State<PathBuf>,
    com_tx_state: tauri::State<Arc<std::sync::mpsc::SyncSender<LnkQuery>>>,
) {
    let db = Arc::clone(&db_state.0);
    let flag = Arc::clone(&is_indexing);
    let tx = Arc::clone(&timer_tx);
    let data_dir = data_dir_state.inner().clone();
    let app_for_thread = app.clone();
    let com_tx = Arc::clone(&com_tx_state);

    std::thread::spawn(move || {
        if flag
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            let settings = crate::store::get_settings(&app_for_thread, &data_dir);
            run_full_index(&db, &data_dir, &settings, &com_tx);
            crate::search::rebuild_index(&app_for_thread);
            flag.store(false, Ordering::Release);
            let _ = tx.lock().unwrap().send(TimerMsg::Reset);
        }
    });
}

// ---- Internal functions ----

/// Enumerate UWP apps using shell:AppsFolder.
pub(crate) fn crawl_apps_folder() -> Vec<(AppRecord, IconSource)> {
    let mut apps = vec![];
    unsafe {
        let folder: IShellItem = match SHGetKnownFolderItem(&FOLDERID_AppsFolder, KF_FLAG_DEFAULT, None) {
            Ok(f) => f,
            Err(_) => return apps,
        };

        let enum_items: IEnumShellItems = match folder.BindToHandler(None, &BHID_EnumItems) {
            Ok(e) => e,
            Err(_) => return apps,
        };

        loop {
            let mut item_opt: [Option<IShellItem>; 1] = [None];
            let mut fetched = 0;
            if enum_items.Next(&mut item_opt, Some(&mut fetched)).is_err() || fetched == 0 {
                break;
            }
            let item = match item_opt[0].take() {
                Some(i) => i,
                None => break,
            };

            let display_name = item.GetDisplayName(SIGDN_NORMALDISPLAY).ok()
                .map(|pwstr| {
                    let s = pwstr.to_string().unwrap_or_default();
                    windows::Win32::System::Com::CoTaskMemFree(Some(pwstr.0 as *const _));
                    s
                })
                .unwrap_or_default();

            let property_store: IPropertyStore = match item.BindToHandler(None, &BHID_PropertyStore) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let aumid_var = match property_store.GetValue(&PKEY_AppUserModel_ID) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let aumid = aumid_var.to_string();
            if aumid.is_empty() {
                continue;
            }

            let record = AppRecord {
                id: format!("uwp:{}", aumid).to_lowercase(),
                name: display_name,
                path: format!("shell:AppsFolder\\{}", aumid),
                icon_path: Some("generic.png".to_string()),
                source: "apps_folder".to_string(),
                last_launched: None,
                launch_count: 0,
            };

            apps.push((record, IconSource::Uwp(aumid)));
        }
    }
    apps
}

pub(crate) fn get_index_paths(settings: &Settings) -> Vec<(PathBuf, &'static str)> {
    let mut paths: Vec<(PathBuf, &'static str)> = vec![];

    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs");
        if p.exists() {
            paths.push((p, "start_menu"));
        }
    }

    if let Ok(programdata) = std::env::var("PROGRAMDATA") {
        let p = PathBuf::from(programdata).join("Microsoft\\Windows\\Start Menu\\Programs");
        if p.exists() {
            paths.push((p, "start_menu"));
        }
    }

    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(userprofile).join("Desktop");
        if p.exists() {
            paths.push((p, "desktop"));
        }
    }

    let public_desktop = PathBuf::from("C:\\Users\\Public\\Desktop");
    if public_desktop.exists() {
        paths.push((public_desktop, "desktop"));
    }

    for additional in &settings.additional_paths {
        let p = PathBuf::from(additional);
        if p.exists() {
            paths.push((p, "additional"));
        }
    }

    paths
}

fn normalize_for_exclusion(p: &Path) -> String {
    let canonical = p
        .canonicalize()
        .unwrap_or_else(|_| p.to_path_buf())
        .to_string_lossy()
        .to_lowercase();
    canonical.trim_end_matches(['/', '\\']).to_string()
}

fn is_generic_launcher_stem(stem: &str) -> bool {
    matches!(
        stem.to_lowercase().as_str(),
        "steam"
            | "epicgameslauncher"
            | "epic games launcher"
            | "galaxyclient"
            | "gog galaxy"
            | "battle.net"
            | "battlenet"
            | "uplay"
            | "ubisoft connect"
            | "origin"
            | "ea app"
    )
}

fn is_protocol_url_shortcut(path: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    content.contains("://")
}

fn parse_url_icon_file(path: &Path) -> Option<PathBuf> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("IconFile=") {
            let p = PathBuf::from(rest.trim());
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}

pub(crate) fn crawl_dir(
    root: &Path,
    source: &'static str,
    excluded: &[String],
    allowlist: &[String],
    com_tx: &std::sync::mpsc::SyncSender<LnkQuery>,
) -> Vec<(AppRecord, IconSource)> {
    let mut apps = vec![];
    let normalized_excluded: Vec<String> = excluded
        .iter()
        .map(|ex| normalize_for_exclusion(Path::new(ex)))
        .collect();

    let walker = walkdir::WalkDir::new(root)
        .max_depth(8)
        .follow_links(false);

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let norm_path = normalize_for_exclusion(path);
        if normalized_excluded
            .iter()
            .any(|ex| norm_path.starts_with(ex.as_str()))
        {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "lnk" => {
                let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(1);
                let _ = com_tx.send(LnkQuery {
                    path: path.to_path_buf(),
                    allowlist: allowlist.to_vec(),
                    reply: reply_tx,
                });
                if let Some(target) = reply_rx.recv().unwrap_or(None) {
                    let lnk_path = path.to_path_buf();
                    let stem = lnk_path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
                    let name = if is_generic_launcher_stem(stem) {
                        lnk_path.parent()
                            .filter(|parent| normalize_for_exclusion(parent) != normalize_for_exclusion(root))
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| stem.to_string())
                    } else {
                        stem.to_string()
                    };
                    let record = AppRecord {
                        id: lnk_path.to_string_lossy().to_lowercase(),
                        name,
                        path: lnk_path.to_string_lossy().to_string(),
                        icon_path: Some("generic.png".to_string()),
                        source: source.to_string(),
                        last_launched: None,
                        launch_count: 0,
                    };
                    apps.push((record, IconSource::File(target)));
                }
            }
            "url" => {
                if !is_protocol_url_shortcut(path) {
                    continue;
                }
                let url_path = path.to_path_buf();
                let stem = url_path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
                let name = if is_generic_launcher_stem(stem) {
                    url_path.parent()
                        .filter(|p| normalize_for_exclusion(p) != normalize_for_exclusion(root))
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| stem.to_string())
                } else {
                    stem.to_string()
                };
                let icon_src = parse_url_icon_file(path).unwrap_or_else(|| url_path.clone());
                let record = AppRecord {
                    id: url_path.to_string_lossy().to_lowercase(),
                    name,
                    path: url_path.to_string_lossy().to_string(),
                    icon_path: Some("generic.png".to_string()),
                    source: source.to_string(),
                    last_launched: None,
                    launch_count: 0,
                };
                apps.push((record, IconSource::File(icon_src)));
            }
            "exe" => {
                let name = get_file_description(path);
                let record = make_app_record(path, source, name);
                apps.push((record, IconSource::File(path.to_path_buf())));
            }
            _ => {}
        }
    }
    apps
}

pub(crate) fn resolve_lnk(lnk_path: &Path, allowlist: &[String]) -> Option<PathBuf> {
    unsafe {
        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let persist_file: IPersistFile = shell_link.cast().ok()?;
        let wide_path: Vec<u16> = lnk_path.as_os_str().encode_wide().chain(std::iter::once(0u16)).collect();
        persist_file.Load(PCWSTR(wide_path.as_ptr()), STGM(0)).ok()?;
        let _ = shell_link.Resolve(HWND(std::ptr::null_mut()), 0x1);

        const EXTENDED_MAX_PATH: usize = 32_767;
        let mut buf = vec![0u16; EXTENDED_MAX_PATH];
        let mut find_data: WIN32_FIND_DATAW = std::mem::zeroed();
        shell_link.GetPath(&mut buf, &mut find_data, 4u32).ok()?;

        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        if len == 0 {
            return Some(lnk_path.to_path_buf());
        }
        let target = PathBuf::from(OsString::from_wide(&buf[..len]));

        if target.extension().and_then(|e| e.to_str()) == Some("lnk") {
            return None;
        }

        let filename = target.file_name().and_then(|f| f.to_str()).unwrap_or("").to_lowercase();
        let allowlisted = allowlist.iter().any(|s| s.to_lowercase() == filename);

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

        if target.exists() && target.extension().and_then(|e| e.to_str()) == Some("exe") {
            Some(target)
        } else {
            None
        }
    }
}

pub(crate) fn make_app_record(
    exe_path: &Path,
    source: &'static str,
    display_name: Option<String>,
) -> AppRecord {
    let name = display_name.unwrap_or_else(|| {
        exe_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
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

pub(crate) fn get_file_description(exe_path: &Path) -> Option<String> {
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
        ) == 0
            || len == 0
            || ptr.is_null()
        {
            return None;
        }

        let slice = std::slice::from_raw_parts(ptr as *const u16, len as usize);
        let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
        let desc = String::from_utf16_lossy(&slice[..end]);
        if desc.is_empty() {
            None
        } else {
            Some(desc)
        }
    }
}

pub(crate) fn icon_filename(exe_path: &str) -> String {
    let mut hash: u64 = 14695981039346656037u64;
    for byte in exe_path.to_lowercase().bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211u64);
    }
    format!("{:016x}.png", hash)
}

pub(crate) fn prune_stale(
    conn: &Connection,
    discovered_ids: &HashSet<String>,
) -> rusqlite::Result<()> {
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

pub(crate) fn ensure_generic_icon(data_dir: &Path) -> std::io::Result<()> {
    let icons_dir = data_dir.join("icons");
    std::fs::create_dir_all(&icons_dir)?;
    let dest = icons_dir.join("generic.png");
    if !dest.exists() {
        std::fs::write(&dest, GENERIC_ICON)?;
    }
    Ok(())
}

/// Extract a 32x32 RGBA PNG from an IconSource.
pub(crate) fn extract_icon_png(source: &IconSource) -> Option<Vec<u8>> {
    match source {
        IconSource::File(path) => {
            let wide: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0u16))
                .collect();

            unsafe {
                let mut hicon_large: isize = 0;
                let count = ExtractIconExW(wide.as_ptr(), 0, &mut hicon_large, std::ptr::null_mut(), 1);
                if count == 0 || hicon_large == 0 {
                    return extract_shell_icon_png(path, false);
                }
                icon_png_from_hicon(hicon_large)
            }
        }
        IconSource::Uwp(aumid) => {
            unsafe {
                let parsing_name = format!("shell:AppsFolder\\{}", aumid);
                let wide_path: Vec<u16> = parsing_name.encode_utf16().chain(std::iter::once(0)).collect();
                let item: IShellItem = match SHCreateItemFromParsingName(PCWSTR(wide_path.as_ptr()), None) {
                    Ok(i) => i,
                    Err(_) => return None,
                };
                let factory: IShellItemImageFactory = match item.cast() {
                    Ok(f) => f,
                    Err(_) => return None,
                };
                let hbitmap = match factory.GetImage(SIZE { cx: 32, cy: 32 }, SIIGBF_RESIZETOFIT) {
                    Ok(b) => b,
                    Err(_) => return None,
                };
                let res = icon_png_from_hbitmap(hbitmap.0 as isize);
                DeleteObject(hbitmap.0 as isize);
                res
            }
        }
    }
}

pub(crate) fn extract_shell_icon_png(path: &Path, is_directory: bool) -> Option<Vec<u8>> {
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect();

    unsafe {
        let mut file_info: SHFILEINFOW = std::mem::zeroed();
        let attributes = if is_directory {
            FILE_ATTRIBUTE_DIRECTORY
        } else {
            FILE_ATTRIBUTE_NORMAL
        };
        let result = SHGetFileInfoW(
            wide.as_ptr(),
            attributes,
            &mut file_info,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON | SHGFI_USEFILEATTRIBUTES,
        );

        if result == 0 || file_info.hIcon == 0 {
            return None;
        }

        icon_png_from_hicon(file_info.hIcon)
    }
}

/// Extract PNG bytes from an HBITMAP handle using GDI.
fn icon_png_from_hbitmap(hbitmap: isize) -> Option<Vec<u8>> {
    unsafe {
        let mut bmp: BITMAP = std::mem::zeroed();
        let got = GetObjectW(
            hbitmap,
            std::mem::size_of::<BITMAP>() as i32,
            &mut bmp as *mut _ as *mut _,
        );
        if got == 0 || bmp.bmWidth == 0 || bmp.bmHeight == 0 {
            return None;
        }

        let width = bmp.bmWidth.unsigned_abs();
        let height = bmp.bmHeight.unsigned_abs();
        let row_bytes = width * 4;
        let mut pixels = vec![0u8; (row_bytes * height) as usize];
        let dc = CreateCompatibleDC(0);

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0,
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
            hbitmap,
            0,
            height,
            pixels.as_mut_ptr() as *mut _,
            &mut bmi,
            DIB_RGB_COLORS,
        );

        DeleteDC(dc);

        if lines == 0 {
            return None;
        }

        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        let img = image::RgbaImage::from_raw(width, height, pixels)?;
        let mut png_bytes: Vec<u8> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png).ok()?;
        Some(png_bytes)
    }
}

fn icon_png_from_hicon(hicon: isize) -> Option<Vec<u8>> {
    unsafe {
        let mut icon_info = ICONINFO {
            fIcon: 0,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: 0,
            hbmColor: 0,
        };
        if GetIconInfo(hicon, &mut icon_info) == 0 {
            DestroyIcon(hicon);
            return None;
        }

        let res = icon_png_from_hbitmap(icon_info.hbmColor);

        DeleteObject(icon_info.hbmColor);
        DeleteObject(icon_info.hbmMask);
        DestroyIcon(hicon);

        res
    }
}

pub(crate) fn try_start_index(
    app: &tauri::AppHandle,
    is_indexing: &Arc<AtomicBool>,
    db: &Arc<Mutex<Connection>>,
    data_dir: &Path,
    settings: &Settings,
    com_tx: std::sync::mpsc::SyncSender<LnkQuery>,
) {
    if is_indexing
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_ok()
    {
        let app = app.clone();
        let flag = Arc::clone(is_indexing);
        let db = Arc::clone(db);
        let data_dir = data_dir.to_path_buf();
        let settings = settings.clone();
        std::thread::spawn(move || {
            run_full_index(&db, &data_dir, &settings, &com_tx);
            crate::search::rebuild_index(&app);
            flag.store(false, Ordering::Release);
        });
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::fs;
    use tempfile::tempdir;

    fn in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::init_db_connection(&conn).unwrap();
        conn
    }

    fn temp_dir_with_exe(name: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().unwrap();
        let exe_path = dir.path().join(name);
        fs::write(&exe_path, b"MZ").unwrap();
        (dir, exe_path)
    }

    fn test_com_tx() -> std::sync::mpsc::SyncSender<LnkQuery> {
        spawn_com_worker()
    }

    #[test]
    fn test_crawl_discovers_exe() {
        let (dir, _exe) = temp_dir_with_exe("foo.exe");
        let apps = crawl_dir(dir.path(), "additional", &[], &[], &test_com_tx());
        assert_eq!(apps.len(), 1);
        assert!(apps[0].0.path.ends_with("foo.exe"));
        match &apps[0].1 {
            IconSource::File(p) => assert!(p.to_string_lossy().ends_with("foo.exe")),
            _ => panic!("Expected IconSource::File"),
        }
    }

    #[test]
    fn test_resolve_lnk_broken() {
        let result = resolve_lnk(Path::new("C:\\nonexistent\\fake.lnk"), &[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_prune_stale() {
        let conn = in_memory_db();
        let app1 = AppRecord {
            id: "app1".to_string(),
            name: "App 1".to_string(),
            path: "C:\\app1.exe".to_string(),
            icon_path: None,
            source: "start_menu".to_string(),
            last_launched: None,
            launch_count: 0,
        };
        let app2 = AppRecord {
            id: "app2".to_string(),
            name: "App 2".to_string(),
            path: "C:\\app2.exe".to_string(),
            icon_path: None,
            source: "start_menu".to_string(),
            last_launched: None,
            launch_count: 0,
        };
        crate::db::upsert_app(&conn, &app1).unwrap();
        crate::db::upsert_app(&conn, &app2).unwrap();
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
        let apps = crawl_dir(dir.path(), "additional", &excluded, &[], &test_com_tx());
        assert_eq!(apps.len(), 1);
        assert!(apps[0].0.path.contains("visible.exe"));
    }

    #[test]
    fn test_icon_filename_stable() {
        let path = "C:\\Windows\\notepad.exe";
        let a = icon_filename(path);
        let b = icon_filename(path);
        assert_eq!(a, b);
        assert!(a.ends_with(".png"));
    }
}
