---
phase: 06-launch-actions
plan: 01
subsystem: api
tags: [rust, tauri, windows-api, shellexecutew, launch, elevation, uac, system-commands]

# Dependency graph
requires:
  - phase: 05-launcher-window-ui
    provides: App.vue with invoke() stubs for launch, launch_elevated, run_system_command
  - phase: 04-search-engine
    provides: search() command, SearchResult with id/kind fields, system command IDs prefixed with "system:"
  - phase: 03-indexer
    provides: DB with apps table; increment_launch_count(); rebuild_index()
  - phase: 02-data-layer
    provides: DbState managed state; rusqlite Connection pattern
provides:
  - launch() Tauri command: DB path lookup, ShellExecuteW NULL verb, window hide-first, MRU update
  - launch_elevated() Tauri command: ShellExecuteW runas verb, UAC cancel detection, post-success hide
  - run_system_command() Tauri command: lock/shutdown/restart/sleep dispatch via windows-sys
  - invoke_handler wired with all 6 commands (3 existing + 3 new)
affects: [07-context-menu, 08-settings-window]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "app.state::<T>() must be bound to a local variable before .0.lock() — temporary dropped while borrowed"
    - "launch_elevated window-hide ownership: Rust owns the hide decision so UAC cancel leaves launcher open"
    - "system: prefix pattern: search index uses 'system:lock' IDs; strip prefix before matching in run_system_command"

key-files:
  created: []
  modified:
    - src-tauri/src/commands.rs
    - src-tauri/src/system_commands.rs
    - src-tauri/src/lib.rs
    - src/App.vue

key-decisions:
  - "launch_elevated does NOT hide window before ShellExecuteW — Rust owns hide decision so UAC cancel leaves launcher open; frontend must not call hideWindow() after invoke('launch_elevated')"
  - "system command IDs include 'system:' prefix from search index — strip in run_system_command before matching"
  - "app.state::<T>() temporary must be bound to local variable before locking — borrow checker requirement"

patterns-established:
  - "Pattern: DB access pattern — let db_state = app.state::<DbState>(); let conn = db_state.0.lock().unwrap();"
  - "Pattern: scoped MutexGuard — wrap DB access in {} block so guard drops before calling rebuild_index"
  - "Pattern: Rust owns hide for asymmetric commands — when a command has conditional hide logic, Rust owns it; frontend must not double-hide"

requirements-completed: [LAUN-01, LAUN-02, LAUN-03, LAUN-04]

# Metrics
duration: 45min
completed: 2026-03-07
---

# Phase 6 Plan 01: Launch Actions Summary

**ShellExecuteW-based launch commands wired end-to-end: normal launch, UAC-aware elevated launch, and system commands (lock/shutdown/restart/sleep) — all verified by smoke test**

## Performance

- **Duration:** ~45 min (implementation + smoke test + bug fixes)
- **Started:** 2026-03-07T00:00:00Z
- **Completed:** 2026-03-07
- **Tasks:** 3/3 (Tasks 1-2 automated, Task 3 human smoke test with fixes)
- **Files modified:** 4

## Accomplishments

- Implemented `launch()`: DB path lookup via rusqlite, window hide-first (LAUN-04), ShellExecuteW with NULL verb, increment_launch_count + rebuild_index on success
- Implemented `launch_elevated()`: ShellExecuteW with "runas" verb, UAC cancel detection via GetLastError/ERROR_CANCELLED 1223, post-success hide — launcher stays open on cancel (LAUN-02)
- Implemented `run_system_command()`: dispatches lock (LockWorkStation), shutdown/restart (shutdown.exe), sleep (SetSuspendState) — all strip the "system:" ID prefix before matching
- `to_wide_null()` helper for &str → null-terminated Vec<u16> with 3 passing unit tests
- lib.rs invoke_handler extended from 3 to 6 commands
- All 37 cargo tests green throughout

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement commands.rs — launch and launch_elevated** - `e2cd046` (feat)
2. **Task 2: Implement system_commands.rs and wire lib.rs invoke_handler** - `95336f5` (feat)
3. **Smoke test bug fix — UAC cancel now leaves launcher open** - `235abe7` (fix)
4. **Smoke test bug fix — system commands dispatch correctly** - `1af2949` (fix)

## Files Created/Modified

- `src-tauri/src/commands.rs` — Full implementation: launch(), launch_elevated(), to_wide_null() + unit tests
- `src-tauri/src/system_commands.rs` — Full implementation: run_system_command() dispatching lock/shutdown/restart/sleep
- `src-tauri/src/lib.rs` — invoke_handler extended with 3 new commands
- `src/App.vue` — launchElevated() fixed: removed unconditional hideWindow() call

## Decisions Made

- **launch_elevated window-hide ownership:** Rust owns the hide-or-not decision for `launch_elevated`. On success, Rust hides; on UAC cancel, Rust does nothing. The frontend must not call `hideWindow()` after `invoke('launch_elevated')` returns — that would always close the launcher, breaking LAUN-02.
- **system: prefix handling:** The search index uses IDs like `"system:lock"` and `"system:shutdown"`. The `run_system_command` command strips the prefix with `strip_prefix("system:")` before the match — this keeps the match arms clean without requiring the search layer to know about implementation details.
- **DB state temporary lifetime:** `app.state::<T>()` returns a temporary that is dropped at end of statement. Binding to a local variable (`let db_state = app.state::<T>()`) before calling `.0.lock()` is the correct pattern across all DB access sites.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Rust temporary lifetime in DB state access**
- **Found during:** Task 1 (first cargo compile)
- **Issue:** Plan showed `let conn = app.state::<crate::db::DbState>().0.lock().unwrap()` inline. Rust drops the `State<T>` temporary at end-of-statement, leaving the MutexGuard pointing to freed memory. Compiler error E0716.
- **Fix:** Bind `app.state::<T>()` to a local variable before calling `.0.lock()` — applied to all 4 DB access sites in commands.rs.
- **Files modified:** src-tauri/src/commands.rs
- **Committed in:** e2cd046 (Task 1 commit, part of implementation)

### Post-Smoke-Test Bug Fixes

**2. [Rule 1 - Bug] UAC cancel closed the launcher (LAUN-02 regression)**
- **Found during:** Task 3 (human smoke test)
- **Issue:** `launchElevated()` in App.vue called `await hideWindow()` unconditionally after `invoke('launch_elevated')` returned. Since Rust returns `Ok(())` on UAC cancel without hiding, the frontend's `hideWindow()` call closed the launcher on cancel.
- **Fix:** Removed the `hideWindow()` call from `launchElevated()`. The Rust command owns the window-hide decision for elevated launch — it hides on success and does nothing on cancel.
- **Files modified:** src/App.vue
- **Committed in:** 235abe7

**3. [Rule 1 - Bug] System commands did nothing (lock/shutdown/restart/sleep all no-op)**
- **Found during:** Task 3 (human smoke test — "> lock" did nothing)
- **Issue:** The search index emits system command IDs as `"system:lock"`, `"system:shutdown"`, etc. `run_system_command()` matched against bare keys (`"lock"`, `"shutdown"`), so every system command fell through to the `_` unknown arm and was silently ignored.
- **Fix:** Added `let cmd_key = cmd.strip_prefix("system:").unwrap_or(cmd.as_str())` and matched on `cmd_key` instead of `cmd`.
- **Files modified:** src-tauri/src/system_commands.rs
- **Committed in:** 1af2949

---

**Total deviations:** 3 auto-fixed (2 bugs from smoke test, 1 borrow-checker fix during compilation)
**Impact on plan:** All three fixes are correctness bugs — no scope creep. LAUN-01 through LAUN-04 all verified.

## Issues Encountered

- Borrow checker rejected the DB access pattern from the plan's inline code (E0716) — required local variable binding. Standard Rust pattern, resolved on first compile.
- Two functional bugs caught by smoke test that were logic errors in either the frontend (unconditional hide) or the Rust backend (missing prefix strip). Both were simple targeted fixes.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Phase 6 complete: all four LAUN requirements verified by smoke test (normal launch, UAC cancel, elevated launch, system commands)
- Phase 7 (Context Menu) can proceed: App.vue is stable, launch paths work, `hideWindow()` pattern is established
- Note for Phase 8: `requires_elevation` field in SearchResult is hardcoded to `false` for all apps — Phase 8 (Settings) or a future phase may want to wire real elevation detection from PE headers

---
*Phase: 06-launch-actions*
*Completed: 2026-03-07*
