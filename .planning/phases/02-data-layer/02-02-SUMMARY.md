---
phase: 02-data-layer
plan: "02"
subsystem: database
tags: [sqlite, rusqlite, tauri-state, tdd]
dependency_graph:
  requires:
    - 02-01  # paths.rs portable-aware data directory
  provides:
    - db::DbState   # Tauri managed state wrapper for shared SQLite connection
    - db::AppRecord # App record struct for indexer and search
    - db::init_db   # SQLite file init with silent corruption reset
    - db::init_db_connection  # Schema DDL (used by unit tests with in-memory DB)
    - db::upsert_app          # Insert/update preserving launch_count
    - db::get_all_apps        # Full table scan returning Vec<AppRecord>
    - db::increment_launch_count  # Atomic launch tracking
  affects:
    - Phase 3 (Indexer) — calls upsert_app and get_all_apps via DbState
    - Phase 4 (Search) — calls get_all_apps via DbState to build nucleo index
    - Phase 6 (Launch Actions) — calls increment_launch_count via DbState
tech_stack:
  added:
    - rusqlite 0.31 with bundled SQLite feature
    - serde/serde_json for AppRecord serialization
  patterns:
    - TDD Red-Green-Refactor for db.rs
    - ON CONFLICT DO UPDATE SET for upsert preserving launch_count
    - Arc<Mutex<Connection>> shared state pattern
    - Silent DB corruption reset (delete + recreate)
    - std::time for Unix timestamps (no chrono dependency)
key_files:
  created:
    - src-tauri/src/db.rs  # Full SQLite data layer implementation with 5 unit tests
  modified:
    - src-tauri/src/lib.rs  # DbState registered as Tauri managed state in setup callback
decisions:
  - ON CONFLICT DO UPDATE SET (not INSERT OR REPLACE) preserves launch_count on re-index
  - init_db_connection() separated from init_db() to enable in-memory testing without AppHandle
  - Silent corruption reset: delete file and retry once on DDL failure
  - tauri::Manager trait import required for app.manage() in Tauri v2
metrics:
  duration: "4min"
  completed_date: "2026-03-06"
  tasks_completed: 2
  files_modified: 2
---

# Phase 2 Plan 02: SQLite Data Layer Summary

SQLite data layer with upsert-safe AppRecord persistence, DbState Tauri managed state, and 5 unit tests; DbState wired into lib.rs setup callback for Phase 3/4 consumption.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Implement db.rs with AppRecord, DbState, all four functions, unit tests | 66279c6 | src-tauri/src/db.rs |
| 2 | Wire DbState into lib.rs setup callback | ef4afa0 | src-tauri/src/lib.rs |

## What Was Built

**src-tauri/src/db.rs** — Full SQLite data layer:
- `DbState(Arc<Mutex<Connection>>)` — Tauri managed state wrapper enabling safe cross-thread access
- `AppRecord` — Serializable struct with 7 fields matching the apps table schema
- `init_db_connection(&Connection)` — Runs CREATE TABLE IF NOT EXISTS DDL; separated for testability
- `init_db(&Path)` — Opens/creates SQLite file, runs DDL; silently resets on corruption
- `upsert_app(&Connection, &AppRecord)` — INSERT with ON CONFLICT DO UPDATE preserving launch_count
- `get_all_apps(&Connection)` — Full table scan; returns Vec<AppRecord> for search index building
- `increment_launch_count(&Connection, &str)` — Atomic count increment + Unix timestamp via std::time
- 5 unit tests covering: schema init, insert, upsert preserving launch_count, empty result, launch tracking

**src-tauri/src/lib.rs** — Setup callback updated:
- Replaced `let _data_dir` placeholder with full DB init sequence
- `let db_path = data_dir.join("launcher.db")` — SQLite file path in portable-aware directory
- `app.manage(crate::db::DbState(Arc::new(Mutex::new(conn))))` — State registered for Phase 3/4 retrieval

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing tauri::Manager trait import for app.manage()**
- **Found during:** Task 2 compilation
- **Issue:** `app.manage()` is a trait method from `tauri::Manager`; calling it without the trait in scope causes E0599
- **Fix:** Added `use tauri::Manager;` at the top of lib.rs
- **Files modified:** src-tauri/src/lib.rs
- **Commit:** ef4afa0

## Verification Results

1. `cargo test --lib db::tests` — 5 tests pass: test_schema_init, test_upsert_insert, test_upsert_update_preserves_launch_count, test_get_all_apps_empty, test_increment_launch_count
2. `cargo test --lib` — 7 tests pass (5 db + 2 paths from Plan 01)
3. Schema: apps table with id (TEXT PK), name (TEXT NOT NULL), path (TEXT NOT NULL), icon_path (TEXT), source (TEXT NOT NULL), last_launched (INTEGER), launch_count (INTEGER DEFAULT 0)
4. launch_count preserved: ON CONFLICT DO UPDATE SET only updates name/path/icon_path/source
5. DbState registered: `app.manage(crate::db::DbState(Arc::new(Mutex::new(conn))))` in lib.rs setup

## Key Design Notes

The upsert design uses `ON CONFLICT(id) DO UPDATE SET` instead of `INSERT OR REPLACE`. This is critical: INSERT OR REPLACE would delete the existing row and insert a new one, resetting launch_count to 0. The ON CONFLICT approach preserves launch_count and last_launched while updating discovery fields — Phase 3 can safely re-index without destroying usage history.

The `init_db_connection()` / `init_db()` split enables true unit testing with `Connection::open_in_memory()` without needing a filesystem or AppHandle. All 5 tests use in-memory connections — fast and hermetic.

## Self-Check: PASSED
