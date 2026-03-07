# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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
│   ├── App.vue                 — entire launcher UI (single component)
│   ├── main.ts                 — Vue app mount
│   ├── assets/
│   │   └── magnifier.svg       — search icon
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
│   │   ├── hotkey.rs           — global shortcut registration
│   │   ├── commands.rs         — launch / launch_elevated Tauri commands
│   │   └── system_commands.rs  — lock / shutdown / restart / sleep
│   ├── icons/                  — app icons (bundled, not runtime data)
│   │   └── system_command.png  — icon embedded into binary, copied to data dir at startup
│   ├── capabilities/
│   │   └── default.json        — Tauri permission declarations
│   ├── tauri.conf.json         — Tauri build and window configuration
│   └── Cargo.toml
├── docs/
│   └── DEV-SETUP.md            — Windows development environment setup guide
├── .planning/                  — GSD planning artifacts (phases, roadmap, state)
├── index.html                  — Vite HTML entry
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

The entire launcher UI lives in a single `src/App.vue` component (no sub-components currently). It handles:
- Search input and result display using `vue-virtual-scroller` (`RecycleScroller`) for virtualized list
- Keyboard navigation (arrows, Enter, Ctrl+Shift+Enter for elevated launch, Escape)
- Window height adjustment via `getCurrentWindow().setSize()` as results change
- Auto-hide on focus loss via `onFocusChanged`
- Icon loading via `convertFileSrc()` pointing to `{data_dir}/icons/{filename}.png`
- Animation modes: `slide` (default), `fade`, `instant`

### Startup sequence (in `lib.rs`)

1. Plugins registered (store, opener, global-shortcut, autostart)
2. `paths::data_dir()` resolves data directory
3. SQLite DB initialized and managed as `DbState(Arc<Mutex<Connection>>)`
4. Settings loaded/persisted from `settings.json`
5. `indexer::run_full_index()` runs synchronously (window hidden during startup)
6. `search::ensure_system_command_icon()` copies embedded icon to data dir
7. `search::init_search_index()` loads all apps into in-memory `SearchIndexState(Arc<RwLock<SearchIndex>>)`
8. Background indexer + file watcher started
9. DWM attributes set (no border, no corner rounding) — CSS owns all visuals

### Key design decisions

- **Portable mode:** detected by presence of `riftle-launcher.portable` file next to exe; data goes in `./data/` instead of `%APPDATA%`
- **Search capped at 50 results** per query
- **System commands** activated by `>` prefix (e.g., `> shutdown`)
- **Window is larger than visible UI** — CSS shadow/border-radius requires transparent overflow area; DWM border/rounding disabled
- **Icons** are extracted as PNG files to `{data_dir}/icons/` during indexing; referenced by filename in search results
- **`isTauriContext`** guard in frontend — some code paths are skipped when running `vite dev` in browser (no Tauri APIs)

## IPC Commands Registered

```rust
// in lib.rs invoke_handler
crate::indexer::reindex       // manual re-index trigger
crate::search::search         // fuzzy search — returns Vec<SearchResult>
crate::store::get_settings_cmd // returns settings as JSON

// planned in commands.rs (Phase 6)
launch, launch_elevated, run_system_command
```

## Testing

Rust unit tests exist in `search.rs`, `store.rs`, and `paths.rs` — run with `cargo test` from `src-tauri/`. No frontend tests currently.
