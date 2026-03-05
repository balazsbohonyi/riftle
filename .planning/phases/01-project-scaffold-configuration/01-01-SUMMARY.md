---
phase: 01-project-scaffold-configuration
plan: 01
subsystem: infra
tags: [tauri, rust, cargo, rusqlite, walkdir, notify, nucleo, windows-sys, tauri-plugin-store, tauri-plugin-global-shortcut, tauri-plugin-autostart]

# Dependency graph
requires: []
provides:
  - Complete Rust dependency graph with all 12 crates declared in Cargo.toml
  - Plugin registration scaffold in lib.rs (store, opener, global-shortcut, autostart)
  - Stub module files for all planned Rust modules (db, store, hotkey, indexer, search, commands, system_commands)
  - Compilable Rust crate ready for phase-by-phase feature addition
affects: [02-data-layer, 03-indexer, 04-search-engine, 06-launch-actions, 09-global-hotkey]

# Tech tracking
tech-stack:
  added:
    - tauri-plugin-store 2.4.2
    - tauri-plugin-global-shortcut 2.3.0
    - tauri-plugin-autostart 2.5.1
    - rusqlite 0.31.x (bundled SQLite)
    - walkdir 2.x
    - notify 6.x
    - nucleo 0.5.x
    - windows-sys 0.52.x (Win32 API bindings)
  patterns:
    - "Tauri v2 desktop-only plugin pattern: global-shortcut and autostart registered inside #[cfg(desktop)] setup callback"
    - "Plugin version strategy: Tauri plugins pinned exact (e.g. 2.4.2), domain crates use caret ranges (e.g. ^0.31)"
    - "Stub module files with phase annotation comments for future implementation"

key-files:
  created:
    - src-tauri/src/db.rs
    - src-tauri/src/store.rs
    - src-tauri/src/hotkey.rs
    - src-tauri/src/indexer.rs
    - src-tauri/src/search.rs
    - src-tauri/src/commands.rs
    - src-tauri/src/system_commands.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/Cargo.lock
    - src-tauri/src/lib.rs

key-decisions:
  - "Tauri plugins pinned to exact versions (store 2.4.2, global-shortcut 2.3.0, autostart 2.5.1) for reproducible builds"
  - "Domain crates use caret ranges (^0.31, ^2, ^6, ^0.5, ^0.52, ^1) per project spec to receive patch updates"
  - "tauri-plugin-opener kept as version 2 (non-pinned) as it was the scaffold default and already working"
  - "global-shortcut and autostart registered in #[cfg(desktop)] setup callback per Tauri v2 desktop-only plugin pattern"
  - "All stub module files created in Phase 1 to prevent import conflicts in later phases"

patterns-established:
  - "Desktop-only plugins: Register inside .setup(|app| { #[cfg(desktop)] { ... } Ok(()) }) not in Builder chain"
  - "MacosLauncher::LaunchAgent is the required enum variant for tauri_plugin_autostart::init() even on Windows"
  - "invoke_handler uses empty [] until command handlers added in later phases"

requirements-completed: [SCAF-03]

# Metrics
duration: 3min
completed: 2026-03-06
---

# Phase 01 Plan 01: Rust Dependency Graph and Plugin Scaffold Summary

**All 12 Rust crates declared in Cargo.toml with correct version pins, four Tauri plugins wired in lib.rs via desktop-only pattern, and seven stub module files created — cargo check passes with zero errors.**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-05T22:41:07Z
- **Completed:** 2026-03-05T22:44:46Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Expanded Cargo.toml with all 12 crates: tauri, tauri-plugin-opener, tauri-plugin-store (2.4.2), tauri-plugin-global-shortcut (2.3.0), tauri-plugin-autostart (2.5.1), rusqlite (^0.31 bundled), walkdir, notify, nucleo, windows-sys (with Win32 features), serde, serde_json
- Replaced greet-scaffold lib.rs with real application entry point: all four plugins registered, empty invoke_handler, #[cfg(desktop)] pattern for desktop-only plugins
- Created seven stub module files (db, store, hotkey, indexer, search, commands, system_commands) annotated with their target phase
- Cargo.lock tracked by git; cargo check exits 0 with no errors

## Task Commits

Each task was committed atomically:

1. **Task 1: Expand Cargo.toml with all required crates** - `40efb9d` (chore)
2. **Task 2: Replace lib.rs and create module stubs** - `3658d04` (feat)

## Files Created/Modified

- `src-tauri/Cargo.toml` - All 12 crates declared with correct version ranges and feature flags
- `src-tauri/Cargo.lock` - Updated after dependency resolution (tracked by git)
- `src-tauri/src/lib.rs` - Plugin registration scaffold; all four plugins wired; empty invoke_handler; greet removed
- `src-tauri/src/db.rs` - Stub: Phase 2 SQLite database layer
- `src-tauri/src/store.rs` - Stub: Phase 2 settings persistence via tauri-plugin-store
- `src-tauri/src/hotkey.rs` - Stub: Phase 9 global hotkey registration
- `src-tauri/src/indexer.rs` - Stub: Phase 3 Windows application indexer
- `src-tauri/src/search.rs` - Stub: Phase 4 nucleo fuzzy search engine
- `src-tauri/src/commands.rs` - Stub: Phase 6 launch commands
- `src-tauri/src/system_commands.rs` - Stub: Phase 6 system commands via windows-sys

## Decisions Made

- Tauri plugins pinned to exact versions (store 2.4.2, global-shortcut 2.3.0, autostart 2.5.1) for reproducible builds aligned with Tauri 2.10.3 core
- Domain crates use caret ranges per project spec: ^0.31 resolves to rusqlite 0.31.0, ^6 resolves to notify 6.1.1, ^0.5 resolves to nucleo 0.5.0
- tauri-plugin-opener kept as "2" (non-exact pin) since it was the working scaffold default
- global-shortcut and autostart registered inside #[cfg(desktop)] setup callback — Tauri v2 requires this pattern for desktop-only plugins; placing them in the Builder chain causes mobile build failures
- MacosLauncher::LaunchAgent is the required constructor parameter even on Windows (it is the enum type for the init function; the underlying mechanism is platform-specific)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. cargo check resolved all dependencies cleanly on first attempt. rusqlite bundled feature compiled the embedded SQLite C code without errors.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Rust crate dependency graph is complete — Phase 2 (Data Layer) can add db.rs logic without Cargo.toml changes
- All stub module files in place — later phases add implementation to existing files, no import conflicts
- lib.rs plugin scaffold ready — Phase 9 (Global Hotkey) adds shortcut registration to hotkey.rs and wires into existing setup callback

---
*Phase: 01-project-scaffold-configuration*
*Completed: 2026-03-06*
