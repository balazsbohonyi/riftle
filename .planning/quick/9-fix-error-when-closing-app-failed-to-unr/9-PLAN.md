# Quick Task 9 Plan

## Goal
Reduce or eliminate the close-time Windows WebView shutdown error:
`Failed to unregister class Chrome_WidgetWin_0. Error = 1412`.

## Tasks

1. Update quit flow in Rust command handler
- files: `src-tauri/src/commands.rs`
- action: replace direct `app.exit(0)` with explicit app cleanup followed by exit.
- verify: code compiles and command remains callable from frontend.
- done: shutdown path is deterministic and documents rationale.

2. Validate backend build/tests
- files: `src-tauri/` (workspace checks)
- action: run Rust tests (or at minimum compile checks) for backend.
- verify: command change introduces no compile/test regressions.
- done: successful command output captured in task summary.

3. Record quick-task execution summary
- files: `.planning/quick/9-fix-error-when-closing-app-failed-to-unr/9-SUMMARY.md`
- action: document change, validation results, and residual risk.
- verify: summary exists and references touched file(s).
- done: quick task artifacts complete.
