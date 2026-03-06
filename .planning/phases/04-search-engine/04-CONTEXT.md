# Phase 4: Search Engine - Context

**Gathered:** 2026-03-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement the `search(query)` Tauri command: nucleo fuzzy matching with MRU-weighted ranking, and `>` prefix routing to system command results. Pure Rust backend — no UI. The result struct returned here is the contract Phase 5 (Launcher Window UI) will consume.

</domain>

<decisions>
## Implementation Decisions

### Empty Query Behavior
- `search("")` returns an **empty list** — nothing shown until user types
- `search(">")` returns **all 4 system commands** immediately (discoverable without needing to type further)
- Any query starting with `>` triggers system command mode — `"> lo"`, `">lo"`, `"> shutdown"` all work (forgiving, no strict syntax)
- System commands **never appear in normal app results** — `>` prefix is the only way to reach them; no mixing

### System Command Icons
- One **shared bundled PNG asset** for all 4 system commands (lock, shutdown, restart, sleep)
- Compiled into the binary with `include_bytes!` (same pattern as generic app icon in Phase 3)
- Copied to `{data_dir}/icons/system_command.png` at startup if missing
- `icon_path` in system command results is `"system_command.png"` (relative filename, same convention as app icons)

### Result Struct Shape
- Fields: `{ id: String, name: String, icon_path: String, path: String, kind: String }`
- `path` is **always included** in every result — Phase 5 decides whether to display it based on `show_path` setting; search() doesn't read UI settings
- System command results have `path: ""` (empty string) and `kind: "system"`; app results have `kind: "app"`
- No `score` field — ranking is an internal detail; Phase 5 just renders the ordered list

### Nucleo Index Lifecycle
- Build once at startup from DB, stored as **Tauri managed state** (`Arc<RwLock<SearchIndex>>`)
- Each `search()` call queries the cached index — no DB hit per keystroke
- After `reindex()` completes, nucleo index **rebuilds asynchronously** in the background
- Race condition handling: `Arc<RwLock<...>>` — writers swap in the new index atomically; readers always get a consistent (possibly briefly stale) snapshot; never returns empty during rebuild

### Claude's Discretion
- Exact nucleo API usage (matcher vs. Nucleo struct, threading model)
- MRU weighting formula — requirements say "secondary sort by launch_count" after fuzzy score tiers
- Acronym match detection logic
- Icon PNG design for system_command.png

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `db::get_all_apps()` — returns `Vec<AppRecord>` with all fields needed to populate nucleo; called during index build
- `db::DbState(Arc<Mutex<Connection>>)` — retrieve in Tauri command handler via `app.state::<DbState>()`
- `paths::data_dir()` — used for `{data_dir}/icons/system_command.png` copy-on-startup
- `search.rs` stub — empty, ready to fill
- `system_commands.rs` stub — Phase 6 fills runtime dispatch; system command *definitions* (id, name, icon_path) live in `search.rs` or a constants block within it
- Generic icon pattern from Phase 3: `include_bytes!("../icons/generic.png")` copied to icons/ at startup — replicate for `system_command.png`

### Established Patterns
- Managed state via `app.manage()` in setup, `app.state::<T>()` in commands — use for `SearchIndex`
- `Arc<Mutex<...>>` for DB state; `Arc<RwLock<...>>` for search index (readers don't need exclusive lock)
- Silent non-fatal failures — if index build fails (empty DB), return empty results without crashing
- `#[cfg(desktop)]` setup callback — index build and background tasks spawned here

### Integration Points
- `lib.rs` setup: call `search::init_search_index(&app)` after indexer completes first full index
- `lib.rs` invoke_handler: register `search` Tauri command
- Phase 3 `reindex()` command: after re-index completes, must trigger `search::rebuild_index(&app)`
- Phase 5 (Launcher Window UI): consumes `Result { id, name, icon_path, path, kind }` — this struct is the contract
- Phase 6 (Launch Actions): uses `id` and `kind` from result to dispatch `launch()` or `run_system_command()`

</code_context>

<specifics>
## Specific Ideas

- No specific UI references — open to standard Rust patterns for nucleo integration
- Performance goal: sub-100ms from keypress to result list update — cached index is the key enabler

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 04-search-engine*
*Context gathered: 2026-03-06*
