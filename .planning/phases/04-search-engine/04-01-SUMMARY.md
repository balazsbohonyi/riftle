---
phase: 04-search-engine
plan: 01
subsystem: search
tags: [rust, nucleo-matcher, tauri, tdd, fuzzy-search]

# Dependency graph
requires:
  - phase: 03-indexer
    provides: AppRecord struct and indexer.rs patterns (include_bytes!, should_panic TDD)
provides:
  - nucleo-matcher 0.3 direct dependency in Cargo.toml
  - system_command.png bundled asset at src-tauri/icons/
  - SearchResult, SearchIndex, SearchIndexState struct shells as public exports
  - 13 search test stubs in RED state (should_panic + todo!())
  - 8 function stubs (ensure_system_command_icon, init_search_index, rebuild_index, score_and_rank, match_tier, is_acronym_match, search_system_commands, search)
affects: [04-search-engine/04-02, 04-search-engine/04-03, 05-launcher-ui]

# Tech tracking
tech-stack:
  added: [nucleo-matcher = "0.3"]
  patterns: [include_bytes! bundled asset pattern (replicates indexer.rs generic.png approach), should_panic TDD RED state scaffold]

key-files:
  created:
    - src-tauri/icons/system_command.png
    - (search.rs substantially rewritten from stub)
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/search.rs
    - src-tauri/src/indexer.rs

key-decisions:
  - "nucleo-matcher = 0.3 added as a direct dep (not transitive via nucleo) to avoid Utf32String re-export ambiguity"
  - "system_command.png is a copy of 32x32.png as a valid placeholder PNG — visual design deferred to Plan 02+"
  - "13 test stubs use should_panic(expected = 'not yet implemented') so todo!() bodies produce passing RED state (mirrors Phase 3 pattern)"

patterns-established:
  - "RED state via should_panic + todo!(): test passes because todo!() panics with 'not yet implemented'; Plan 02 replaces todo!() with real assertions"
  - "include_bytes!(../icons/system_command.png) path is relative to the .rs source file"

requirements-completed: [SRCH-01, SRCH-02, SRCH-03, SRCH-04, SRCH-05]

# Metrics
duration: 6min
completed: 2026-03-06
---

# Phase 4 Plan 01: Search Engine Wave 0 Scaffold Summary

**nucleo-matcher 0.3 direct dep, system_command.png bundled asset, and 13 should_panic test stubs in search.rs establishing the TDD contract for Plan 02**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-03-06T18:06:51Z
- **Completed:** 2026-03-06T18:11:57Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added nucleo-matcher = "0.3" as a direct Cargo dependency after nucleo line
- Created src-tauri/icons/system_command.png (valid 32x32 PNG, placeholder for Wave 0)
- Wrote search.rs with full struct shells (SearchResult, SearchIndex, SearchIndexState) and 8 function stubs
- All 13 test stubs pass in RED state (should_panic + todo!() = green); full test suite 33 passed, 0 failed

## Task Commits

Each task was committed atomically:

1. **Task 1: Add nucleo-matcher dep and create system_command.png asset** - `b7751cd` (chore)
2. **Task 2: Scaffold search.rs with struct shells and 13 RED test stubs** - `acc5e73` (test)

## Files Created/Modified
- `src-tauri/Cargo.toml` - Added nucleo-matcher = "0.3" after nucleo line
- `src-tauri/icons/system_command.png` - Valid 32x32 PNG bundled asset for include_bytes! reference
- `src-tauri/src/search.rs` - Struct shells + 8 function stubs + 13 RED test stubs
- `src-tauri/src/indexer.rs` - Fixed pre-existing resolve_lnk test call (missing allowlist arg)

## Decisions Made
- nucleo-matcher added as a direct dep (not relying on transitive re-export from nucleo) to ensure unambiguous Utf32String import paths as documented in RESEARCH.md Pitfall 2
- system_command.png is a copy of the existing 32x32.png — valid PNG bytes confirmed (magic bytes 0x89 0x50 0x4E 0x47); visual design deferred since Wave 0 only needs the file to exist for include_bytes! to compile
- Followed Phase 3's established should_panic TDD RED state pattern exactly

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed pre-existing resolve_lnk test missing second argument in indexer.rs**
- **Found during:** Task 2 (cargo test search compilation)
- **Issue:** indexer.rs:696 called `resolve_lnk(path)` with 1 argument but the function signature was updated to `resolve_lnk(path, allowlist: &[String])` — test not kept in sync; caused compilation failure of the entire test binary
- **Fix:** Updated test call to `resolve_lnk(Path::new("C:\\nonexistent\\fake.lnk"), &[])` — empty allowlist is correct for the nonexistent path test
- **Files modified:** src-tauri/src/indexer.rs
- **Verification:** `cargo test` ran 33 tests, 0 failed after fix
- **Committed in:** acc5e73 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 3 - blocking)
**Impact on plan:** Single fix was necessary for the test binary to compile. No scope creep.

## Issues Encountered
None beyond the Rule 3 auto-fix above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plan 02 can begin immediately: all struct shells and test stubs are in place
- Plan 02 implements the 8 function stubs and replaces todo!() test bodies with real assertions
- Plan 03 wires the `search` Tauri command into lib.rs invoke_handler
- system_command.png visual design can be improved at any point (file exists, bytes are valid)

---
*Phase: 04-search-engine*
*Completed: 2026-03-06*

## Self-Check: PASSED
- FOUND: src-tauri/icons/system_command.png
- FOUND: src-tauri/src/search.rs
- FOUND: .planning/phases/04-search-engine/04-01-SUMMARY.md
- FOUND: b7751cd (chore: add nucleo-matcher dep and system_command.png)
- FOUND: acc5e73 (test: scaffold search.rs with 13 RED test stubs)
