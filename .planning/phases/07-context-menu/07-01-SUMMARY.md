---
phase: 07-context-menu
plan: 01
subsystem: ui
tags: [tauri, vue, context-menu, rust, typescript]

# Dependency graph
requires:
  - phase: 06-launch-actions
    provides: launch/launch_elevated commands and App.vue architecture
provides:
  - quit_app Tauri command (AppHandle::exit(0), no plugin needed)
  - Right-click context menu overlay with Settings and Quit Launcher items
  - Native browser context menu suppressed globally on launcher window
  - Keyboard (Escape) and click-outside (backdrop) dismissal wiring
affects:
  - 08-settings-window (open_settings_window invoke already wired, silently catches error until Phase 8 implements it)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "@mousedown.prevent on menu items to avoid focus-loss auto-hide race (not @click)"
    - "Backdrop (z-index 99) + menu (z-index 100) layering for click-outside dismissal"
    - "position: fixed; inset: 0 backdrop covers window regardless of scroll"
    - "AppHandle::exit(0) for clean quit — no tauri-plugin-process needed"
    - "menuX clamped to Math.min(e.clientX, 500 - 170) to prevent overflow on 500px window"

key-files:
  created: []
  modified:
    - src-tauri/src/commands.rs
    - src-tauri/src/lib.rs
    - src/App.vue

key-decisions:
  - "AppHandle::exit(0) used for quit_app — no tauri-plugin-process needed, available natively in Tauri v2"
  - "Backdrop uses position: fixed; inset: 0 so it covers entire window viewport regardless of scroll"
  - "@mousedown.prevent on menu items prevents focus-loss from triggering auto-hide before click completes"
  - "Menu state reset in both hideWindow() and launcher-show listener so menu never reappears with launcher on next show"

patterns-established:
  - "Custom context menu: backdrop (fixed, z-99) + overlay (absolute, z-100) pattern"
  - "Escape key guard: check menuVisible first, close menu, return — then hideWindow on second press"

requirements-completed:
  - MENU-01
  - MENU-02
  - MENU-03

# Metrics
duration: 2min
completed: 2026-03-07
---

# Phase 07 Plan 01: Context Menu Summary

**Right-click context menu with Settings and Quit Launcher items using backdrop dismissal, Escape key awareness, and AppHandle::exit(0) Rust quit command**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-07T23:35:39Z
- **Completed:** 2026-03-07T23:37:43Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `quit_app` Tauri command to `commands.rs` using `app.exit(0)` and registered it in `lib.rs` invoke_handler
- Implemented custom HTML context menu overlay in `App.vue` with backdrop/menu z-index layering, cursor-coordinate positioning, and clamped X to prevent overflow
- Wired all dismissal paths: Escape key (menu-first guard), click-outside backdrop, hideWindow reset, launcher-show reset

## Task Commits

Each task was committed atomically:

1. **Task 1: Add quit_app Rust command and register it** - `207aa69` (feat)
2. **Task 2: Add context menu overlay to App.vue** - `2a28969` (feat)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified

- `src-tauri/src/commands.rs` - Added `pub fn quit_app(app: AppHandle)` using `app.exit(0)`
- `src-tauri/src/lib.rs` - Registered `crate::commands::quit_app` in `generate_handler!`
- `src/App.vue` - menuVisible/menuX/menuY refs, closeMenu/onContextMenu/openSettings/quitApp functions, updated Escape handler, backdrop+overlay template, context menu CSS

## Decisions Made

- AppHandle::exit(0) used for quit_app — no `tauri-plugin-process` needed; this method is available natively in Tauri v2 as specified by the plan
- Backdrop uses `position: fixed; inset: 0` so it covers the entire window viewport regardless of scroll position
- `@mousedown.prevent` used on menu items (not `@click`) to prevent focus loss from triggering auto-hide before the click handler completes — key research finding from RESEARCH.md Pitfall 1
- Menu state is reset in both `hideWindow()` and the `launcher-show` listener so the menu never reappears when the launcher is summoned after being hidden with an open menu

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Context menu complete with Settings and Quit Launcher items
- `openSettings` already wired to `invoke('open_settings_window').catch(console.error)` — silently no-ops until Phase 8 implements the settings window
- Phase 8 (Settings Window) can implement `open_settings_window` Tauri command and it will immediately connect to the existing frontend invocation

---
*Phase: 07-context-menu*
*Completed: 2026-03-07*
