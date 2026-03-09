# Riftle Architecture Map

## System Shape
- Riftle is a desktop app with a strict split: Vue 3 UI in `src/` and Rust/Tauri backend in `src-tauri/src/`.
- OS-facing behavior lives in Rust commands and services (`launch`, hotkeys, indexing, system power actions).
- UI-facing behavior lives in Vue components and pages (`App.vue`, `Settings.vue`).
- IPC boundary is Tauri `invoke()`/`#[tauri::command]` plus event bus (`listen`, `emitTo`).

## Primary Layers
- Presentation layer: launcher window in `src/App.vue`, settings window in `src/Settings.vue`.
- UI composition layer: reusable controls in `src/components/` and `src/components/ui/`.
- Application/service layer (Rust): `src-tauri/src/indexer.rs`, `search.rs`, `hotkey.rs`, `commands.rs`, `system_commands.rs`.
- Persistence layer: `src-tauri/src/db.rs` (SQLite), `store.rs` (settings JSON via plugin-store).
- Platform abstraction layer: Windows API integration in `indexer.rs`, `commands.rs`, `system_commands.rs`, and DWM setup in `lib.rs`.

## Entry Points
- Frontend launcher entry: `index.html` -> `<script src="/src/main.ts">` -> `src/main.ts` -> `src/App.vue`.
- Frontend settings entry: `settings.html` -> `<script src="/src/settings-main.ts">` -> `src/settings-main.ts` -> `src/Settings.vue`.
- Backend process entry: `src-tauri/src/main.rs` calls `riftle_lib::run()`.
- Backend app boot orchestration: `src-tauri/src/lib.rs::run()`.

## Boot Sequence (Backend)
- `lib.rs` registers plugins: store, opener, dialog, global-shortcut, autostart.
- `paths::data_dir()` resolves portable vs installed data root (`paths.rs`).
- `db::init_db()` opens/creates `launcher.db` and applies schema (`db.rs`).
- `store::get_settings()` + `set_settings()` ensures `settings.json` exists (`store.rs`).
- `indexer::run_full_index()` performs synchronous initial crawl (`indexer.rs`).
- `search::ensure_system_command_icon()` writes bundled icon if missing (`search.rs`).
- `search::init_search_index()` loads DB apps into in-memory index (`search.rs`).
- `hotkey::register()` binds global shortcut and may fallback to `Alt+Space` (`hotkey.rs`).
- `indexer::start_background_tasks()` starts timer + filesystem watch threads.

## State Model
- Shared DB state: `DbState(pub Arc<Mutex<Connection>>)` in `db.rs`.
- Search cache state: `SearchIndexState(pub Arc<RwLock<SearchIndex>>)` in `search.rs`.
- Startup/session UI state: `SettingsCentered(AtomicBool)` in `lib.rs`.
- Background indexing guard: `Arc<AtomicBool>` managed in `lib.rs` and used in `indexer.rs`.

## IPC Surface (Rust Commands)
- Registered in `lib.rs` `invoke_handler(...)`.
- Index/search: `indexer::reindex`, `search::search`.
- Settings: `store::get_settings_cmd`, `store::set_settings_cmd`.
- Launch: `commands::launch`, `commands::launch_elevated`, `commands::quit_app`.
- System actions: `system_commands::run_system_command`.
- Hotkey mutation: `hotkey::update_hotkey`.
- Window management: `open_settings_window`.

## Runtime Data Flow
- Query flow: `App.vue` watches `query` -> `invoke('search')` -> `search.rs` ranks results -> UI renders virtualized list.
- Launch flow: `App.vue` `invoke('launch'|'launch_elevated')` -> `commands.rs` ShellExecuteW -> `db::increment_launch_count` -> `search::rebuild_index`.
- System command flow: query prefix `>` -> `search.rs` returns synthetic `kind: "system"` rows -> `invoke('run_system_command')`.
- Settings flow: `Settings.vue` loads via `get_settings_cmd`, persists via `set_settings_cmd`, rebinding via `update_hotkey`.
- Reindex flow: `Settings.vue` path/reindex actions -> `invoke('reindex')` -> `indexer::run_full_index` -> `search::rebuild_index`.

## Event Flow (Non-Invoke)
- Hotkey press in `hotkey.rs` emits `launcher-show` to window.
- `App.vue` listens for `launcher-show` and resets query/results, focuses input, triggers animation.
- `Settings.vue` emits `launcher-show` on close (ESC/X), then hides settings window.
- `Settings.vue` emits `settings-changed`; `App.vue` listens and applies theme/show_path updates.

## Search/Ranking Pattern
- Matching engine in `search.rs`: `nucleo-matcher` fuzzy scoring.
- Tiered ranking strategy: prefix tier (2) > acronym tier (1) > fuzzy tier (0).
- Tie-break by score, then `launch_count` (MRU-like bias).
- Hard cap: 50 results in `score_and_rank()`.

## Indexing Pattern
- Source discovery in `indexer.rs::get_index_paths()` from Start Menu/Desktop + `settings.additional_paths`.
- Crawl pattern in `crawl_dir()`: `.lnk` resolves via COM `IShellLinkW`, `.exe` parsed for file description.
- De-dup key: lowercased executable path used as `AppRecord.id`.
- Icon extraction runs async per app; generic placeholder is available immediately.
- Stale row cleanup via `prune_stale()` using discovered ID set.

## Platform/Boundary Decisions
- Windows-only integrations are explicit (`windows`/`windows-sys` usage in backend modules).
- DWM border/corner overrides in `lib.rs` keep OS chrome disabled; CSS controls visual shell.
- Launcher auto-hide on focus loss is frontend-driven (`App.vue` `onFocusChanged`).
- Capability permissions declared in `src-tauri/capabilities/default.json` for both windows.

## Architectural Patterns Observed
- Command pattern for frontend->backend actions via `#[tauri::command]`.
- Service-module pattern in Rust: each concern isolated by file (`search`, `indexer`, `hotkey`, etc.).
- State container pattern via Tauri managed state (`app.manage(...)`).
- Event-driven UI updates (`launcher-show`, `settings-changed`) alongside request/response IPC.
- Multi-window composition pattern with shared backend and separate frontend entries.
