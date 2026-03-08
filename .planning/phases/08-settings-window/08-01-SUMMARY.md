---
phase: 08-settings-window
plan: 01
subsystem: ui
tags: [vite, css-tokens, tauri, rust, multi-page, settings]

# Dependency graph
requires:
  - phase: 07-context-menu
    provides: quit_app command and App.vue context menu infrastructure
provides:
  - CSS design tokens shared across both Vite entry points (tokens.css)
  - settings.html Vite entry point and settings-main.ts stub
  - Multi-page Vite build (main + settings rollupOptions)
  - set_settings_cmd Tauri command for full settings persistence
  - open_settings_window Tauri command (show + focus settings webview)
  - Extended get_settings_cmd returning all fields plus is_portable
  - tauri-plugin-dialog wired throughout (Cargo.toml, lib.rs, capabilities)
affects: [08-02-token-refactor, 08-03-settings-ui, 08-04-settings-integration]

# Tech tracking
tech-stack:
  added: [tauri-plugin-dialog v2.6.0]
  patterns:
    - CSS custom properties in :root with light theme override and media query
    - Multi-page Vite build with rollupOptions.input for two HTML entry points
    - tauri.conf.json window url field pointing to named HTML page
    - open_settings_window as inline lib.rs command (show + set_focus pattern)

key-files:
  created:
    - src/styles/tokens.css
    - settings.html
    - src/settings-main.ts
  modified:
    - src/main.ts
    - vite.config.ts
    - src-tauri/tauri.conf.json
    - src-tauri/src/store.rs
    - src-tauri/src/lib.rs
    - src-tauri/Cargo.toml
    - src-tauri/capabilities/default.json

key-decisions:
  - "settings-main.ts created as minimal stub (tokens.css import only) — full settings Vue component is Plan 03's responsibility"
  - "tauri-plugin-dialog added as caret range '2' consistent with tauri-plugin-opener pattern in project"
  - "open_settings_window placed inline in lib.rs (not a separate module) — single-command pattern not worth a new file"
  - "is_portable detection uses data_dir.ends_with('data') + portable file existence check — mirrors paths.rs logic"

patterns-established:
  - "tokens.css: import as first import in both main.ts and settings-main.ts — establishes CSS variable baseline before any component styles"
  - "Multi-page Vite: each window gets its own HTML entry + TS entry — rollupOptions.input is the wiring"

requirements-completed: [SETT-01, SETT-02]

# Metrics
duration: 3min
completed: 2026-03-08
---

# Phase 8 Plan 01: Settings Window Foundation Summary

**CSS design tokens, multi-page Vite build, settings window URL, and all Rust backend commands (set_settings_cmd, open_settings_window, extended get_settings_cmd, tauri-plugin-dialog) wired as foundation for Plans 02-04.**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-08T08:12:09Z
- **Completed:** 2026-03-08T08:15:09Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Created src/styles/tokens.css with complete :root block (25 tokens), light theme override, and media query
- Established multi-page Vite build: settings.html + settings-main.ts entry point, vite.config.ts updated with rollupOptions
- Extended get_settings_cmd to return all 10 settings fields plus is_portable; added set_settings_cmd Tauri command
- Added open_settings_window command in lib.rs and registered all new commands in invoke_handler
- Registered tauri-plugin-dialog throughout (Cargo.toml, lib.rs plugin chain, capabilities/default.json)

## Task Commits

Each task was committed atomically:

1. **Task 1: CSS tokens, settings.html, and Vite multi-page config** - `eba9dbb` (feat)
2. **Task 2: Rust backend — set_settings_cmd, open_settings_window, extended get_settings_cmd, plugin-dialog** - `48e3dd1` (feat)

## Files Created/Modified

- `src/styles/tokens.css` - Full CSS custom properties: 25 design tokens, light theme override, dark/light media query
- `settings.html` - Second Vite entry point referencing src/settings-main.ts
- `src/settings-main.ts` - Minimal stub entry point (tokens.css import; full UI in Plan 03)
- `src/main.ts` - Added tokens.css as first import
- `vite.config.ts` - Added path import + multi-page rollupOptions.input (main + settings)
- `src-tauri/tauri.conf.json` - Settings window: decorations:false, added url:settings.html
- `src-tauri/src/store.rs` - Extended get_settings_cmd (all fields + is_portable); added set_settings_cmd
- `src-tauri/src/lib.rs` - Registered dialog plugin; added open_settings_window command + invoke_handler entries
- `src-tauri/Cargo.toml` - Added tauri-plugin-dialog = "2"
- `src-tauri/capabilities/default.json` - Added dialog:default permission

## Decisions Made

- `settings-main.ts` created as a minimal stub (only imports tokens.css) — the actual Settings Vue component is Plan 03's work; the entry point must exist for Vite to resolve the build without error.
- `tauri-plugin-dialog` added with caret range `"2"` consistent with `tauri-plugin-opener` (not pinned like store/global-shortcut/autostart, since dialog was not in the original exact-pin list).
- `open_settings_window` placed inline in `lib.rs` rather than a new module — single-function commands don't warrant a file.
- `is_portable` check in `get_settings_cmd` uses `data_dir.ends_with("data")` plus portable marker file existence — mirrors the logic in paths.rs.

## Deviations from Plan

**1. [Rule 2 - Missing Critical] Created src/settings-main.ts stub**
- **Found during:** Task 1 (settings.html creation)
- **Issue:** settings.html references `/src/settings-main.ts` but the file didn't exist; pnpm build would fail resolving the entry point
- **Fix:** Created minimal stub with tokens.css import and a comment indicating Plan 03 will fill in the full UI
- **Files modified:** src/settings-main.ts
- **Verification:** pnpm build completed without errors (built in 1.11s)
- **Committed in:** eba9dbb (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 2 - missing critical file for build correctness)
**Impact on plan:** Necessary for build correctness. No scope creep — stub is intentionally minimal.

## Issues Encountered

None — both pnpm build and cargo check passed on first attempt.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 02-04 can now proceed without further infrastructure changes
- Plan 02 (token refactor): tokens.css exists and importable from App.vue
- Plan 03 (settings UI): settings.html URL resolves, all Tauri commands registered
- Plan 04 (integration): set_settings_cmd and open_settings_window available for invoke()

---
*Phase: 08-settings-window*
*Completed: 2026-03-08*
