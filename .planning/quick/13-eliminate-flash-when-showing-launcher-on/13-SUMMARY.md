---
phase: 13-eliminate-flash-when-showing-launcher
plan: 01
subsystem: lib.rs
tags: [bugfix, dpi, multi-monitor, flash, perf]
requires: []
provides: [DPI-FIX-02]
affects: [src-tauri/src/lib.rs]
tech-stack:
  added: []
  patterns:
    - "Monitor resolution before show() — compute PhysicalSize from logical × scale_factor"
    - "set_size(PhysicalSize) before show() to eliminate stale-geometry flash"
key-files:
  created: []
  modified:
    - src-tauri/src/lib.rs
decisions:
  - "Resolve monitor before show() to compute physical size from correct scale factor"
  - "Use PhysicalSize instead of LogicalSize for set_size() — bypasses stale DPI conversion"
metrics:
  duration: "about 5 min"
  completed: "2026-06-14"
---


# Phase 13 Plan 1: Eliminate Flash When Showing Launcher — Summary

**One-liner:** Rewrite `show_positioned_launcher` to resolve target monitor, compute `PhysicalSize` from the correct `scale_factor`, and call `set_size()` before `show()` — eliminating the one-frame stale-size flash on DPI change.

## Task Results

| Task | Name                                                                        | Commit   | Files Modified              |
| ---- | --------------------------------------------------------------------------- | -------- | --------------------------- |
| 1    | Move monitor resolution before show(), use PhysicalSize for set_size()      | `0188694` | `src-tauri/src/lib.rs`      |

## Verification Results

| Check              | Result |
| ------------------ | ------ |
| `cargo test`       | ✅ 101/101 passed |
| `pnpm build`       | ✅ passed |
| Order: monitor → `set_size(PhysicalSize)` → `show()` | ✅ Confirmed in code |
| `PhysicalSize` used (not `LogicalSize`)             | ✅ |
| `PhysicalSize` computed from logical × scale_factor | ✅ |
| `scale_factor` reused for both sizing and position  | ✅ |

## Execution Details

**Approach:** The root cause of the flash was that `show()` revealed the window at its stale cached geometry (from previous monitor's DPI), and only then `set_size(LogicalSize)` resized it correctly. The fix:

1. **Resolve target monitor FIRST** (cursor → primary → current) — before the window is shown
2. **Extract `scale_factor`** from the resolved monitor (fallback 1.0)
3. **`set_size(PhysicalSize::new(...))`** — computed as `(logical_width × scale_factor, logical_height × scale_factor)`. `PhysicalSize` bypasses DPI conversion entirely, so the window gets correct pixel dimensions regardless of stale internal DPI state.
4. **`show()`** — window is now visible at the correct size from frame 1 — no flash
5. **Position** using same monitor and `scale_factor`
6. **`set_focus()`**

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- [x] File exists: `src-tauri/src/lib.rs` — shows correct `PhysicalSize` ordering
- [x] Commit exists: `0188694` — "fix(13-eliminate-flash-when-showing-launcher): move monitor resolution before show(), use PhysicalSize"
- [x] `cargo test` passes: 101/101
- [x] `pnpm build` passes
