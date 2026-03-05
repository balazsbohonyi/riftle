# Requirements

## v1 — Current Milestone

### R01 · Project Scaffold
- Tauri v2 project initialised with Vue 3 + TypeScript + Vite via `pnpm create tauri-app .`
- All Rust crates added to `Cargo.toml` (rusqlite bundled, windows-sys with required features)
- Two windows declared in `tauri.conf.json`: `launcher` (frameless, skip_taskbar, always_on_top) and `settings` (normal, hidden by default)
- `pnpm tauri dev` starts without errors
- **Phase:** 1

### R02 · Data Layer — SQLite Schema
- SQLite database initialised at startup in the correct path (portable-aware: `data/` next to exe if `launcher.portable` marker exists, otherwise `%APPDATA%\launcher\`)
- Schema: `apps` table (id, name, path, icon_path, source, last_launched, launch_count); `settings` table is handled by tauri-plugin-store, not SQLite
- `db.rs` exposes `init_db()`, `upsert_app()`, `get_all_apps()`, `increment_launch_count()`
- **Phase:** 2

### R03 · Data Layer — Settings Persistence
- Settings persisted via `tauri-plugin-store` to `settings.json` (portable-aware path)
- Default settings: hotkey `Alt+Space`, theme `system`, opacity `1.0`, show_path `false`, autostart `false`, additional_paths `[]`, excluded_paths `[]`, reindex_interval `15` (minutes)
- `store.rs` exposes `get_settings()` and `set_settings(patch)` with typed `Settings` struct
- **Phase:** 2

### R04 · Portable Mode
- On startup, detect if `launcher.portable` file exists adjacent to the executable
- If portable: resolve all data paths relative to the exe directory (`./data/`)
- If installed: use `%APPDATA%\launcher\`
- `tauri-plugin-autostart` is silently skipped (no-op) in portable mode
- Single `app_data_dir()` helper used by both `db.rs` and `store.rs`
- **Phase:** 2

### R05 · Indexer — Windows Path Crawl
- On startup (and on manual reindex), crawl:
  - `%AppData%\Microsoft\Windows\Start Menu\Programs` (recursive)
  - `%ProgramData%\Microsoft\Windows\Start Menu\Programs` (recursive)
  - `%USERPROFILE%\Desktop`
  - `%PUBLIC%\Desktop`
  - All directories in `%PATH%` (non-recursive, `.exe` only)
  - All user-defined additional paths from settings (recursive)
- `.lnk` shortcut targets are resolved to the actual executable path
- Excluded paths from settings are skipped
- Index stored in SQLite `apps` table; stale entries removed on each full index
- **Phase:** 3

### R06 · Indexer — Icon Extraction
- App icons extracted via `ExtractIconEx` (Windows API) from the resolved executable
- Icons saved as `.png` to `{data_dir}/icons/{app_id}.png`
- Falls back to a generic app icon if extraction fails
- Icon extraction runs asynchronously after the index build; launcher shows placeholder until icon is ready
- **Phase:** 3

### R07 · Indexer — Background Re-index
- Background thread re-indexes on a configurable interval (default: 15 min, from settings)
- `notify` crate watches Start Menu directories for filesystem changes and triggers incremental re-index
- Re-index is debounced (500ms) to avoid thrashing on rapid file changes
- **Phase:** 3

### R08 · Search — Fuzzy Matching
- `search(query: string)` Tauri command returns ranked `Result[]`
- Uses `nucleo` for fuzzy matching; scoring order: exact prefix > acronym match > fuzzy substring
- Results ranked by: match score → launch frequency (MRU from `launch_count`)
- Maximum 50 results returned
- **Phase:** 4

### R09 · Search — System Commands
- If query starts with `>`, return only system command results (no app results)
- Built-in system commands: `lock`, `shutdown`, `restart`, `sleep`
- System command results carry a distinct `kind: "system"` field and a fixed icon
- Matching is prefix-based (e.g. `> sh` matches both `shutdown` and `sleep`)
- **Phase:** 4

### R10 · Launcher Window — Layout & Keyboard Nav
- Frameless floating window, centered on the primary monitor, always-on-top, no taskbar entry
- Fixed width 640px; height grows with result count (min: input only, max: input + 8 rows)
- Search input is autofocused when the window appears
- `↑` / `↓` navigate the result list; wraps at boundaries
- `Enter` launches the selected result
- `Ctrl+Shift+Enter` triggers elevated launch (see R13)
- `Escape` hides the window
- Window auto-hides when it loses focus
- **Phase:** 5

### R11 · Launcher Window — Result Rows
- Each result row displays: app icon (16×16 or 32×32) · app name
- The **currently selected row** additionally shows the full executable path below the name, indented — but only when `show_path` is `true` in settings
- When `Ctrl+Shift` is held down, a small `[Admin]` badge appears in the selected row's hint area
- Placeholder icon shown while actual icon is loading
- Result list is virtualised for performance (vue-virtual-scroller or equivalent)
- **Phase:** 5

### R12 · Launcher Window — System Command Hint
- When no query is typed, the search input placeholder reads: `Search apps, or > for system commands…`
- System command results render with a ⚙️ icon and no path line regardless of `show_path` setting
- **Phase:** 5

### R13 · Launch Actions
- `launch(id)`: opens the app using the default shell verb (`ShellExecuteW` with `lpVerb = NULL`)
- `launch_elevated(id)`: opens with `lpVerb = "runas"`; if user cancels UAC, no error is surfaced — launcher stays open
- `run_system_command(cmd)` dispatches to:
  - `lock` → `LockWorkStation()`
  - `shutdown` → `InitiateSystemShutdownEx` (or `shutdown /s /t 0`)
  - `restart` → `shutdown /r /t 0`
  - `sleep` → `SetSuspendState(FALSE, TRUE, FALSE)`
- All launch actions hide the launcher window after execution
- **Phase:** 6

### R14 · Context Menu
- Right-click anywhere on the launcher window shows a custom HTML overlay (Vue component)
- v1 menu items: **Settings** (opens/focuses settings window) · **Quit Launcher** (exits process)
- Menu dismisses on click-outside or `Escape`
- Menu is absolutely positioned relative to the cursor at the time of right-click
- **Phase:** 7

### R15 · Settings Window — Shell
- Separate Tauri window with label `settings`; normal framed window, min size 600×400px
- Single-instance: `open_settings_window()` focuses the existing window if already open
- Accessible via context menu → Settings, and via `Ctrl+,` when the launcher is focused
- **Phase:** 8

### R16 · Settings Window — Sections
- **General:** Launch at startup toggle (tauri-plugin-autostart; disabled/hidden in portable mode)
- **Hotkey:** Rebind global hotkey via a key-capture input; takes effect immediately (deregisters old, registers new)
- **Search:** Add/remove additional index paths (folder picker); add/remove excluded paths; "Re-index now" button; re-index interval selector
- **Appearance:** Theme selector (System / Light / Dark); window opacity slider (0.85–1.0); "Show path for selected result" toggle — all changes are reactive on the open launcher
- **Phase:** 8

### R17 · Global Hotkey
- Register `Alt+Space` (or user-configured hotkey) via `tauri-plugin-global-shortcut` on app start
- Hotkey toggles launcher visibility (show if hidden, hide if visible)
- When hotkey fires and window is shown: window is brought to front, search input is cleared and focused
- When hotkey is changed in Settings: old hotkey is deregistered, new one is registered immediately
- **Phase:** 9

### R18 · Packaging — Installer Builds
- `pnpm tauri build` produces both NSIS (`.exe`) and MSI (`.msi`) installers
- NSIS configured with `installMode: currentUser` (no admin rights required for install)
- MSI configured with WiX for enterprise / silent install scenarios
- WebView2 handled via Evergreen Bootstrapper (downloaded at install time)
- Code signing documented but not enforced in CI for v1 (SmartScreen warning acceptable for personal distribution)
- **Phase:** 10

### R19 · Packaging — Portable Build
- Portable artifact: the raw `.exe` from `target/release/` bundled in a zip with a `launcher.portable` marker file
- `README_portable.txt` included explaining: place both files in the same folder, run the exe, data stored in `./data/`
- Auto-start is silently disabled in portable mode (no registry entry written)
- **Phase:** 10

---

## v2 — Next Milestone (out of scope for now)

- Web search with configurable engines
- Inline calculator and unit converter
- Shell / PowerShell command runner
- Plugin / extension API
- Window switcher
- Browser bookmark search
- Clipboard history

## v3+ — Future

- Everything by voidtools integration
- Custom theme editor
- Auto-updater (tauri-plugin-updater)
- File preview pane
- Multi-monitor placement
