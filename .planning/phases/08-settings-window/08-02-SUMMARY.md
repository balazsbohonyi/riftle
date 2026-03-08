---
phase: 08-settings-window
plan: 02
subsystem: ui
tags: [vue, css-tokens, design-system, components, settings]

# Dependency graph
requires:
  - phase: 08-01
    provides: tokens.css CSS custom properties, settings window scaffold

provides:
  - App.vue style block fully tokenized (no hardcoded hex colors, font families, or sizes)
  - Section.vue: heading + divider + slot wrapper for settings sections
  - Row.vue: label-left / control-right layout primitive with optional hint text
  - Toggle.vue: accessible role=switch toggle with v-model (modelValue + update:modelValue)
  - KeyCapture.vue: hotkey capture input emitting Modifier+Key strings for tauri-plugin-global-shortcut
  - PathList.vue: add/remove folder path list using @tauri-apps/plugin-dialog

affects:
  - 08-03 (Settings.vue composes all five primitives)
  - 09-global-hotkey (KeyCapture.vue wires to update_hotkey Tauri command)

# Tech tracking
tech-stack:
  added:
    - "@tauri-apps/plugin-dialog (JS package, was present in Rust Cargo.toml but missing from package.json)"
  patterns:
    - "CSS token substitution: all design values reference var(--*) from tokens.css"
    - "UI primitives pattern: small scoped-style Vue SFCs composing layout and behavior"
    - "isTauriContext guard: PathList.vue checks __TAURI_INTERNALS__ before dynamic import"
    - "Dynamic import for dialog: await import('@tauri-apps/plugin-dialog') inside addPath()"

key-files:
  created:
    - src/components/ui/Section.vue
    - src/components/ui/Row.vue
    - src/components/ui/Toggle.vue
    - src/components/ui/KeyCapture.vue
    - src/components/ui/PathList.vue
  modified:
    - src/App.vue (style block tokenized)
    - package.json (added @tauri-apps/plugin-dialog)

key-decisions:
  - "@tauri-apps/plugin-dialog JS package installed alongside existing Rust crate — was missing from package.json causing TS2307 error"
  - "PathList.vue uses dynamic import for plugin-dialog inside addPath() to avoid top-level import errors in browser dev mode"
  - "Two pure #ffffff overrides retained in App.vue (selected .app-name and .menu-item:hover) — intentional design, no --color-white token exists"

patterns-established:
  - "UI primitives: scoped styles only; all values via CSS tokens from tokens.css"
  - "Tauri plugin guard: check __TAURI_INTERNALS__ before any plugin API usage in component"

requirements-completed: [SETT-05, SETT-06]

# Metrics
duration: 3min
completed: 2026-03-08
---

# Phase 8 Plan 02: CSS Token Refactor + UI Primitives Summary

**App.vue style block fully tokenized and five composable settings UI primitives built (Section, Row, Toggle, KeyCapture, PathList) ready for Settings.vue assembly in Plan 03**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-08T17:37:26Z
- **Completed:** 2026-03-08T17:40:31Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Replaced all hardcoded design values in App.vue's unscoped style block with CSS custom property references from tokens.css — visual output is pixel-identical
- Created Section.vue and Row.vue layout primitives for structuring settings content
- Created Toggle.vue with role=switch ARIA semantics and full v-model support
- Created KeyCapture.vue that records keyboard combos in "Modifier+Key" format compatible with tauri-plugin-global-shortcut
- Created PathList.vue with folder picker integration via @tauri-apps/plugin-dialog

## Task Commits

Each task was committed atomically:

1. **Task 1: Refactor App.vue style block to CSS tokens** - `d46c0f5` (refactor)
2. **Task 2: Build five UI primitive components** - `c7c1f0f` (feat)

**Plan metadata:** (see final commit below)

## Files Created/Modified
- `src/App.vue` - Style block tokenized; style tag remains unscoped
- `src/components/ui/Section.vue` - Section wrapper with heading, divider, and slot
- `src/components/ui/Row.vue` - Label + control layout with optional hint text
- `src/components/ui/Toggle.vue` - Accessible on/off toggle with v-model
- `src/components/ui/KeyCapture.vue` - Hotkey capture producing Modifier+Key strings
- `src/components/ui/PathList.vue` - Add/remove folder list with plugin-dialog integration
- `package.json` - Added @tauri-apps/plugin-dialog JS package

## Decisions Made
- @tauri-apps/plugin-dialog JS package installed (was missing from package.json, Rust crate was already present)
- PathList.vue uses dynamic `await import('@tauri-apps/plugin-dialog')` to avoid top-level import failures in browser dev mode
- Two `#ffffff` overrides retained in App.vue (`.result-row.selected .app-name` and `.menu-item:hover`) — intentional pure-white design overrides, no corresponding token

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Installed missing @tauri-apps/plugin-dialog JS package**
- **Found during:** Task 2 (building PathList.vue)
- **Issue:** `@tauri-apps/plugin-dialog` was added to Rust Cargo.toml in Plan 01 but its JS counterpart was missing from package.json, causing TS2307 type error and build failure
- **Fix:** Ran `pnpm add @tauri-apps/plugin-dialog`
- **Files modified:** package.json, pnpm-lock.yaml
- **Verification:** pnpm build passes after install
- **Committed in:** c7c1f0f (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (blocking dependency)
**Impact on plan:** Necessary fix; JS package must match Rust plugin. No scope creep.

## Issues Encountered
None beyond the auto-fixed missing dependency.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All five UI primitives importable and ready for composition in Settings.vue (Plan 03)
- App.vue launcher appearance unchanged — token values match the hardcoded values they replaced
- KeyCapture.vue produces output format accepted by tauri-plugin-global-shortcut (e.g., "Alt+Space")
- PathList.vue wired to @tauri-apps/plugin-dialog with isTauriContext guard for browser dev mode safety

---
*Phase: 08-settings-window*
*Completed: 2026-03-08*
