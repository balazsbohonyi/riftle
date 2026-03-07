# Phase 6: Launch Actions - Context

**Gathered:** 2026-03-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement all Tauri launch commands in Rust — `launch(id)`, `launch_elevated(id)`, `run_system_command(cmd)` — using windows-sys APIs. Wire these commands into the invoke_handler. The Vue stubs already call these commands; this phase fills the Rust implementation side only. No new UI changes.

</domain>

<decisions>
## Implementation Decisions

### Command signatures (fixed by Phase 5 stubs)
- `launch(id: String)` — opens app by looking up path from SQLite, calls ShellExecuteW with lpVerb = NULL
- `launch_elevated(id: String)` — same lookup, calls ShellExecuteW with lpVerb = "runas"
- `run_system_command(cmd: String)` — dispatches on cmd string: "lock", "shutdown", "restart", "sleep"
- All three commands take the app `AppHandle` to access DbState (for path lookup) and the launcher window (for hide)

### Window hide behavior (LAUN-04)
- **Successful launch:** Window hides before ShellExecuteW fires — faster perceived performance, launcher is out of the way immediately
- **UAC cancellation (LAUN-02 specific):** Launcher remains open with no error displayed — this is the only case where the window does NOT hide
- **Failed regular launch (ShellExecuteW error):** Window hides regardless; error is silently discarded. No popup or error feedback in the launcher. (App path broken = stale index; user will see "app didn't open" naturally.)

### Launch count tracking
- `launch()` and `launch_elevated()` both call `db::increment_launch_count(id)` after successful ShellExecuteW dispatch
- After incrementing, call `search::rebuild_index(app)` to refresh the nucleo index so MRU ranking updates immediately
- System commands do NOT increment launch counts

### requires_elevation field
- Keep `requires_elevation: false` on all SearchResult construction — no real PE manifest detection in Phase 6
- The [Admin] badge (LWND-09) is driven by this field; it will remain invisible until Phase 8 wires real detection or the user explicitly defers to a future phase
- Phase 6 only implements the launch commands, not elevation detection

### System command dispatch (LAUN-03)
- `"lock"` → `LockWorkStation()` (Windows API via windows-sys)
- `"shutdown"` → spawn `shutdown /s /t 0` subprocess (simpler + handles privilege elevation automatically)
- `"restart"` → spawn `shutdown /r /t 0` subprocess
- `"sleep"` → `SetSuspendState(FALSE, FALSE, FALSE)` (Windows API) — hibernate=false, force=false, wake_timer=false
- Window hides before dispatching all system commands (system commands are irreversible — hide first)

### Error handling
- ShellExecuteW returns HINSTANCE > 32 on success; ≤ 32 on error — check and log to stderr, but don't surface to frontend
- System command errors are also logged to stderr only
- UAC cancellation: ShellExecuteW returns ERROR_CANCELLED — detect and return Ok(()) without hiding window

### Claude's Discretion
- Exact windows-sys feature flags needed for ShellExecuteW, LockWorkStation, SetSuspendState
- Whether to use `ShellExecuteW` directly or the `Shell32` binding pattern from prior art in the codebase
- Spawn approach for shutdown/restart (std::process::Command vs CreateProcessW)
- Type casting for HWND in ShellExecuteW call (using launcher window HWND or HWND(0) for no parent)

</decisions>

<specifics>
## Specific Ideas

- UAC cancellation detection: ShellExecuteW error code 1223 = ERROR_CANCELLED — this is the specific case where launcher stays open
- All other ShellExecuteW failures = hide window + silent log (not ERROR_CANCELLED)
- Window hide before launch feels snappier — standard pattern in Flow Launcher, Wox, PowerToys Run

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `db::increment_launch_count(conn, id)` — exists in db.rs from Phase 2, ready to use
- `db::get_all_apps(conn)` — available but not needed; better to query by id for launch path lookup
- `search::rebuild_index` / `search::init_search_index` — Phase 4 established pattern for refreshing nucleo index after DB changes
- `DbState(Arc<Mutex<Connection>>)` — managed state pattern for DB access inside commands
- `SearchIndexState` — managed state for nucleo index; rebuild_index takes `&AppHandle`

### Established Patterns
- `app.state::<DbState>().0.lock().unwrap()` — DB access pattern from Phase 4 reindex command
- `app.get_webview_window("launcher").unwrap().hide()` — window hide from Phase 5 auto-hide
- `app.state::<SearchIndexState>()` — nucleo index state access pattern
- All Tauri commands use `#[tauri::command]` attribute and are registered in `lib.rs` invoke_handler
- Non-fatal errors use `eprintln!` and return `Ok(())` — established in Phase 3 background tasks

### Integration Points
- `lib.rs` invoke_handler: add `crate::commands::launch`, `crate::commands::launch_elevated`, `crate::system_commands::run_system_command`
- `commands.rs` stub: currently empty — Phase 6 fills both launch commands here
- `system_commands.rs` stub: currently empty — Phase 6 fills run_system_command here
- `windows-sys` already in Cargo.toml with Win32_Foundation features — may need Win32_UI_Shell (ShellExecuteW) and Win32_System_Shutdown (LockWorkStation, SetSuspendState) features added

</code_context>

<deferred>
## Deferred Ideas

- Real `requires_elevation` detection from PE manifest — deferred to Phase 8 or later
- Visual error feedback for failed launches (toast notification, etc.) — out of scope; launcher hides silently

</deferred>

---

*Phase: 06-launch-actions*
*Context gathered: 2026-03-07*
