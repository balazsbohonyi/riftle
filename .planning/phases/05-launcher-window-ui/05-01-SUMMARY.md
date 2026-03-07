---
phase: 05-launcher-window-ui
plan: 01
subsystem: backend
tags: [rust, tauri, settings, animation, asset-protocol, csp, serde]

# Dependency graph
requires:
  - phase: 02-data-layer
    provides: Settings struct and store.rs (get_settings/set_settings) that this plan extends
  - phase: 04-search-engine
    provides: lib.rs invoke_handler pattern this plan adds to
provides:
  - Settings.animation field with serde default "slide"
  - get_settings_cmd Tauri command returning show_path, animation, data_dir, hotkey, theme, opacity
  - tauri.conf.json asset protocol + CSP for convertFileSrc() icon images
affects:
  - 05-launcher-window-ui (App.vue uses invoke('get_settings_cmd') and convertFileSrc())
  - 08-settings-window (will read/write animation setting)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Tauri command returns serde_json::Value — avoids needing a separate DTO struct for partial field projection"
    - "tauri::State<PathBuf> injection in Tauri command for data_dir access without re-calling paths::data_dir"
    - "assetProtocol scope ['**'] allows any local path — icons live in user AppData or portable data dir"

key-files:
  created: []
  modified:
    - src-tauri/src/store.rs
    - src-tauri/src/lib.rs
    - src-tauri/tauri.conf.json

key-decisions:
  - "get_settings_cmd placed in store.rs (not a separate commands file) — settings-related Tauri commands belong alongside settings logic"
  - "assetProtocol scope ['**'] chosen over explicit path list — icon paths vary between installed and portable mode"
  - "CSP includes data: and blob: in img-src for future inline image fallbacks"

patterns-established:
  - "Tauri command returning serde_json::Value for partial Settings projection (not full struct) — reusable in Phase 8"
  - "serde default functions follow naming convention default_{field_name}() — consistent with existing store.rs pattern"

requirements-completed: [LWND-07, LWND-08, LWND-12]

# Metrics
duration: 7min
completed: 2026-03-06
---

# Phase 05 Plan 01: Backend Settings Extension and Asset Protocol Summary

**Animation field added to Settings struct + get_settings_cmd Tauri command + asset:// CSP unblocking convertFileSrc() icon images**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-06T20:51:46Z
- **Completed:** 2026-03-06T20:58:31Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Settings struct extended with `animation: String` field (serde default "slide"), default function, and updated `impl Default`
- `get_settings_cmd` Tauri command added to store.rs and registered in lib.rs invoke_handler — returns show_path, animation, data_dir, hotkey, theme, opacity
- `tauri.conf.json` security block updated with `assetProtocol.enable: true` and CSP `img-src asset:` scheme so `convertFileSrc()` icon URLs are not blocked
- All 4 store unit tests GREEN (34 total, 2 ignored); no regressions

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Failing tests for animation field** - `24b2fe1` (test)
2. **Task 1 GREEN: animation field + get_settings_cmd implementation** - `54e025f` (feat)
3. **Task 2: lib.rs + tauri.conf.json wiring** - `e5ba290` (feat — included in 05-02 branch commit)

_Note: Task 2 changes (lib.rs invoke_handler, tauri.conf.json) were already present in the branch's existing 05-02 commit. Both files verified correct before Task 2 ran._

## Files Created/Modified

- `src-tauri/src/store.rs` - Added `animation` field with `default_animation()`, updated `Default`, added `get_settings_cmd` Tauri command, updated 3 unit tests
- `src-tauri/src/lib.rs` - Added `crate::store::get_settings_cmd` to `invoke_handler` macro
- `src-tauri/tauri.conf.json` - Replaced `"csp": null` with full CSP + `assetProtocol` config block

## Decisions Made

- `get_settings_cmd` placed in `store.rs` alongside settings logic (not a separate commands module) — consistent with the file's responsibility
- `assetProtocol.scope: ["**"]` used instead of a restrictive path list because icon paths differ between installed (%APPDATA%) and portable (target/debug/data/) modes
- CSP includes `data:` and `blob:` in `img-src` to support potential inline image fallbacks in Phase 5 UI

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - both tasks completed cleanly on first attempt.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Frontend (App.vue) can now call `invoke('get_settings_cmd')` and receive `{ show_path, animation, data_dir, hotkey, theme, opacity }`
- `<img>` tags using `convertFileSrc()` will resolve to `asset://` URLs that pass CSP
- Phase 5 Plan 02 (launcher UI) is unblocked — backend contract fulfilled

---
*Phase: 05-launcher-window-ui*
*Completed: 2026-03-06*
