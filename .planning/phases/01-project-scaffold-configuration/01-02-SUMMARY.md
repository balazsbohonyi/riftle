---
phase: 01-project-scaffold-configuration
plan: 02
subsystem: infra
tags: [tauri, vue, tauri-conf, capabilities, plugin-store, plugin-global-shortcut, plugin-autostart]

# Dependency graph
requires:
  - phase: 01-01
    provides: Cargo.toml with 12 crates, lib.rs with plugin scaffold, seven stub module files

provides:
  - tauri.conf.json with launcher window (frameless/transparent/shadow/alwaysOnTop/skipTaskbar/visible:false) and settings window (normal/minWidth:600/minHeight:400/visible:false)
  - identifier changed to com.riftle.launcher
  - capabilities/default.json covering both window labels with all five plugin permissions
  - App.vue minimal transparent shell (no greet, body transparent)
  - JS plugin packages installed (plugin-store 2.4.2, plugin-global-shortcut 2.3.1, plugin-autostart 2.5.1)

affects: [05-launcher-window-ui, 08-settings-window, 09-global-hotkey]

# Tech tracking
tech-stack:
  added:
    - "@tauri-apps/plugin-store 2.4.2"
    - "@tauri-apps/plugin-global-shortcut 2.3.1"
    - "@tauri-apps/plugin-autostart 2.5.1"
  patterns:
    - "Two-window Tauri app: launcher (frameless/transparent/hidden) and settings (normal/hidden) both declared in tauri.conf.json"
    - "capabilities/default.json must list all window labels explicitly — if window label not listed IPC calls fail silently"
    - "Vue body { background: transparent } required for transparent Tauri window — without it launcher renders white"
    - "camelCase window flags in Tauri v2 JSON: skipTaskbar, alwaysOnTop, not snake_case"

key-files:
  created: []
  modified:
    - src-tauri/tauri.conf.json
    - src-tauri/capabilities/default.json
    - src/App.vue
    - package.json
    - pnpm-lock.yaml

key-decisions:
  - "Bundle identifier changed from com.balazs.bohonyi.riftle to com.riftle.launcher per user decision in plan"
  - "Both windows start with visible:false — launcher shown via hotkey (Phase 9), settings via menu action"
  - "Launcher shadow:true retained — provides floating appearance even though window is frameless and transparent"
  - "App.vue intentionally minimal with no Tauri API imports — launcher UI deferred to Phase 5"

patterns-established:
  - "Tauri v2 window flags are camelCase in JSON: skipTaskbar, alwaysOnTop, decorations, transparent, shadow"
  - "capabilities/default.json windows array must match tauri.conf.json labels exactly; mismatches cause silent IPC failures"
  - "Transparent Tauri windows require body { background: transparent } in CSS — the Rust/config alone is insufficient"

requirements-completed: [SCAF-02]

# Metrics
duration: 5min
completed: 2026-03-06
---

# Phase 01 Plan 02: Tauri Two-Window Configuration Summary

**Two-window Tauri configuration with frameless/transparent launcher and normal settings window, capabilities covering both labels, and three JS plugin packages installed — smoke test pending human verification.**

## Performance

- **Duration:** ~5 min (automated tasks; human smoke test pending)
- **Started:** 2026-03-05T22:46:31Z
- **Completed:** 2026-03-05T22:51:01Z (Tasks 1-2; Task 3 awaits human verification)
- **Tasks:** 2 of 3 auto-tasks complete (checkpoint:human-verify pending)
- **Files modified:** 5

## Accomplishments

- Rewrote tauri.conf.json to declare two named windows with correct camelCase flags: launcher (frameless, transparent, shadow, skipTaskbar, alwaysOnTop, visible:false, 640x60) and settings (decorations:true, minWidth:600, minHeight:400, visible:false)
- Updated capabilities/default.json with windows ["launcher", "settings"] and all five permission entries (core:default, opener:default, store:default, global-shortcut:default, autostart:default)
- Replaced greet-scaffold App.vue with minimal transparent shell — no imports, no API calls, body background:transparent to prevent white rectangle on transparent window
- Installed three JS plugin packages providing TypeScript bindings for Rust plugins already registered in lib.rs

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewrite tauri.conf.json, update capabilities, install JS plugin packages** - `f58e0a7` (feat)
2. **Task 2: Replace App.vue with minimal transparent launcher shell** - `ba92b07` (feat)
3. **Task 3: Smoke-test pnpm tauri dev and verify window flags** - PENDING human verification

## Files Created/Modified

- `src-tauri/tauri.conf.json` - Two-window configuration with launcher (frameless/transparent/always-on-top/no-taskbar/hidden) and settings (normal/hidden); identifier com.riftle.launcher
- `src-tauri/capabilities/default.json` - Windows ["launcher","settings"]; five plugin permissions
- `src/App.vue` - Minimal transparent shell; no greet references; body { background: transparent }
- `package.json` - Three new plugin dependencies added
- `pnpm-lock.yaml` - Lockfile updated with plugin-store, plugin-global-shortcut, plugin-autostart

## Decisions Made

- Bundle identifier changed from `com.balazs.bohonyi.riftle` to `com.riftle.launcher` — matches project branding decision documented in plan
- Both windows start hidden (`visible: false`) — launcher appears via hotkey (Phase 9 scope), settings via menu action
- Launcher retains `shadow: true` for floating appearance even though frameless and transparent
- App.vue is intentionally a minimal placeholder — all launcher UI is Phase 5 scope

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. pnpm install resolved all three plugin packages cleanly in 2 seconds. cargo check passes in 0.67s (all dependencies already cached from Plan 01).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Task 3 (smoke test) requires `pnpm tauri dev` to be run manually on Windows with a display — this is a human-verify checkpoint
- After smoke test passes: Phase 1 is fully complete (SCAF-02 and SCAF-04 both satisfied)
- Phase 2 (Data Layer) ready to begin once Phase 1 smoke test confirmed
- JS plugin TypeScript bindings available for import in later phases without package.json changes

---
*Phase: 01-project-scaffold-configuration*
*Completed: 2026-03-06 (Task 3 pending)*
