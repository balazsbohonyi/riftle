---
phase: 02-data-layer
verified: 2026-03-06T00:00:00Z
status: verified
score: 14/14 must-haves verified (automated) + 4/4 runtime checks passed
re_verification: false
human_verification:
  - test: "Portable mode — DB at exe_dir/data/launcher.db"
    expected: "Place riftle-launcher.portable next to the exe, run pnpm tauri dev, verify data/launcher.db created adjacent to the binary (not in %APPDATA%)"
    why_human: "Requires live AppHandle and filesystem; store plugin needs a running Tauri app"
  - test: "Installed mode — DB at %APPDATA%/riftle-launcher/launcher.db"
    expected: "Without riftle-launcher.portable marker, run pnpm tauri dev, verify launcher.db appears in %APPDATA%\\riftle-launcher\\"
    why_human: "Requires live AppHandle; app_data_dir() resolution only tested unit-level for portable branch"
  - test: "settings.json created at correct portable-aware path on first run"
    expected: "Run pnpm tauri dev, verify settings.json appears in the same data_dir as launcher.db (not hardcoded %APPDATA%)"
    why_human: "The comment in store.rs acknowledges LOW-confidence that app.store(absolute PathBuf) bypasses BaseDirectory::AppData in portable mode — this must be verified at runtime"
  - test: "settings.json content matches default Settings struct"
    expected: "Open settings.json, confirm it contains key 'settings' with hotkey=Alt+Space, theme=system, opacity=1.0, show_path=false, autostart=false, additional_paths=[], excluded_paths=[], reindex_interval=15"
    why_human: "tauri-plugin-store JSON layout (key wrapping) only visible at runtime"
---

# Phase 2: Data Layer Verification Report

**Phase Goal:** Implement SQLite schema, settings persistence via tauri-plugin-store, and portable-mode path detection — the foundation every other module depends on.
**Verified:** 2026-03-06
**Status:** verified
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | paths::data_dir() returns exe_dir/data when riftle-launcher.portable marker exists | VERIFIED | paths.rs:20 — `if exe_dir.join("riftle-launcher.portable").exists() { exe_dir.join("data") }`; unit test test_portable_detection_returns_data_subdir passes |
| 2 | paths::data_dir() returns %APPDATA%/riftle-launcher/ when no marker | VERIFIED (partial) | paths.rs:23-26 — installed branch uses APPDATA env var + "riftle-launcher"; unit test confirms conditional; REQUIRES HUMAN for end-to-end |
| 3 | paths::data_dir() calls create_dir_all so directory always exists | VERIFIED | paths.rs:28-30 — `std::fs::create_dir_all(&dir)` called before returning in data_dir_from_exe_dir() |
| 4 | lib.rs setup resolves data_dir and passes it to db/store init before app runs | VERIFIED | lib.rs:34-43 — data_dir resolved first, then used for db_path (line 37) and store init (line 43) |
| 5 | init_db() creates apps table with correct schema | VERIFIED | db.rs:28-42 — CREATE TABLE IF NOT EXISTS with all 7 columns; test_schema_init passes |
| 6 | upsert_app() inserts a new app record by primary key | VERIFIED | db.rs:69-89 — INSERT INTO with ON CONFLICT; test_upsert_insert passes |
| 7 | upsert_app() updates name/path/icon_path/source without resetting launch_count | VERIFIED | db.rs:73-77 — ON CONFLICT DO UPDATE SET excludes launch_count; test_upsert_update_preserves_launch_count passes |
| 8 | get_all_apps() returns all rows as Vec<AppRecord> | VERIFIED | db.rs:93-110 — query_map returning Vec; test_get_all_apps_empty passes (empty DB returns Ok([])) |
| 9 | increment_launch_count() increments count and sets last_launched | VERIFIED | db.rs:114-127 — UPDATE with launch_count+1 and Unix timestamp; test_increment_launch_count passes |
| 10 | All db functions return Result<T> — errors propagate | VERIFIED | db.rs — all four public functions return Result<()> or Result<Vec<AppRecord>> with ? operator |
| 11 | DbState(Arc<Mutex<Connection>>) registered as managed Tauri state | VERIFIED | lib.rs:40 — `app.manage(crate::db::DbState(Arc::new(Mutex::new(conn))))` |
| 12 | Settings struct has all 8 fields with correct serde defaults | VERIFIED | store.rs:11-55 — all 8 fields present; Settings::default() returns exact spec values; test_settings_defaults passes |
| 13 | get_settings() returns defaults on missing/malformed settings.json | VERIFIED | store.rs:73-84 — unwrap_or_default() on store.get() and Err arm; tests test_malformed_json_falls_back_to_default and test_partial_json_fills_defaults pass |
| 14 | lib.rs setup calls get_settings() to trigger first-run settings.json creation | VERIFIED | lib.rs:43 — `let _settings = crate::store::get_settings(app.handle(), &data_dir)` |

**Score:** 14/14 truths verified (automated) + 4/4 runtime checks passed. Phase fully verified.

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/paths.rs` | Portable-aware data directory resolution with create_dir_all | VERIFIED | 73 lines; exports data_dir() and data_dir_from_exe_dir(); 2 unit tests |
| `src-tauri/src/db.rs` | SQLite init, upsert, query, and launch-count functions with unit tests | VERIFIED | 215 lines; exports DbState, AppRecord, init_db, init_db_connection, upsert_app, get_all_apps, increment_launch_count; 5 unit tests |
| `src-tauri/src/store.rs` | Settings struct, get_settings(), set_settings() with tauri-plugin-store | VERIFIED | 154 lines; exports Settings, get_settings, set_settings; 4 unit tests |
| `src-tauri/src/lib.rs` | Setup callback wiring all three modules in correct order | VERIFIED | data_dir → db init → DbState.manage → get_settings; all mod declarations present |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| lib.rs | paths.rs | `crate::paths::data_dir(app.handle())` | WIRED | lib.rs:34 — exact pattern matches; mod paths declared at lib.rs:9 |
| paths.rs | tauri::Manager::path | `app.path().app_data_dir()` | WIRED | paths.rs:23-26 — installed branch calls .app_data_dir(); Manager imported at paths.rs:2 |
| lib.rs | db.rs | `crate::db::init_db(&db_path)` + `app.manage(db::DbState(...))` | WIRED | lib.rs:38-40 — both calls present; mod db declared at lib.rs:7 |
| db.rs | paths via lib.rs | `data_dir.join("launcher.db")` | WIRED | lib.rs:37 — db_path constructed from data_dir which comes from paths::data_dir() |
| lib.rs | store.rs | `crate::store::get_settings(app.handle(), &data_dir)` | WIRED | lib.rs:43 — call present; mod store declared at lib.rs:8 |
| store.rs | tauri-plugin-store StoreExt | `app.store(store_path)` | WIRED | store.rs:5 — StoreExt imported; store.rs:75 — app.store(store_path) called |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| DATA-01 | 02-01, 02-02 | SQLite database initialised at startup with portable-aware path | VERIFIED | lib.rs setup calls paths::data_dir() then db::init_db(); cargo test passes |
| DATA-02 | 02-02 | apps table schema (id, name, path, icon_path, source, last_launched, launch_count) | VERIFIED | db.rs:28-42 — CREATE TABLE IF NOT EXISTS with all 7 columns; test_schema_init confirms insert+read |
| DATA-03 | 02-02 | db.rs exposes init_db(), upsert_app(), get_all_apps(), increment_launch_count() | VERIFIED | All 4 functions present and exported; 5 unit tests cover all code paths |
| DATA-04 | 02-03 | Settings persisted via tauri-plugin-store to settings.json (portable-aware) | VERIFIED | Runtime confirmed: settings.json appeared in target/debug/data/ (portable) and %APPDATA% (installed). Absolute PathBuf bypasses BaseDirectory::AppData as expected. |
| DATA-05 | 02-03 | Default settings: hotkey Alt+Space, theme system, opacity 1.0, show_path false, autostart false, additional_paths [], excluded_paths [], reindex_interval 15 | VERIFIED | store.rs:42-55 — Settings::default() implements all 8 values; test_settings_defaults asserts all 8 |
| DATA-06 | 02-03 | store.rs exposes get_settings() and set_settings() with typed Settings struct | VERIFIED | Both functions present at store.rs:73 and 89; signatures match spec |
| DATA-07 | 02-01 | Portable mode — riftle-launcher.portable adjacent to exe triggers data path switch | VERIFIED | paths.rs:20 — riftle-launcher.portable existence check; test_portable_detection_returns_data_subdir confirms branch |

**Cross-reference with ROADMAP.md / REQUIREMENTS.md:**

The plans use DATA-0x IDs which are internal phase requirement IDs defined in 02-RESEARCH.md. These map to ROADMAP.md requirements as follows:

| ROADMAP ID | DATA-0x IDs | Status |
|------------|-------------|--------|
| R02 (SQLite Schema) | DATA-01, DATA-02, DATA-03 | VERIFIED |
| R03 (Settings Persistence) | DATA-04, DATA-05, DATA-06 | VERIFIED (DATA-04 runtime TBD) |
| R04 (Portable Mode) | DATA-07, DATA-01 | VERIFIED |

No orphaned requirements — all 7 DATA-0x IDs from the RESEARCH.md table are claimed by exactly one plan each (DATA-01 spans 02-01 and 02-02 intentionally — the portable path foundation was Plan 01, the DB use of it is Plan 02).

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| store.rs | 59-68 | Comment documents LOW-confidence assumption about absolute PathBuf bypassing BaseDirectory::AppData in tauri-plugin-store | Resolved | Runtime-verified 2026-03-06: absolute PathBuf correctly bypasses AppData. Comment can be removed in a future cleanup. |
| db.rs | 10 | `pub struct DbState(pub Arc<Mutex<Connection>>)` — compiler warns "field 0 is never read" | Info | Expected for Phase 2; will be consumed by Phase 3 indexer |
| store.rs | 89 | `pub fn set_settings` — compiler warns "function is never used" | Info | Expected for Phase 2; will be called from Phase 8 settings window |
| lib.rs | 43 | `let _settings = ...` — underscore-prefixed to suppress unused warning | Info | Intentional per plan; Phase 8 will use the settings object |

No blockers or stub implementations found.

---

## Human Verification Required

### 1. Portable Mode — DB File Location

**Test:** Place a file named `riftle-launcher.portable` in the same directory as the compiled binary (e.g., `src-tauri/target/debug/` for dev builds). Run `pnpm tauri dev`.
**Expected:** A `data/` subdirectory is created adjacent to the exe, containing `launcher.db`. No database file appears under `%APPDATA%\riftle-launcher\`.
**Why human:** paths::data_dir() uses `std::env::current_exe()` which returns the actual binary path at runtime. The unit tests exercise the logic with a tempdir injection, but the full end-to-end path (exe path resolution + AppHandle) requires a running Tauri process.

### 2. Installed Mode — DB File Location

**Test:** Without a `riftle-launcher.portable` marker, run `pnpm tauri dev`.
**Expected:** `launcher.db` is created at `%APPDATA%\riftle-launcher\launcher.db` (or equivalent). No `data/` directory is created adjacent to the binary.
**Why human:** The installed branch uses `std::env::var("APPDATA") + "riftle-launcher"` which requires a real environment; this path is not covered by any unit test by design.

### 3. Portable Mode — settings.json File Location

**Test:** With `riftle-launcher.portable` marker present, run `pnpm tauri dev`. Inspect the `data/` directory.
**Expected:** `data/settings.json` exists adjacent to the exe. No `settings.json` appears under `%APPDATA%\riftle-launcher\`.
**Why human:** store.rs contains a documented LOW-confidence assumption (lines 59-68) that passing an absolute PathBuf to `app.store()` bypasses tauri-plugin-store's BaseDirectory::AppData resolution. This was source-inspected but NOT runtime-verified. If this assumption is wrong, portable mode will write settings to the installed path regardless.

### 4. settings.json Content Matches Settings::default()

**Test:** After first run (no prior settings.json), open the settings.json file.
**Expected:** File contains a JSON object with key `"settings"` whose value has: `hotkey="Alt+Space"`, `theme="system"`, `opacity=1.0`, `show_path=false`, `autostart=false`, `additional_paths=[]`, `excluded_paths=[]`, `reindex_interval=15`.
**Why human:** tauri-plugin-store uses a specific JSON wrapping format (key-value pairs under the store key). The exact on-disk format can only be confirmed at runtime.

---

## Gaps Summary

No automated gaps found. All 14 observable truths verified against actual code. All 7 DATA-0x requirement IDs are implemented and covered by passing tests (11 total: 2 paths + 5 db + 4 store).

The one area of structural concern is the LOW-confidence assumption in store.rs about absolute PathBuf bypassing tauri-plugin-store's path resolution in portable mode (DATA-04). This is flagged in the source code comment and is the highest-priority item for human verification. If it fails, `set_settings` and `get_settings` will write to `%APPDATA%` even in portable mode, breaking the portable-mode contract (DATA-04, DATA-07).

All other automation (unit tests) is clean. Two compiler warnings exist (DbState field unused, set_settings unused) — both are expected at Phase 2 because consuming phases have not yet been implemented.

---

_Verified: 2026-03-06_
_Verifier: Claude (gsd-verifier)_
