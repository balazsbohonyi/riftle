# Riftle Repository Structure Map

## Top-Level Layout
- `src/`: Vue launcher + settings frontend code.
- `src-tauri/`: Rust backend, Tauri config, capabilities, icons.
- `docs/`: developer documentation (`docs/DEV-SETUP.md`).
- `.planning/`: planning artifacts; mapping outputs go to `.planning/codebase/`.
- `public/`: static assets (`vite.svg`, `tauri.svg`).
- Build/config roots: `package.json`, `vite.config.ts`, `tsconfig*.json`, `index.html`, `settings.html`.

## Frontend Tree (`src/`)
- `src/main.ts`: launcher app bootstrap imports tokens/fonts and mounts `App.vue`.
- `src/settings-main.ts`: settings app bootstrap mounts `Settings.vue`.
- `src/App.vue`: launcher page (query input, result list, keyboard handling, context menu, Tauri events).
- `src/Settings.vue`: settings page (general/hotkey/search/appearance sections, persistence and event emitters).
- `src/styles/tokens.css`: design tokens and theme variable sets.
- `src/assets/magnifier.svg`: launcher search icon.
- `src/components/Dropdown.vue`: keyboard-aware custom select.
- `src/components/ui/Button.vue`: reusable button primitive.
- `src/components/ui/KeyCapture.vue`: hotkey capture control.
- `src/components/ui/PathList.vue`: add/remove folder list using dialog plugin.
- `src/components/ui/Row.vue`: settings row layout wrapper.
- `src/components/ui/Section.vue`: settings section wrapper/title.
- `src/components/ui/Toggle.vue`: boolean switch primitive.
- `src/vite-env.d.ts`: Vite typing shim.

## Backend Tree (`src-tauri/`)
- `src-tauri/src/main.rs`: binary entrypoint, delegates to library run.
- `src-tauri/src/lib.rs`: backend composition root; plugin setup, state registration, command registration.
- `src-tauri/src/paths.rs`: data directory resolution (portable marker vs `%APPDATA%`).
- `src-tauri/src/db.rs`: SQLite schema + CRUD helpers + launch count updates.
- `src-tauri/src/store.rs`: `Settings` model and settings.json persistence via plugin-store.
- `src-tauri/src/indexer.rs`: crawl/index pipelines, icon extraction, background timer/watcher, `reindex` command.
- `src-tauri/src/search.rs`: in-memory search index, ranking, system command virtual results, `search` command.
- `src-tauri/src/hotkey.rs`: global shortcut registration and `update_hotkey` command.
- `src-tauri/src/commands.rs`: app launch commands + quit command.
- `src-tauri/src/system_commands.rs`: lock/shutdown/restart/sleep execution.
- `src-tauri/tauri.conf.json`: window definitions (`launcher`, `settings`), security/csp, bundling.
- `src-tauri/capabilities/default.json`: granted Tauri permission set for both windows.
- `src-tauri/Cargo.toml`: Rust crate metadata and dependency graph.
- `src-tauri/icons/`: app bundle icons + `generic.png` and `system_command.png` source assets.
- `src-tauri/build.rs`: Tauri build script hook.

## Build and Runtime Entrypoint Files
- `index.html`: launcher web entry references `/src/main.ts`.
- `settings.html`: settings web entry references `/src/settings-main.ts`.
- `vite.config.ts`: multi-page Rollup input (`main`, `settings`) and Tauri-friendly dev server settings.
- `package.json`: scripts (`dev`, `build`, `tauri`) and frontend/Tauri JS dependencies.

## Layered Responsibility by Directory
- `src/`: view logic and interaction handling only; no direct OS calls.
- `src-tauri/src/`: all privileged OS and persistence operations.
- `src-tauri/capabilities/`: security boundary declarations for IPC/window/plugin permissions.
- `src-tauri/icons/`: binary-bundled icon assets used during runtime bootstrapping.

## Data and Artifact Locations
- SQLite database file target: `{data_dir}/launcher.db` (constructed in `lib.rs`).
- Settings store target: `{data_dir}/settings.json` (in `store.rs`).
- Extracted runtime icon cache: `{data_dir}/icons/*.png` (managed by `indexer.rs` and `search.rs`).
- Portable marker checked at executable directory: `riftle-launcher.portable` (`paths.rs`).

## Module Coupling and Import Directions
- Frontend imports Tauri API from `@tauri-apps/api/*`; backend is never imported directly by frontend.
- Backend modules are wired through `lib.rs` `mod` declarations and command registration.
- `search.rs` depends on `db.rs` data shape (`AppRecord`) and managed state.
- `commands.rs` depends on both `db.rs` and `search.rs` to update launch stats and rebuild index.
- `indexer.rs` depends on `db.rs` (upsert/prune), `store.rs` (settings inputs), and `search.rs` (rebuild after manual reindex).

## Concurrency-Related Structure
- Shared DB connection uses `Arc<Mutex<Connection>>` (`db.rs` state wrapper).
- Search cache uses `Arc<RwLock<SearchIndex>>` (`search.rs`).
- Background work threads are spawned in `indexer.rs` for timer/watcher/icon extraction.
- Single-run index protection uses `Arc<AtomicBool>` managed in `lib.rs` and consumed in `indexer.rs`.

## Tests by Location
- Backend unit tests colocated inside Rust modules:
- `db.rs` tests schema/upsert/launch count behavior.
- `paths.rs` tests portable detection branches.
- `search.rs` tests ranking/system-command behavior.
- `indexer.rs` tests crawl/prune/icon/timer guard paths.
- `store.rs` tests defaults and serde fallback behavior.
- No dedicated frontend test directories are present in `src/`.

## Structure Signals for Maintainers
- Core runtime behavior is concentrated in `src-tauri/src/lib.rs` startup sequence.
- UI windows are physically separated by entry files (`main.ts` vs `settings-main.ts`) but share tokenized styling.
- Most cross-cutting behavior is integration-through-IPC, not shared source files.
- The repository follows a clear dual-app layout: one Rust host, two Vue entry pages, one shared stateful backend.
