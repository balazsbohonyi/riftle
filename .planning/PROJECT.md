# Launcher — Minimal Windows App Launcher

## What This Is

A keyboard-first, minimal application launcher for Windows built with Tauri v2 (Rust + Vue 3 + TypeScript). Users summon it with a global hotkey, type to fuzzy-search installed apps, and launch with Enter — entirely without touching the mouse. Inspired by Flow Launcher but intentionally smaller in scope, with a clean architecture designed for future extensibility.

## Core Value

Sub-100ms hotkey-to-visible response time with zero mouse required — if the shortcut doesn't fire instantly and the search doesn't feel snappy, nothing else matters.

## Requirements

### Validated

- ✓ Tauri v2 project scaffolded with Vue 3 + TypeScript + Vite — existing

### Active

- [ ] Global hotkey registers and summons/dismisses the launcher window
- [ ] Frameless floating window with no taskbar entry and no tray icon
- [ ] App indexer crawls Windows paths and stores results in SQLite
- [ ] Fuzzy search over indexed apps returns ranked results
- [ ] Keyboard navigation (arrow keys, Enter to launch) works fully
- [ ] Normal app launch and elevated (Run as Admin) launch supported
- [ ] System commands (lock / shutdown / restart / sleep) via prefix
- [ ] Right-click context menu on results (run as admin, open file location)
- [ ] Settings window: hotkey config, search paths, autostart toggle
- [ ] Settings persisted via tauri-plugin-store, portable-aware

### Out of Scope

- Plugin/extension system — future milestone (listed separately below)
- Plugin/extension system — future milestone
- Non-Windows platforms — Windows-only by design
- Web search or external queries — intentional scope limit

## Context

- Stack is locked: Tauri v2, Rust backend, Vue 3 frontend, SQLite via rusqlite (bundled), pnpm, nucleo for fuzzy matching
- Design spec already in PROJECT.md at repo root — all IPC commands and module breakdown defined there
- Scaffolding only: lib.rs has the default greet command, App.vue is the default template — nothing of the real app is implemented yet
- Windows-only: uses windows-sys for ShellExecuteW, SetForegroundWindow, etc.

## Constraints

- **Performance**: Sub-100ms hotkey-to-visible — every Rust path must be fast; no blocking on the main thread
- **Platform**: Windows 10 1803+ and Windows 11 only
- **Architecture**: Single IPC boundary — Rust handles all OS interactions, Vue handles all UI. No OS calls from JS.
- **Portability**: All data paths must be portable-aware (installer vs. portable mode)
- **Package manager**: pnpm only

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri v2 over Electron | Native performance, smaller binary, Rust backend | — Pending |
| nucleo for fuzzy matching | Fast, Rust-native, better than fzf bindings | — Pending |
| rusqlite bundled | No external SQLite dependency, portable | — Pending |
| No tray icon | Minimal footprint — hotkey only | — Pending |
| SQLite over in-memory index | Persistent across restarts, fast queries | — Pending |

---
*Last updated: 2026-03-05 after initialization*
