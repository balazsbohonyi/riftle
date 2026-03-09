# Coding Conventions (Quality Mapping)

## Scope
- This document summarizes coding conventions observed in this repository.
- Evidence is based on current source files under `src/` and `src-tauri/src/`.

## Architecture Conventions
- Clear backend/frontend boundary: OS integration stays in Rust, UI logic stays in Vue.
- Evidence: Tauri commands in `src-tauri/src/lib.rs` expose backend behavior via `invoke_handler`.
- Evidence: Frontend calls `invoke()` in `src/App.vue` and `src/Settings.vue`.
- Multi-window architecture is explicit and stable.
- Evidence: launcher window logic in `src/App.vue`; settings window logic in `src/Settings.vue`; opener command in `src-tauri/src/lib.rs` (`open_settings_window`).
- Shared runtime state uses Tauri managed state wrappers.
- Evidence: `DbState`, `SearchIndexState`, `SettingsCentered` in `src-tauri/src/db.rs`, `src-tauri/src/search.rs`, `src-tauri/src/lib.rs`.

## Rust Style and Naming
- Modules are single-responsibility and named by domain (`db`, `store`, `indexer`, `search`, `hotkey`).
- Evidence: `src-tauri/src/*.rs` layout.
- Data structs use Rust `snake_case` field names for JSON compatibility with frontend payloads.
- Evidence: `Settings` fields in `src-tauri/src/store.rs`, `SearchResult` in `src-tauri/src/search.rs`.
- Constructors/defaults are explicit, not implicit magic.
- Evidence: `impl Default for Settings` and dedicated `default_*` functions in `src-tauri/src/store.rs`.
- Internal helpers are scoped with `pub(crate)` where cross-file exposure is not needed.
- Evidence: helpers in `src-tauri/src/indexer.rs` (`get_index_paths`, `crawl_dir`, `resolve_lnk`, etc.).
- Unsafe FFI blocks are isolated around Win32 API calls.
- Evidence: `src-tauri/src/commands.rs`, `src-tauri/src/system_commands.rs`, `src-tauri/src/indexer.rs`, `src-tauri/src/lib.rs`.

## Frontend Style and Naming
- Vue components use `<script setup lang="ts">` with Composition API refs/computed/watch.
- Evidence: `src/App.vue`, `src/Settings.vue`, `src/components/ui/*.vue`.
- Component files and exported UI primitives use PascalCase naming.
- Evidence: `src/components/ui/Button.vue`, `Toggle.vue`, `KeyCapture.vue`, `PathList.vue`.
- Reactive state variables and functions use lower camelCase naming.
- Evidence: `selectedIndex`, `launchInProgress`, `updateWindowHeight` in `src/App.vue`.
- Frontend payload interfaces mirror Rust JSON fields (snake_case across IPC).
- Evidence: `SettingsData`, `SettingsResponse` in `src/Settings.vue` use `show_path`, `reindex_interval`.
- Design values are centralized in CSS tokens, then consumed by components.
- Evidence: `src/styles/tokens.css` and token usage in `src/App.vue`/`src/Settings.vue` styles.

## Error-Handling Conventions
- UI-facing Tauri commands frequently return `Result<(), String>` for IPC-friendly errors.
- Evidence: `launch`, `launch_elevated`, `update_hotkey`, `run` setup command paths.
- Non-fatal operational failures are usually logged with `eprintln!` and execution continues.
- Evidence: icon creation/index failures in `src-tauri/src/lib.rs` and `src-tauri/src/indexer.rs`.
- Fallback behavior is preferred over hard failure for user experience.
- Evidence: hotkey fallback to `Alt+Space` in `src-tauri/src/hotkey.rs`.
- Data-load failures commonly degrade to defaults (`unwrap_or_default`) rather than bubbling errors.
- Evidence: settings parsing and search index loads in `src-tauri/src/store.rs` and `src-tauri/src/search.rs`.
- Panics are still used in some environment-critical paths.
- Evidence: `expect()`/`panic!` around data-dir resolution in `src-tauri/src/paths.rs`, DB init expect in `src-tauri/src/lib.rs`.

## Concurrency and State Safety Patterns
- Shared mutable data is synchronized with `Arc<Mutex<...>>` and `Arc<RwLock<...>>`.
- Evidence: DB and search index state wrappers in `src-tauri/src/db.rs` and `src-tauri/src/search.rs`.
- Re-entrancy control uses `AtomicBool` compare/exchange guards.
- Evidence: indexing guard in `src-tauri/src/indexer.rs` (`try_start_index`, `reindex`).
- Background work is spawned to avoid blocking UI commands.
- Evidence: threads in `src-tauri/src/indexer.rs` and command execution flow in `src-tauri/src/commands.rs`.

## Observed Convention Risks
- Logging is mostly ad-hoc `println!/eprintln!`; no structured logging layer yet.
- Evidence: repeated `eprintln!` use across `src-tauri/src/*.rs`.
- Some comments contain stale phase references or implementation drift.
- Evidence: `reindex` comment in `src-tauri/src/indexer.rs` mentions defaults/future wiring while command is active.
- Frontend style consistency is good, but formatting compactness varies between files.
- Evidence: dense one-line CSS in `src/components/ui/KeyCapture.vue` vs expanded style blocks in `src/App.vue`.
