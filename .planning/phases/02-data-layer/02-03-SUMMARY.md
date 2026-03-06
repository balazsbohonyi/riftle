---
phase: 02-data-layer
plan: "03"
subsystem: database
tags: [tauri-plugin-store, serde, serde_json, settings, persistence, rust]

# Dependency graph
requires:
  - phase: 02-data-layer/02-01
    provides: paths::data_dir() portable-aware data directory resolution
  - phase: 02-data-layer/02-02
    provides: db::init_db and DbState — established setup callback pattern this plan extends
provides:
  - Settings struct with 8 fields and serde defaults for forward-compatible JSON
  - get_settings() returning Settings::default() on missing or malformed JSON (silent reset)
  - set_settings() persisting full Settings struct via tauri-plugin-store
  - lib.rs setup wired to call get_settings() ensuring settings.json created on first run
affects:
  - 08-settings-window
  - 09-global-hotkey
  - 03-indexer
  - 04-search-engine

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Settings as single full-struct replace (no partial patch merge)
    - serde default functions for primitive fields (default_hotkey, default_theme, default_opacity, default_reindex_interval)
    - unwrap_or_default() for silent reset on malformed JSON
    - app.store(absolute PathBuf) via StoreExt to bypass BaseDirectory::AppData resolution

key-files:
  created:
    - src-tauri/src/store.rs
  modified:
    - src-tauri/src/lib.rs

key-decisions:
  - "Settings uses full-struct replace (not partial patch) for simplicity — Phase 8 reads current, updates fields, writes full struct back"
  - "Silent reset on malformed settings.json (unwrap_or_default) — avoids startup failure if user corrupts file"
  - "get_settings() called in lib.rs setup to trigger first-run settings.json creation before any other subsystem needs it"
  - "eprintln! for store error logging — lightweight, no tracing crate needed until later phases"

patterns-established:
  - "Serde default functions pattern: fn default_X() -> T used for #[serde(default = \"default_X\")] on primitive fields"
  - "AppHandle store functions: pub fn get_settings(app: &AppHandle, data_dir: &Path) signature — no partial application needed"

requirements-completed: [DATA-04, DATA-05, DATA-06]

# Metrics
duration: 3min
completed: 2026-03-06
---

# Phase 2 Plan 03: Settings Persistence Summary

**Settings struct with 8 serde-defaulted fields persisted via tauri-plugin-store 2.4.2, with silent reset on malformed JSON and first-run initialization wired in lib.rs setup**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-06T01:13:45Z
- **Completed:** 2026-03-06T01:16:28Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Implemented `Settings` struct with all 8 fields (hotkey, theme, opacity, show_path, autostart, additional_paths, excluded_paths, reindex_interval) with correct serde defaults
- Implemented `get_settings()` with silent reset on missing/malformed JSON and `set_settings()` persisting full struct via tauri-plugin-store
- Wired `get_settings()` into lib.rs setup callback to ensure settings.json is written with defaults on first run
- 4 unit tests pass: defaults, serde round-trip, partial JSON deserialization, malformed JSON fallback

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement store.rs with Settings struct, get_settings, set_settings, and unit tests** - `c304b48` (feat)
2. **Task 2: Wire get_settings into lib.rs setup callback** - `b0683e5` (feat)

**Plan metadata:** TBD (docs: complete plan)

_Note: TDD tasks may have multiple commits (test -> feat -> refactor). Tasks 1 combined tests + implementation per plan spec (unit tests cannot fail independently since implementation is inlined)._

## Files Created/Modified
- `src-tauri/src/store.rs` - Settings struct, get_settings(), set_settings() with 4 unit tests
- `src-tauri/src/lib.rs` - Added get_settings() call in setup callback for first-run initialization

## Decisions Made
- Settings uses full-struct replace (not partial patch) for simplicity — Phase 8 reads current, updates fields, writes full struct back
- Silent reset on malformed settings.json (unwrap_or_default) avoids startup failure if user corrupts file
- eprintln! for store error logging — lightweight, no tracing crate needed until later phases

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None - cargo test ran cleanly, all 4 store tests and all 11 lib tests (paths + db + store) pass on first attempt.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Settings contract established: DATA-04, DATA-05, DATA-06 requirements complete
- Phase 3 (Indexer) can reference `Settings.additional_paths` and `Settings.excluded_paths`
- Phase 8 (Settings Window) can read/write settings via `get_settings`/`set_settings` commands
- Phase 9 (Global Hotkey) can read `Settings.hotkey`
- All Phase 2 plans (01, 02, 03) are now complete — data layer foundation ready

## Self-Check: PASSED

- src-tauri/src/store.rs: FOUND
- src-tauri/src/lib.rs: FOUND
- .planning/phases/02-data-layer/02-03-SUMMARY.md: FOUND
- Commit c304b48 (Task 1): FOUND
- Commit b0683e5 (Task 2): FOUND
- All 11 lib tests pass (4 store + 2 paths + 5 db)

---
*Phase: 02-data-layer*
*Completed: 2026-03-06*
