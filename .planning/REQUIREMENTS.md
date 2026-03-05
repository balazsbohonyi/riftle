# Requirements: Launcher — Minimal Windows App Launcher

**Defined:** 2026-03-05
**Core Value:** Sub-100ms hotkey-to-visible response time with zero mouse required

## v1 Requirements

### Scaffold

- [x] **SCAF-01**: Tauri v2 project initialised with Vue 3 + TypeScript + Vite (existing scaffold)
- [ ] **SCAF-02**: Both windows declared in tauri.conf.json (launcher: frameless, skip_taskbar, always_on_top; settings: normal, hidden by default)
- [ ] **SCAF-03**: All required Rust crates added to Cargo.toml (rusqlite bundled, windows-sys with required features)
- [ ] **SCAF-04**: `pnpm tauri dev` starts without errors

### Data Layer

- [ ] **DATA-01**: SQLite database initialised at startup with portable-aware path detection
- [ ] **DATA-02**: Schema: apps table (id, name, path, icon_path, source, last_launched, launch_count)
- [ ] **DATA-03**: db.rs exposes init_db(), upsert_app(), get_all_apps(), increment_launch_count()
- [ ] **DATA-04**: Settings persisted via tauri-plugin-store to settings.json (portable-aware path)
- [ ] **DATA-05**: Default settings: hotkey Alt+Space, theme system, opacity 1.0, show_path false, autostart false, additional_paths [], excluded_paths [], reindex_interval 15
- [ ] **DATA-06**: store.rs exposes get_settings() and set_settings(patch) with typed Settings struct
- [ ] **DATA-07**: Portable mode detection — launcher.portable file adjacent to exe triggers data path switch to ./data/

### Indexer

- [ ] **INDX-01**: On startup and on manual reindex, crawl Start Menu (both AppData and ProgramData), Desktop (user + public), PATH directories (.exe only), and user-defined additional paths
- [ ] **INDX-02**: .lnk shortcut targets resolved to actual executable paths
- [ ] **INDX-03**: Excluded paths from settings are skipped; stale entries removed on each full index
- [ ] **INDX-04**: App icons extracted via ExtractIconEx, saved as .png to {data_dir}/icons/{app_id}.png; falls back to generic icon
- [ ] **INDX-05**: Icon extraction runs asynchronously; launcher shows placeholder until icon is ready
- [ ] **INDX-06**: Background re-index on configurable interval (default 15 min)
- [ ] **INDX-07**: notify crate watches Start Menu directories; incremental re-index on change, debounced 500ms
- [ ] **INDX-08**: reindex() Tauri command triggers a full manual re-index on demand (exposed to frontend for Settings "Re-index now" button)

### Search

- [ ] **SRCH-01**: search(query) Tauri command returns ranked Result[] using nucleo fuzzy matching
- [ ] **SRCH-02**: Scoring order: exact prefix > acronym match > fuzzy substring; secondary sort by launch_count
- [ ] **SRCH-03**: Maximum 50 results returned
- [ ] **SRCH-04**: Query starting with > returns only system command results (prefix-based matching)
- [ ] **SRCH-05**: Built-in system commands: lock, shutdown, restart, sleep — carry kind: "system" and fixed icon

### Launcher Window

- [ ] **LWND-01**: Frameless floating window, centered on primary monitor, always-on-top, no taskbar entry
- [ ] **LWND-02**: Fixed width 640px; height grows with result count (min: input only, max: input + 8 rows)
- [ ] **LWND-03**: Search input autofocused when window appears; cleared when summoned via hotkey
- [ ] **LWND-04**: ↑/↓ navigate result list (wraps at boundaries); Enter launches selected; Escape hides window
- [ ] **LWND-05**: Ctrl+Shift+Enter triggers elevated launch
- [ ] **LWND-06**: Window auto-hides on focus loss
- [ ] **LWND-07**: Each result row: app icon (16×16 or 32×32) · app name
- [ ] **LWND-08**: Selected row shows full executable path below name when show_path setting is true
- [ ] **LWND-09**: [Admin] badge in selected row hint area when Ctrl+Shift is held
- [ ] **LWND-10**: Result list virtualised for performance
- [ ] **LWND-11**: Placeholder: "Search apps, or > for system commands…" when no query
- [ ] **LWND-12**: System command results render with ⚙️ icon and no path line

### Launch Actions

- [ ] **LAUN-01**: launch(id) opens app via ShellExecuteW with lpVerb = NULL
- [ ] **LAUN-02**: launch_elevated(id) opens with lpVerb = "runas"; UAC cancellation silently absorbed
- [ ] **LAUN-03**: run_system_command dispatches: lock → LockWorkStation(), shutdown → shutdown /s /t 0, restart → shutdown /r /t 0, sleep → SetSuspendState
- [ ] **LAUN-04**: All launch actions hide the launcher window after execution

### Context Menu

- [ ] **MENU-01**: Right-click on launcher shows custom HTML Vue overlay, absolutely positioned at cursor
- [ ] **MENU-02**: v1 menu items: Settings (opens/focuses settings window) · Quit Launcher (exits process)
- [ ] **MENU-03**: Menu dismisses on click-outside or Escape

### Settings Window

- [ ] **SETT-01**: Separate Tauri window (label: settings), normal framed, min size 600×400px
- [ ] **SETT-02**: Single-instance: open_settings_window() focuses existing window if already open
- [ ] **SETT-03**: Accessible via context menu → Settings and Ctrl+, when launcher focused
- [ ] **SETT-04**: General section: launch at startup toggle (disabled/hidden in portable mode)
- [ ] **SETT-05**: Hotkey section: key-capture input to rebind; takes effect immediately
- [ ] **SETT-06**: Search section: add/remove additional paths (folder picker), add/remove excluded paths, Re-index now button, re-index interval selector
- [ ] **SETT-07**: Appearance section: theme (System/Light/Dark), opacity slider (0.85–1.0), show_path toggle — all reactive on open launcher

### Global Hotkey

- [ ] **HKEY-01**: Register Alt+Space (or user-configured) via tauri-plugin-global-shortcut on app start
- [ ] **HKEY-02**: Hotkey toggles launcher visibility; on show: bring to front, clear input, focus
- [ ] **HKEY-03**: Hotkey change in Settings: deregister old, register new immediately

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
| SCAF-02 | Phase 1 | Pending |
| SCAF-03 | Phase 1 | Pending |
| SCAF-04 | Phase 1 | Pending |
| DATA-01 | Phase 2 | Pending |
| DATA-02 | Phase 2 | Pending |
| DATA-03 | Phase 2 | Pending |
| DATA-04 | Phase 2 | Pending |
| DATA-05 | Phase 2 | Pending |
| DATA-06 | Phase 2 | Pending |
| DATA-07 | Phase 2 | Pending |
| INDX-01 | Phase 3 | Pending |
| INDX-02 | Phase 3 | Pending |
| INDX-03 | Phase 3 | Pending |
| INDX-04 | Phase 3 | Pending |
| INDX-05 | Phase 3 | Pending |
| INDX-06 | Phase 3 | Pending |
| INDX-07 | Phase 3 | Pending |
| INDX-08 | Phase 3 | Pending |
| SRCH-01 | Phase 4 | Pending |
| SRCH-02 | Phase 4 | Pending |
| SRCH-03 | Phase 4 | Pending |
| SRCH-04 | Phase 4 | Pending |
| SRCH-05 | Phase 4 | Pending |
| LWND-01 | Phase 5 | Pending |
| LWND-02 | Phase 5 | Pending |
| LWND-03 | Phase 5 | Pending |
| LWND-04 | Phase 5 | Pending |
| LWND-05 | Phase 5 | Pending |
| LWND-06 | Phase 5 | Pending |
| LWND-07 | Phase 5 | Pending |
| LWND-08 | Phase 5 | Pending |
| LWND-09 | Phase 5 | Pending |
| LWND-10 | Phase 5 | Pending |
| LWND-11 | Phase 5 | Pending |
| LWND-12 | Phase 5 | Pending |
| LAUN-01 | Phase 6 | Pending |
| LAUN-02 | Phase 6 | Pending |
| LAUN-03 | Phase 6 | Pending |
| LAUN-04 | Phase 6 | Pending |
| MENU-01 | Phase 7 | Pending |
| MENU-02 | Phase 7 | Pending |
| MENU-03 | Phase 7 | Pending |
| SETT-01 | Phase 8 | Pending |
| SETT-02 | Phase 8 | Pending |
| SETT-03 | Phase 8 | Pending |
| SETT-04 | Phase 8 | Pending |
| SETT-05 | Phase 8 | Pending |
| SETT-06 | Phase 8 | Pending |
| SETT-07 | Phase 8 | Pending |
| HKEY-01 | Phase 9 | Pending |
| HKEY-02 | Phase 9 | Pending |
| HKEY-03 | Phase 9 | Pending |
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
