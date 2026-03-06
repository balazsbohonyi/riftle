---
phase: 03-indexer
plan: 01
subsystem: indexer
tags: [rust, lnk, notify-debouncer-mini, image, windows-sys, cargo, tdd, scaffold]

# Dependency graph
requires:
  - phase: 02-data-layer
    provides: AppRecord, upsert_app, get_all_apps, init_db_connection, DbState, Settings
provides:
  - All public function signatures for indexer: run_full_index, start_background_tasks, reindex, get_index_paths, crawl_dir, resolve_lnk, make_app_record, icon_filename, prune_stale, ensure_generic_icon, extract_icon_png, try_start_index
  - Cargo dependencies: lnk ^0.3, notify-debouncer-mini 0.4, image ^0.25 (png), Win32_Graphics_Gdi, Win32_UI_WindowsAndMessaging
  - generic.png compiled into binary via include_bytes! at src-tauri/icons/generic.png
  - 7 RED test stubs with correct should_panic, 4 ignored (2 LNK, 2 timer)
affects: [03-02, 03-03, 03-04, 03-05]

# Tech tracking
tech-stack:
  added:
    - lnk ^0.3 (Windows .lnk shortcut resolution without COM)
    - notify-debouncer-mini 0.4 (filesystem watcher with debounce)
    - image ^0.25 png-only (icon PNG encoding)
    - tempfile 3 (dev-dependency for test temp dirs)
    - Win32_Graphics_Gdi feature (GetIconInfo, GetDIBits, CreateCompatibleDC)
    - Win32_UI_WindowsAndMessaging feature (ICONINFO, DestroyIcon)
  patterns:
    - Nyquist scaffold pattern: all public API signatures written with todo! before implementation
    - TDD RED state: should_panic(expected = "not yet implemented") for todo! stubs
    - include_bytes! for bundling generic icon at compile time from src-tauri/icons/
    - #[ignore] for tests requiring external fixtures (LNK files, Plan 05 timer impl)

key-files:
  created:
    - src-tauri/icons/generic.png (32x32 PNG, 974 bytes, compiled into binary)
    - src-tauri/src/indexer.rs (complete scaffold: 12 stubs + 11 test stubs)
  modified:
    - src-tauri/Cargo.toml (added lnk, notify-debouncer-mini, image, tempfile, 2 windows-sys features)
    - src-tauri/Cargo.lock (updated with new crates)

key-decisions:
  - "include_bytes! path is ../icons/generic.png relative to src/indexer.rs (not ../../)"
  - "should_panic(expected = 'not yet implemented') not 'todo' — todo!() macro produces 'not yet implemented: ...' messages"
  - "test_timer_fires and test_timer_reset marked #[ignore] — bodies don't call any stub so should_panic can never fire"
  - "generic.png copied from 32x32.png (existing Tauri icon) — real PNG required for include_bytes! at compile time"

patterns-established:
  - "Nyquist scaffold: define all public signatures with todo! bodies before implementation so later plans compile against contracts"
  - "TDD RED state via should_panic(expected = 'not yet implemented') — tests pass in RED state (todo panics), turn to real assertions in GREEN"
  - "Timer/watcher stub tests use #[ignore] when test body has no callable function to exercise"

requirements-completed: [INDX-01, INDX-02, INDX-03, INDX-04, INDX-05, INDX-06, INDX-07, INDX-08]

# Metrics
duration: 7min
completed: 2026-03-06
---

# Phase 3 Plan 01: Indexer Scaffold Summary

**Wave 0 Nyquist scaffold: lnk/notify-debouncer-mini/image crates added, all 12 indexer function stubs defined, 11 test stubs in RED state, generic.png compiled in via include_bytes!**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-06T08:57:23Z
- **Completed:** 2026-03-06T09:03:53Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Extended Cargo.toml with 3 new domain crates (lnk, notify-debouncer-mini, image) and 2 new windows-sys features; `cargo check` passes
- Created src-tauri/icons/generic.png (copied from existing 32x32.png) for compile-time bundling via `include_bytes!`
- Wrote complete indexer.rs scaffold: 12 public function stubs with `todo!()` bodies, 11 test stubs in proper RED state
- All 18 library tests pass (11 Phase 2 + 7 Phase 3 RED); 4 ignored (LNK + timer stubs)

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend Cargo.toml with Phase 3 dependencies** - `f46bc86` (chore)
2. **Task 2: Create generic.png and write indexer.rs scaffold with failing test stubs** - `73bbbe6` (feat)

## Files Created/Modified

- `src-tauri/Cargo.toml` - Added lnk ^0.3, notify-debouncer-mini 0.4, image ^0.25 (png-only), Win32_Graphics_Gdi, Win32_UI_WindowsAndMessaging, tempfile 3 dev-dep
- `src-tauri/Cargo.lock` - Updated with resolved new crate versions
- `src-tauri/icons/generic.png` - 974-byte PNG (copy of 32x32.png) for include_bytes! at compile time
- `src-tauri/src/indexer.rs` - Complete scaffold: 12 public stubs + 11 test stubs (7 should_panic RED, 4 ignored)

## Decisions Made

- **include_bytes! path correction:** The plan specified `../../icons/generic.png` but the correct path from `src-tauri/src/indexer.rs` is `../icons/generic.png` (one level up to `src-tauri/`, then `icons/`). Fixed before first compile.
- **should_panic expected string:** `todo!()` macro produces panic messages of the form `"not yet implemented: ..."` not just `"todo"`. Changed expected string to `"not yet implemented"` so RED tests register as passed (should_panic satisfied).
- **Timer test stubs marked #[ignore]:** `test_timer_fires` and `test_timer_reset` have no-op bodies that never call any todo! stub. `#[should_panic]` can never fire on an empty body — marked `#[ignore]` per the LNK test pattern.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed include_bytes! path from ../../ to ../**
- **Found during:** Task 2 (first cargo test run)
- **Issue:** Plan specified `include_bytes!("../../icons/generic.png")` but from `src-tauri/src/indexer.rs`, two levels up is the project root, not `src-tauri/`. The generic.png is at `src-tauri/icons/` so one level up is correct.
- **Fix:** Changed path to `../icons/generic.png`
- **Files modified:** `src-tauri/src/indexer.rs`
- **Verification:** `cargo test -p riftle --lib indexer` compiled without path error
- **Committed in:** `73bbbe6` (Task 2 commit)

**2. [Rule 1 - Bug] Fixed should_panic expected string: "todo" → "not yet implemented"**
- **Found during:** Task 2 (test run showed "panic did not contain expected string")
- **Issue:** `todo!("message")` expands to `panic!("not yet implemented: message")` — the substring "todo" does not appear in the actual panic message.
- **Fix:** Changed all `#[should_panic(expected = "todo")]` to `#[should_panic(expected = "not yet implemented")]`
- **Files modified:** `src-tauri/src/indexer.rs`
- **Verification:** 7 should_panic tests now report "ok" (panic contains expected string)
- **Committed in:** `73bbbe6` (Task 2 commit)

**3. [Rule 1 - Bug] Marked test_timer_fires and test_timer_reset as #[ignore]**
- **Found during:** Task 2 (after fixing should_panic string, these 2 tests still failed)
- **Issue:** Both timer test bodies are no-ops (no calls to any todo! stub). `#[should_panic]` requires the test body to actually panic — an empty body returns normally, failing the should_panic constraint.
- **Fix:** Changed from `#[should_panic(expected = "not yet implemented")]` to `#[ignore]`, matching the LNK stub pattern used in the plan for tests without callable implementations.
- **Files modified:** `src-tauri/src/indexer.rs`
- **Verification:** Tests now show as "ignored" (4 total ignored: 2 LNK + 2 timer)
- **Committed in:** `73bbbe6` (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (all Rule 1 - bug)
**Impact on plan:** All three fixes were necessary for the scaffold to compile and produce the correct RED state. The intent of the plan (Nyquist scaffold with failing tests) is preserved exactly — only the mechanical details of how Rust expresses that needed correction.

## Issues Encountered

None beyond the auto-fixed deviations above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All 12 public function signatures are defined and compiled — Plans 02-05 can implement against these contracts
- generic.png is bundled at compile time via include_bytes! — Plan 03 ensure_generic_icon implementation can proceed
- tempfile dev-dependency available for test infrastructure in Plans 02-05
- Phase 2 tests unaffected — 11 db/paths/store tests still pass

---
*Phase: 03-indexer*
*Completed: 2026-03-06*

## Self-Check: PASSED

- FOUND: src-tauri/src/indexer.rs
- FOUND: src-tauri/icons/generic.png
- FOUND: .planning/phases/03-indexer/03-01-SUMMARY.md
- FOUND commit: f46bc86 (chore: extend Cargo.toml)
- FOUND commit: 73bbbe6 (feat: indexer.rs scaffold + generic.png)
