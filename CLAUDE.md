This file provides guidance to AI coding agents when working with code in this repository.

## What This Is

Riftle is a keyboard-first, minimal Windows application launcher built with Tauri v2 (Rust backend + Vue 3 TypeScript frontend). Users summon it with a global hotkey (default: Alt+Space), type to fuzzy-search installed apps, and launch with Enter.

## Commands

```sh
# Install dependencies
pnpm install

# Run in development mode (starts both Vite dev server and Tauri)
pnpm tauri dev

# Type-check frontend only (fast check without building)
pnpm build   # runs vue-tsc --noEmit && vite build

# Run Rust tests only
cd src-tauri && cargo test

# Run a specific Rust test
cd src-tauri && cargo test test_search_prefix_beats_fuzzy

# Build release installer (NSIS + MSI in src-tauri/target/release/bundle/)
pnpm tauri build
```

## Project Structure

```
riftle/
├── src/                        — Vue 3 frontend
│   ├── App.vue                 — launcher UI (search input, result list, keyboard nav, context menu)
│   ├── main.ts                 — launcher Vue app mount
│   ├── Settings.vue            — settings window UI (General, Hotkey, Search, Appearance sections)
│   ├── settings-main.ts        — settings Vue app mount (multi-page build entry)
│   ├── assets/
│   │   └── magnifier.svg       — search icon
│   ├── components/
│   │   ├── Dropdown.vue        — reusable dropdown selector
│   │   └── ui/                 — settings UI primitives
│   │       ├── Button.vue
│   │       ├── KeyCapture.vue  — hotkey capture input
│   │       ├── PathList.vue    — add/remove path list editor
│   │       ├── Row.vue         — labeled settings row
│   │       ├── Section.vue     — settings section header
│   │       └── Toggle.vue      — boolean toggle switch
│   ├── styles/
│   │   └── tokens.css          — CSS design tokens (colors, spacing, typography)
│   └── vite-env.d.ts
├── src-tauri/
│   ├── src/                    — Rust backend
│   │   ├── lib.rs              — app entry, startup sequence, plugin registration
│   │   ├── main.rs             — binary entry point
│   │   ├── paths.rs            — data directory resolution (portable vs installed)
│   │   ├── db.rs               — SQLite schema and queries
│   │   ├── store.rs            — settings persistence + Settings struct
│   │   ├── indexer.rs          — Windows app crawler + background re-index
│   │   ├── search.rs           — Nucleo fuzzy search + ranking
│   │   ├── hotkey.rs           — global shortcut registration + update_hotkey command
│   │   ├── commands.rs         — launch / launch_elevated / quit_app Tauri commands
│   │   └── system_commands.rs  — lock / shutdown / restart / sleep
│   ├── icons/                  — app icons (bundled, not runtime data)
│   │   └── system_command.png  — icon embedded into binary, copied to data dir at startup
│   ├── capabilities/
│   │   └── default.json        — Tauri permission declarations
│   ├── tauri.conf.json         — Tauri build and window configuration (launcher + settings windows)
│   └── Cargo.toml
├── docs/
│   └── DEV-SETUP.md            — Windows development environment setup guide
├── .planning/                  — GSD planning artifacts (phases, roadmap, state)
├── index.html                  — Vite HTML entry (launcher window)
├── settings.html               — Vite HTML entry (settings window)
├── vite.config.ts
├── tsconfig.json
└── package.json                — scripts: dev, build, tauri
```

## Architecture

**Single IPC boundary:** Rust handles all OS interactions (indexing, search, hotkeys, launching, SQLite); Vue handles all UI. Communication is exclusively via Tauri `invoke()` calls.

### Rust backend (`src-tauri/src/`)

| File | Role |
|---|---|
| `lib.rs` | App entry point — plugin registration, startup sequence, window DWM setup |
| `paths.rs` | Data directory resolution — portable mode (`./data/`) vs installed (`%APPDATA%\riftle-launcher\`) |
| `db.rs` | SQLite schema (via `rusqlite` bundled) — app records, launch counts |
| `store.rs` | Settings persistence via `tauri-plugin-store`; `Settings` struct with serde defaults |
| `indexer.rs` | Windows path crawler (Start Menu, Desktop, user-defined paths) + background re-index timer + `notify` file watcher |
| `search.rs` | Nucleo fuzzy search with 3-tier ranking: prefix match (tier 2) > acronym match (tier 1) > fuzzy (tier 0), then MRU tiebreak |
| `hotkey.rs` | Global shortcut registration via `tauri-plugin-global-shortcut` |
| `commands.rs` | Tauri `#[command]` handlers — `launch`, `launch_elevated` (ShellExecuteW runas) |
| `system_commands.rs` | Lock/shutdown/restart/sleep via Windows API |

### Vue frontend (`src/`)

**Multi-page build:** `index.html` → `App.vue` (launcher), `settings.html` → `Settings.vue` (settings window).

`src/App.vue` handles:
- Search input and result display using `vue-virtual-scroller` (`RecycleScroller`) for virtualized list
- Keyboard navigation (arrows, Enter, Ctrl+Shift+Enter for elevated launch, Escape)
- Window height adjustment via `getCurrentWindow().setSize()` as results change
- Auto-hide on focus loss via `onFocusChanged`
- Icon loading via `convertFileSrc()` pointing to `{data_dir}/icons/{filename}.png`
- Animation modes: `slide` (default), `fade`, `instant`
- Right-click context menu overlay with Settings and Quit actions
- `launcher-show` event listener — shows, focuses, and runs appear animation; emitted by both hotkey.rs and Settings.vue
- `settings-changed` event listener — reactively applies theme/show_path/animation changes without restart
- `Ctrl+,` shortcut to open the settings window

`src/Settings.vue` handles:
- Four sections: General (autostart), Hotkey (live rebind), Search (extra/excluded paths, re-index), Appearance (theme, show_path, animation)
- Single-instance: opened via `open_settings_window` command; subsequent opens bring to focus
- ESC hides settings window and re-shows the launcher
- Autostart toggle disabled (but visible) in portable mode

### Startup sequence (in `lib.rs`)

1. Plugins registered (store, opener, dialog, global-shortcut, autostart)
2. `paths::data_dir()` resolves data directory
3. SQLite DB initialized and managed as `DbState(Arc<Mutex<Connection>>)`
4. Settings loaded/persisted from `settings.json`
5. `indexer::run_full_index()` runs synchronously (window hidden during startup)
6. `search::ensure_system_command_icon()` copies embedded icon to data dir
7. `search::init_search_index()` loads all apps into in-memory `SearchIndexState(Arc<RwLock<SearchIndex>>)`
8. `hotkey::register()` registers global shortcut; falls back to `Alt+Space` and persists fallback if OS rejects the key
9. Background indexer + file watcher started
10. `SettingsCentered(AtomicBool)` managed state initialized (centers settings window on first open only)
11. DWM attributes set (no border, no corner rounding) — CSS owns all visuals

### Key design decisions

- **Portable mode:** detected by presence of `riftle-launcher.portable` file next to exe; data goes in `./data/` instead of `%APPDATA%`
- **Search capped at 50 results** per query
- **System commands** activated by `>` prefix (e.g., `> shutdown`)
- **Window is larger than visible UI** — CSS shadow/border-radius requires transparent overflow area; DWM border/rounding disabled
- **Icons** are extracted as PNG files to `{data_dir}/icons/` during indexing; referenced by filename in search results
- **`isTauriContext`** guard in frontend — some code paths are skipped when running `vite dev` in browser (no Tauri APIs)
- **Settings window** calls `.hide()` not `.close()` — kept alive so `open_settings_window` can re-show it without recreating state
- **`launcher-show` event** is the unified show-and-focus signal emitted by both hotkey.rs (toggle) and Settings.vue (ESC); App.vue listener runs the appear animation
- **Hotkey fallback:** `hotkey::register()` returns the actually-registered hotkey string; if the OS rejects the configured key, falls back to `Alt+Space` and persists the fallback so it is not retried on next startup
- **CSS design tokens** in `src/styles/tokens.css` — all colors, spacing, and typography are token-driven; theme switching swaps the token set

## IPC Commands Registered

```rust
// in lib.rs invoke_handler
crate::indexer::reindex              // manual re-index trigger (Search section in Settings)
crate::search::search                // fuzzy search — returns Vec<SearchResult>
crate::store::get_settings_cmd       // returns settings + is_portable flag as JSON
crate::store::set_settings_cmd       // persists full Settings struct to settings.json
crate::commands::launch              // launch app by id
crate::commands::launch_elevated     // launch app elevated (ShellExecuteW runas)
crate::system_commands::run_system_command // lock/shutdown/restart/sleep
crate::hotkey::update_hotkey         // deregister old hotkey, register new, persist to settings.json
                                     // returns Err when fallback is used so UI can update displayed key
crate::commands::quit_app            // exit(0) via AppHandle — context menu "Quit Launcher"
open_settings_window                 // show settings window; center on first open, restore position after
```

## Testing

Rust unit tests exist in `search.rs`, `store.rs`, and `paths.rs` — run with `cargo test` from `src-tauri/`. No frontend tests currently.

## Tool priority: LSP FIRST

**When navigating code, ALWAYS use LSP before grep, glob, bash, or Read-and-scan.** LSP is faster, more accurate, and cheaper on tokens. Only fall back to grep/glob for non-code text search or regex patterns. This is a hard rule, not a suggestion.

## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- If graphify-out/wiki/index.md exists, navigate it instead of reading raw files
- After modifying code files in this session, run `graphify update .` to keep the graph current (AST-only, no API cost)
