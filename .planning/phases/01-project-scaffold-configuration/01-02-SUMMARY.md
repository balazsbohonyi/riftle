---
phase: 01-project-scaffold-configuration
plan: "02"
subsystem: infra
tags: [tauri, vue, tauri-conf, capabilities, transparent-window, plugin-store, global-shortcut, autostart]

# Dependency graph
requires:
  - phase: 01-01
    provides: Cargo.toml with 12 crates, lib.rs with plugin scaffold, seven stub module files

provides:
  - tauri.conf.json with launcher window (frameless/transparent/shadow/alwaysOnTop/skipTaskbar/visible:false) and settings window (normal/minWidth:600/minHeight:400/visible:false)
  - identifier changed to com.riftle.launcher
  - capabilities/default.json covering both window labels with all five plugin permissions
  - App.vue minimal transparent shell (no greet, body transparent, height chain correct)
  - JS plugin packages installed (plugin-store, plugin-global-shortcut, plugin-autostart)
  - Smoke test approved by user — pnpm tauri dev starts cleanly with correct window flags

affects:
  - 05-launcher-window-ui
  - 08-settings-window
  - 09-global-hotkey

# Tech tracking
tech-stack:
  added:
    - "@tauri-apps/plugin-store"
    - "@tauri-apps/plugin-global-shortcut"
    - "@tauri-apps/plugin-autostart"
  patterns:
    - "Two-window Tauri app: launcher (frameless/transparent/hidden) and settings (normal/hidden) both declared in tauri.conf.json"
    - "capabilities/default.json must list all window labels explicitly — if window label not listed IPC calls fail silently"
    - "Vue body { background: transparent } required for transparent Tauri window — without it launcher renders white"
    - "camelCase window flags in Tauri v2 JSON: skipTaskbar, alwaysOnTop, not snake_case"
    - "body background color must match app background color to prevent white webview corner bleed-through on transparent windows"

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
  - "body background color matched to app background color to eliminate white webview corner bleed-through"

patterns-established:
  - "Tauri v2 window flags are camelCase in JSON: skipTaskbar, alwaysOnTop, decorations, transparent, shadow"
  - "capabilities/default.json windows array must match tauri.conf.json labels exactly; mismatches cause silent IPC failures"
  - "Transparent Tauri windows require body { background: transparent } in CSS — the Rust/config alone is insufficient"
  - "Height chain for transparent windows: html, body, and #app must all have explicit height:100% for the div to fill the viewport"

requirements-completed: [SCAF-02, SCAF-04]

# Metrics
duration: ~25min
completed: 2026-03-06
---

# Phase 01 Plan 02: Tauri Two-Window Configuration Summary

**Two-window Tauri v2 app configured with frameless transparent launcher (640x60, hidden at start) and normal settings window, capabilities covering both labels, three JS plugin packages installed, and smoke test approved by user**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-03-06T00:49:00Z
- **Completed:** 2026-03-06T01:15:00Z
- **Tasks:** 3 of 3 complete (including human-verify checkpoint approved)
- **Files modified:** 5

## Accomplishments

- Rewrote tauri.conf.json to declare two named windows with correct camelCase flags: launcher (frameless, transparent, shadow, skipTaskbar, alwaysOnTop, visible:false, 640x60) and settings (decorations:true, minWidth:600, minHeight:400, visible:false)
- Updated capabilities/default.json with windows ["launcher", "settings"] and all five permission entries (core:default, opener:default, store:default, global-shortcut:default, autostart:default)
- Replaced greet-scaffold App.vue with minimal transparent shell — no imports, no API calls, body background:transparent to prevent white rectangle on transparent window, with correct height chain
- Installed three JS plugin packages providing TypeScript bindings for Rust plugins already registered in lib.rs
- Smoke test approved by user: pnpm tauri dev compiles cleanly, launcher window is frameless, transparent, shadow-bearing, always-on-top, no taskbar entry

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewrite tauri.conf.json, update capabilities, install JS plugin packages** - `f58e0a7` (feat)
2. **Task 2: Replace App.vue with minimal transparent launcher shell** - `ba92b07` (feat)
3. **Task 3: Smoke-test pnpm tauri dev and verify window flags** - human-verify checkpoint, approved by user

**Deviation fix commits (during smoke test checkpoint resolution):**
- `ccd7c95` — fix: solid content area with rounded corners on transparent launcher window
- `1ce3ac0` — fix: height chain so #app fills viewport for solid background
- `00e52ae` — fix: match body background to app color to hide white webview corners

## Files Created/Modified

- `src-tauri/tauri.conf.json` - Two-window configuration with launcher (frameless/transparent/always-on-top/no-taskbar/hidden) and settings (normal/hidden); identifier com.riftle.launcher
- `src-tauri/capabilities/default.json` - Windows ["launcher","settings"]; five plugin permissions
- `src/App.vue` - Minimal transparent shell; no greet references; body { background: transparent }; correct height chain for full viewport coverage
- `package.json` - Three new plugin dependencies added
- `pnpm-lock.yaml` - Lockfile updated with plugin-store, plugin-global-shortcut, plugin-autostart

## Decisions Made

- Bundle identifier changed from `com.balazs.bohonyi.riftle` to `com.riftle.launcher` — matches project branding decision documented in plan
- Both windows start hidden (`visible: false`) — launcher appears via hotkey (Phase 9 scope), settings via menu action
- Launcher retains `shadow: true` for floating appearance even though frameless and transparent
- App.vue is intentionally a minimal placeholder — all launcher UI is Phase 5 scope
- body background color must match the app element background to prevent white webview corner bleed-through on transparent windows

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] White webview corners on transparent launcher window during smoke test**
- **Found during:** Task 3 (smoke test / human-verify checkpoint)
- **Issue:** The WebView rendered visible white corners where the rounded-corner `#app` div did not cover the full viewport edges, and the body background did not match the app background color causing bleed-through
- **Fix:** Added `border-radius` to `#app`; gave `html`, `body`, and `#app` explicit `height: 100%` so the element fills the viewport; matched `body { background }` to the same color as `#app`
- **Files modified:** `src/App.vue`
- **Verification:** User confirmed correct appearance during smoke test — window approved
- **Committed in:** `ccd7c95`, `1ce3ac0`, `00e52ae`

---

**Total deviations:** 1 auto-fixed (Rule 1 — visual bug)
**Impact on plan:** Fix necessary for correct transparent window appearance. No scope creep — all changes confined to App.vue CSS.

## Issues Encountered

During the smoke test, the transparent window showed white webview corners because the body background and the app div height chain were not fully specified. Three iterative fix commits resolved the issue. Root cause: on Windows, the WebView compositing area extends to the full window rect; CSS must cover it completely with matching colors.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 1 is fully complete — SCAF-02 (two-window config) and SCAF-04 (clean dev start) both satisfied
- Both windows configured with correct labels and flags; Phase 9 can target "launcher" label directly
- All plugin JS packages installed — Phases 3-9 can import them without further package.json changes
- App.vue is a clean placeholder — Phase 5 can replace the template content directly
- Phase 2 (Data Layer) is ready to begin

---
*Phase: 01-project-scaffold-configuration*
*Completed: 2026-03-06*
