---
phase: 04-search-engine
plan: 02
subsystem: search
tags: [rust, nucleo-matcher, tdd, fuzzy-search, ranking, mru]

# Dependency graph
requires:
  - phase: 04-search-engine/04-01
    provides: search.rs struct shells + 13 RED test stubs + nucleo-matcher dep + system_command.png asset
  - phase: 03-indexer
    provides: AppRecord struct from db.rs
provides:
  - score_and_rank: nucleo fuzzy scoring with tier+score+launch_count sort, capped at 50
  - match_tier: prefix=2, acronym=1 (query>=2 chars), fuzzy=0
  - is_acronym_match: first-char initials starts_with check (lowercased)
  - search_system_commands: SYSTEM_COMMANDS filter by .contains() substring match
  - ensure_system_command_icon: create icons dir + write bundled PNG if missing
  - 14 search tests GREEN (13 original + 1 bonus ensure_system_command_icon round-trip)
affects: [04-search-engine/04-03, 05-launcher-ui]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "TDD RED-GREEN cycle: real assertions in tests before implementing functions; tests drive implementation shape"
    - "ScoredResult private helper struct local to module — not pub, used only in score_and_rank"
    - "nucleo Pattern::parse + pattern.score(haystack.slice(..), &mut matcher) pipeline for fuzzy scoring"
    - "sort_unstable_by with .then_with() chaining: tier DESC -> score DESC -> launch_count DESC"

key-files:
  created: []
  modified:
    - src-tauri/src/search.rs

key-decisions:
  - "score_and_rank truncates to 50 inside the function (not in search() Tauri command) for unit testability — search() not unit-testable without AppHandle mock"
  - "Plan spec error corrected: search_system_commands('sh') returns 1 result (Shutdown only) — 'Sleep' does not contain 'sh'; plan spec listed both incorrectly"
  - "REFACTOR phase skipped: code was already clean after GREEN; no redundancy or structural issues found"
  - "ensure_system_command_icon test added as bonus (tempdir round-trip) — validates idempotency"

patterns-established:
  - "Tier-based ranking (u8): prefix=2 > acronym=1 > fuzzy=0; acronym only applies when query.len() >= 2"
  - "is_acronym_match: split_whitespace -> first char per word -> collect String -> to_lowercase -> starts_with"

requirements-completed: [SRCH-01, SRCH-02, SRCH-03, SRCH-04, SRCH-05]

# Metrics
duration: 3min
completed: 2026-03-06
---

# Phase 4 Plan 02: Search Pure Functions Summary

**nucleo fuzzy ranking with tier-based (prefix > acronym > fuzzy) + MRU tiebreak, all 14 search tests GREEN after RED-GREEN TDD cycle**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-06T18:14:11Z
- **Completed:** 2026-03-06T18:17:35Z
- **Tasks:** 3 (RED, GREEN, REFACTOR — refactor was no-op)
- **Files modified:** 1

## Accomplishments
- Replaced all 13 `should_panic + todo!()` test stubs with real assertions (RED phase)
- Implemented 5 pure functions: `ensure_system_command_icon`, `score_and_rank`, `match_tier`, `is_acronym_match`, `search_system_commands` (GREEN phase)
- All 14 search tests pass; full suite 34 passed, 0 failed, 2 ignored — no regressions
- Score ranking: nucleo pattern score + tier (prefix=2/acronym=1/fuzzy=0) + launch_count tiebreak, capped at 50

## Task Commits

Each task was committed atomically:

1. **Task 1: RED — replace should_panic stubs with real assertions** - `c0e2d00` (test)
2. **Task 2: GREEN — implement all pure search functions** - `be8fb5a` (feat)
3. **Task 3: REFACTOR — no changes needed** (no commit — code already clean)

## Files Created/Modified
- `src-tauri/src/search.rs` - All 5 pure search functions implemented; 14 tests GREEN; ScoredResult private helper struct; todo!() stubs for init_search_index, rebuild_index, search (Plan 03 scope)

## Decisions Made
- `score_and_rank` truncates to 50 internally (not in the Tauri `search()` command) because `search()` is not unit-testable without AppHandle mock — this satisfies SRCH-03 directly in a testable way
- Plan spec error noted and corrected: `search_system_commands("sh")` should return 1 result (Shutdown only) because "Sleep" does not contain the substring "sh" — the plan spec was wrong listing Sleep as a match
- REFACTOR phase produced no commits — ScoredResult is already private, no redundancy exists, code is idiomatic

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected plan spec error for search_system_commands("sh") test**
- **Found during:** Task 2 (GREEN — running cargo test after implementation)
- **Issue:** Plan specified `search_system_commands("sh")` returns 2 results: Shutdown and Sleep. However, "sleep".contains("sh") is false — Sleep does not contain "sh". The implementation (`.contains()` filter) is correct; the plan's expected count of 2 was wrong.
- **Fix:** Updated test assertion from `assert_eq!(results.len(), 2)` to `assert_eq!(results.len(), 1)` and removed the Sleep assertion, adding a comment explaining the spec error
- **Files modified:** src-tauri/src/search.rs
- **Verification:** All 14 search tests pass, `cargo test` 34 passed 0 failed
- **Committed in:** be8fb5a (Task 2 GREEN commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - plan spec bug in test expectation)
**Impact on plan:** Spec correction was necessary for correctness — implementing wrong behavior would silently return wrong results. No scope creep.

## Issues Encountered
None beyond the spec correction above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plan 03 can begin immediately: all pure search functions are implemented and verified
- Plan 03 wires `init_search_index`, `rebuild_index`, and the `search` Tauri command into lib.rs
- `search()` Tauri command stub remains as `todo!()` — Plan 03 scope
- system_command.png visual design can be improved at any point (file exists, bytes are valid)

---
*Phase: 04-search-engine*
*Completed: 2026-03-06*

## Self-Check: PASSED
- FOUND: src-tauri/src/search.rs
- FOUND: .planning/phases/04-search-engine/04-02-SUMMARY.md
- FOUND: c0e2d00 (test: replace should_panic stubs with real assertions RED)
- FOUND: be8fb5a (feat: implement pure search functions GREEN)
