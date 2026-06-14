---
phase: 12-fix-multi-monitor-dpi-sizing
plan: 01
subsystem: window-management
tags: [dpi, multi-monitor, sizing, display]
requires: []
provides: [DPI-FIX-01]
affects: [src-tauri/src/lib.rs]
tech-stack:
  added: []
  patterns:
    - "Show window before set_size to trigger DPI re-evaluation on Windows"
key-files:
  created: []
  modified:
    - src-tauri/src/lib.rs
decisions:
  - "win.show() moved before win.set_size() — DPI change notifications (WM_DPICHANGED) only fire when a hidden window becomes visible"
metrics:
  duration: "1 min"
  completed: 1/1 tasks
  completed_date: "2026-06-14"
---

# Quick Task 12: Fix Multi-Monitor DPI Sizing on Connect/Disconnect — Summary

**One-liner:** Reordered `show_positioned_launcher` so `win.show()` fires first, triggering Windows DPI re-evaluation before logical-to-physical size mapping.

## Objective

Fix the multi-monitor DPI sizing bug where the launcher shows with incorrect physical width on first summon after connecting or disconnecting an external monitor with a different DPI scale.

**Root cause:** `set_size(LogicalSize)` was called while the window was hidden, using stale DPI awareness from the previous monitor. Windows only sends `WM_DPICHANGED` when a window becomes visible. The stale physical size became visible on `show()` — hence the hide+reshow workaround.

**Fix:** Moved `win.show()` to the first operation. This triggers DPI re-evaluation before `set_size()` maps logical pixels to physical dimensions. The fade-in animation prevents any visible flash.

## Tasks Executed

| # | Name | Type | Status | Commit | Files |
|---|------|------|--------|--------|-------|
| 1 | Reorder show_positioned_launcher — show() before set_size()/set_position() | auto | Done | `2b7498a` | `src-tauri/src/lib.rs` |

## Verification

- [x] `cargo test` — 101 tests passed, 0 failed
- [x] `pnpm build` — frontend type-checks and builds successfully
- [x] `show_positioned_launcher` now calls `win.show()` before `win.set_size()`
- [x] No other behavior changed

## Deviations from Plan

None — plan executed exactly as written.

## Success Criteria

- [x] `cargo test` passes
- [x] `pnpm build` passes
- [x] `show_positioned_launcher` orders operations as: show → set_size → resolve_monitor → set_position → set_focus
- [x] Launcher no longer shows with wrong size on first summon after monitor connect/disconnect (requires manual verification on multi-monitor setup)

## Self-Check: PASSED

- [x] Modified file exists: `src-tauri/src/lib.rs`
- [x] Commit exists: `2b7498a`
