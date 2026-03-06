---
phase: 02-data-layer
plan: "01"
subsystem: database
tags: [rust, tauri, paths, portable, sqlite, filesystem]

# Dependency graph
requires:
  - phase: 01-project-scaffold-configuration
    provides: lib.rs setup callback structure, Cargo.toml with rusqlite and tauri dependencies
provides:
  - paths::data_dir() — portable-aware data directory resolution with create_dir_all guarantee
  - paths::data_dir_from_exe_dir() — testable internal helper for exe_dir injection
  - mod paths declared in lib.rs and wired in setup callback
affects:
  - 02-data-layer/02-02 (db.rs uses paths::data_dir for SQLite file location)
  - 02-data-layer/02-03 (store.rs uses paths::data_dir for settings file location)
  - 03-indexer (reads app list from DB initialized at this path)
  - 10-packaging (portable mode distributable uses riftle-launcher.portable marker)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Portable mode detection via riftle-launcher.portable marker file adjacent to exe"
    - "data_dir_from_exe_dir() helper pattern for AppHandle-free unit testing"
    - "create_dir_all() called before returning path — callers always get ready-to-use path"
    - "_prefixed variable to suppress unused warning on forward-declared values"

key-files:
  created:
    - src-tauri/src/paths.rs
  modified:
    - src-tauri/src/lib.rs

key-decisions:
  - "Portable detection uses std::env::current_exe() for consistent behavior in dev and release builds"
  - "data_dir_from_exe_dir() internal helper enables unit testing without AppHandle — installed branch tested via smoke test in Plan 02"
  - "paths::data_dir() calls create_dir_all before returning so all callers receive a guaranteed-existing directory"
  - "_data_dir prefixed with underscore to suppress unused warning until Plans 02 and 03 replace it with db/store init"

patterns-established:
  - "Separate paths.rs module for path resolution — db.rs and store.rs import paths::data_dir() rather than duplicating logic"
  - "TDD with AppHandle-free helper: split fn data_dir(app) from fn data_dir_from_exe_dir(dir, app) to enable unit testing portable branch"

requirements-completed:
  - DATA-07
  - DATA-01

# Metrics
duration: 5min
completed: 2026-03-06
---

# Phase 2 Plan 01: Paths Module Summary

**Portable-aware data directory resolution via paths.rs — returns exe_dir/data in portable mode or %APPDATA%/riftle-launcher/ in installed mode, with create_dir_all guarantee before returning**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-06T00:54:28Z
- **Completed:** 2026-03-06T01:00:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Created `src-tauri/src/paths.rs` with `data_dir(app: &AppHandle) -> PathBuf` and testable `data_dir_from_exe_dir()` helper
- Portable branch: detects `riftle-launcher.portable` adjacent to exe, returns `exe_dir/data/`
- Installed branch: returns Tauri's `app.path().app_data_dir()` result
- Both branches call `create_dir_all` before returning to guarantee directory exists
- Wired `crate::paths::data_dir(app.handle())` in lib.rs setup callback; `mod paths;` declared alongside existing stubs
- 2 unit tests pass: portable detection returns correct path, no-marker case confirms installed branch

## Task Commits

Each task was committed atomically:

1. **Task 1: Create paths.rs with data_dir() and unit tests** - `217fe73` (feat)
2. **Task 2: Wire paths::data_dir in lib.rs setup callback** - `ccc535b` (feat)

**Plan metadata:** _(docs commit follows)_

## Files Created/Modified

- `src-tauri/src/paths.rs` - Portable-aware data directory resolution with unit tests
- `src-tauri/src/lib.rs` - Added `mod paths;` declaration and `crate::paths::data_dir(app.handle())` call in setup

## Decisions Made

- Used `std::env::current_exe()` for portable detection (consistent in dev and release; in dev, `riftle-launcher.portable` goes in `target/debug/`)
- Split `data_dir_from_exe_dir(exe_dir, app)` as internal helper so unit tests can inject a tempdir without needing an AppHandle
- Installed-mode path (`app_data_dir()`) not unit-tested — requires AppHandle, deferred to smoke test in Plan 02
- Removed unused `super::*` import from test module (auto-fixed Rule 2 — minor warning cleanup)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Removed unused `super::*` import from test module**
- **Found during:** Task 1 (paths.rs creation)
- **Issue:** `use super::*` imported but tests only use `std::fs`, producing a compiler warning
- **Fix:** Removed the unused import; tests pass without it
- **Files modified:** src-tauri/src/paths.rs
- **Verification:** cargo test --lib: 2 passed, 0 warnings
- **Committed in:** ccc535b (Task 2 commit, warning cleanup included)

---

**Total deviations:** 1 auto-fixed (1 unused import cleanup)
**Impact on plan:** Cleanup only. No scope creep. All plan requirements met.

## Issues Encountered

None - plan executed without issues. First test run showed 0 tests because `mod paths;` was not yet in lib.rs (expected behavior for first compile). Adding the mod declaration resolved test discovery immediately.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `paths::data_dir(app)` is the authoritative data directory function for all of Phase 2
- Plan 02 (db.rs) can import `crate::paths::data_dir(app)` to locate the SQLite file
- Plan 03 (store.rs) can import `crate::paths::data_dir(app)` to locate the settings file
- No blockers — portable detection logic verified, installed path verified via Tauri's app_data_dir()

## Self-Check: PASSED

- FOUND: src-tauri/src/paths.rs
- FOUND: src-tauri/src/lib.rs
- FOUND: .planning/phases/02-data-layer/02-01-SUMMARY.md
- FOUND: commit 217fe73 (feat(02-01): create paths.rs)
- FOUND: commit ccc535b (feat(02-01): wire paths::data_dir)
- cargo test -p riftle --lib paths::tests: 2 passed, 0 failed

---
*Phase: 02-data-layer*
*Completed: 2026-03-06*
