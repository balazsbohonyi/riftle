---
phase: 5
slug: launcher-window-ui
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-06
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + manual smoke (frontend — vitest deferred to Phase 8) |
| **Config file** | src-tauri/Cargo.toml (existing) |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test` + manual UAT checklist |
| **Estimated runtime** | ~10 seconds (cargo test) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test` + manual smoke test of launcher window
- **Before `/gsd:verify-work`:** Full `cargo test` green + manual UAT checklist
- **Max feedback latency:** ~10 seconds (cargo test)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-W0-rust | 01 | 0 | store.rs animation field | unit | `cargo test` | ✅ existing | ⬜ pending |
| 05-W1-lwnd01 | 01 | 1 | LWND-01 | manual-smoke | N/A — visual/config | N/A | ⬜ pending |
| 05-W1-lwnd02 | 01 | 1 | LWND-02 | manual-smoke | N/A — visual height | N/A | ⬜ pending |
| 05-W1-lwnd03 | 01 | 1 | LWND-03 | manual-smoke | N/A — Tauri focus API | N/A | ⬜ pending |
| 05-W1-lwnd04 | 02 | 1 | LWND-04 | manual-smoke | N/A — keyboard nav | N/A | ⬜ pending |
| 05-W1-lwnd05 | 02 | 1 | LWND-05 | manual-smoke | N/A — Ctrl+Shift | N/A | ⬜ pending |
| 05-W1-lwnd06 | 02 | 1 | LWND-06 | manual-smoke | N/A — Tauri focus event | N/A | ⬜ pending |
| 05-W1-lwnd07 | 02 | 1 | LWND-07 | manual-smoke | N/A — visual icons | N/A | ⬜ pending |
| 05-W1-lwnd08 | 02 | 1 | LWND-08 | manual-smoke | N/A — conditional render | N/A | ⬜ pending |
| 05-W1-lwnd09 | 02 | 1 | LWND-09 | manual-smoke | N/A — badge visual | N/A | ⬜ pending |
| 05-W1-lwnd10 | 02 | 1 | LWND-10 | manual-smoke | N/A — performance visual | N/A | ⬜ pending |
| 05-W1-lwnd11 | 02 | 1 | LWND-11 | manual-smoke | N/A — placeholder text | N/A | ⬜ pending |
| 05-W1-lwnd12 | 02 | 1 | LWND-12 | manual-smoke | N/A — system cmd render | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Add `animation: String` field to `Settings` struct in `store.rs` with default `"slide"` — enables frontend to read animation mode
- [ ] Add `get_settings` Tauri command returning full Settings struct — frontend reads `show_path`, `animation`, `data_dir`
- [ ] `cargo test` green after Rust changes

*Frontend vitest infrastructure deferred to Phase 8 — all LWND requirements use manual smoke verification.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Frameless window, always-on-top, no taskbar | LWND-01 | Visual/OS behavior | `pnpm tauri dev`, confirm window has no frame, no taskbar entry |
| Height grows with results (max 8 rows) | LWND-02 | Visual layout | Type queries, count visible rows, verify max at 8 |
| Input autofocused + cleared on show | LWND-03 | Tauri focus API | Toggle window via hotkey stub, confirm cursor in input, field empty |
| ↑/↓ nav, Enter, Escape, wrap | LWND-04 | Keyboard interaction | Navigate list fully, test wrap-around at top/bottom |
| Ctrl+Shift+Enter elevated launch | LWND-05 | Keyboard modifier | Hold Ctrl+Shift, confirm Admin badge; release, confirm badge gone |
| Auto-hide on focus loss | LWND-06 | Tauri focus event | Click away from launcher, confirm it hides |
| App icons displayed | LWND-07 | Visual + asset protocol | Confirm icons load for indexed apps |
| Path line on selected row only | LWND-08 | Conditional render | Enable show_path, navigate rows, confirm path shows on selected only |
| Admin badge right side of selected row | LWND-09 | Visual modifier state | Hold Ctrl+Shift, confirm badge position |
| Virtualised list — no lag at 50 items | LWND-10 | Performance visual | Trigger 50-result search, scroll rapidly |
| Placeholder text when no query | LWND-11 | Visual | Clear input, confirm placeholder text present |
| System commands: ⚙️ icon, no path | LWND-12 | Visual render | Type ">", confirm all 4 cmds with ⚙️, no path line |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
