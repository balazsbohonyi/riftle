# INTEGRATIONS

## Scope
- This document lists external integrations and system touchpoints implemented in code.
- Evidence comes from concrete calls/imports in `src-tauri/src/*`, `src/*`, and Tauri config/capabilities files.

## Tauri IPC Boundary (Frontend <-> Backend)
- Frontend invokes Rust commands using `invoke` from `@tauri-apps/api/core`: `src/App.vue`, `src/Settings.vue`.
- Registered backend commands are defined centrally in `src-tauri/src/lib.rs` via `tauri::generate_handler![...]`.
- Actively used IPC commands include:
- `search` (`src-tauri/src/search.rs`), called from query watcher in `src/App.vue`.
- `launch` / `launch_elevated` (`src-tauri/src/commands.rs`), called in `launchItem`/`launchElevated` in `src/App.vue`.
- `run_system_command` (`src-tauri/src/system_commands.rs`), called for `kind === 'system'` in `src/App.vue`.
- `get_settings_cmd` / `set_settings_cmd` (`src-tauri/src/store.rs`), called in `src/App.vue` and `src/Settings.vue`.
- `update_hotkey` (`src-tauri/src/hotkey.rs`), called in `src/Settings.vue`.
- `reindex` (`src-tauri/src/indexer.rs`), called in `src/Settings.vue`.
- `open_settings_window` + `quit_app` (`src-tauri/src/lib.rs`, `src-tauri/src/commands.rs`), called in launcher context menu (`src/App.vue`).

## Tauri Plugins (Rust + JS)
- Store plugin:
- Rust registration: `tauri_plugin_store::Builder::new().build()` in `src-tauri/src/lib.rs`.
- JS-side usage indirect via settings commands; direct store access in Rust `src-tauri/src/store.rs` (`StoreExt`).
- Global shortcut plugin:
- Rust registration: `tauri_plugin_global_shortcut::Builder` in `src-tauri/src/lib.rs`.
- Runtime usage: `app.global_shortcut().on_shortcut(...)` and `.unregister(...)` in `src-tauri/src/hotkey.rs`.
- Autostart plugin:
- Rust init: `tauri_plugin_autostart::init(...)` in `src-tauri/src/lib.rs`.
- Frontend API use: dynamic import of `@tauri-apps/plugin-autostart` (`isEnabled/enable/disable`) in `src/Settings.vue`.
- Dialog plugin:
- Rust registration: `tauri_plugin_dialog::init()` in `src-tauri/src/lib.rs`.
- Opener plugin:
- Rust registration: `tauri_plugin_opener::init()` in `src-tauri/src/lib.rs`.
- Capability permissions for these plugins are declared in `src-tauri/capabilities/default.json` (`store`, `global-shortcut`, `autostart`, `dialog`, `opener`).

## Window/Event Integrations
- Window APIs from `@tauri-apps/api/window` used in `src/App.vue` and `src/Settings.vue` (`show`, `hide`, `setFocus`, `setSize`, `center`, focus listener).
- Event bus integration with `@tauri-apps/api/event`:
- `listen('launcher-show')` and `listen('settings-changed')` in `src/App.vue`.
- `emitTo('launcher', 'launcher-show')` and `emitTo('launcher', 'settings-changed', ...)` in `src/Settings.vue`.
- Rust emits launcher event via Tauri emitter in hotkey handler: `win_clone.emit("launcher-show", ())` in `src-tauri/src/hotkey.rs`.

## OS and Win32 Integrations
- Executable launch integration:
- `ShellExecuteW` default open in `src-tauri/src/commands.rs::launch`.
- `ShellExecuteW` with `runas` for elevation in `src-tauri/src/commands.rs::launch_elevated`.
- UAC cancellation handling via `GetLastError` in `launch_elevated`.
- System power/session controls:
- `LockWorkStation` and `SetSuspendState` in `src-tauri/src/system_commands.rs`.
- Shutdown/restart through shell command `shutdown /s` and `shutdown /r` in `system_commands.rs`.
- Window Manager integration:
- DWM attribute calls (`DwmSetWindowAttribute`) to disable border/rounding in `src-tauri/src/lib.rs`.

## Windows Shell/COM Integrations
- `.lnk` shortcut resolution uses COM shell interfaces in `src-tauri/src/indexer.rs::resolve_lnk`:
- `CoInitializeEx`, `CoCreateInstance`, `IPersistFile`, `IShellLinkW::GetPath`.
- Start Menu/Desktop crawling uses Windows environment paths in `src-tauri/src/indexer.rs::get_index_paths` (`APPDATA`, `PROGRAMDATA`, `USERPROFILE`, public desktop).
- File metadata reads via version-info APIs in `indexer.rs::get_file_description` (`GetFileVersionInfoSizeW`, `GetFileVersionInfoW`, `VerQueryValueW`).

## Icon and Graphics Integrations
- Icon extraction from executables uses Win32 shell + GDI in `src-tauri/src/indexer.rs::extract_icon_png`:
- `ExtractIconExW`, `GetIconInfo`, `GetDIBits`, `CreateCompatibleDC`, `DeleteObject`, `DestroyIcon`.
- PNG encoding uses Rust `image` crate (`image::RgbaImage::write_to`) in `extract_icon_png`.
- Frontend icon rendering uses Tauri asset conversion `convertFileSrc` in `src/App.vue` to load icons from `{data_dir}/icons/*`.

## Filesystem and Watcher Integrations
- Recursive filesystem scan via `walkdir` in `src-tauri/src/indexer.rs::crawl_dir`.
- Background file watching via `notify-debouncer-mini` + `notify` in `src-tauri/src/indexer.rs::start_background_tasks`.
- Debounced watcher currently targets Start Menu directories and triggers reindex attempts.
- Persistent icon assets written to runtime data dir via `ensure_generic_icon` and `ensure_system_command_icon` (`indexer.rs`, `search.rs`).

## Data and Persistence Integrations
- SQLite database file `launcher.db` created in app data directory in `src-tauri/src/lib.rs`.
- Schema and mutations live in `src-tauri/src/db.rs` (`apps` table, upsert, read, launch count updates).
- Settings persisted to `settings.json` using Tauri Store plugin in `src-tauri/src/store.rs`.
- Data directory mode integration (portable marker file `riftle-launcher.portable`) in `src-tauri/src/paths.rs`.

## Search Engine Integration
- Fuzzy search implementation integrates `nucleo-matcher` in `src-tauri/src/search.rs`.
- In-memory index refresh integrates DB reads (`get_all_apps`) with app-managed state (`SearchIndexState`) in `search.rs`.
- Special command namespace integration for system actions using `>` prefix in `search.rs::search` and `search_system_commands`.

## Frontend Dependency/Build Integrations
- Multi-page build integration for launcher/settings windows in `vite.config.ts` (`rollupOptions.input.main/settings`).
- Tauri build pipeline integration in `src-tauri/tauri.conf.json`:
- `beforeDevCommand: pnpm dev` and `devUrl` for dev server.
- `beforeBuildCommand: pnpm build` and `frontendDist` for production bundle handoff.
- TS + Vue type tooling integrated via `vue-tsc` and strict `tsconfig.json` compiler rules.

## Security/Capability Integrations
- Tauri CSP and asset protocol integration configured in `src-tauri/tauri.conf.json` (`security.csp`, `assetProtocol.enable/scope`).
- Capability grants scoped to `launcher` and `settings` windows in `src-tauri/capabilities/default.json`.
- Window privilege surface includes show/hide/set-size/start-dragging/set-focus/center permissions declared in capabilities file.

## Packaging/Distribution Integrations
- Bundle targets configured as `all` in `src-tauri/tauri.conf.json` with icon set (`.png`, `.icns`, `.ico`).
- README and setup docs reference NSIS/MSI outputs and WiX dependency for MSI (`README.md`, `docs/DEV-SETUP.md`).

## Integration Notes
- Integration style is intentionally thin frontend/native bridge: UI in Vue, OS interactions in Rust commands.
- Most critical external dependencies are Windows APIs and Tauri plugins; no cloud/SaaS network API integration is present in repository code.
