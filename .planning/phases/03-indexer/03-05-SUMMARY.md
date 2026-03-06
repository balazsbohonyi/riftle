---
phase: 03-indexer
plan: 05
subsystem: indexer
tags: [rust, threading, atomic, mpsc, notify-debouncer-mini, tauri-command, background-tasks]

# Dependency graph
requires:
  - phase: 03-indexer/03-04
    provides: run_full_index synchronous indexer entry point
  - phase: 02-data-layer/02-02
    provides: DbState (Arc<Mutex<Connection>>)
  - phase: 02-data-layer/02-03
    provides: Settings struct with reindex_interval field
provides:
  - try_start_index: AtomicBool-guarded indexer trigger (silently drops concurrent calls)
  - start_background_tasks: timer thread (reindex_interval deadline, 1s poll) + filesystem watcher (500ms debounce on Start Menu dirs)
  - reindex: Tauri command for fire-and-forget frontend-triggered re-index with timer reset
  - lib.rs integration: run_full_index wired into setup(), all state managed, reindex registered in invoke_handler
affects:
  - 04-search (will read from the DB that indexer keeps fresh)
  - 08-settings (Phase 8 will wire settings changes to trigger reindex via managed state)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "compare_exchange(false, true, AcqRel, Relaxed): standard AtomicBool guard — only one thread wins, second drops silently"
    - "Timer thread: 1s sleep poll + try_recv() deadline reset via mpsc::Sender<()>"
    - "Filesystem watcher: notify-debouncer-mini with 500ms debounce, non-fatal errors (eprintln! + return)"
    - "Tauri managed state triple: Arc<AtomicBool> + PathBuf + Arc<Mutex<mpsc::Sender<()>>> for reindex() command"

key-files:
  created: []
  modified:
    - src-tauri/src/indexer.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "Settings::default() in reindex() command — real settings not in managed state yet; Phase 8 will wire set_settings changes to trigger reindex"
  - "Filesystem watcher watches Start Menu dirs only (user + ProgramData) — not desktop/PATH; per INDX-07 locked scope decision"
  - "Watcher and timer failures are non-fatal — eprintln! and thread return; app continues normally"
  - "app.manage(data_dir.clone()) stores PathBuf directly as managed state for reindex() command retrieval via tauri::State<PathBuf>"

patterns-established:
  - "AtomicBool compare_exchange guard: use AcqRel/Relaxed ordering; reset with store(false, Release) after work completes"
  - "Background tasks in #[cfg(desktop)] block — consistent with plugin registration pattern from Phase 1"
  - "Timer reset via mpsc channel: Sender stored as Arc<Mutex<Sender<()>>> in managed state, send() called after manual reindex"

requirements-completed:
  - INDX-06
  - INDX-07
  - INDX-08

# Metrics
duration: 3min
completed: 2026-03-06
---

# Phase 3 Plan 05: Background Coordination Layer Summary

**AtomicBool-guarded try_start_index, background timer + filesystem watcher via notify-debouncer-mini, and reindex Tauri command wired into lib.rs setup() — all 20 tests GREEN (2 ignored), 0 compile errors**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-06T09:29:51Z
- **Completed:** 2026-03-06T09:32:56Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Replaced three remaining todo!() stubs (try_start_index, start_background_tasks, reindex) with full implementations
- Timer thread uses 1s sleep poll with mpsc::Sender<()> reset signal — fires every reindex_interval minutes
- Filesystem watcher uses notify-debouncer-mini with 500ms debounce on Start Menu directories (user + ProgramData)
- lib.rs setup() now calls run_full_index synchronously then start_background_tasks; all state managed correctly
- All 20 non-ignored tests pass across db, store, paths, and indexer modules

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement try_start_index, start_background_tasks, and reindex command** - `6d4b763` (feat)
2. **Task 2: Wire indexer into lib.rs setup() and register reindex command** - `e5f3349` (feat)

**Plan metadata:** _(final docs commit — see below)_

## Files Created/Modified
- `src-tauri/src/indexer.rs` - try_start_index, start_background_tasks, reindex implemented; 3 test stubs converted from ignore/should_panic to proper GREEN tests
- `src-tauri/src/lib.rs` - AtomicBool import added; #[cfg(desktop)] Phase 3 block with run_full_index + start_background_tasks + managed state; reindex registered in invoke_handler

## Decisions Made
- `Settings::default()` used in reindex() command rather than reading from store — real settings are not managed state in Phase 3. Phase 8 will wire settings changes properly. This is a documented known simplification.
- Filesystem watcher scope limited to Start Menu dirs (user APPDATA + PROGRAMDATA paths) per INDX-07 locked decision — desktop/PATH not watched since they change rarely.
- Watcher and timer setup failures are non-fatal: eprintln! and thread returns silently. App continues without background refresh rather than crashing — correct per plan's "non-fatal errors" truth.
- `app.manage(data_dir.clone())` stores raw `PathBuf` as managed state (not wrapped in Arc/Mutex) since PathBuf is Clone and reindex() only needs to read it.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- cargo test argument syntax for multiple named tests required `--` separator (`cargo test -p riftle --lib -- test1 test2`), not positional args without separator. Corrected immediately, no functional impact.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 3 (Indexer) is now complete — all 5 plans executed
- Index runs on startup, refreshes on timer, re-indexes on Start Menu filesystem changes, and is callable from frontend
- Phase 4 (Search Engine) can proceed — apps table is populated and maintained; search reads from the same DB

## Self-Check: PASSED

- FOUND: src-tauri/src/indexer.rs
- FOUND: src-tauri/src/lib.rs
- FOUND: .planning/phases/03-indexer/03-05-SUMMARY.md
- Verified: commit 6d4b763 (feat(03-05): implement try_start_index, start_background_tasks, reindex)
- Verified: commit e5f3349 (feat(03-05): wire indexer into lib.rs setup())
- Verified: cargo test -p riftle --lib → 20 passed, 0 failed, 2 ignored
- Verified: cargo check -p riftle → 0 errors

---
*Phase: 03-indexer*
*Completed: 2026-03-06*
