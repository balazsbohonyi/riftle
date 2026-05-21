---
quick_task: 3
title: Hotkey reset button for Ctrl+Space default in settings window
date: 2026-03-08
commit: 3d26264
files_modified:
  - src/Settings.vue
---

# Quick Task 3: Hotkey Reset Button Summary

**One-liner:** Added a conditional "Reset" text-link button inline with KeyCapture to restore the Ctrl+Space default hotkey without needing to type it.

## Decision Amendment - 2026-05-21

The default hotkey is now Ctrl+Space. The reset button continues to restore the current default rather than preserving the earlier default from the original quick task.

## Problem

Users who set a custom hotkey had no way to restore the default Ctrl+Space without recording it manually.

## Solution

Added a small "Reset" button next to the KeyCapture input in the Hotkey row. The button:

- Only renders when `settings.value.hotkey !== defaultHotkey` (hidden when already at default)
- Calls `onHotkeyChange(defaultHotkey)` on click, which invokes `update_hotkey` in Rust and persists to `settings.json`
- Styled as a subtle text link (accent color, underline, no border or background) so it does not dominate the row

## Changes

### `src/Settings.vue`

- Wrapped KeyCapture in a `.hotkey-row` flex container
- Added a Reset button keyed to the current default hotkey
- Added `.hotkey-row`, `.reset-link`, `.reset-link:focus`, `.reset-link:hover` CSS rules

## Deviations

None - plan executed exactly as written.
