# Phase 6: Launch Actions - Research

**Researched:** 2026-03-07
**Domain:** Windows API (ShellExecuteW, LockWorkStation, SetSuspendState) via windows-sys 0.52 in Rust/Tauri
**Confidence:** HIGH

## Summary

Phase 6 is a pure Rust backend phase — no Vue changes needed. The two stub files (`commands.rs` and `system_commands.rs`) are currently empty comment placeholders. The task is to fill them with three `#[tauri::command]` handlers: `launch`, `launch_elevated`, and `run_system_command`, then register all three in `lib.rs`'s `invoke_handler`.

All required Windows API features are already present in `Cargo.toml`: `Win32_UI_Shell` (ShellExecuteW), `Win32_System_Shutdown` (LockWorkStation), `Win32_System_Power` (SetSuspendState), and `Win32_Foundation` (BOOL, GetLastError, ERROR_CANCELLED). No new Cargo dependencies are needed.

The main technical considerations are: correct PCWSTR encoding from Rust strings, the HINSTANCE > 32 success check for ShellExecuteW, detecting UAC cancellation via GetLastError() == ERROR_CANCELLED (1223) as the special no-hide case, and the `db::increment_launch_count` + `search::rebuild_index` post-launch sequence to keep MRU ranking current.

**Primary recommendation:** Use `windows-sys` directly (already in Cargo.toml) for all three system-level operations; use `std::process::Command` for shutdown/restart subprocesses (simpler, already available, no additional Win32 binding needed).

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Command signatures (fixed by Phase 5 stubs):**
- `launch(id: String)` — opens app by looking up path from SQLite, calls ShellExecuteW with lpVerb = NULL
- `launch_elevated(id: String)` — same lookup, calls ShellExecuteW with lpVerb = "runas"
- `run_system_command(cmd: String)` — dispatches on cmd string: "lock", "shutdown", "restart", "sleep"
- All three commands take the app `AppHandle` to access DbState (for path lookup) and the launcher window (for hide)

**Window hide behavior (LAUN-04):**
- Successful launch: Window hides before ShellExecuteW fires
- UAC cancellation (LAUN-02 specific): Launcher remains open with no error displayed — this is the only case where the window does NOT hide
- Failed regular launch (ShellExecuteW error): Window hides regardless; error is silently discarded

**Launch count tracking:**
- `launch()` and `launch_elevated()` both call `db::increment_launch_count(id)` after successful ShellExecuteW dispatch
- After incrementing, call `search::rebuild_index(app)` to refresh the nucleo index
- System commands do NOT increment launch counts

**requires_elevation field:**
- Keep `requires_elevation: false` on all SearchResult construction — no real PE manifest detection in Phase 6

**System command dispatch (LAUN-03):**
- `"lock"` → `LockWorkStation()` (Windows API via windows-sys)
- `"shutdown"` → spawn `shutdown /s /t 0` subprocess
- `"restart"` → spawn `shutdown /r /t 0` subprocess
- `"sleep"` → `SetSuspendState(FALSE, FALSE, FALSE)` (Windows API)
- Window hides before dispatching all system commands

**Error handling:**
- ShellExecuteW returns HINSTANCE > 32 on success; ≤ 32 on error
- UAC cancellation: ShellExecuteW returns ERROR_CANCELLED — detect and return `Ok(())` without hiding window
- All other errors: log to stderr with `eprintln!`, return `Ok(())`

### Claude's Discretion
- Exact windows-sys feature flags needed for ShellExecuteW, LockWorkStation, SetSuspendState
- Whether to use `ShellExecuteW` directly or the `Shell32` binding pattern from prior art in the codebase
- Spawn approach for shutdown/restart (std::process::Command vs CreateProcessW)
- Type casting for HWND in ShellExecuteW call (using launcher window HWND or HWND(0) for no parent)

### Deferred Ideas (OUT OF SCOPE)
- Real `requires_elevation` detection from PE manifest — deferred to Phase 8 or later
- Visual error feedback for failed launches (toast notification, etc.) — out of scope; launcher hides silently
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LAUN-01 | launch(id) opens app via ShellExecuteW with lpVerb = NULL | ShellExecuteW signature verified; HINSTANCE > 32 success check documented; PCWSTR encoding pattern documented |
| LAUN-02 | launch_elevated(id) opens with lpVerb = "runas"; UAC cancellation silently absorbed | GetLastError() == 1223 (ERROR_CANCELLED) detection pattern documented; no-hide-on-cancel path confirmed |
| LAUN-03 | run_system_command dispatches: lock → LockWorkStation(), shutdown → shutdown /s /t 0, restart → shutdown /r /t 0, sleep → SetSuspendState | All three Windows API signatures verified; std::process::Command recommended for shutdown/restart |
| LAUN-04 | All launch actions hide the launcher window after execution | app.get_webview_window("launcher").unwrap().hide() pattern exists in Phase 5; hide-before-launch pattern confirmed |
</phase_requirements>

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| windows-sys | ^0.52 (already in Cargo.toml) | Raw Windows API bindings (ShellExecuteW, LockWorkStation, SetSuspendState, GetLastError) | Already present; zero-overhead FFI-only crate; no extra dependencies |
| std::process::Command | stdlib | Spawn shutdown.exe subprocess for shutdown/restart | Simpler than CreateProcessW; handles privilege automatically |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| windows (already in Cargo.toml) | 0.58 | Higher-level Windows bindings with COM support | Already used in lib.rs for DWM; NOT needed for Phase 6 (windows-sys is sufficient) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| std::process::Command for shutdown | InitiateShutdownW (windows-sys) | Command is simpler and handles token privileges automatically; Win32 approach requires SeShutdownPrivilege |
| HWND(0) null parent | Launcher HWND | Using null parent (HWND(0)) is standard for app launchers — avoids blocking launcher window; simpler |

**Installation:**
No new dependencies needed. All required Windows API features are already declared in `src-tauri/Cargo.toml`:
```toml
windows-sys = { version = "^0.52", features = [
  "Win32_UI_Shell",        # ShellExecuteW
  "Win32_System_Shutdown", # LockWorkStation
  "Win32_System_Power",    # SetSuspendState
  "Win32_Foundation",      # BOOL, GetLastError, ERROR_CANCELLED
  ...
] }
```

---

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/
├── commands.rs         # launch(), launch_elevated() — fill this stub
├── system_commands.rs  # run_system_command() — fill this stub
└── lib.rs              # add 3 commands to invoke_handler
```

### Pattern 1: PCWSTR Encoding for Windows API Strings
**What:** Convert Rust `&str` to null-terminated UTF-16 `Vec<u16>` for Windows wide-string APIs.
**When to use:** Every time a `PCWSTR` parameter is needed (ShellExecuteW lpFile, lpVerb, lpOperation).
**Example:**
```rust
// Source: std::os::windows::ffi::OsStrExt — stdlib, no extra dep
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

fn to_wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}
```
Note: Keep the `Vec<u16>` alive for the entire duration of the unsafe call — the pointer must not dangle.

### Pattern 2: ShellExecuteW Call Structure
**What:** Invoke ShellExecuteW with correct parameter types.
**When to use:** launch() and launch_elevated() commands.
**Example:**
```rust
// Source: https://docs.rs/windows-sys/latest/windows_sys/Win32/UI/Shell/fn.ShellExecuteW.html
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

let file = to_wide_null(&path_string);
let verb = to_wide_null("runas"); // or use std::ptr::null() for lpVerb=NULL
let result = unsafe {
    ShellExecuteW(
        0,           // hwnd: HWND(0) — no parent window
        verb.as_ptr(),  // lpOperation: "runas" or null()
        file.as_ptr(),  // lpFile: path to executable
        std::ptr::null(), // lpParameters: no args
        std::ptr::null(), // lpDirectory: inherit working dir
        1,           // nShowCmd: SW_SHOWNORMAL = 1
    )
};
// Success: result > 32 (cast as isize for comparison)
if result as isize <= 32 {
    // failure — check GetLastError
}
```
Note: For `lpVerb = NULL` (normal launch), pass `std::ptr::null()`. The Windows API interprets NULL verb as default open action.

### Pattern 3: UAC Cancellation Detection
**What:** Distinguish UAC-cancelled launch from other errors to decide window hide behavior.
**When to use:** launch_elevated() only — when ShellExecuteW returns ≤ 32.
**Example:**
```rust
// Source: Windows SDK docs — ERROR_CANCELLED = 1223
use windows_sys::Win32::Foundation::GetLastError;

const ERROR_CANCELLED: u32 = 1223;

if result as isize <= 32 {
    let err = unsafe { GetLastError() };
    if err == ERROR_CANCELLED {
        // UAC cancelled — do NOT hide window, return Ok silently
        return Ok(());
    }
    // Other error — hide window, log to stderr
    eprintln!("[launch_elevated] ShellExecuteW failed: error {}", err);
}
```

### Pattern 4: DB Path Lookup
**What:** Retrieve an app's filesystem path from SQLite by ID, for use in ShellExecuteW.
**When to use:** Both launch() and launch_elevated() before calling ShellExecuteW.
**Example:**
```rust
// Source: established pattern from db.rs + Phase 4 command patterns
let conn = app.state::<crate::db::DbState>().0.lock().unwrap();
let path: Option<String> = conn.query_row(
    "SELECT path FROM apps WHERE id = ?1",
    rusqlite::params![id],
    |row| row.get(0),
).ok();
let Some(path) = path else {
    eprintln!("[launch] app not found in DB: {}", id);
    return Ok(()); // stale id — silent no-op
};
```

### Pattern 5: Post-Launch Index Refresh
**What:** Increment launch count in DB and rebuild nucleo index so MRU ranking updates immediately.
**When to use:** After successful ShellExecuteW in both launch() and launch_elevated().
**Example:**
```rust
// Source: db.rs increment_launch_count + search.rs rebuild_index (both established in prior phases)
{
    let conn = app.state::<crate::db::DbState>().0.lock().unwrap();
    let _ = crate::db::increment_launch_count(&conn, &id);
} // drop conn lock before rebuild_index
crate::search::rebuild_index(&app);
```
Note: Drop the DB MutexGuard before calling `rebuild_index` — rebuild_index also acquires the DB lock internally, deadlock risk if held across the call.

### Pattern 6: System Command Dispatch
**What:** Dispatch lock/shutdown/restart/sleep based on string command.
**When to use:** `run_system_command` handler.
**Example:**
```rust
// Source: windows-sys docs + std::process::Command stdlib
use windows_sys::Win32::System::Shutdown::LockWorkStation;
use windows_sys::Win32::System::Power::SetSuspendState;

match cmd.as_str() {
    "lock" => {
        unsafe { LockWorkStation(); }
    }
    "shutdown" => {
        let _ = std::process::Command::new("shutdown").args(["/s", "/t", "0"]).spawn();
    }
    "restart" => {
        let _ = std::process::Command::new("shutdown").args(["/r", "/t", "0"]).spawn();
    }
    "sleep" => {
        // bHibernate=false, bForce=false, bWakeupEventsDisabled=false
        unsafe { SetSuspendState(0, 0, 0); }
    }
    _ => {
        eprintln!("[system_command] unknown command: {}", cmd);
    }
}
```

### Pattern 7: Window Hide (Before Launch)
**What:** Hide the launcher window before executing launch action.
**When to use:** All launch paths except UAC-cancelled elevated launch.
**Example:**
```rust
// Source: Phase 5 established pattern in lib.rs / App.vue
if let Some(win) = app.get_webview_window("launcher") {
    let _ = win.hide();
}
```
Note: The Vue frontend also calls `hideWindow()` after `invoke(...)` returns, but the Rust side hides first for perceived performance. This double-hide is safe — hiding an already-hidden window is a no-op.

### Pattern 8: Command Registration in lib.rs
**What:** Add the three new commands to the Tauri invoke_handler.
**When to use:** Once all three command functions exist.
**Example:**
```rust
// Source: lib.rs invoke_handler (currently missing these three)
.invoke_handler(tauri::generate_handler![
    crate::indexer::reindex,
    crate::search::search,
    crate::store::get_settings_cmd,
    crate::commands::launch,           // add
    crate::commands::launch_elevated,  // add
    crate::system_commands::run_system_command, // add
])
```

### Anti-Patterns to Avoid
- **Holding DB MutexGuard across rebuild_index:** `rebuild_index` acquires the same lock — always drop before calling it (use a block `{ let conn = ...; ... }`)
- **Forgetting null terminator on wide strings:** `encode_wide()` alone does NOT add `\0` — always chain `once(0)`
- **Using the HINSTANCE return value directly as a pointer:** Cast to `isize` for numeric comparison against 32; do NOT dereference
- **Calling GetLastError after other Win32 calls:** Call it immediately after ShellExecuteW returns ≤ 32, before any other Win32 function that might overwrite the last-error code
- **Holding the wide string Vec past its use:** The `Vec<u16>` must stay alive (not dropped) for the entire unsafe block using its pointer

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| UTF-16 string conversion | Custom encoder | `OsStr::encode_wide().chain(once(0)).collect()` | stdlib — handles surrogate pairs, already correct |
| Process spawning for shutdown | CreateProcessW binding | `std::process::Command` | Simpler; handles privilege automatically; already stdlib |
| UAC elevation | Custom token manipulation | ShellExecuteW "runas" verb | OS handles the UAC dialog and token duplication |
| Window hide | Direct HWND manipulation | `app.get_webview_window("launcher").hide()` | Tauri's managed API — already established in Phase 5 |

**Key insight:** Windows API FFI via `windows-sys` is deliberately low-level. The stdlib `OsStrExt::encode_wide()` trait is the canonical UTF-16 conversion path — no extra crate needed.

---

## Common Pitfalls

### Pitfall 1: Dangling PCWSTR Pointer
**What goes wrong:** Wide string `Vec<u16>` is created in a temporary, dropped before the unsafe call completes.
**Why it happens:** Rust's temporary lifetime rules can drop a Vec at end of statement if not bound to a variable.
**How to avoid:** Always bind `let file = to_wide_null(&path);` as a named variable before the `unsafe {}` block. Keep all wide string Vecs in scope for the entire unsafe block.
**Warning signs:** Segfaults or garbled strings in launched application name.

### Pitfall 2: GetLastError Called Too Late
**What goes wrong:** GetLastError returns 0 or a wrong error code, failing to detect UAC cancellation.
**Why it happens:** Any Win32 call between ShellExecuteW and GetLastError can overwrite the thread-local last-error.
**How to avoid:** Call `GetLastError()` immediately as the first thing after detecting `result as isize <= 32`. No other Win32 calls in between.
**Warning signs:** UAC-cancelled launches incorrectly hide the window.

### Pitfall 3: DB Lock Deadlock in rebuild_index
**What goes wrong:** `rebuild_index` hangs indefinitely at startup or panics with MutexGuard poison.
**Why it happens:** The DB `MutexGuard` is still held when `rebuild_index` tries to acquire the same lock.
**How to avoid:** Use a block `{ let conn = ...; increment_launch_count(&conn, &id); }` so the guard drops before `rebuild_index` is called.
**Warning signs:** Launch command hangs forever; cargo test hangs.

### Pitfall 4: SW_SHOWNORMAL Value
**What goes wrong:** Wrong nShowCmd value hides the launched app or opens it minimized.
**Why it happens:** SW_SHOWNORMAL = 1 is not imported by default; easy to forget the constant.
**How to avoid:** Either import `windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL` or just pass `1i32` directly (well-known stable value).
**Warning signs:** Launched apps appear in taskbar but not on screen.

### Pitfall 5: system_commands.rs "lock" Behavior
**What goes wrong:** LockWorkStation() is called but the workstation doesn't lock.
**Why it happens:** User is not in an interactive session, or the current session is already locked.
**How to avoid:** This is expected OS behavior — not a code bug. Log the BOOL return value to stderr if it fails.
**Warning signs:** Lock seems to silently fail in certain session types (RDP, etc.) — acceptable behavior.

---

## Code Examples

Verified patterns from official sources:

### Full launch() Command Skeleton
```rust
// Source: windows-sys docs + established codebase patterns (db.rs, search.rs, Phase 5 hide)
#[tauri::command]
pub fn launch(id: String, app: tauri::AppHandle) -> Result<(), String> {
    // 1. Look up path in DB
    let path = {
        let conn = app.state::<crate::db::DbState>().0.lock().unwrap();
        conn.query_row(
            "SELECT path FROM apps WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get::<_, String>(0),
        ).ok()
    };
    let Some(path) = path else {
        eprintln!("[launch] app not found: {}", id);
        return Ok(());
    };

    // 2. Hide window before launch (LAUN-04)
    if let Some(win) = app.get_webview_window("launcher") {
        let _ = win.hide();
    }

    // 3. ShellExecuteW with lpVerb = NULL
    let file = to_wide_null(&path);
    let result = unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            0,
            std::ptr::null(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1, // SW_SHOWNORMAL
        )
    };

    if result as isize > 32 {
        // 4. Increment launch count and rebuild index
        {
            let conn = app.state::<crate::db::DbState>().0.lock().unwrap();
            let _ = crate::db::increment_launch_count(&conn, &id);
        }
        crate::search::rebuild_index(&app);
    } else {
        eprintln!("[launch] ShellExecuteW failed with code {}", result as isize);
    }

    Ok(())
}

fn to_wide_null(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}
```

### UAC Cancellation Detection in launch_elevated()
```rust
// Source: Windows SDK ERROR_CANCELLED = 1223, GetLastError pattern
const ERROR_CANCELLED: u32 = 1223;

let result = unsafe {
    windows_sys::Win32::UI::Shell::ShellExecuteW(
        0,
        runas.as_ptr(), // "runas\0" as Vec<u16>
        file.as_ptr(),
        std::ptr::null(),
        std::ptr::null(),
        1,
    )
};

if result as isize <= 32 {
    let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
    if err == ERROR_CANCELLED {
        // UAC cancelled — window stays open (LAUN-02), no error shown
        return Ok(());
    }
    // Other error — window already hidden before launch attempt
    eprintln!("[launch_elevated] ShellExecuteW error: {}", err);
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| winapi crate | windows-sys crate | ~2021 | windows-sys is the current Microsoft-maintained replacement; winapi is deprecated |
| ShellExecute (ANSI) | ShellExecuteW (Unicode) | Legacy → current | Always use W variants for proper Unicode path support |
| CreateProcessW for all subprocess | std::process::Command for simple CLI tools | N/A | Command is idiomatic Rust for subprocess spawning without COM overhead |

**Deprecated/outdated:**
- `winapi` crate: Superseded by `windows-sys` and `windows`. Project already uses `windows-sys`.
- `ShellExecuteA`: ANSI variant — never use; paths with non-ASCII chars silently corrupt.

---

## Open Questions

1. **ERROR_CANCELLED constant availability in windows-sys**
   - What we know: Value is 1223 (standard Windows SDK constant, confirmed in std::sys internals)
   - What's unclear: Whether `windows_sys::Win32::Foundation::ERROR_CANCELLED` is exported or must be defined as `const ERROR_CANCELLED: u32 = 1223`
   - Recommendation: Define as local const `1223u32` — safe, stable, avoids potential import path variance

2. **HWND type for ShellExecuteW hwnd parameter**
   - What we know: windows-sys uses `HWND` as `isize` in 0.52; passing `0` satisfies the "no parent" case
   - What's unclear: Whether the type requires explicit `0isize` vs `0` with type inference
   - Recommendation: Pass literal `0` — Rust type inference resolves from function signature; if ambiguous, cast as `0 as windows_sys::Win32::Foundation::HWND`

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | None (standard `[cfg(test)]` modules in each .rs file) |
| Quick run command | `cd src-tauri && cargo test` |
| Full suite command | `cd src-tauri && cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LAUN-01 | launch(id) opens app via ShellExecuteW | manual-only | N/A — requires real Windows GDI context and installed apps | N/A |
| LAUN-02 | UAC cancellation: window stays open, no error | manual-only | N/A — requires interactive UAC dialog | N/A |
| LAUN-03 | run_system_command dispatches correctly | manual-only | N/A — requires live OS (lock/sleep/shutdown are system-wide) | N/A |
| LAUN-04 | Window hides after launch | manual-only | N/A — requires running Tauri window | N/A |

**Justification for manual-only:** All launch commands produce irreversible side effects (launching processes, locking workstation, sleep, shutdown) or require an interactive Windows session with UAC. Unit-testing ShellExecuteW and system calls in a CI `cargo test` context is impractical and potentially disruptive (a `cargo test` that sleeps the machine would be catastrophic). The established project pattern (Phase 3: GDI icon extraction also manual-only) confirms this is acceptable.

**Unit testable portions** (can be extracted into pure functions and tested):
- `to_wide_null`: string → Vec<u16> conversion (pure, no Win32 call)
- DB path lookup logic (uses rusqlite in-memory — established pattern from db.rs tests)

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test` (ensures no compile errors, DB tests green)
- **Per wave merge:** `cd src-tauri && cargo test` + manual smoke test: summon launcher, type app name, press Enter
- **Phase gate:** Cargo test green + manual verification: normal launch, elevated launch (UAC prompt appears and cancels correctly), lock workstation, before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/commands.rs` — currently empty stub; test for `to_wide_null` helper if extracted
- [ ] `src-tauri/src/system_commands.rs` — currently empty stub

*(Existing `cargo test` infrastructure covers all DB layer tests; no new test framework install needed.)*

---

## Sources

### Primary (HIGH confidence)
- [windows_sys::Win32::UI::Shell::ShellExecuteW](https://docs.rs/windows-sys/latest/windows_sys/Win32/UI/Shell/fn.ShellExecuteW.html) — function signature, parameter types verified
- [windows_sys::Win32::System::Shutdown (LockWorkStation)](https://docs.rs/windows-sys/latest/windows_sys/Win32/System/Shutdown/) — LockWorkStation confirmed in module
- [windows_sys::Win32::System::Power::SetSuspendState](https://docs.rs/windows-sys/latest/windows_sys/Win32/System/Power/fn.SetSuspendState.html) — signature verified: `(bhibernate: bool, bforce: bool, bwakeupeventsdisabled: bool) -> bool`
- [std::os::windows::ffi::OsStrExt](https://doc.rust-lang.org/std/os/windows/ffi/trait.OsStrExt.html) — `encode_wide()` for PCWSTR conversion, stdlib
- Codebase (db.rs, search.rs, lib.rs) — `increment_launch_count`, `rebuild_index`, `get_webview_window().hide()` patterns confirmed by direct file reading

### Secondary (MEDIUM confidence)
- [Windows SDK docs — ShellExecuteW](https://github.com/MicrosoftDocs/sdk-api/blob/docs/sdk-api-src/content/shellapi/nf-shellapi-shellexecutew.md) — HINSTANCE > 32 success check, ERROR_CANCELLED (1223) for UAC
- [Rust std::sys windows ERROR_CANCELLED](https://stdrs.dev/nightly/x86_64-pc-windows-gnu/std/sys/windows/c/windows_sys/constant.ERROR_CANCELLED.html) — value confirmed as 1223

### Tertiary (LOW confidence)
- Community search results confirming `std::process::Command` for shutdown/restart is common Rust pattern

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies already in Cargo.toml; confirmed from codebase
- Architecture: HIGH — Windows API signatures verified from official docs; established codebase patterns confirmed by direct file reading
- Pitfalls: HIGH — PCWSTR dangling pointer and mutex deadlock are verified Rust-specific hazards; GetLastError ordering is Windows API documented behavior

**Research date:** 2026-03-07
**Valid until:** 2026-09-07 (windows-sys 0.52 API is stable; Windows API signatures do not change)
