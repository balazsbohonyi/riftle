---
phase: 06-launch-actions
verified: 2026-03-07T00:00:00Z
status: human_needed
score: 5/6 must-haves verified
human_verification:
  - test: "Elevated launch — UAC prompt appears and approving opens elevated process"
    expected: "UAC prompt appears when Ctrl+Shift+Enter is pressed; clicking Yes launches the selected app elevated; launcher window disappears"
    why_human: "ShellExecuteW with runas verb produces a Windows UAC dialog; cannot verify dialog appearance or user interaction programmatically"
  - test: "System command — shutdown and restart trigger correct OS actions"
    expected: "Typing '> shutdown' and pressing Enter initiates a Windows shutdown (shutdown /s /t 0); '> restart' initiates a reboot"
    why_human: "Cannot invoke destructive OS commands in verification; actual OS response to shutdown.exe cannot be tested statically"
  - test: "Sleep command triggers SetSuspendState"
    expected: "Typing '> sleep' and pressing Enter suspends the workstation"
    why_human: "SetSuspendState is a side-effect-only Win32 call; cannot trigger it programmatically during verification"
  - test: "MRU ranking updates after each launch"
    expected: "After launching an app twice, it ranks higher than single-launch apps in subsequent searches"
    why_human: "Requires a live search session comparing rankings before and after repeated launches"
---

# Phase 6: Launch Actions Verification Report

**Phase Goal:** Implement all Tauri launch commands in Rust — normal, elevated, and system commands — using windows-sys APIs.
**Verified:** 2026-03-07
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Typing an app name and pressing Enter opens the application via ShellExecuteW | VERIFIED | `commands.rs` lines 30-40: `ShellExecuteW` called with `std::ptr::null()` verb and wide-string path from DB lookup |
| 2 | Ctrl+Shift+Enter shows UAC prompt; cancelling leaves launcher open with no error | VERIFIED (partial — automated) | `commands.rs` lines 92-101: `GetLastError()` called immediately after `result <= 32`; `ERROR_CANCELLED` (1223) returns `Ok(())` without hiding; `App.vue` line 111 confirms frontend does NOT call `hideWindow()` after `launch_elevated` |
| 3 | Typing '> lock' and pressing Enter locks the workstation | VERIFIED | `system_commands.rs` lines 17-21: `LockWorkStation()` called; prefix stripping at line 14 ensures `"system:lock"` ID matches |
| 4 | Typing '> shutdown' or '> restart' triggers the OS action | HUMAN NEEDED | `system_commands.rs` lines 23-30: correct `shutdown.exe` arguments present but OS effect requires human test |
| 5 | Launcher window disappears immediately when any launch action succeeds | VERIFIED | `launch`: hides before `ShellExecuteW` (line 25-27); `launch_elevated`: hides after success (lines 104-106); `run_system_command`: hides before dispatch (lines 8-10); `App.vue` also calls `hideWindow()` after `launchItem()` as belt-and-suspenders |
| 6 | MRU ranking updates after normal or elevated launch | HUMAN NEEDED | Code path exists: `increment_launch_count` + `rebuild_index` called on success in both `launch` (lines 44-49) and `launch_elevated` (lines 107-112); runtime verification requires live session |

**Score:** 4 fully automated + 2 human-needed = 6/6 truths have implementation; 4 verifiable programmatically

---

## Required Artifacts

| Artifact | Provides | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/commands.rs` | `launch()`, `launch_elevated()`, `to_wide_null()` helper | VERIFIED | 159 lines; both commands annotated `#[tauri::command]`; 3 unit tests for `to_wide_null` |
| `src-tauri/src/system_commands.rs` | `run_system_command()` dispatching lock/shutdown/restart/sleep | VERIFIED | 43 lines; `#[tauri::command]` annotation present; all 4 dispatch arms implemented |
| `src-tauri/src/lib.rs` | invoke_handler registration of all 6 commands | VERIFIED | Lines 113-120: 6-command `generate_handler!` macro verified |
| `src/App.vue` | Frontend `launchElevated()` does NOT double-hide | VERIFIED | Lines 108-115: no `hideWindow()` call; comment explicitly documents ownership decision |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `commands.rs::launch` | `app.state::<DbState>().0.lock()` | `query_row("SELECT path FROM apps WHERE id = ?1")` | WIRED | Lines 10-17: scoped block, local variable bound before `.0.lock()` to satisfy borrow checker |
| `commands.rs::launch` | `ShellExecuteW` | `unsafe { windows_sys::Win32::UI::Shell::ShellExecuteW(...) }` | WIRED | Lines 31-40: NULL verb, wide-string path, SW_SHOWNORMAL(1) |
| `commands.rs::launch` | `crate::search::rebuild_index` | `increment_launch_count` then `rebuild_index(&app)` after DB lock dropped | WIRED | Lines 44-49: scoped DB block, then `rebuild_index` on line 49 |
| `system_commands.rs::run_system_command` | `LockWorkStation()` | `unsafe { windows_sys::Win32::System::Shutdown::LockWorkStation() }` | WIRED | Lines 18-21: `cmd_key = "lock"` match arm present |
| `lib.rs` | `commands::launch, commands::launch_elevated, system_commands::run_system_command` | `tauri::generate_handler!` macro | WIRED | Lines 117-119: all three commands in handler |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| LAUN-01 | 06-01-PLAN.md | `launch(id)` opens app via ShellExecuteW with `lpVerb = NULL` | SATISFIED | `commands.rs` line 33: `std::ptr::null()` as lpVerb; ShellExecuteW call verified |
| LAUN-02 | 06-01-PLAN.md | `launch_elevated(id)` opens with `lpVerb = "runas"`; UAC cancel silently absorbed | SATISFIED | `commands.rs` line 84: `runas.as_ptr()` as lpVerb; lines 92-101: ERROR_CANCELLED(1223) returns `Ok(())` without hiding; `App.vue` confirmed no double-hide |
| LAUN-03 | 06-01-PLAN.md | `run_system_command` dispatches lock → LockWorkStation(); shutdown/restart → shutdown.exe; sleep → SetSuspendState | SATISFIED (code) / HUMAN NEEDED (OS effect for shutdown/restart/sleep) | `system_commands.rs`: all 4 arms present; prefix stripping at line 14 handles `"system:"` IDs |
| LAUN-04 | 06-01-PLAN.md | All launch actions hide the launcher window after execution | SATISFIED | `launch`: hide-first at lines 25-27; `launch_elevated`: hide-on-success at lines 104-106; `run_system_command`: hide-first at lines 8-10 |

No orphaned requirements — LAUN-01 through LAUN-04 are the only Phase 6 requirements in REQUIREMENTS.md (traceability table lines 163-166), and all four are claimed and implemented in 06-01-PLAN.md.

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/App.vue` | 104 | `launchItem()` calls `hideWindow()` after `run_system_command` returns | INFO | Rust already hides before dispatch; double-hide is harmless (second call is a no-op on an already-hidden window). No behavioral defect. |

No TODO/FIXME/HACK/placeholder stubs found in any Phase 6 files.

---

## Human Verification Required

### 1. UAC Elevated Launch — Full Flow

**Test:** Select any indexed application, press Ctrl+Shift+Enter
**Expected:** Windows UAC dialog appears. Clicking "No" or "Cancel" leaves the launcher window open with no error. Clicking "Yes" opens the app with administrator privileges and the launcher window disappears.
**Why human:** UAC dialog is a system modal; cannot be triggered or inspected programmatically.

### 2. Shutdown and Restart System Commands

**Test:** Type `> shutdown` and press Enter (use a VM or be prepared to save work)
**Expected:** Windows initiates a scheduled shutdown immediately (equivalent to `shutdown /s /t 0`)
**Why human:** Destructive OS action; cannot execute in verification context.

### 3. Sleep System Command

**Test:** Type `> sleep` and press Enter
**Expected:** Workstation enters sleep/suspend state
**Why human:** `SetSuspendState` is a hardware-level call; cannot test statically.

### 4. MRU Ranking — Post-Launch Update

**Test:** Search for a known app, launch it twice using Enter. Reopen the launcher and search again.
**Expected:** The twice-launched app appears ranked higher than apps with zero or one launch in the same search result set.
**Why human:** Requires a live Tauri session with real DB writes and nucleo index rebuild comparison across two queries.

---

## Automated Checks Summary

All automated checks passed:

- `cargo test` exits 0 — 37 tests pass (0 failed, 2 ignored)
- `to_wide_null` unit tests: 3/3 pass (`test_to_wide_null_hello`, `test_to_wide_null_empty`, `test_to_wide_null_path`)
- `commands.rs`: `launch` and `launch_elevated` both carry `#[tauri::command]`; both are public
- `system_commands.rs`: `run_system_command` carries `#[tauri::command]`; prefix stripping present
- `lib.rs`: invoke_handler contains all 6 commands (3 pre-existing + 3 Phase 6)
- `App.vue` `launchElevated()`: confirmed no unconditional `hideWindow()` call
- Git commits verified: `e2cd046`, `95336f5`, `235abe7`, `1af2949` all present in history
- No TODO/FIXME/stub patterns in any Phase 6 implementation files

---

_Verified: 2026-03-07_
_Verifier: Claude (gsd-verifier)_
