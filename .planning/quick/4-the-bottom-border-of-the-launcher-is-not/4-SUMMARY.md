---
phase: quick-4
plan: 4
subsystem: frontend
tags: [window-sizing, border, webview2, tauri]
dependency_graph:
  requires: []
  provides: [visible-bottom-border]
  affects: [src/App.vue]
tech_stack:
  added: []
  patterns: [BOTTOM_PAD constant, consistent height padding]
key_files:
  created: []
  modified:
    - src/App.vue
decisions:
  - BOTTOM_PAD = 8px added as a named constant so the extra transparent area is documented and referenced from one place; 8px chosen to comfortably contain the 1px border pixel inside the WebView2-rendered transparent area
metrics:
  duration: 3min
  completed: 2026-03-08
---

# Quick Task 4: Fix Invisible Bottom Border — Summary

**One-liner:** Added `BOTTOM_PAD = 8` constant applied to all four `setSize()` height calculations so the 1px bottom CSS border renders reliably inside the transparent OS window area instead of being clipped by WebView2.

## What Was Done

The launcher window bottom border was invisible because `setSize()` sized the OS window exactly to the CSS content height, placing the border pixel at the very edge of the transparent window where WebView2 does not reliably paint it.

The fix adds `BOTTOM_PAD = 8` (8 transparent pixels of headroom) to every height passed to `getCurrentWindow().setSize()`:

1. `updateWindowHeight()` — normal search state height
2. `watch(menuVisible)` restore handler — height after context menu closes
3. `onContextMenu()` overflow calculation — base height used to detect overflow
4. `launcher-show` listener reset — empty-state height on window summon

No CSS changes were made. The extra 8px is transparent OS window area below the `.launcher-app` component; it is not visible to the user.

## Verification

- `pnpm build` exits 0 — no type errors
- Manual verification required in `pnpm tauri dev`: summon launcher, confirm bottom border visible in empty state and with results; confirm border is stable (not flickering) when context menu opens/closes

## Commits

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Add BOTTOM_PAD constant, apply to all four height sites | 3ce3e0a | src/App.vue |

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check

- [x] `src/App.vue` modified — confirmed
- [x] Commit `3ce3e0a` exists — confirmed
- [x] `pnpm build` exits 0 — confirmed

## Self-Check: PASSED
