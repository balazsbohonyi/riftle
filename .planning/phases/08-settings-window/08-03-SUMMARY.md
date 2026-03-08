---
phase: 08-settings-window
plan: 03
subsystem: ui
tags: [vue, settings, ui, tauri, autostart, hotkey, search, appearance]

# Dependency graph
requires:
  - phase: 08-01
    provides: tokens.css, settings.html, Tauri commands (get_settings_cmd, set_settings_cmd, update_hotkey, open_settings_window)
  - phase: 08-02
    provides: Section, Row, Toggle, KeyCapture, PathList UI primitives

provides:
  - src/settings-main.ts: complete entry point with font imports and Settings.vue mount
  - src/Settings.vue: full settings window root component with all four sections

affects:
  - 08-04 (human verification of settings window)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Dynamic import of @tauri-apps/plugin-autostart inside handler functions to avoid top-level plugin issues"
    - "emitTo('launcher', ...) for cross-window event targeting without self-reception"
    - "data-theme attribute on root element for settings window self-theming"
    - "isTauriContext guard pattern: ref checked before any invoke() or plugin call"

key-files:
  created:
    - src/Settings.vue
  modified:
    - src/settings-main.ts

key-decisions:
  - "Dynamic import of @tauri-apps/plugin-autostart inside onAutostartChange/onMounted — consistent with plugin-dialog dynamic import pattern from Plan 02's PathList.vue"
  - "emitTo('launcher', 'settings-changed', payload) used (not emit()) to target launcher window only and avoid self-handling on settings window"
  - "Autostart isEnabled() called only in non-portable mode on mount — avoids plugin errors in portable context"
  - "Global html/body/#app height:100% and margin:0 placed in unscoped <style> block — required for settings window to fill viewport without scrollbar"

patterns-established:
  - "Settings window self-theming: data-theme set on documentElement on mount and on theme change"
  - "saveSettings() always called after every change handler for consistency, even when Tauri command (update_hotkey) already persists internally"

requirements-completed: [SETT-01, SETT-02, SETT-04, SETT-05, SETT-06, SETT-07]

# Metrics
duration: 2min
completed: 2026-03-08
---

# Phase 8 Plan 03: Settings Window UI Summary

**Full settings window built: settings-main.ts entry point with font imports + Settings.vue root component composing all four sections (General, Hotkey, Search, Appearance) across all five UI primitives from Plan 02, with every change persisting via set_settings_cmd and emitting to the launcher window.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-08T17:42:30Z
- **Completed:** 2026-03-08T17:44:04Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Completed settings-main.ts: added Inter 400/500, JetBrains Mono 400 font imports and mounted Settings.vue
- Created Settings.vue with four sections: General (autostart), Hotkey (KeyCapture), Search (PathList x2, interval, re-index), Appearance (theme, opacity, show_path)
- Autostart toggle disabled with hint text in portable mode; uses enable()/disable() from plugin-autostart in installed mode
- Hotkey section calls invoke('update_hotkey') then saveSettings() on change
- Search paths call invoke('reindex') after saving; Re-index button shows "Indexing..." feedback for 1s
- Appearance changes emit to launcher via emitTo('launcher', 'settings-changed') for live updates
- Custom frameless header with data-tauri-drag-region, app title, and close button
- Opacity slider range 0.85–1.0 per SETT-07

## Task Commits

Each task was committed atomically:

1. **Task 1: Complete settings-main.ts entry point** - `9720629` (feat)
2. **Task 2: Build Settings.vue with all four sections** - `5cf36a2` (feat)

## Files Created/Modified

- `src/settings-main.ts` - Upgraded from stub: font imports + Settings.vue mount
- `src/Settings.vue` - Full settings window root with all four sections (323 lines)

## Decisions Made

- Dynamic import of `@tauri-apps/plugin-autostart` inside handler functions — consistent with Plan 02's PathList.vue `plugin-dialog` dynamic import pattern, avoids top-level import failures in browser dev mode
- `emitTo('launcher', 'settings-changed', payload)` used throughout (not `emit()`) — targets only the launcher window, prevents the settings window from reacting to its own events
- Global `html, body, #app` height/margin rules placed in an unscoped `<style>` block — scoped styles cannot target elements outside the component's shadow root; required for the settings window to fill the Tauri webview correctly

## Deviations from Plan

None — plan executed exactly as written. All section implementations match plan specification. pnpm build passed on first attempt.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Plan 04 (human verification): settings window is ready to open via invoke('open_settings_window') and verify all four sections visually and functionally
- All SETT-04, SETT-05, SETT-06, SETT-07 requirements implemented

## Self-Check: PASSED

- FOUND: src/settings-main.ts
- FOUND: src/Settings.vue
- FOUND: 9720629 (Task 1 commit)
- FOUND: 5cf36a2 (Task 2 commit)
- emitTo('launcher', 'settings-changed') present (4 occurrences)
- update_hotkey invoked on hotkey change (1 occurrence)
- set_settings_cmd invoked via saveSettings() (1 occurrence)
- reindex invoked on path change and Re-index button (2 occurrences)
- Portable mode guard on autostart (6 isPortable references)
- Opacity slider min=0.85 confirmed

---
*Phase: 08-settings-window*
*Completed: 2026-03-08*
