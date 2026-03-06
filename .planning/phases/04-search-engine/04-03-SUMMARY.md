---
phase: 04-search-engine
plan: 03
subsystem: search
tags: [rust, tauri, nucleo-matcher, managed-state, rwlock, search-engine, command-registration]

# Dependency graph
requires:
  - phase: 04-search-engine/04-01
    provides: search.rs struct shells (SearchIndex, SearchIndexState, SearchResult), system_command.png asset
  - phase: 04-search-engine/04-02
    provides: score_and_rank, search_system_commands, ensure_system_command_icon pure functions implemented and tested
  - phase: 03-indexer
    provides: run_full_index, reindex() Tauri command, DbState managed state, AppRecord struct
provides:
  - search() Tauri command registered in invoke_handler and callable from frontend
  - init_search_index(): reads DB via DbState at startup, manages SearchIndexState(Arc<RwLock<SearchIndex>>)
  - rebuild_index(): atomically swaps SearchIndex after reindex() via RwLock write guard
  - SearchIndexState populated at startup (after run_full_index) and refreshed after each reindex
  - ensure_system_command_icon() called at startup with non-fatal error handling
  - Full search pipeline wired end-to-end: frontend query -> search() -> score_and_rank or search_system_commands
affects: [05-launcher-ui, 06-launch-actions]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Tauri v2 try_state returns Option<State<T>> not Result — use if let Some(state) pattern"
    - "RwLock write swap pattern for atomic index rebuild: acquire write guard, deref assign *guard = new_index"
    - "AppHandle propagation: add app: tauri::AppHandle to Tauri command signature to pass to helper functions"
    - "Non-fatal startup errors: if let Err(e) = ... { eprintln!(...) } — app continues on icon write failure"

key-files:
  created: []
  modified:
    - src-tauri/src/search.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/indexer.rs

key-decisions:
  - "Tauri v2 try_state returns Option<State<T>> (not Result as plan spec assumed) — use if let Some pattern for rebuild_index"
  - "reindex() command required app: tauri::AppHandle parameter addition to support rebuild_index(&app) call — plan spec gap auto-fixed"
  - "PoisonError<_> type annotation required in closure for RwLock write() unwrap_or_else in Tauri v2 context"

patterns-established:
  - "SearchIndexState managed state lifecycle: init (app.manage) at startup, swap (write lock deref) on rebuild — never empty during rebuild"
  - "search() command: empty guard -> '>' system routing -> fuzzy app search pipeline"

requirements-completed: [SRCH-01, SRCH-02, SRCH-03, SRCH-04, SRCH-05]

# Metrics
duration: 3min
completed: 2026-03-06
---

# Phase 4 Plan 03: Search Engine Tauri Wiring Summary

**search() Tauri command wired end-to-end with SearchIndexState managed state, startup init, and post-reindex atomic rebuild via RwLock write swap**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-06T18:20:34Z
- **Completed:** 2026-03-06T18:24:10Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Replaced 3 `todo!()` stubs with real implementations: `init_search_index`, `rebuild_index`, `search`
- `search()` Tauri command registered in invoke_handler and callable from frontend
- `SearchIndexState` populated at startup via `init_search_index()` called after `run_full_index()`
- Atomic index rebuild wired into `reindex()` background thread via `rebuild_index(&app)`
- `ensure_system_command_icon()` called at startup with eprintln non-fatal error handling
- Full test suite: 34 passed, 0 failed, 2 ignored — no regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement search() command handler, init_search_index(), rebuild_index()** - `2f20272` (feat)
2. **Task 2: Wire search into lib.rs and hook rebuild_index into reindex()** - `1ad50a9` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src-tauri/src/search.rs` - init_search_index, rebuild_index, search() implemented; tauri::Manager + get_all_apps imports added; no todo!() stubs remain
- `src-tauri/src/lib.rs` - ensure_system_command_icon + init_search_index calls added after run_full_index; crate::search::search added to invoke_handler
- `src-tauri/src/indexer.rs` - app: tauri::AppHandle added to reindex() signature; crate::search::rebuild_index(&app) called after run_full_index

## Decisions Made
- `try_state::<T>()` returns `Option<State<T>>` in Tauri v2 — plan spec referenced `Result` pattern; corrected to `if let Some(state)` idiom
- Added `app: tauri::AppHandle` to `reindex()` command signature — plan spec stated it was already present but it was not; adding is the correct fix to enable `rebuild_index(&app)` call
- `PoisonError<_>` explicit type annotation required in the `unwrap_or_else` closure for `RwLock::write()` to resolve type inference in Tauri v2 build context

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Tauri v2 try_state returns Option not Result**
- **Found during:** Task 1 (implement rebuild_index)
- **Issue:** Plan spec showed `if let Ok(state) = app.try_state::<SearchIndexState>()` but Tauri v2's `try_state` returns `Option<State<T>>`, not `Result`. The `Ok` pattern causes E0308 mismatched types.
- **Fix:** Changed to `if let Some(state) = app.try_state::<SearchIndexState>()` and added `std::sync::PoisonError<_>` type annotation to closure parameter
- **Files modified:** src-tauri/src/search.rs
- **Verification:** cargo test search — 14/14 passed
- **Committed in:** 2f20272 (Task 1 commit)

**2. [Rule 3 - Blocking] reindex() lacked app: tauri::AppHandle parameter**
- **Found during:** Task 2 (hook rebuild_index into reindex)
- **Issue:** Plan spec stated "The reindex() function already has `app: tauri::AppHandle` as its first parameter" but the actual function had no AppHandle parameter, making `crate::search::rebuild_index(&app)` undefined.
- **Fix:** Added `app: tauri::AppHandle` as first parameter to `reindex()` Tauri command; Tauri v2 automatically injects AppHandle when declared as a command parameter
- **Files modified:** src-tauri/src/indexer.rs
- **Verification:** cargo test — 34/34 passed, cargo build succeeded
- **Committed in:** 1ad50a9 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 Rule 1 spec mismatch, 1 Rule 3 blocking)
**Impact on plan:** Both auto-fixes necessary for compilation. No scope creep — same semantic intent as plan.

## Issues Encountered
- Tauri v2 API mismatch on `try_state` return type (plan referenced v1-style Result pattern) — resolved immediately via compile error
- reindex() signature gap in plan spec — resolved by adding AppHandle parameter per Tauri v2 injection pattern

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 4 complete: search() command is fully wired and callable from frontend
- Phase 5 (Launcher Window UI) can begin: invoke('search', { query }) will return Vec<SearchResult>
- SearchResult fields: id, name, icon_path, path, kind ("app" or "system")
- System commands route via '>' prefix; empty query returns empty list

---
*Phase: 04-search-engine*
*Completed: 2026-03-06*

## Self-Check: PASSED
- FOUND: src-tauri/src/search.rs
- FOUND: src-tauri/src/lib.rs
- FOUND: src-tauri/src/indexer.rs
- FOUND: .planning/phases/04-search-engine/04-03-SUMMARY.md
- FOUND: 2f20272 (feat: search.rs implementations)
- FOUND: 1ad50a9 (feat: lib.rs and indexer.rs wiring)
