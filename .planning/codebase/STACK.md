# STACK

## Scope
- Repository analyzed: `riftle` (Windows-first desktop launcher).
- Primary evidence sources: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src/` and `src-tauri/src/` runtime code.

## Languages
- Rust (backend/native): `src-tauri/src/*.rs`, crate config in `src-tauri/Cargo.toml`.
- TypeScript (frontend logic): `src/main.ts`, `src/settings-main.ts`, `<script setup lang="ts">` in `src/App.vue` and `src/Settings.vue`.
- Vue SFC + HTML + CSS: `src/*.vue`, entry HTML in `index.html` and `settings.html`, design tokens in `src/styles/tokens.css`.
- JSON configs/manifests: `package.json`, `tsconfig.json`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`.

## Runtime Model
- Desktop runtime is Tauri v2: `tauri = { version = "2" }` in `src-tauri/Cargo.toml`.
- Web UI runtime is WebView2-hosted Vue app via Tauri windows: launcher/settings windows in `src-tauri/tauri.conf.json`.
- Backend entrypoint: `src-tauri/src/main.rs` -> `riftle_lib::run()`.
- App lifecycle and plugin wiring: `src-tauri/src/lib.rs` (`tauri::Builder::default()` with `.plugin(...)`, `.setup(...)`, `.invoke_handler(...)`).

## Frontend Stack
- Framework: Vue 3 (`"vue": "^3.5.13"` in `package.json`).
- Build tool: Vite 6 (`"vite": "^6.0.3"`, config in `vite.config.ts`).
- Vue plugin: `@vitejs/plugin-vue` in `package.json` and `vite.config.ts`.
- Type checking: `vue-tsc` (`"build": "vue-tsc --noEmit && vite build"` in `package.json`).
- Virtualized list rendering: `vue-virtual-scroller` in `package.json`, used in `src/App.vue` (`RecycleScroller`).
- Fonts packaged via npm: `@fontsource/inter`, `@fontsource/jetbrains-mono` imported in `src/main.ts` and `src/settings-main.ts`.

## Backend Stack
- Rust edition 2021: `edition = "2021"` in `src-tauri/Cargo.toml`.
- Crate outputs include `staticlib`, `cdylib`, `rlib`: `[lib] crate-type` in `src-tauri/Cargo.toml`.
- Tauri build script: `src-tauri/build.rs` running `tauri_build::build()`.
- Tauri plugin crates in Rust manifest:
- `tauri-plugin-store`
- `tauri-plugin-opener`
- `tauri-plugin-dialog`
- `tauri-plugin-global-shortcut`
- `tauri-plugin-autostart`

## State, Storage, and Data
- Primary persistent data store: SQLite (`rusqlite` with `bundled` feature) in `src-tauri/Cargo.toml`.
- Database schema and queries: `src-tauri/src/db.rs` (`apps` table, upsert/read/increment logic).
- Connection held as managed app state: `DbState(Arc<Mutex<Connection>>)` in `src-tauri/src/db.rs` and managed in `src-tauri/src/lib.rs`.
- Settings persistence: `tauri-plugin-store` JSON file (`settings.json`) in `src-tauri/src/store.rs`.
- Data directory resolution (portable vs installed): `src-tauri/src/paths.rs`.
- Search index is in-memory (`Arc<RwLock<SearchIndex>>`): `src-tauri/src/search.rs`.

## Search/Indexing Pipeline
- File crawler: `walkdir` crate in `src-tauri/Cargo.toml`, implemented in `src-tauri/src/indexer.rs`.
- File watch/debounce: `notify` + `notify-debouncer-mini` in `src-tauri/Cargo.toml`, used in `start_background_tasks` in `indexer.rs`.
- Fuzzy matcher: `nucleo` + `nucleo-matcher` in `src-tauri/Cargo.toml`, ranking logic in `src-tauri/src/search.rs`.
- Ranking strategy: tiered prefix/acronym/fuzzy + MRU tie-break + cap 50 results in `score_and_rank` (`search.rs`).

## Packaging and Build
- JS package manager and lockfile: pnpm (`pnpm-lock.yaml`, scripts in `package.json`).
- Dev orchestration: `pnpm tauri dev` (Tauri config `beforeDevCommand: pnpm dev` in `src-tauri/tauri.conf.json`).
- Production build: `pnpm build` then Tauri bundle (`beforeBuildCommand` + `frontendDist` in `src-tauri/tauri.conf.json`).
- Multi-page frontend bundling inputs: `vite.config.ts` (`index.html`, `settings.html`).
- Installer/bundle targets: `"bundle": { "active": true, "targets": "all" }` in `src-tauri/tauri.conf.json`.

## TypeScript/Compiler Setup
- TS target and module config: `ES2020`/`ESNext` in `tsconfig.json`.
- Bundler module resolution mode: `"moduleResolution": "bundler"` in `tsconfig.json`.
- Strict checks enabled: `strict`, `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch` in `tsconfig.json`.

## Platform and OS Constraints
- Product is explicitly Windows-oriented in implementation:
- Win32 APIs via `windows-sys` and `windows` crates in `src-tauri/Cargo.toml`.
- Windows-specific modules in `indexer.rs`, `commands.rs`, `system_commands.rs`, `lib.rs` DWM handling.
- Startup docs define Windows prerequisites/tooling: `docs/DEV-SETUP.md`.

## Security/Permissions Surface (Stack-Relevant)
- Tauri capability permissions are explicitly declared in `src-tauri/capabilities/default.json`.
- App security config includes CSP and enabled asset protocol scope in `src-tauri/tauri.conf.json`.

## Testing Stack
- Rust unit tests embedded alongside modules (`db.rs`, `store.rs`, `search.rs`, `indexer.rs`, `paths.rs`).
- Test command documented in repo: `cd src-tauri && cargo test` (`AGENTS.md` and `README.md`).
- No dedicated frontend test framework is configured in `package.json` scripts.
