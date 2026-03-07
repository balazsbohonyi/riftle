---
phase: 6
slug: launch-actions
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-07
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | None (standard `[cfg(test)]` modules in each .rs file) |
| **Quick run command** | `cd src-tauri && cargo test` |
| **Full suite command** | `cd src-tauri && cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test`
- **After every plan wave:** Run `cd src-tauri && cargo test` + manual smoke test: summon launcher, type app name, press Enter
- **Before `/gsd:verify-work`:** Full suite must be green + manual verification of all launch paths
- **Max feedback latency:** ~5 seconds (cargo test) + manual smoke

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 6-01-01 | 01 | 1 | LAUN-01 | manual | `cd src-tauri && cargo test` (compile check) | ❌ W0 | ⬜ pending |
| 6-01-02 | 01 | 1 | LAUN-02 | manual | `cd src-tauri && cargo test` (compile check) | ❌ W0 | ⬜ pending |
| 6-01-03 | 01 | 1 | LAUN-03 | manual | `cd src-tauri && cargo test` (compile check) | ❌ W0 | ⬜ pending |
| 6-01-04 | 01 | 1 | LAUN-04 | manual | `cd src-tauri && cargo test` (compile check) | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/commands.rs` — stub exists but empty; fill with `launch`, `launch_elevated`, and `to_wide_null` helper
- [ ] `src-tauri/src/system_commands.rs` — stub exists but empty; fill with `run_system_command`

*Existing `cargo test` infrastructure covers all DB layer tests; no new test framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| launch(id) opens application via ShellExecuteW | LAUN-01 | Requires real Windows GDI context and installed apps | Summon launcher, type app name, press Enter — app opens |
| launch_elevated(id) shows UAC, cancels correctly | LAUN-02 | Requires interactive UAC dialog in a live session | Summon launcher, use Ctrl+Shift+Enter on an app, cancel UAC — launcher stays open, no error |
| run_system_command("lock") locks workstation | LAUN-03 | Requires live OS interactive session | Type `> lock`, press Enter — workstation locks |
| run_system_command("sleep") suspends system | LAUN-03 | Requires live OS (would suspend test runner) | Type `> sleep`, press Enter — system sleeps |
| Window hides after all launch actions | LAUN-04 | Requires running Tauri window | Verify launcher disappears after each launch action in all manual tests above |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
