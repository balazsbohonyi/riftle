---
phase: 03-indexer
plan: 02
subsystem: indexer
tags: [rust, lnk, walkdir, fnv, tdd, windows, path-discovery]

# Dependency graph
requires:
  - phase: 03-indexer
    plan: 01
    provides: All indexer function stubs + RED test state
  - phase: 02-data-layer
    provides: AppRecord, upsert_app, get_all_apps, init_db_connection, DbState, Settings
provides:
  - get_index_paths: collects APPDATA/PROGRAMDATA Start Menu, USERPROFILE/Public Desktop, PATH, additional_paths
  - crawl_dir: walks root directory, resolves .lnk (non-PATH), finds .exe, skips excluded paths
  - resolve_lnk: extracts target from working_dir+relative_path; None for broken/chained/non-exe
  - make_app_record: builds AppRecord with normalized id, stem name, generic.png placeholder
  - icon_filename: FNV-1a 64-bit hash producing stable 16-char hex + .png string
  - prune_stale: removes DB rows whose id is absent from discovered_ids set
affects: [03-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - FNV-1a 64-bit inline hash (no external crate) for stable icon filename generation
    - lnk crate API via working_dir()+relative_path() (public fields only — no link_info() method)
    - walkdir with max_depth(1) for PATH source, follow_links(true) for other sources
    - prune_stale queries id column directly to avoid loading full AppRecord structs

key-files:
  created: []
  modified:
    - src-tauri/src/indexer.rs (replaced 5 todo! stubs with working implementations; 5 tests turned GREEN)

key-decisions:
  - "lnk crate (0.3.0) has no public link_info() method — used working_dir()+relative_path() to reconstruct target path instead of link_info().local_base_path()"
  - "prune_stale uses inline query_map with .get::<_, String>(0) type annotation to resolve lifetime — avoids stmt borrow issue"
  - "crawl_dir implements PATH source with max_depth(1) and no .lnk resolution per spec"

# Metrics
duration: 5min
completed: 2026-03-06
---

# Phase 3 Plan 02: Path Discovery and Crawling Summary

**Five indexer primitives implemented: get_index_paths, crawl_dir, resolve_lnk, make_app_record, icon_filename, prune_stale — all 7 stub tests turned from RED to GREEN**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-06T09:06:59Z
- **Completed:** 2026-03-06T09:11:25Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Implemented all six internal indexer functions replacing `todo!()` stubs
- Turned all 5 targeted tests GREEN (test_resolve_lnk_broken, test_icon_filename_stable, test_crawl_discovers_exe, test_crawl_excludes_path, test_prune_stale)
- Full lib suite: 18 passed, 0 failed, 4 ignored — Phase 2 tests unaffected
- Remaining stubs (test_generic_icon_bootstrap, test_atomic_guard_prevents_double_index) correctly still failing with "not yet implemented" for Plans 03 and 05

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement path discovery, LNK resolution, and make_app_record** - `07f79df` (feat)
2. **Task 2: Implement crawl_dir and prune_stale** - `2c25b86` (feat)

## Files Created/Modified

- `src-tauri/src/indexer.rs` - Replaced 6 todo! stubs with working implementations; removed should_panic from 5 tests (test_resolve_lnk_broken, test_icon_filename_stable, test_crawl_discovers_exe, test_crawl_excludes_path, test_prune_stale)

## Decisions Made

- **lnk crate API deviation:** The plan specified `shortcut.link_info().as_ref()?.local_base_path().as_ref()?` but `ShellLink` in lnk 0.3.0 has no public `link_info()` method — the field is private with no accessor. Used `working_dir()` + `relative_path()` public methods instead, which is the standard way `new_simple()` writes shortcut targets.
- **prune_stale lifetime fix:** The plan's code `stmt.query_map([], |row| row.get(0))?.filter_map().collect()` caused a "stmt does not live long enough" error. Fixed with inline query_map using `.get::<_, String>(0)` type annotation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] lnk crate has no public link_info() method**
- **Found during:** Task 1 (compile error: `no method named link_info found for struct ShellLink`)
- **Issue:** Plan specified `shortcut.link_info().as_ref()?.local_base_path().as_ref()?` but `ShellLink` in lnk 0.3.0 exposes no `link_info()` public method. The field is private.
- **Fix:** Reconstructed target path from public methods `working_dir()` + `relative_path()`. Windows shortcuts created by `ShellLink::new_simple()` store parent dir in `working_dir` and `"./filename.exe"` in `relative_path`.
- **Files modified:** `src-tauri/src/indexer.rs`
- **Commit:** `07f79df`

**2. [Rule 1 - Bug] prune_stale stmt lifetime error**
- **Found during:** Task 1 compile (same compile run)
- **Issue:** `stmt.query_map([], |row| row.get(0))?` inside a block caused "stmt does not live long enough" — the mapped iterator borrows `stmt` but `stmt` drops at block end before the collect.
- **Fix:** Changed to `conn.prepare(...)?.query_map(...)?.filter_map().collect()` — the prepared statement is consumed by `query_map` via the `?` chain, avoiding the borrow conflict. Added explicit type annotation `.get::<_, String>(0)`.
- **Files modified:** `src-tauri/src/indexer.rs`
- **Commit:** `07f79df`

## Issues Encountered

None beyond the two auto-fixed deviations above.

## User Setup Required

None.

## Next Phase Readiness

- All five core indexer primitives are implemented and tested
- Plan 03 can now implement `ensure_generic_icon` and `extract_icon_png`
- Plan 04 can now implement `run_full_index` which assembles these primitives
- Plan 05 can now implement `start_background_tasks`, `reindex`, and `try_start_index`

---
*Phase: 03-indexer*
*Completed: 2026-03-06*

## Self-Check: PASSED

- FOUND: src-tauri/src/indexer.rs
- FOUND: .planning/phases/03-indexer/03-02-SUMMARY.md
- FOUND commit: 07f79df (feat: Task 1 implementation)
- FOUND commit: 2c25b86 (feat: Task 2 implementation)
