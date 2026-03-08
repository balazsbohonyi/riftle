# Requirements: Launcher — Minimal Windows App Launcher

**Defined:** 2026-03-05
**Core Value:** Sub-100ms hotkey-to-visible response time with zero mouse required

## v1 Requirements

### Scaffold

- [x] **SCAF-01**: Tauri v2 project initialised with Vue 3 + TypeScript + Vite (existing scaffold)
- [x] **SCAF-02**: Both windows declared in tauri.conf.json (launcher: frameless, skip_taskbar, always_on_top; settings: normal, hidden by default)
- [x] **SCAF-03**: All required Rust crates added to Cargo.toml (rusqlite bundled, windows-sys with required features)
- [x] **SCAF-04**: `pnpm tauri dev` starts without errors

### Data Layer

- [x] **DATA-01**: SQLite database initialised at startup with portable-aware path detection
- [x] **DATA-02**: Schema: apps table (id, name, path, icon_path, source, last_launched, launch_count)
- [x] **DATA-03**: db.rs exposes init_db(), upsert_app(), get_all_apps(), increment_launch_count()
- [x] **DATA-04**: Settings persisted via tauri-plugin-store to settings.json (portable-aware path)
- [x] **DATA-05**: Default settings: hotkey Alt+Space, theme system, opacity 1.0, show_path false, autostart false, additional_paths [], excluded_paths [], reindex_interval 15
- [x] **DATA-06**: store.rs exposes get_settings() and set_settings(patch) with typed Settings struct
- [x] **DATA-07**: Portable mode detection — launcher.portable file adjacent to exe triggers data path switch to ./data/

### Indexer

- [x] **INDX-01**: On startup and on manual reindex, crawl Start Menu (both AppData and ProgramData), Desktop (user + public), PATH directories (.exe only), and user-defined additional paths
- [x] **INDX-02**: .lnk shortcut targets resolved to actual executable paths
- [x] **INDX-03**: Excluded paths from settings are skipped; stale entries removed on each full index
- [x] **INDX-04**: App icons extracted via ExtractIconEx, saved as .png to {data_dir}/icons/{app_id}.png; falls back to generic icon
- [x] **INDX-05**: Icon extraction runs asynchronously; launcher shows placeholder until icon is ready
- [x] **INDX-06**: Background re-index on configurable interval (default 15 min)
- [x] **INDX-07**: notify crate watches Start Menu directories; incremental re-index on change, debounced 500ms
- [x] **INDX-08**: reindex() Tauri command triggers a full manual re-index on demand (exposed to frontend for Settings "Re-index now" button)

### Search

- [x] **SRCH-01**: search(query) Tauri command returns ranked Result[] using nucleo fuzzy matching
- [x] **SRCH-02**: Scoring order: exact prefix > acronym match > fuzzy substring; secondary sort by launch_count
- [x] **SRCH-03**: Maximum 50 results returned
- [x] **SRCH-04**: Query starting with > returns only system command results (prefix-based matching)
- [x] **SRCH-05**: Built-in system commands: lock, shutdown, restart, sleep — carry kind: "system" and fixed icon

### Launcher Window

- [x] **LWND-01**: Frameless floating window, centered on primary monitor, always-on-top, no taskbar entry
- [x] **LWND-02**: Fixed width 500px; height grows with result count (min: input only, max: input + 5 rows) — intentional design adjustment from original 640px/8-row spec
- [x] **LWND-03**: Search input autofocused when window appears; cleared when summoned via hotkey
- [x] **LWND-04**: ↑/↓ navigate result list (wraps at boundaries); Enter launches selected; Escape hides window
- [x] **LWND-05**: Ctrl+Shift+Enter triggers elevated launch
- [x] **LWND-06**: Window auto-hides on focus loss
- [x] **LWND-07**: Each result row: app icon (16×16 or 32×32) · app name
- [x] **LWND-08**: Selected row shows full executable path below name when show_path setting is true
- [x] **LWND-09**: [Admin] badge shown on any result row where requires_elevation is true (data-driven; Phase 6/8 populates real elevation data)
- [x] **LWND-10**: Result list virtualised for performance
- [x] **LWND-11**: Placeholder: "Search apps, or > for system commands…" when no query
- [x] **LWND-12**: System command results render with ⚙️ icon and no path line

### Launch Actions

- [x] **LAUN-01**: launch(id) opens app via ShellExecuteW with lpVerb = NULL
- [x] **LAUN-02**: launch_elevated(id) opens with lpVerb = "runas"; UAC cancellation silently absorbed
- [x] **LAUN-03**: run_system_command dispatches: lock → LockWorkStation(), shutdown → shutdown /s /t 0, restart → shutdown /r /t 0, sleep → SetSuspendState
- [x] **LAUN-04**: All launch actions hide the launcher window after execution

### Context Menu

- [x] **MENU-01**: Right-click on launcher shows custom HTML Vue overlay, absolutely positioned at cursor
- [x] **MENU-02**: v1 menu items: Settings (opens/focuses settings window) · Quit Launcher (exits process)
- [x] **MENU-03**: Menu dismisses on click-outside or Escape

### Settings Window

- [x] **SETT-01**: Separate Tauri window (label: settings), normal framed, min size 600×400px
- [x] **SETT-02**: Single-instance: open_settings_window() focuses existing window if already open
- [ ] **SETT-03**: Accessible via context menu → Settings and Ctrl+, when launcher focused
- [ ] **SETT-04**: General section: launch at startup toggle (disabled/hidden in portable mode)
- [x] **SETT-05**: Hotkey section: key-capture input to rebind; takes effect immediately
- [x] **SETT-06**: Search section: add/remove additional paths (folder picker), add/remove excluded paths, Re-index now button, re-index interval selector
- [ ] **SETT-07**: Appearance section: theme (System/Light/Dark), opacity slider (0.85–1.0), show_path toggle — all reactive on open launcher

### Global Hotkey

- [x] **HKEY-01**: Register Alt+Space (or user-configured) via tauri-plugin-global-shortcut on app start
- [x] **HKEY-02**: Hotkey toggles launcher visibility; on show: bring to front, clear input, focus
- [x] **HKEY-03**: Hotkey change in Settings: deregister old, register new immediately

### Packaging

- [ ] **PACK-01**: pnpm tauri build produces NSIS (.exe) and MSI (.msi) installers
- [ ] **PACK-02**: NSIS configured with installMode: currentUser (no admin required)
- [ ] **PACK-03**: WebView2 Evergreen Bootstrapper handled at install time
- [ ] **PACK-04**: Portable artifact: raw .exe + launcher.portable marker in a zip with README_portable.txt
- [ ] **PACK-05**: Portable build stores data in ./data/ adjacent to exe

## v2 Requirements

- Web search with configurable engines
- Inline calculator and unit converter
- Shell / PowerShell command runner
- Plugin / extension API
- Window switcher
- Browser bookmark search
- Clipboard history

## v3+ Future

- Everything by voidtools integration
- Custom theme editor
- Auto-updater (tauri-plugin-updater)
- File preview pane
- Multi-monitor placement

## Out of Scope

| Feature | Reason |
|---------|--------|
| Non-Windows platforms | Windows-only by design |
| Plugin/extension system | Future milestone |
| Web search / external queries | Out of scope for a local app launcher |
| Code signing in CI | SmartScreen warning acceptable for personal distribution |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SCAF-01 | Phase 1 | Complete |
| SCAF-02 | Phase 1 | Complete |
| SCAF-03 | Phase 1 | Complete |
| SCAF-04 | Phase 1 | Complete |
| DATA-01 | Phase 2 | Complete |
| DATA-02 | Phase 2 | Complete |
| DATA-03 | Phase 2 | Complete |
| DATA-04 | Phase 2 | Complete |
| DATA-05 | Phase 2 | Complete |
| DATA-06 | Phase 2 | Complete |
| DATA-07 | Phase 2 | Complete |
| INDX-01 | Phase 3 | Complete |
| INDX-02 | Phase 3 | Complete |
| INDX-03 | Phase 3 | Complete |
| INDX-04 | Phase 3 | Complete |
| INDX-05 | Phase 3 | Complete |
| INDX-06 | Phase 3 | Complete |
| INDX-07 | Phase 3 | Complete |
| INDX-08 | Phase 3 | Complete |
| SRCH-01 | Phase 4 | Complete |
| SRCH-02 | Phase 4 | Complete |
| SRCH-03 | Phase 4 | Complete |
| SRCH-04 | Phase 4 | Complete |
| SRCH-05 | Phase 4 | Complete |
| LWND-01 | Phase 5 | Complete |
| LWND-02 | Phase 5 | Complete |
| LWND-03 | Phase 5 | Complete |
| LWND-04 | Phase 5 | Complete |
| LWND-05 | Phase 5 | Complete |
| LWND-06 | Phase 5 | Complete |
| LWND-07 | Phase 5 | Complete |
| LWND-08 | Phase 5 | Complete |
| LWND-09 | Phase 5 | Complete |
| LWND-10 | Phase 5 | Complete |
| LWND-11 | Phase 5 | Complete |
| LWND-12 | Phase 5 | Complete |
| LAUN-01 | Phase 6 | Complete |
| LAUN-02 | Phase 6 | Complete |
| LAUN-03 | Phase 6 | Complete |
| LAUN-04 | Phase 6 | Complete |
| MENU-01 | Phase 7 | Complete |
| MENU-02 | Phase 7 | Complete |
| MENU-03 | Phase 7 | Complete |
| SETT-01 | Phase 8 | Complete |
| SETT-02 | Phase 8 | Complete |
| SETT-03 | Phase 8 | Pending |
| SETT-04 | Phase 8 | Pending |
| SETT-05 | Phase 8 | Complete |
| SETT-06 | Phase 8 | Complete |
| SETT-07 | Phase 8 | Pending |
| HKEY-01 | Phase 9 | Complete |
| HKEY-02 | Phase 9 | Complete |
| HKEY-03 | Phase 9 | Complete |
| PACK-01 | Phase 10 | Pending |
| PACK-02 | Phase 10 | Pending |
| PACK-03 | Phase 10 | Pending |
| PACK-04 | Phase 10 | Pending |
| PACK-05 | Phase 10 | Pending |

**Coverage:**
- v1 requirements: 52 total
- Mapped to phases: 52
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-05*
*Last updated: 2026-03-05 after initial definition*
