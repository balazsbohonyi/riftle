---
phase: 03-indexer
plan: 04
subsystem: indexer
tags: [rust, sqlite, threading, arc-mutex, icon-extraction, windows]

# Dependency graph
requires:
  - phase: 03-indexer/03-02
    provides: crawl_dir, get_index_paths, make_app_record, icon_filename, prune_stale primitives
  - phase: 03-indexer/03-03
    provides: ensure_generic_icon, extract_icon_png GDI pipeline
  - phase: 02-data-layer/02-02
    provides: upsert_app, AppRecord, DbState (Arc<Mutex<Connection>>)
provides:
  - run_full_index: synchronous entry point wiring all indexer primitives into a complete index run

affects:
  - 03-indexer/03-05 (start_background_tasks calls run_full_index or try_start_index; lib.rs wiring in Plan 05)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Lock scope as block: acquire Arc<Mutex<Connection>>.lock().unwrap() inside {} block — releases before thread spawn"
    - "Per-app icon thread: skip spawn if icon_file.exists() (re-index optimization)"
    - "rusqlite::params! macro used directly in spawned thread's conn.execute() call"

key-files:
  created: []
  modified:
    - src-tauri/src/indexer.rs

key-decisions:
  - "app.id used as icon_filename key (not app.path) — id is already normalized lowercase path, cleaner canonical key"
  - "DB lock scope via block (not explicit drop) — lock released before std::thread::spawn, never held across GDI calls"
  - "Icon skip optimization: icon_file.exists() check prevents re-extraction on re-index runs for unchanged apps"
  - "eprintln! for ensure_generic_icon failure — non-fatal, consistent with Phase 2 pattern"

patterns-established:
  - "DB lock scope isolation: always use {} block to drop Mutex guard before spawning threads — prevents deadlock"
  - "Icon thread independence: thread owns Arc clone, PathBuf clone, String clone — no borrows from outer scope"

requirements-completed:
  - INDX-01
  - INDX-03
  - INDX-04
  - INDX-05

# Metrics
duration: 2min
completed: 2026-03-06
---

# Phase 3 Plan 04: run_full_index Implementation Summary

**run_full_index implemented — synchronous indexer entry point wiring crawl, upsert, per-app GDI icon threads, and stale pruning; DB lock scoped to block (never held across GDI); 7 tests GREEN**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-06T09:22:32Z
- **Completed:** 2026-03-06T09:24:02Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Replaced `run_full_index` todo!() stub with complete implementation assembling all Plan 02 and 03 primitives
- DB lock correctly scoped to a block (drops before thread spawn) — prevents holding lock during GDI calls
- Icon skip optimization: `icon_file.exists()` check avoids redundant extraction on re-index runs
- cargo check produces 0 errors; all 7 previously GREEN indexer tests remain GREEN

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement run_full_index** - `3ec6d12` (feat)

**Plan metadata:** _(final docs commit — see below)_

## Files Created/Modified
- `src-tauri/src/indexer.rs` - run_full_index implemented; todo!() stub replaced with 60-line implementation

## Decisions Made
- `app.id` used as icon_filename key (not `app.path`): id is the normalized lowercase path — the stable canonical key used throughout the indexer. Both are equivalent since `id = path.to_string_lossy().to_lowercase()`, but `id` is cleaner.
- Lock scope as `{}` block ensures the `MutexGuard` is dropped before `std::thread::spawn` — critical to avoid holding DB lock during GDI extraction (which can take 5-50ms per exe).
- Icon file existence check prevents re-spawning extraction threads on re-index for apps already indexed — key performance optimization documented in the plan.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `run_full_index` ready for wiring into `lib.rs` setup() in Plan 05
- `start_background_tasks` and `try_start_index` stubs remain as todo!() — Plan 05 responsibility
- No blockers for Plan 05

## Self-Check: PASSED

- FOUND: src-tauri/src/indexer.rs
- FOUND: .planning/phases/03-indexer/03-04-SUMMARY.md
- FOUND: commit 3ec6d12 (feat(03-04): implement run_full_index)
- Verified: cargo check -p riftle → 0 errors
- Verified: 7 indexer tests GREEN, 0 failures, 4 ignored (Plan 05 scope)

---
*Phase: 03-indexer*
*Completed: 2026-03-06*
