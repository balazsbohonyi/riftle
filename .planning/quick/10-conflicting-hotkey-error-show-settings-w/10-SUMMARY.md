---
phase: 10-conflicting-hotkey
plan: "01"
subsystem: hotkey-conflict-ux
tags: [hotkey, settings, tauri-events, ux, error-display]
dependency_graph:
  requires: [hotkey.rs register(), show_settings_window()]
  provides: [startup hotkey conflict detection, hotkey-conflict event, Settings inline error display]
  affects: [src-tauri/src/lib.rs, src/Settings.vue]
tech_stack:
  patterns: [Tauri event emission (emit), dynamic import @tauri-apps/api/event, Vue ref scrollIntoView]
key_files:
  modified:
    - src-tauri/src/lib.rs
    - src/Settings.vue
decisions:
  - hotkey_conflict declared as Option<String> before #[cfg(desktop)] block so it is accessible after the block for the Settings window notification
  - listen() placed outside the try/catch in onMounted but after the isTauriContext guard — settings load errors do not prevent conflict listener from registering
  - scrollIntoView called via nextTick to ensure DOM has updated before scroll
  - unlistenConflictRef cleaned up in onUnmounted to prevent memory leaks
metrics:
  duration: "3min"
  completed_date: "2026-05-16"
  tasks_completed: 2
  files_modified: 2
---

# Quick Task 10: Conflicting Hotkey Error — Show Settings Window Summary

**One-liner:** Auto-opens Settings on startup hotkey conflict with inline error "'[key]' is already in use by another app" and scrolls hotkey section into view.

## What Was Built

When hotkey registration fails at startup (the configured key is already taken by another app), the application now:

1. Opens the Settings window automatically
2. Emits a `hotkey-conflict` Tauri event to the settings window carrying the failed key name
3. Settings.vue listens for the event, sets `hotkeyError` with an actionable message, and scrolls the hotkey `<Row>` into view

The error clears automatically when the user successfully sets a new hotkey via the existing `onHotkeyChange` handler (which already calls `hotkeyError.value = null` at the top).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Detect startup hotkey conflict in lib.rs and open Settings with event | f23816f | src-tauri/src/lib.rs |
| 2 | Handle hotkey-conflict event in Settings.vue with error display and scroll | 38797a4 | src/Settings.vue |

## Key Changes

**src-tauri/src/lib.rs**
- Added `hotkey_conflict: Option<String>` variable before the `#[cfg(desktop)]` block
- Inside the desktop block, assigns `Some(failed_key)` when `actual_hotkey != settings.hotkey` (fallback triggered)
- After full setup, calls `show_settings_window()` and `emit("hotkey-conflict", &failed_key)` when conflict detected

**src/Settings.vue**
- Added `hotkeyRowRef: ref<HTMLElement | null>(null)` targeting the hotkey Row component
- Added `unlistenConflictRef: (() => void) | undefined` for cleanup
- Added `hotkey-conflict` listener in `onMounted` (after `isTauriContext` guard): sets `hotkeyError`, scrolls `hotkeyRowRef` into view via `nextTick`
- Added `unlistenConflictRef?.()` in `onUnmounted`
- Added `ref="hotkeyRowRef"` to `<Row label="Global shortcut">`

## Verification

- `cargo test`: 104 passed, 0 failed, 2 ignored
- `pnpm build`: TypeScript and Vite build succeeded with no errors
- Normal startup (no conflict): Settings window does not open — `hotkey_conflict` remains `None`

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- `src-tauri/src/lib.rs` modified and committed at f23816f
- `src/Settings.vue` modified and committed at 38797a4
- Both commits verified in git log
