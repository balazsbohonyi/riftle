# Phase 3: Indexer - Research

**Researched:** 2026-03-06
**Domain:** Rust Windows backend — filesystem crawling, LNK resolution, icon extraction, notify watcher, SQLite persistence
**Confidence:** HIGH (core stack verified via official docs and crates.io; COM/GDI patterns verified via Rust forum + docs.rs)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Startup index timing:** First index runs synchronously in `setup()` — blocks until complete before app is "ready". No time cap. After first index, spawn background timer thread.
- **Timer reset:** Timer interval counts from end of first index. `reindex()` (INDX-08) is fire-and-forget — spawns background thread and returns immediately.
- **Re-index coordination:** If a re-index is already running when a new trigger fires (timer, watcher, manual): skip/drop the new trigger. Watcher events suppressed during a full re-index.
- **After manual `reindex()`:** Reset the timer so next auto-index is 15 min from then.
- **Broken/orphan shortcuts:** Unresolvable .lnk files: skip silently. Chained .lnk → .lnk: one level only, skip entirely. PATH crawl: .exe only.
- **Generic icon fallback:** Bundled PNG compiled into binary. Copy to `{data_dir}/icons/generic.png` only if missing. `icon_path` in SQLite stores `"generic.png"` for failures. Extract icons for ALL indexed apps.

### Claude's Discretion

- Threading model for background timer and watcher (std::thread vs tokio)
- Atomic boolean or Mutex<bool> for "is_indexing" flag
- Windows API approach for .lnk resolution
- PNG conversion approach for ExtractIconEx HICON output
- Icon filename convention in {data_dir}/icons/

### Deferred Ideas (OUT OF SCOPE)

- None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INDX-01 | Crawl Start Menu (AppData + ProgramData), Desktop (user + public), PATH (.exe only), additional_paths | walkdir + env vars/SHGetKnownFolderPath section |
| INDX-02 | .lnk shortcut targets resolved to actual executable paths | LNK resolution section — lnk crate approach |
| INDX-03 | Excluded paths skipped; stale entries removed on each full index | Stale pruning pattern + walkdir filter |
| INDX-04 | Icons extracted via ExtractIconEx, saved as .png; falls back to generic icon | Icon extraction section — GDI pipeline + image crate |
| INDX-05 | Icon extraction runs asynchronously; launcher shows placeholder until ready | Async icon extraction pattern — spawn per-app thread |
| INDX-06 | Background re-index on configurable interval (default 15 min) | Timer thread section |
| INDX-07 | notify crate watches Start Menu dirs; incremental re-index debounced 500ms | notify-debouncer-mini section |
| INDX-08 | reindex() Tauri command triggers full manual re-index on demand | Tauri command pattern section |
</phase_requirements>

---

## Summary

Phase 3 is a pure Rust backend phase. The implementation has three distinct concerns: (1) synchronous full-index on startup with stale pruning and icon extraction, (2) background timer re-index, and (3) filesystem watcher triggering incremental re-index. All three share the `Arc<Mutex<Connection>>` DB handle via `DbState` and coordinate with an `Arc<AtomicBool>` is-indexing flag.

The most nuanced areas are LNK resolution and icon extraction. For LNK resolution, `windows-sys` 0.52 does **not** expose IShellLink COM vtable structs — those live only in the `windows` crate. The recommended approach is the pure-Rust `lnk` crate (already adds no COM overhead, no new unsafe surface). Icon extraction via `ExtractIconEx` requires GDI calls (GetIconInfo, GetDIBits) and the `image` crate for PNG encoding — two new additions to Cargo.toml. The `notify-debouncer-mini` crate (v0.4.1, compatible with the project's existing `notify ^6`) is the recommended debouncing layer.

**Primary recommendation:** Use the `lnk` crate for LNK parsing (no COM), `notify-debouncer-mini = "0.4"` for debounced watching, `image = "^0.25"` for PNG encoding, and `std::thread` throughout (no tokio). Use `Arc<AtomicBool>` for the is-indexing guard and a channel-based approach for timer reset signalling.

---

## Standard Stack

### Core (all already in Cargo.toml)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| walkdir | ^2 | Recursive directory crawl with error handling | Zero overhead, handles permissions errors gracefully, yields DirEntry |
| notify | ^6 | Filesystem event watcher | Already in project; stable ^6 API |
| windows-sys | ^0.52 | ExtractIconEx, SHGetKnownFolderPath, GDI bitmap APIs | Already in project |
| rusqlite (via DbState) | ^0.31 | SQLite persistence | Already in project; established pattern |

### New Dependencies Required
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| lnk | ^0.3 | Pure-Rust .lnk parser — no COM required | Avoids unsafe COM vtable machinery; no `windows` crate needed; link_target() extracts path from LINK_INFO |
| notify-debouncer-mini | 0.4 | 500ms debounce layer over notify | Compatible with existing notify ^6; single new_debouncer() call |
| image | ^0.25 | PNG encoding for extracted HICON pixel data | Standard Rust image crate; RgbaImage::save() writes PNG |

### New windows-sys Features Required
| Feature | Why Needed |
|---------|------------|
| `Win32_Graphics_Gdi` | GetIconInfo, GetDIBits, DeleteObject, CreateCompatibleDC |
| `Win32_UI_WindowsAndMessaging` | ICONINFO struct, DestroyIcon |
| `Win32_System_Com` | CoInitializeEx, CoTaskMemFree (if using COM path) |

**Note:** `Win32_UI_Shell` already in Cargo.toml covers ExtractIconExW and SHGetKnownFolderPath.

### Updated Cargo.toml Section
```toml
# New domain crates — Phase 3
lnk = "^0.3"
notify-debouncer-mini = "0.4"
image = { version = "^0.25", default-features = false, features = ["png"] }

# Update windows-sys features:
windows-sys = { version = "^0.52", features = [
  "Win32_UI_Shell",
  "Win32_System_Shutdown",
  "Win32_System_Power",
  "Win32_System_RemoteDesktop",
  "Win32_Graphics_Gdi",          # NEW: GetIconInfo, GetDIBits, DeleteObject
  "Win32_UI_WindowsAndMessaging", # NEW: ICONINFO, DestroyIcon
] }
```

**Installation:**
```bash
cargo add lnk@0.3
cargo add notify-debouncer-mini@0.4
cargo add image@0.25 --no-default-features --features png
```

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| lnk crate | windows + IShellLinkW COM | COM approach is more reliable for edge-case shortcuts but requires adding the `windows` crate (different from `windows-sys`), COM initialization per thread, and substantial unsafe code. lnk crate handles 95%+ of real-world Start Menu shortcuts reliably. |
| notify-debouncer-mini | Manual debounce with `std::time::Instant` | Manual approach is 20+ lines of state per watcher; debouncer-mini is 1 line. Both compatible with notify ^6. |
| image crate | Raw Win32 HBITMAP to PNG (GDI+ or WIC) | Raw approach avoids an extra crate but requires WIC COM or GDI+ C++ interop — far more complex. image crate is the established Rust solution. |
| std::thread timer | tokio::time::interval | std::thread is simpler and already the project pattern (no tokio dependency). |

---

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/indexer.rs    # All indexer logic (single module)
  ├── pub fn run_full_index()         # Called from lib.rs setup()
  ├── pub fn start_background_tasks() # Spawns timer + watcher threads
  ├── #[tauri::command] pub fn reindex() # Tauri command — fire and forget
  ├── fn crawl_paths()                # walkdir over all source dirs
  ├── fn resolve_lnk()                # lnk crate .lnk → target path
  ├── fn extract_icon_async()         # Spawns icon extraction thread
  ├── fn extract_icon_sync()          # GDI pipeline: HICON → PNG bytes
  ├── fn prune_stale()                # DELETE FROM apps WHERE id NOT IN (...)
  └── fn get_start_menu_paths()       # SHGetKnownFolderPath / env fallback

icons/generic.png           # Bundled in src-tauri/ — included via include_bytes!
```

### Pattern 1: AtomicBool is-indexing Guard

**What:** `Arc<AtomicBool>` shared across timer thread, watcher thread, and Tauri command. Prevents concurrent index runs.

**When to use:** Any trigger point (timer, watcher, manual reindex) checks this flag before starting.

```rust
// Source: std::sync::atomic docs + Rust Atomics and Locks
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// In lib.rs setup — create shared state
let is_indexing = Arc::new(AtomicBool::new(false));

// At any trigger point — skip if already running:
fn try_start_index(is_indexing: &Arc<AtomicBool>, db: &Arc<Mutex<Connection>>, data_dir: &Path) {
    // compare_exchange: only one thread can flip false → true
    if is_indexing
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_ok()
    {
        let flag = Arc::clone(is_indexing);
        let db = Arc::clone(db);
        let data_dir = data_dir.to_path_buf();
        std::thread::spawn(move || {
            do_full_index(&db, &data_dir);
            flag.store(false, Ordering::Release); // always release
        });
    }
    // else: already indexing — silently drop trigger
}
```

**Key detail:** Use `Ordering::AcqRel` on the CAS and `Ordering::Release` on the store. Use `Ordering::Acquire` on reads. This ensures the indexing work is fully visible when the flag is reset.

### Pattern 2: Timer Thread with Reset Channel

**What:** Background thread sleeps for reindex_interval, then triggers index. A `std::sync::mpsc` channel lets the reindex command reset the timer.

**When to use:** After first index completes; timer resets after each manual reindex.

```rust
// Source: std::thread + std::sync::mpsc standard library
use std::sync::mpsc::{self, TryRecvError};
use std::time::{Duration, Instant};

fn start_timer_thread(
    interval_mins: u32,
    is_indexing: Arc<AtomicBool>,
    db: Arc<Mutex<Connection>>,
    data_dir: PathBuf,
) -> mpsc::Sender<()> {  // Sender returned so reindex() can reset timer
    let (tx, rx) = mpsc::channel::<()>();
    std::thread::spawn(move || {
        let interval = Duration::from_secs(interval_mins as u64 * 60);
        let mut deadline = Instant::now() + interval;
        loop {
            std::thread::sleep(Duration::from_secs(1)); // coarse 1s poll
            match rx.try_recv() {
                Ok(()) => { deadline = Instant::now() + interval; } // reset on manual index
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {}
            }
            if Instant::now() >= deadline {
                try_start_index(&is_indexing, &db, &data_dir);
                deadline = Instant::now() + interval;
            }
        }
    });
    tx
}
```

**Key detail:** 1-second poll granularity is fine — timer doesn't need millisecond precision. Sender is stored in Tauri state so `reindex()` command can call `tx.send(())`.

### Pattern 3: notify-debouncer-mini Watcher

**What:** Start Menu directories watched for changes. 500ms debounce before triggering incremental re-index.

**When to use:** Start background watcher after first full index completes.

```rust
// Source: docs.rs/notify-debouncer-mini/0.4.1 + verified 2026 blog example
use notify_debouncer_mini::{notify::RecursiveMode, new_debouncer, DebounceEventResult};
use std::time::Duration;

fn start_watcher(
    watch_paths: Vec<PathBuf>,
    is_indexing: Arc<AtomicBool>,
    db: Arc<Mutex<Connection>>,
    data_dir: PathBuf,
) {
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_millis(500), move |res: DebounceEventResult| {
            let _ = tx.send(res);
        }).expect("failed to create debouncer");

        for path in &watch_paths {
            debouncer.watcher().watch(path.as_ref(), RecursiveMode::Recursive)
                .unwrap_or_else(|e| eprintln!("[watcher] failed to watch {:?}: {}", path, e));
        }

        for result in rx {
            if let Ok(_events) = result {
                // Suppress events during full re-index (flag check inside try_start_index)
                try_start_index(&is_indexing, &db, &data_dir);
            }
        }
    });
}
```

**Key detail:** Watcher thread owns the `Debouncer` — keep it alive by holding in the thread loop (the debouncer drops when out of scope). Calling `try_start_index()` reuses the same AtomicBool guard — watcher events during a full re-index are silently dropped.

### Pattern 4: LNK Resolution (lnk crate)

**What:** Parse .lnk file with pure-Rust `lnk` crate. Extract target path from LINK_INFO structure.

**When to use:** Any file with `.lnk` extension found during walkdir.

```rust
// Source: docs.rs/lnk/latest + forum-verified approach
use lnk::ShellLink;

fn resolve_lnk(lnk_path: &Path) -> Option<PathBuf> {
    use lnk::encoding::WINDOWS_1252;
    let shortcut = ShellLink::open(lnk_path, WINDOWS_1252).ok()?;

    // link_target() builds path from LINK_INFO — returns None if structure absent
    let target_str = shortcut.link_target()?;
    let target = PathBuf::from(&target_str);

    // One-level only: skip if resolved target is also a .lnk
    if target.extension().and_then(|e| e.to_str()) == Some("lnk") {
        return None;
    }

    // Must exist and be an .exe
    if target.exists() && target.extension().and_then(|e| e.to_str()) == Some("exe") {
        Some(target)
    } else {
        None // Skip silently — broken or non-exe target
    }
}
```

**Key detail:** `lnk::ShellLink::open()` requires an encoding argument. Use `WINDOWS_1252` for maximum compatibility. Returns `None` on any failure — caller skips silently per locked decision.

**Limitation:** `link_target()` reads from `LINK_INFO` only. Some .lnk files (especially those with `LINK_TARGET_ID_LIST` but no `LINK_INFO`) return `None`. These are silently skipped. In practice, all modern Windows Start Menu shortcuts include LINK_INFO.

### Pattern 5: Icon Extraction Pipeline (GDI + image crate)

**What:** Extract HICON from exe via ExtractIconExW, convert to RGBA pixel data via GDI, encode as PNG via image crate.

**When to use:** For each indexed app; runs in a spawned thread (INDX-05 async requirement).

```rust
// Source: docs.rs/windows-sys/0.52 + Rust forum HICON-to-PNG thread (verified)
// Features needed: Win32_UI_Shell, Win32_Graphics_Gdi, Win32_UI_WindowsAndMessaging
use windows_sys::Win32::UI::Shell::ExtractIconExW;
use windows_sys::Win32::UI::WindowsAndMessaging::{ICONINFO, GetIconInfo, DestroyIcon};
use windows_sys::Win32::Graphics::Gdi::{
    GetDIBits, GetObjectW, DeleteObject, CreateCompatibleDC, DeleteDC,
    BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS,
};

/// Returns PNG bytes for the app at exe_path, or None on failure.
/// Call this in a spawned thread — GDI calls are thread-safe.
fn extract_icon_png(exe_path: &Path) -> Option<Vec<u8>> {
    // Convert path to wide string
    let wide: Vec<u16> = exe_path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        // Extract large icon (32x32) at index 0
        let mut hicon_large: isize = 0;
        let count = ExtractIconExW(wide.as_ptr(), 0, &mut hicon_large, std::ptr::null_mut(), 1);
        if count == 0 || hicon_large == 0 { return None; }

        // Get icon bitmap info
        let mut icon_info = ICONINFO {
            fIcon: 0, xHotspot: 0, yHotspot: 0,
            hbmMask: 0, hbmColor: 0,
        };
        if GetIconInfo(hicon_large, &mut icon_info) == 0 {
            DestroyIcon(hicon_large);
            return None;
        }

        // Get bitmap dimensions
        let mut bmp: BITMAP = std::mem::zeroed();
        GetObjectW(icon_info.hbmColor, std::mem::size_of::<BITMAP>() as i32,
                   &mut bmp as *mut _ as *mut _);
        let width = bmp.bmWidth as u32;
        let height = bmp.bmHeight as u32;

        // Allocate pixel buffer and extract via GetDIBits
        let row_bytes = width * 4; // BGRA
        let mut pixels = vec![0u8; (row_bytes * height) as usize];

        let dc = CreateCompatibleDC(0);
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32), // negative = top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0, // BI_RGB
                biSizeImage: 0,
                biXPelsPerMeter: 0, biYPelsPerMeter: 0,
                biClrUsed: 0, biClrImportant: 0,
            },
            bmiColors: [std::mem::zeroed()],
        };
        GetDIBits(dc, icon_info.hbmColor, 0, height,
                  pixels.as_mut_ptr() as *mut _, &mut bmi, DIB_RGB_COLORS);

        // Cleanup GDI resources
        DeleteDC(dc);
        DeleteObject(icon_info.hbmColor);
        DeleteObject(icon_info.hbmMask);
        DestroyIcon(hicon_large);

        // BGRA → RGBA swap
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // B ↔ R
        }

        // Encode as PNG via image crate
        let img = image::RgbaImage::from_raw(width, height, pixels)?;
        let mut png_bytes: Vec<u8> = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png).ok()?;
        Some(png_bytes)
    }
}
```

**Key detail:** Use negative `biHeight` to get top-down pixel order (simpler). Swap B and R channels (Windows GDI uses BGRA, image crate expects RGBA). Use `GetDIBits` not `GetBitmapBits` — the latter doesn't handle 32-bit icons correctly.

**Async pattern (INDX-05):** Icon extraction spawns per-app threads during the full index. The DB upsert happens twice: first with `icon_path = None` (placeholder), then the icon thread calls `upsert_app()` again with the resolved icon path after extraction.

### Pattern 6: Stale Entry Pruning

**What:** After a full crawl, delete SQLite rows for apps no longer found on disk.

**When to use:** At the end of every full index run.

```rust
// Source: SQLite docs + rusqlite patterns
fn prune_stale(conn: &Connection, discovered_ids: &HashSet<String>) -> rusqlite::Result<()> {
    // Build parameterized IN clause
    // For ~hundreds of apps, this is fast enough (no temp table needed)
    let existing: Vec<String> = {
        let mut stmt = conn.prepare("SELECT id FROM apps")?;
        stmt.query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect()
    };
    for id in existing {
        if !discovered_ids.contains(&id) {
            conn.execute("DELETE FROM apps WHERE id = ?1", rusqlite::params![id])?;
        }
    }
    Ok(())
}
```

**Alternative (single SQL DELETE):** For larger indexes, use a single parameterized DELETE:
```sql
DELETE FROM apps WHERE id NOT IN (SELECT value FROM json_each(?1))
```
Using `serde_json::to_string(&ids_vec)` to build the JSON array. This avoids N round-trips for N stale entries. Use the single-DELETE approach if app count exceeds ~500.

**Key detail:** `id` in the apps table is the normalized exe path (per DATA-02). Use `path.to_string_lossy().to_lowercase()` as the canonical ID to match upsert_app() behavior.

### Pattern 7: Walk Source Directories

**What:** Discover Start Menu paths using Windows known folder API or env var fallback. Crawl with walkdir.

**When to use:** At the start of every full index.

```rust
// Source: windows-sys docs + env var documentation (HIGH confidence for env var approach)
fn get_index_paths(settings: &Settings) -> Vec<(PathBuf, &'static str)> {
    let mut paths = vec![];

    // Start Menu user: %APPDATA%\Microsoft\Windows\Start Menu\Programs
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs");
        if p.exists() { paths.push((p, "start_menu")); }
    }

    // Start Menu ProgramData: %PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs
    if let Ok(pdata) = std::env::var("PROGRAMDATA") {
        let p = PathBuf::from(pdata).join("Microsoft\\Windows\\Start Menu\\Programs");
        if p.exists() { paths.push((p, "start_menu")); }
    }

    // Desktop user: %USERPROFILE%\Desktop
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(&profile).join("Desktop");
        if p.exists() { paths.push((p, "desktop")); }
    }

    // Desktop public: C:\Users\Public\Desktop
    let public_desktop = PathBuf::from("C:\\Users\\Public\\Desktop");
    if public_desktop.exists() { paths.push((public_desktop, "desktop")); }

    // PATH directories — only .exe files (per locked decision)
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if dir.exists() { paths.push((dir, "path")); }
        }
    }

    // User-defined additional paths from settings
    for ap in &settings.additional_paths {
        let p = PathBuf::from(ap);
        if p.exists() { paths.push((p, "additional")); }
    }

    paths
}

fn crawl_dir(root: &Path, source: &str, excluded: &[String]) -> Vec<AppRecord> {
    let mut apps = vec![];
    for entry in walkdir::WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok()) // skip permission errors silently
    {
        let path = entry.path();

        // Skip excluded paths
        if excluded.iter().any(|ex| path.starts_with(ex)) { continue; }

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
```

**Key detail:** PATH directories are crawled for `.exe` only — no `.lnk` resolution from PATH (per locked decision). Start Menu and Desktop resolve `.lnk`. `walkdir::WalkDir::follow_links(true)` handles symlinks correctly.

### Pattern 8: Generic Icon Bootstrapping

**What:** On startup, copy bundled generic PNG to `{data_dir}/icons/generic.png` if not present.

**When to use:** Called once at the top of `run_full_index()`.

```rust
// Source: Rust std::fs docs + Tauri include_bytes! pattern
static GENERIC_ICON: &[u8] = include_bytes!("../../icons/generic.png"); // path relative to indexer.rs

fn ensure_generic_icon(data_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(data_dir.join("icons"))?;
    let dest = data_dir.join("icons").join("generic.png");
    if !dest.exists() {
        std::fs::write(&dest, GENERIC_ICON)?;
    }
    Ok(())
}
```

**Key detail:** The generic icon PNG must exist at `src-tauri/icons/generic.png` (relative to the lib) at compile time. Create it as a 32x32 blank/placeholder PNG. The path in the `include_bytes!` macro is relative to the current source file's location.

### Pattern 9: Icon Filename Convention

**Recommendation (Claude's Discretion):** Use the SQLite row `id` (normalized exe path) as filename: hash the path to a hex string. Specifically, use `format!("{:016x}.png", FNV-hash-of-path)`. This gives stable filenames across re-indexes and avoids path separator issues.

**Alternative:** Use the integer rowid from SQLite. Simpler but requires reading the rowid after upsert.

**Simplest approach:** Convert the normalized path to a base64url string. Since paths can be long, a 16-character FNV hash is preferred. Use a minimal inline FNV-1a implementation (no new crate):

```rust
fn icon_filename(exe_path: &str) -> String {
    // FNV-1a 64-bit hash — no crate needed
    let mut hash: u64 = 14695981039346656037;
    for byte in exe_path.to_lowercase().bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{:016x}.png", hash)
}
```

### Anti-Patterns to Avoid

- **Holding DB lock during icon extraction:** GDI calls are slow (~10ms per icon). Never hold `Arc<Mutex<Connection>>` lock during `extract_icon_png()`. Upsert with `None` icon first, release lock, extract icon, re-acquire lock to update.
- **Walking PATH non-recursively:** PATH directories should be crawled non-recursively (`walkdir::WalkDir::max_depth(1)`) — only top-level .exe files are meaningful from PATH.
- **Not calling CoInitializeEx on icon thread:** If you ever switch to COM-based icon extraction in future, remember COM must be initialized per-thread. The GDI approach (current recommendation) does NOT require COM.
- **Blocking the main thread on watcher creation:** Create debouncer inside the spawned thread, not in setup().
- **Timer thread panic on channel disconnect:** If the AppHandle goes out of scope, the timer's receive end disconnects. Handle `TryRecvError::Disconnected` by breaking the loop cleanly.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| .lnk binary parsing | Custom binary parser for MS-SHLLINK spec | `lnk` crate | 300-page spec with dozens of optional structures; lnk crate handles real-world shortcuts |
| Debounce logic | Manual `Instant` tracking + mpsc debounce | `notify-debouncer-mini` | Event coalescence across multiple rapid changes is subtle; debouncer handles event deduplication correctly |
| Icon hash/filename | UUID library | Inline FNV-1a hash (5 lines) | No crate needed for a 64-bit hash; FNV is sufficient for path-to-filename mapping |
| PNG encoding from raw bytes | Raw PNG chunk writer | `image` crate | PNG spec (CRC, deflate, filter) has many correctness requirements |
| Wide string conversion | Manual UTF-16 encoding | `OsStr::encode_wide()` | Already in std; handles surrogate pairs correctly |

**Key insight:** The Windows icon extraction GDI pipeline looks short but has ~10 failure modes (empty hbmColor, top-down vs bottom-up bitmap, alpha premultiplication). The pattern above handles the common cases; always check return values and fall back to `generic.png`.

---

## Common Pitfalls

### Pitfall 1: lnk LINK_INFO absence
**What goes wrong:** `ShellLink::link_target()` returns `None` for shortcuts that store the target only in the `LINK_TARGET_ID_LIST` structure (no `LINK_INFO`). This affects some older or custom shortcuts.
**Why it happens:** The lnk crate's `link_target()` builds the path from LINK_INFO only.
**How to avoid:** Accept None silently — these shortcuts are a small minority on typical Windows systems. If needed in future, fall back to COM (IShellLink::GetPath) for None cases.
**Warning signs:** Large numbers of Start Menu apps not appearing in the index.

### Pitfall 2: Icon extraction on UI thread
**What goes wrong:** Calling `ExtractIconExW` from the Tauri setup callback or main thread blocks the UI.
**Why it happens:** GDI icon extraction can take 5–50ms per exe (disk read).
**How to avoid:** Always spawn a separate thread per-app or a worker thread pool for icon extraction. The locked decision specifies async extraction (INDX-05).
**Warning signs:** Slow startup, frozen window during indexing.

### Pitfall 3: Bottom-up bitmap from GetDIBits
**What goes wrong:** Icon appears vertically flipped in the saved PNG.
**Why it happens:** Default BITMAPINFOHEADER `biHeight` is positive = bottom-up scanline order.
**How to avoid:** Set `biHeight = -(height as i32)` (negative) to get top-down order, eliminating the need to flip rows.
**Warning signs:** PNG icons appear upside-down.

### Pitfall 4: DbState mutex contention during bulk icon upsert
**What goes wrong:** Multiple icon extraction threads try to `upsert_app()` simultaneously, causing lock contention and potential timeout/deadlock.
**Why it happens:** `Arc<Mutex<Connection>>` serializes all access — many concurrent writers queue up.
**How to avoid:** Use a bounded icon-update queue — spawn a single "icon writer" thread that receives completed icon data via `mpsc::channel` and batches writes. Alternatively, use a semaphore to limit concurrent icon threads to 4.
**Warning signs:** High lock wait time; indexing taking much longer than expected.

### Pitfall 5: notify watcher path survival
**What goes wrong:** Watcher stops receiving events after the paths are modified or re-created.
**Why it happens:** `ReadDirectoryChangesW` (Windows backend) can drop the watch if the watched directory is deleted and re-created.
**How to avoid:** In the watcher thread, on error events, attempt to re-register the watch paths.
**Warning signs:** Watcher events stop arriving after a Windows update or Start Menu reorganization.

### Pitfall 6: Cargo.toml windows-sys feature gaps
**What goes wrong:** Compile error `cannot find function/struct X in module Win32::...`
**Why it happens:** windows-sys features are granular — missing `Win32_Graphics_Gdi` prevents GDI function access.
**How to avoid:** Add all three new features listed in Standard Stack section. Check each function's Required Features note in docs.rs.
**Warning signs:** Compilation fails with unresolved name in windows-sys path.

### Pitfall 7: Timer thread outliving AppHandle
**What goes wrong:** Timer thread panics when trying to access data after app shutdown.
**Why it happens:** `std::thread::spawn` creates a detached thread — it doesn't know about Tauri lifecycle.
**How to avoid:** Store the timer reset `Sender<()>` in managed state. When the sender is dropped (app shutdown), the timer thread's `TryRecvError::Disconnected` will break its loop cleanly.
**Warning signs:** Panic message mentioning send on a closed channel after app exit.

---

## Code Examples

### Full Index Entry Point
```rust
// Called synchronously from lib.rs setup()
pub fn run_full_index(db: &Arc<Mutex<Connection>>, data_dir: &Path) {
    // 1. Ensure icons dir and generic icon exist
    let icons_dir = data_dir.join("icons");
    let _ = std::fs::create_dir_all(&icons_dir);
    ensure_generic_icon(data_dir).unwrap_or_else(|e| eprintln!("[indexer] generic icon: {}", e));

    // 2. Load settings
    // (settings loaded by caller and passed in, or re-read here)

    // 3. Crawl all source directories
    let settings = /* read from store */ Settings::default();
    let source_dirs = get_index_paths(&settings);
    let mut discovered_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (dir, source) in &source_dirs {
        for app in crawl_dir(dir, source, &settings.excluded_paths) {
            discovered_ids.insert(app.id.clone());
            let conn = db.lock().unwrap();
            let _ = upsert_app(&conn, &app);
            drop(conn);
            // Spawn icon extraction asynchronously
            let icon_path_for_app = icons_dir.join(icon_filename(&app.path));
            let db_clone = Arc::clone(db);
            let exe_path = PathBuf::from(&app.path);
            let app_id = app.id.clone();
            std::thread::spawn(move || {
                if !icon_path_for_app.exists() {
                    match extract_icon_png(&exe_path) {
                        Some(bytes) => {
                            let _ = std::fs::write(&icon_path_for_app, bytes);
                            // Update icon_path in DB
                            let filename = icon_path_for_app.file_name()
                                .unwrap().to_string_lossy().to_string();
                            let conn = db_clone.lock().unwrap();
                            let _ = conn.execute(
                                "UPDATE apps SET icon_path = ?1 WHERE id = ?2",
                                rusqlite::params![filename, app_id],
                            );
                        }
                        None => {} // stays as generic.png (already set by upsert)
                    }
                }
            });
        }
    }

    // 4. Prune stale entries
    let conn = db.lock().unwrap();
    let _ = prune_stale(&conn, &discovered_ids);
}
```

### Tauri Command: reindex
```rust
// Source: Tauri v2 command pattern (established in project)
#[tauri::command]
pub fn reindex(
    app: tauri::AppHandle,
    db_state: tauri::State<DbState>,
    is_indexing: tauri::State<Arc<AtomicBool>>,
    timer_tx: tauri::State<Arc<Mutex<mpsc::Sender<()>>>>,
    // data_dir passed as Tauri state or re-derived
) {
    let db = Arc::clone(&db_state.0);
    let flag = Arc::clone(&is_indexing);
    let tx = Arc::clone(&timer_tx);

    std::thread::spawn(move || {
        if flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed).is_ok() {
            let data_dir = /* derive from app */ PathBuf::new();
            run_full_index(&db, &data_dir);
            flag.store(false, Ordering::Release);
            // Reset timer so next auto-index is 15 min from now
            let _ = tx.lock().unwrap().send(());
        }
    });
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `notify` built-in debouncing | `notify-debouncer-mini` separate crate | notify v5+ | Cleaner separation; debouncer-mini is simpler API |
| `winapi` crate for Windows APIs | `windows-sys` / `windows` crates | 2021-2022 | `winapi` is no longer maintained; windows-sys is the official Microsoft-maintained binding |
| SHGetFolderPathW for known folders | SHGetKnownFolderPath (or env vars) | Vista+ | SHGetFolderPathW deprecated; env vars are simpler and sufficient |
| image crate `PngEncoder::encode()` | `PngEncoder::write_image()` | image 0.24+ | Old encode() had byte-order issues; write_image() is correct |

**Deprecated/outdated:**
- `winapi` crate: no longer maintained since 2021 — do not add
- `SHGetFolderPathW`: deprecated since Vista — use env vars or `SHGetKnownFolderPath`
- `notify` v5 raw API for debouncing: use `notify-debouncer-mini` instead

---

## Open Questions

1. **lnk crate vs COM for edge-case shortcuts**
   - What we know: `lnk` crate handles standard LINK_INFO shortcuts correctly
   - What's unclear: Percentage of real-world shortcuts on a typical Windows 11 system that lack LINK_INFO (store target only in LINK_TARGET_ID_LIST)
   - Recommendation: Ship with lnk crate; if users report missing apps, add COM fallback in a follow-up

2. **Icon extraction thread count / pool size**
   - What we know: Each extraction spawns a thread; GDI is thread-safe; but many concurrent threads cause mutex contention
   - What's unclear: Optimal parallelism for typical system with 200-500 Start Menu apps
   - Recommendation: Use a bounded channel with a pool of 4 icon worker threads; or use `std::thread::spawn` with a semaphore (AtomicUsize counting active extractions, max 8)

3. **notify watcher scope**
   - What we know: CONTEXT.md specifies "watch Start Menu directories"
   - What's unclear: Whether to also watch Desktop and additional_paths for real-time updates
   - Recommendation: Watch only Start Menu (user + ProgramData) per the requirement INDX-07; Desktop and PATH changes are caught by the 15-min timer

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test (`#[cfg(test)]` + `cargo test`) |
| Config file | None — Cargo.toml `[lib]` section |
| Quick run command | `cargo test -p riftle --lib indexer` |
| Full suite command | `cargo test -p riftle --lib` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INDX-01 | Crawl function discovers .exe files from a temp test directory | unit | `cargo test -p riftle --lib indexer::tests::test_crawl_discovers_exe` | ❌ Wave 0 |
| INDX-01 | Crawl function discovers .lnk files from a temp test directory | unit | `cargo test -p riftle --lib indexer::tests::test_crawl_discovers_lnk` | ❌ Wave 0 |
| INDX-02 | resolve_lnk() returns Some(exe_path) for a valid .lnk file | unit | `cargo test -p riftle --lib indexer::tests::test_resolve_lnk_valid` | ❌ Wave 0 |
| INDX-02 | resolve_lnk() returns None for a broken/non-exe .lnk | unit | `cargo test -p riftle --lib indexer::tests::test_resolve_lnk_broken` | ❌ Wave 0 |
| INDX-03 | prune_stale() removes DB rows for paths no longer in discovered set | unit | `cargo test -p riftle --lib indexer::tests::test_prune_stale` | ❌ Wave 0 |
| INDX-03 | Excluded path is skipped during crawl | unit | `cargo test -p riftle --lib indexer::tests::test_crawl_excludes_path` | ❌ Wave 0 |
| INDX-04 | icon_filename() produces stable hex string for same exe path | unit | `cargo test -p riftle --lib indexer::tests::test_icon_filename_stable` | ❌ Wave 0 |
| INDX-04 | ensure_generic_icon() writes file if missing, skips if present | unit | `cargo test -p riftle --lib indexer::tests::test_generic_icon_bootstrap` | ❌ Wave 0 |
| INDX-05 | Async icon extraction — icon_path in DB starts as None/generic, updates after extraction | integration | Manual smoke test (requires real exe on disk) | ❌ manual |
| INDX-06 | Timer thread sends trigger after configured interval | unit | `cargo test -p riftle --lib indexer::tests::test_timer_fires` (use 100ms interval) | ❌ Wave 0 |
| INDX-06 | Timer resets when Sender receives message | unit | `cargo test -p riftle --lib indexer::tests::test_timer_reset` | ❌ Wave 0 |
| INDX-07 | AtomicBool guard prevents concurrent index runs | unit | `cargo test -p riftle --lib indexer::tests::test_atomic_guard_prevents_double_index` | ❌ Wave 0 |
| INDX-07 | Watcher debounce — tested via notify-debouncer-mini behavior | manual | Start app, modify a Start Menu .lnk, verify re-index within ~500ms | manual |
| INDX-08 | reindex() Tauri command is registered and callable | integration | `cargo test -p riftle --lib` (command compile check) | ❌ Wave 0 |

**Notes on manual tests:**
- INDX-05 requires a real Windows system with actual executables on disk — cannot be unit-tested without OS integration
- INDX-07 watcher behavior requires running process — verify in smoke test

### Sampling Rate
- **Per task commit:** `cargo test -p riftle --lib indexer`
- **Per wave merge:** `cargo test -p riftle --lib`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/indexer.rs` — fill stub with all public functions and test module
- [ ] `src-tauri/icons/generic.png` — create 32x32 placeholder PNG (must exist at compile time for `include_bytes!`)
- [ ] Test helper: `fn temp_dir_with_exes()` — creates temp directory with dummy .exe and .lnk files for crawl tests
- [ ] Test helper: `fn in_memory_db()` — reuse pattern from db.rs tests (`Connection::open_in_memory()`)

*(All test functions are new — indexer.rs is currently a stub comment only)*

---

## Sources

### Primary (HIGH confidence)
- `docs.rs/notify-debouncer-mini/0.4.1` — confirmed notify ^6.1.1 dependency, new_debouncer API
- `docs.rs/windows-sys/0.52.0/features` — confirmed Win32_UI_Shell, Win32_System_Com, Win32_Graphics_Gdi feature flags exist (232 total)
- `docs.rs/windows-sys/latest/windows_sys/Win32/UI/Shell/fn.ExtractIconExW.html` — confirmed ExtractIconExW parameters and return type
- `docs.rs/windows-sys/latest/windows_sys/Win32/System/Com/` — confirmed CoCreateInstance, CoInitializeEx in Win32_System_Com
- `docs.rs/lnk/latest/lnk/` — confirmed ShellLink::open() API and link_target() method
- `docs.rs/image/0.25.9` — confirmed RgbaImage, write_to(), ImageFormat::Png API
- `notify-rs/notify` Cargo workspace — confirmed notify-debouncer-mini 0.7.0 current; 0.4.1 targets notify ^6.1.1

### Secondary (MEDIUM confidence)
- [Rust Forum: HICON to PNG](https://users.rust-lang.org/t/how-to-convert-hicon-to-png/90975) — GetIconInfo + GetDIBits pipeline; BGR swap requirement; GetBitmapBits caveat (multiple community-verified responses)
- [notify-rs documentation](https://docs.rs/notify/6.1.1/notify/) — RecommendedWatcher multiple-directory watching pattern
- [File Watcher with Debouncing in Rust (2026-01-25)](https://oneuptime.com/blog/post/2026-01-25-file-watcher-debouncing-rust/view) — notify-debouncer-mini 500ms pattern
- [winsafe IShellLink docs](https://rodrigocfd.github.io/winsafe/winsafe/struct.IShellLink.html) — CoCreateInstance + IShellLink pattern (confirms COM approach requires windows crate, not windows-sys)

### Tertiary (LOW confidence)
- SHGetKnownFolderPath feature flag specifics within Win32_UI_Shell — env var fallback recommended as more reliable and simpler
- Icon extraction thread count recommendation (4-8 workers) — based on general Windows GDI threading guidance, not benchmarked

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crate versions verified via docs.rs and crates.io
- LNK resolution (lnk crate): MEDIUM-HIGH — API verified; edge cases (LINK_INFO absence) noted
- Icon extraction (GDI pipeline): MEDIUM — code pattern sourced from Rust forum (multiple responders); not personally compiled
- Architecture/threading: HIGH — AtomicBool + std::thread patterns are stable Rust idioms
- notify-debouncer-mini: HIGH — version compatibility confirmed via docs.rs 0.4.1

**Research date:** 2026-03-06
**Valid until:** 2026-06-06 (90 days — stable crates; lnk and notify-debouncer-mini are slow-moving)
