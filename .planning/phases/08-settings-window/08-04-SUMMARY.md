---
phase: 08-settings-window
plan: 04
subsystem: ui
tags: [vue, tauri, settings, reactive, keyboard-shortcut, css-custom-properties, single-instance-window]

# Dependency graph
requires:
  - phase: 08-03
    provides: Settings.vue with emitTo('launcher', 'settings-changed') event emissions

provides:
  - App.vue listens for settings-changed event and applies theme and show_path in real time
  - Ctrl+, keyboard shortcut opens settings window from launcher
  - applyTheme() sets data-theme attribute on documentElement for CSS theme switching
  - Settings window hides (not closes) on X so state is preserved between opens
  - launcher-show emitted from Settings.vue closeWindow() so launcher reappears when settings closes
  - hotkey::register() falls back to Alt+Space when OS rejects requested key

affects: [09-global-hotkey, 10-packaging]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "listen<SettingsPayload>('settings-changed') for typed reactive event handling between windows"
    - "applyTheme() removes data-theme for system, sets it for explicit themes"
    - "SettingsCentered(AtomicBool) managed state to center settings window only on first open"
    - "Settings window uses hide() not close() — preserves state and last position between opens"
    - "launcher-show emitted by Settings.vue on close so App.vue show/focus path handles both hotkey and settings-close"
    - "Hotkey register() returns actually-registered hotkey and falls back to Alt+Space on OS rejection"

key-files:
  created: []
  modified:
    - src/App.vue
    - src/Settings.vue
    - src-tauri/src/hotkey.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "Opacity setting removed — launcher opacity slider and --launcher-opacity CSS variable removed; opacity does not make sense as a user setting"
  - "Settings window uses hide() not close() — SettingsCentered(AtomicBool) managed state tracks first-open for centering; subsequent opens restore last position"
  - "hotkey::register() returns the actually-registered hotkey and falls back to Alt+Space if OS rejects requested key (e.g. Ctrl+Space blocked by Windows IME); fallback persisted to settings.json"
  - "launcher-show emitted by Settings.vue closeWindow() before hiding — App.vue launcher-show handler now calls show() + setFocus(), making it handle both hotkey-path (Rust) and settings-close-path (Vue)"
  - "Background gradient uses solid CSS color tokens (not rgba); opacity: 0→1 on container is the only opacity transition — text/icons always at full opacity"
  - "Settings window width set to 450px (minWidth also 450) — prior 800px was too wide"
  - "@mousedown.stop on close button required: Tauri drag region intercepts mousedown on all children, preventing click from registering"
  - "data-tauri-drag-region added to title span so full header width is draggable"
  - "Custom scrollbar uses var(--color-border) for thumb — contrasts with background without being distracting, adapts to theme"

patterns-established:
  - "Interactive elements inside data-tauri-drag-region need @mousedown.stop to receive click events"
  - "Hide-not-close pattern for secondary windows: preserves state, use AtomicBool managed state to detect first-open for centering"
  - "launcher-show as unified show-and-focus signal — both Rust (hotkey) and Vue (settings close) emit the same event, App.vue handles it identically"
  - "Hotkey registration with OS fallback: attempt requested key, fall back to safe default, persist whatever was actually registered"

requirements-completed:
  - SETT-03
  - SETT-07

# Metrics
duration: 18min
completed: 2026-03-08
---

# Phase 8 Plan 04: Settings Integration Summary

**Reactive settings propagation via settings-changed listener + Ctrl+, shortcut in App.vue, hide-not-close window lifecycle, hotkey fallback to Alt+Space, and launcher-show unified show path — all SETT-01 through SETT-07 requirements human-verified**

## Performance

- **Duration:** 18 min (Task 1: 3 min + verification fixes: 15 min)
- **Started:** 2026-03-08T17:46:00Z
- **Completed:** 2026-03-08T18:10:00Z
- **Tasks:** 2 of 2 complete
- **Files modified:** 4

## Accomplishments
- settings-changed Tauri event listener added in App.vue onMounted — applies theme and show_path from Settings window in real time
- applyTheme() sets/removes data-theme on documentElement for CSS-driven theme switching
- Ctrl+, keyboard shortcut added as first check in onKeyDown — opens settings from launcher without disrupting other key handling
- Settings window uses hide() instead of close() — preserves DOM state and last position between opens; SettingsCentered(AtomicBool) centers only on first open
- Settings.vue closeWindow() emits launcher-show before hiding — App.vue launcher-show handler now calls show() + setFocus() so launcher reappears correctly from both hotkey and settings-close paths
- hotkey::register() falls back to Alt+Space when OS rejects requested key; fallback persisted to settings.json to avoid retry loop on next startup
- Opacity slider and --launcher-opacity removed — opacity is not a meaningful user setting
- All seven SETT requirements (SETT-01 through SETT-07) human-verified and complete

## Task Commits

1. **Task 1: Add settings-changed listener and Ctrl+, shortcut to App.vue** - `72d9393` (feat)
2. **Fix: settings window width, close button, drag region, custom scrollbar** - `1930985` (fix)

**Plan metadata:** `bdaf5da` / `935a4fe` (docs commits)

## Files Created/Modified
- `src/App.vue` — Added SettingsPayload interface, applyTheme(), settings-changed listener, Ctrl+, shortcut, initial theme from settings, onUnmounted cleanup; launcher-show handler updated to call show() + setFocus(); opacity refs and CSS variable removed
- `src/Settings.vue` — Fixed close button (@mousedown.stop), data-tauri-drag-region on title span, custom scrollbar CSS, closeWindow() emits launcher-show before hide(); opacity slider section removed
- `src-tauri/src/hotkey.rs` — register() returns actually-registered hotkey string; falls back to "Alt+Space" on OS rejection and persists fallback to settings.json
- `src-tauri/src/lib.rs` — SettingsCentered(AtomicBool) managed state; open_settings_window centers on first open only

## Decisions Made

1. **Opacity setting removed.** The launcher opacity slider and --launcher-opacity CSS variable were removed. A semi-transparent launcher would require backdrop-filter for the frosted-glass effect to look intentional; plain opacity just makes text harder to read. Not a useful setting.

2. **Settings window uses hide() not close().** Closing the settings window hides it so it can be reopened without re-mounting the Vue component (preserves scroll position, pending edits, etc.). `SettingsCentered(AtomicBool)` managed state tracks whether the window has been shown before; first open centers the window, subsequent opens restore the last user-dragged position.

3. **Hotkey fallback to Alt+Space.** `hotkey::register()` now catches OS rejection (e.g. Ctrl+Space blocked by Windows IME) and falls back to Alt+Space. It returns the string of whichever key was actually registered. The fallback is persisted to settings.json immediately so the bad key is not retried on next startup.

4. **launcher-show as unified show-and-focus signal.** Settings.vue `closeWindow()` calls `emitTo('launcher', 'launcher-show')` before calling `getCurrentWindow().hide()`. App.vue's existing launcher-show listener was updated to also call `win.show()` + `win.setFocus()` — it now handles both the Rust hotkey path and the Vue settings-close path identically, with no duplicate logic.

5. **Solid color tokens in background gradient.** Background uses CSS color tokens directly (not rgba). The only opacity transition is the container's 0→1 appear animation; text and icons are always at full opacity.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed opacity slider/CSS variable — would silently break show/hide animation**
- **Found during:** Task 1 (reviewing template addition for opacity binding)
- **Issue:** Plan specified `:style="{ opacity: launcherOpacity }"` on root element, but inline style specificity overrides CSS class transitions (`.anim-slide.visible { opacity: 1 }`), breaking show/hide animations
- **Fix:** Initially used CSS custom property as workaround; after human verification, opacity setting was removed entirely as it is not a meaningful user-facing feature
- **Files modified:** src/App.vue, src/Settings.vue
- **Committed in:** 72d9393 / 1930985

**2. [Rule 1 - Bug] Fixed four settings window UI issues found during human verification**
- **Found during:** Task 2 (human verification)
- **Issue 1:** Window 800px wide — too wide for a settings panel
- **Issue 2:** Close button non-functional — Tauri drag region intercepted mousedown on all children
- **Issue 3:** Header not fully draggable — drag region only on the div, not the title span
- **Issue 4:** Default browser scrollbar — visually inconsistent with app theme
- **Fix:** tauri.conf.json width/minWidth to 450; `@mousedown.stop` on close button; `data-tauri-drag-region` on title span; custom CSS scrollbar (6px thumb, var(--color-border), transparent track)
- **Files modified:** src-tauri/tauri.conf.json, src/Settings.vue
- **Committed in:** 1930985

---

**Total deviations:** 2 auto-fixed (both Rule 1 — animation bug prevention + UI bug fixes from verification)
**Impact on plan:** All fixes necessary for correctness and usability. Opacity removal reduces scope (one less setting to maintain). No unplanned scope added.

## Issues Encountered
- Tauri drag region silently absorbs mousedown from all child elements including buttons — `@mousedown.stop` required on interactive children inside `data-tauri-drag-region` divs.
- OS can reject hotkey registration silently (Windows IME blocks Ctrl+Space and others) — fallback + persist pattern is now the standard approach.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 8 (Settings Window) fully complete — all SETT-01 through SETT-07 requirements human-verified
- Phase 9 (Global Hotkey) ready to begin — update_hotkey Tauri command and fallback logic already in hotkey.rs
- No blockers

## Self-Check: PASSED

---
*Phase: 08-settings-window*
*Completed: 2026-03-08*
