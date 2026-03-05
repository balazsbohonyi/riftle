---
phase: 1
slug: project-scaffold-configuration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-05
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | None — Tauri app; compilation + smoke test only |
| **Config file** | src-tauri/Cargo.toml |
| **Quick run command** | `cargo check --manifest-path src-tauri/Cargo.toml` |
| **Full suite command** | `pnpm tauri dev` (manual smoke test — requires Windows display) |
| **Estimated runtime** | cargo check ~10s; pnpm tauri dev ~60s first run |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --manifest-path src-tauri/Cargo.toml`
- **After every plan wave:** Run `pnpm tauri dev` (full smoke test, manual)
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~10 seconds (cargo check)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| Cargo.toml update | 01 | 1 | SCAF-03 | compilation | `cargo check --manifest-path src-tauri/Cargo.toml` | ✅ | ⬜ pending |
| tauri.conf.json rewrite | 01 | 1 | SCAF-02 | manual-only | `cargo check` (JSON validation) + visual inspect | ✅ | ⬜ pending |
| capabilities update | 01 | 1 | SCAF-02 | manual-only | IPC accessible after dev start | ✅ | ⬜ pending |
| lib.rs replacement | 01 | 1 | SCAF-03 | compilation | `cargo check --manifest-path src-tauri/Cargo.toml` | ✅ | ⬜ pending |
| pnpm tauri dev | 01 | 2 | SCAF-04 | smoke test | `pnpm tauri dev` — manual, launcher window appears | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

No test framework installation needed — all phase requirements are configuration/compilation checks.

- [ ] `cargo check --manifest-path src-tauri/Cargo.toml` — validates Rust compilation after crate changes

*Existing infrastructure covers all automated checks for this phase.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Launcher window appears frameless with shadow | SCAF-02 | Visual inspection required | Run `pnpm tauri dev`, verify window has no title bar, has system shadow, is transparent-background |
| Settings window hidden at launch | SCAF-02 | Visual inspection required | Run `pnpm tauri dev`, verify no settings window appears |
| Always-on-top and skip-taskbar | SCAF-02 | OS-level behavior | Launch, switch to another app, verify launcher stays on top; check taskbar has no entry |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
