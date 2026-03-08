---
quick_task: 3
title: Hotkey reset button for Alt+Space default in settings window
date: 2026-03-08
commit: 3d26264
files_modified:
  - src/Settings.vue
---

# Quick Task 3: Hotkey Reset Button Summary

**One-liner:** Added a conditional "Reset" text-link button inline with KeyCapture to restore the Alt+Space default hotkey without needing to type it.

## Problem

Users who set a custom hotkey (e.g. Ctrl+Alt+Space) had no way to restore the default Alt+Space. Pressing Alt+Space inside the Settings window triggers the OS/browser context menu rather than being captured by the KeyCapture component — this is an OS-level interception that cannot be reliably suppressed.

## Solution

Added a small "Reset" button next to the KeyCapture input in the Hotkey row. The button:

- Only renders when `settings.value.hotkey !== 'Alt+Space'` (hidden when already at default)
- Calls `onHotkeyChange('Alt+Space')` on click, which invokes `update_hotkey` in Rust and persists to `settings.json`
- Styled as a subtle text link (accent color, underline, no border or background) so it does not dominate the row

## Changes

### `src/Settings.vue`

- Wrapped KeyCapture in a `.hotkey-row` flex container
- Added `v-if="settings.hotkey !== 'Alt+Space'"` Reset button
- Added `.hotkey-row`, `.reset-link`, `.reset-link:focus`, `.reset-link:hover` CSS rules

## Deviations

None — plan executed exactly as written.
