---
phase: 05-launcher-window-ui
plan: 04
subsystem: ui
tags: [vue, rust, tauri, search, admin-badge, elevation]

# Dependency graph
requires:
  - phase: 05-03
    provides: UAT gap analysis identifying admin badge visibility as major defect (Test 8)
  - phase: 04-search-engine
    provides: SearchResult struct in search.rs serialized via Tauri invoke('search')
provides:
  - requires_elevation: bool field on SearchResult Rust struct
  - requires_elevation: boolean field on TypeScript SearchResult interface
  - Badge v-if decoupled from keyboard state — uses item.requires_elevation
affects:
  - 05-05 (future gap closure plans)
  - 06-launch-actions (elevation detection can populate requires_elevation: true)
  - 08-settings-window (no impact)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Per-result elevation flag pattern: badge visibility driven by stable data field not transient keyboard ref
    - Rust bool serialized to TypeScript boolean via Tauri Serde JSON bridge

key-files:
  created: []
  modified:
    - src-tauri/src/search.rs
    - src/App.vue

key-decisions:
  - "All SearchResult construction sites default requires_elevation to false — Phase 6/8 can wire real elevation detection later without breaking changes"
  - "adminMode ref and keyboard handlers preserved — Ctrl+Shift+Enter elevated launch still works; only badge visibility decoupled"

patterns-established:
  - "Per-result stable flag pattern: badge visibility driven by item.requires_elevation (stable data), not adminMode (transient keyboard state)"

requirements-completed:
  - LWND-09

# Metrics
duration: 4min
completed: 2026-03-07
---

# Phase 5 Plan 04: Admin Badge Visibility Fix Summary

**Admin badge decoupled from Ctrl+Shift keyboard state — now driven by per-result requires_elevation field on SearchResult struct in Rust and TypeScript**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-07T10:21:35Z
- **Completed:** 2026-03-07T10:26:22Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added `requires_elevation: bool` to SearchResult Rust struct in `search.rs`, populated at all construction sites
- Mirrored `requires_elevation: boolean` in TypeScript SearchResult interface in `App.vue`
- Fixed badge v-if from `index === selectedIndex && adminMode` to `item.requires_elevation`, closing UAT gap Test 8
- All 34 Rust tests pass; pnpm build succeeds with no TypeScript errors

## Task Commits

Each task was committed atomically:

1. **Task 1: Add requires_elevation field to SearchResult Rust struct** - `c5b99ca` (feat)
2. **Task 2: Mirror requires_elevation in TypeScript interface and fix badge v-if** - `5791d11` (fix)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src-tauri/src/search.rs` - Added `requires_elevation: bool` to SearchResult struct; populated `requires_elevation: false` in `score_and_rank` (app results) and `search_system_commands` (system results)
- `src/App.vue` - Added `requires_elevation: boolean` to TypeScript interface; changed badge `v-if="index === selectedIndex && adminMode"` to `v-if="item.requires_elevation"`

## Decisions Made

- All SearchResult construction sites default `requires_elevation` to `false` — this is the correct baseline since Phase 6 or 8 will wire real elevation detection (e.g., manifest requiresAdministrator) when needed
- The `adminMode` ref and `onKeyUp` handler were preserved — Ctrl+Shift+Enter elevated launch still uses adminMode in the Enter key handler; only the badge visibility was decoupled from keyboard state

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `pnpm build` failed initially due to missing `node_modules` in the worktree (not installed). Ran `pnpm install` first (Rule 3 — blocking), then build succeeded. This is a worktree setup condition, not a code issue.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Admin badge gap (UAT Test 8) is closed: badge appears on all rows where `requires_elevation` is true, independent of keyboard or selection state
- Currently all results return `requires_elevation: false` so no badge will be visible until Phase 6/8 wires elevation detection
- Phase 5 gap closure series can continue (05-05 and beyond if planned)

---
*Phase: 05-launcher-window-ui*
*Completed: 2026-03-07*
