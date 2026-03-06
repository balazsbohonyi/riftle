# Phase 3: Indexer - Context

**Gathered:** 2026-03-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the Windows application indexer as a pure Rust backend system: crawl configured paths (Start Menu, Desktop, PATH, user-defined), resolve .lnk shortcuts to their target executables, extract app icons via ExtractIconEx and save as PNG, persist everything to SQLite, and keep the index fresh via a background timer and filesystem watcher. No Tauri commands are exposed to the frontend except `reindex()`.

</domain>

<decisions>
## Implementation Decisions

### Startup index timing
- First index runs **synchronously in setup()** — blocks until complete before app is "ready"
- Window starts hidden (Phase 1 decision), so hotkey-to-visible is unaffected by setup blocking
- **No time cap** — let the full index finish; typical Windows system indexes in under 2 seconds
- After first index completes, spawn the background timer thread at end of setup()
- Timer interval counts from end of first index — no redundant auto-index shortly after startup
- `reindex()` Tauri command (INDX-08) is **fire-and-forget** — spawns a background thread and returns immediately; frontend shows spinner/loading state

### Re-index coordination
- If a re-index is already running when a new trigger fires (timer, watcher, or manual): **skip/drop the new trigger**
- Simplest correct behavior — a missed 15-min timer tick is harmless; avoids DB lock contention
- Filesystem watcher events are **suppressed during a full re-index** — the full crawl covers those changes anyway
- After a manual `reindex()` completes, **reset the timer** so next auto-index is 15 min from then, not from original schedule

### Broken/orphan shortcut handling
- Unresolvable .lnk shortcuts (missing target, broken link, permission denied): **skip silently** — no error, no log entry; orphan shortcuts are noise on Windows
- Chained .lnk → .lnk shortcuts: **one level only** — if resolved target is also a .lnk, skip the shortcut entirely
- PATH directory crawl: **only .exe files** — no .bat/.cmd/.ps1 scripts; Start Menu .lnk files cover the apps users actually want

### Generic icon fallback
- Failed icon extractions fall back to a **bundled generic app icon PNG** compiled into the Tauri binary
- On startup, copy generic icon to `{data_dir}/icons/generic.png` **only if missing** (first run or after data dir deletion)
- `icon_path` in SQLite stores `"generic.png"` (relative filename) for apps where extraction failed
- **Extract icons for all indexed apps** regardless of source — .lnk-resolved targets and direct .exe files from PATH/Desktop

### Claude's Discretion
- Threading model for background timer and watcher (std::thread vs tokio)
- Atomic boolean or Mutex<bool> for "is_indexing" flag to coordinate trigger suppression
- Windows API approach for .lnk resolution (IShellLink COM interface via windows-sys)
- PNG conversion approach for ExtractIconEx HICON output (image crate or GDI APIs)
- Icon filename convention in {data_dir}/icons/ (e.g., using SQLite row id or a hash of the exe path)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `db::DbState(Arc<Mutex<Connection>>)` — established managed state; indexer retrieves via `app.state::<DbState>()` in the Tauri command handler
- `db::upsert_app()` — ready to call per discovered app; `get_all_apps()` useful for stale entry pruning
- `paths::data_dir()` — returns ready-to-use PathBuf; indexer uses it for `{data_dir}/icons/` directory
- `store::get_settings()` — indexer reads `additional_paths`, `excluded_paths`, and `reindex_interval` at index time
- `indexer.rs` stub exists — ready to be filled in Phase 3
- `walkdir`, `notify`, `windows-sys` already in `Cargo.toml` — no new crate additions needed for core indexing/watching

### Established Patterns
- `#[cfg(desktop)]` setup callback — background threads and watcher must be spawned inside this block
- `Arc<Mutex<Connection>>` pattern — indexer acquires the lock, runs upsert_app() calls, releases; same pattern as db.rs functions
- `Result<T, E>` propagation — indexer functions return errors to callers; don't swallow internally (Phase 2 pattern)
- Silent recovery on startup failures — consistent with Phase 2 DB/settings reset behavior

### Integration Points
- `lib.rs` setup callback: call `indexer::run_full_index()` synchronously, then `indexer::start_background_watcher()` at end of setup
- `lib.rs` invoke_handler: register `reindex` command (indexer::reindex Tauri command)
- Phase 5 (Launcher Window UI): icon_path from SQLite is a filename in `{data_dir}/icons/`; Phase 5 needs to serve icons from this path via Tauri asset protocol or IPC
- Phase 8 (Settings Window): `reindex()` command wired to "Re-index now" button; settings changes to `additional_paths`/`excluded_paths` should trigger a fresh index

</code_context>

<specifics>
## Specific Ideas

- No specific references — open to standard Rust threading and Windows API patterns

</specifics>

<deferred>
## Deferred Ideas

- None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-indexer*
*Context gathered: 2026-03-06*
