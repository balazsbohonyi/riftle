---
phase: 03-indexer
plan: 03
subsystem: indexer
tags: [rust, windows-gdi, icon-extraction, png, image-crate, unsafe]

# Dependency graph
requires:
  - phase: 03-indexer/03-01
    provides: ensure_generic_icon and extract_icon_png stubs, GENERIC_ICON static bytes, image crate dependency
provides:
  - ensure_generic_icon: idempotent bootstrap of generic.png to {data_dir}/icons/
  - extract_icon_png: full GDI pipeline returning RGBA PNG bytes or None on failure
affects:
  - 03-indexer/03-04 (run_full_index calls ensure_generic_icon; icon extraction loop calls extract_icon_png)
  - 03-indexer/03-05 (background tasks use extract_icon_png in spawned threads)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Win32 GDI icon extraction: ExtractIconExW -> GetIconInfo -> GetObjectW -> GetDIBits -> BGRA swap -> image::RgbaImage -> PNG"
    - "Negative biHeight in BITMAPINFOHEADER for top-down scanline order (avoids post-flip)"
    - "BGRA to RGBA via chunks_exact_mut(4).swap(0,2) inline swap"
    - "Idempotent file bootstrap: create_dir_all + dest.exists() guard before write"

key-files:
  created: []
  modified:
    - src-tauri/src/indexer.rs
    - src-tauri/Cargo.toml

key-decisions:
  - "Win32_Foundation feature required for DeleteDC, DeleteObject, DestroyIcon, GetIconInfo — added to windows-sys features in Cargo.toml"
  - "All GDI failure paths return None via early return — no panic, no unwrap"
  - "No unit test for extract_icon_png — GDI requires real Windows context with actual exe files; test_generic_icon_bootstrap covers ensure_generic_icon"

patterns-established:
  - "Unsafe GDI blocks: always cleanup (DestroyIcon, DeleteDC, DeleteObject) before returning None on any failure path"
  - "BGRA->RGBA swap inline before image::RgbaImage::from_raw — GDI always delivers BGRA"

requirements-completed:
  - INDX-04
  - INDX-05

# Metrics
duration: 4min
completed: 2026-03-06
---

# Phase 3 Plan 03: Icon Extraction Pipeline Summary

**Windows GDI icon extraction pipeline (ExtractIconExW -> GetIconInfo -> GetDIBits -> BGRA->RGBA -> PNG) and idempotent generic.png bootstrap implemented; test_generic_icon_bootstrap GREEN**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-06T09:14:28Z
- **Completed:** 2026-03-06T09:18:42Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Replaced `ensure_generic_icon` todo!() stub with idempotent implementation that creates `{data_dir}/icons/` and writes GENERIC_ICON bytes only when generic.png is absent
- Replaced `extract_icon_png` todo!() stub with full Windows GDI extraction pipeline using unsafe Rust
- Added `Win32_Foundation` windows-sys feature (required for DeleteDC, DeleteObject, DestroyIcon, GetIconInfo)
- All 18 lib tests remain GREEN; 0 regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement ensure_generic_icon** - `7da8773` (feat - TDD GREEN)
2. **Task 2: Implement extract_icon_png GDI pipeline** - `82c4903` (feat)

**Plan metadata:** _(final docs commit — see below)_

_Note: Task 1 used TDD — test existed in RED state (should_panic on todo!()); implementation made it GREEN by removing should_panic and implementing the function._

## Files Created/Modified
- `src-tauri/src/indexer.rs` - ensure_generic_icon and extract_icon_png implemented; GDI imports added
- `src-tauri/Cargo.toml` - Added Win32_Foundation to windows-sys features

## Decisions Made
- `Win32_Foundation` feature added to `windows-sys` — required by `DeleteDC`, `DeleteObject`, `DestroyIcon`, `GetIconInfo` which gated behind `#[cfg(feature = "Win32_Foundation")]` in windows-sys 0.52. Plan's feature list omitted this; discovered during cargo check (Rule 3 auto-fix).
- No unit test for `extract_icon_png` — GDI calls require a real Windows context with valid exe files. This is documented in the plan and VALIDATION.md (manual smoke test only for INDX-05).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added Win32_Foundation feature to windows-sys in Cargo.toml**
- **Found during:** Task 2 (extract_icon_png compilation)
- **Issue:** `DeleteDC`, `DeleteObject`, `DestroyIcon`, and `GetIconInfo` are gated behind `#[cfg(feature = "Win32_Foundation")]` in windows-sys 0.52. The plan's Cargo.toml feature list omitted this feature, causing unresolved import errors.
- **Fix:** Added `"Win32_Foundation"` to the windows-sys features array in Cargo.toml
- **Files modified:** src-tauri/Cargo.toml
- **Verification:** `cargo check -p riftle` produced 0 errors after the fix
- **Committed in:** `82c4903` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking dependency/feature)
**Impact on plan:** Essential for compilation — no scope creep.

## Issues Encountered
- None beyond the Win32_Foundation feature gap documented above.

## Next Phase Readiness
- `ensure_generic_icon` ready for call in `run_full_index` (Plan 04)
- `extract_icon_png` ready for use in icon extraction loop spawned in Plan 04
- Both functions compile and follow the documented None-on-failure contract
- No blockers for Plan 04

## Self-Check: PASSED

- FOUND: src-tauri/src/indexer.rs
- FOUND: src-tauri/Cargo.toml
- FOUND: .planning/phases/03-indexer/03-03-SUMMARY.md
- FOUND: commit 7da8773 (feat(03-03): implement ensure_generic_icon)
- FOUND: commit 82c4903 (feat(03-03): implement extract_icon_png GDI pipeline)
- Verified: 18 lib tests GREEN, 0 failures

---
*Phase: 03-indexer*
*Completed: 2026-03-06*
