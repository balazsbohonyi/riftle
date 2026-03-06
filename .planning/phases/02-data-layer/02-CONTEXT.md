# Phase 2: Data Layer - Context

**Gathered:** 2026-03-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement SQLite schema, settings persistence via tauri-plugin-store, and portable-mode path detection. Pure Rust backend — no Tauri commands exposed to the frontend yet. This is the foundation every other module (indexer, search, settings window) will build on.

</domain>

<decisions>
## Implementation Decisions

### Error Recovery
- Corrupted SQLite file at startup → silent reset: delete and recreate a fresh database. Launch history and frequency counts are lost, but the app starts normally.
- Unparseable or malformed settings.json → silent reset to defaults: overwrite with the default Settings struct. User loses customizations but app starts normally. Consistent behavior between DB and settings recovery.
- Runtime DB errors (e.g., upsert_app fails mid-index) → return `Result<T, rusqlite::Error>`, let callers decide. A failed upsert logs a warning but doesn't crash the app. db.rs functions never swallow errors internally.

### DB Init Timing
- `init_db()` runs synchronously in the Tauri `setup` callback (blocking). For an empty/small SQLite file, this is <5ms. Simpler — all later modules can assume the DB is ready immediately after setup.
- DB connection shared via Tauri managed state: `Arc<Mutex<Connection>>`. Stored in app state after `init_db()`, retrieved by indexer, search, and commands via `app.state::<DbState>()`. Standard Tauri v2 pattern.
- db.rs unit tests using `Connection::open_in_memory()` in Phase 2 — DATA-03 requires this. Tests cover init_db(), upsert_app(), get_all_apps(), increment_launch_count().

### Settings Persistence
- `set_settings()` accepts the **full Settings struct** every time — no partial JSON patch. Frontend (Phase 8 Settings Window) always sends the complete struct. No merge logic needed in store.rs.
- `get_settings()` and `set_settings()` are **internal Rust functions only** in Phase 2. Tauri IPC commands that expose settings to the frontend are deferred to Phase 8 (or Phase 5 as needed).
- Settings struct uses `#[serde(default)]` on every field for forward compatibility. Missing fields in settings.json auto-fill with defaults — old settings files survive schema additions without triggering a reset.

### Portable Path Detection
- Always use `std::env::current_exe()` to locate the `launcher.portable` marker file. Same code path in dev and release. In dev, the binary is in `target/debug/` — place `launcher.portable` there to test portable mode.
- Portable path detection and data_dir resolution live in a **separate `paths.rs` module**. Both db.rs and store.rs import `paths::data_dir()` — avoids duplication, makes detection testable independently.
- `paths::data_dir()` returns a `PathBuf` AND calls `std::fs::create_dir_all()` to ensure the directory exists. Callers always get a ready-to-use path without needing to mkdir themselves.

### Claude's Discretion
- Exact type aliases or newtype wrappers for AppId / app record structs
- Logging approach (eprintln vs tracing vs log crate) for error/warning output
- Whether paths.rs uses dirs-rs crate or manual env var lookup for %APPDATA%

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `tauri-plugin-store` already registered in `lib.rs` — store.rs just needs to call `app.store()` to get a handle
- `serde` + `serde_json` already in `Cargo.toml` — Settings struct can derive Serialize/Deserialize immediately
- `rusqlite` with `bundled` feature already in `Cargo.toml`
- All seven module stubs exist — db.rs and store.rs are empty placeholders ready to be filled

### Established Patterns
- Phase 1 used `#[cfg(desktop)]` setup callback for plugin registration — db init should follow the same pattern (run in setup callback)
- Tauri managed state pattern: `app.manage(...)` in setup, `app.state::<T>()` in commands — use this for DbState

### Integration Points
- `lib.rs` `setup` callback is where `init_db()` and initial `get_settings()` should be called
- Phase 3 (Indexer) and Phase 4 (Search) will call `db.rs` functions via the managed DbState
- Phase 8 (Settings Window) will call `store.rs` functions, probably via Tauri commands added in that phase
- A new `paths.rs` module needs to be added to `lib.rs` mod declarations alongside the existing stubs

</code_context>

<specifics>
## Specific Ideas

- No specific references — open to standard Tauri v2 managed state patterns

</specifics>

<deferred>
## Deferred Ideas

- None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-data-layer*
*Context gathered: 2026-03-06*
