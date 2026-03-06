---
phase: 2
slug: data-layer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-06
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`cargo test`) |
| **Config file** | None needed — `#[cfg(test)]` modules in source files |
| **Quick run command** | `cargo test -p riftle --lib` |
| **Full suite command** | `cargo test -p riftle` |
| **Estimated runtime** | ~2 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p riftle --lib`
- **After every plan wave:** Run `cargo test -p riftle`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 2 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 2-01-01 | 01 | 0 | DATA-07 | unit | `cargo test -p riftle --lib paths::tests` | ❌ W0 | ⬜ pending |
| 2-01-02 | 01 | 1 | DATA-01 | integration (smoke) | `cargo test -p riftle` + verify file on disk | ✅ | ⬜ pending |
| 2-02-01 | 02 | 0 | DATA-02, DATA-03 | unit | `cargo test -p riftle --lib db::tests` | ❌ W0 | ⬜ pending |
| 2-02-02 | 02 | 1 | DATA-02 | unit | `cargo test -p riftle --lib db::tests::test_schema` | ❌ W0 | ⬜ pending |
| 2-02-03 | 02 | 1 | DATA-03 | unit | `cargo test -p riftle --lib db::tests` | ❌ W0 | ⬜ pending |
| 2-03-01 | 03 | 0 | DATA-05, DATA-06 | unit | `cargo test -p riftle --lib store::tests` | ❌ W0 | ⬜ pending |
| 2-03-02 | 03 | 1 | DATA-04 | integration (smoke) | `cargo test -p riftle` + verify file on disk | ✅ | ⬜ pending |
| 2-03-03 | 03 | 1 | DATA-05 | unit | `cargo test -p riftle --lib store::tests::test_defaults` | ❌ W0 | ⬜ pending |
| 2-03-04 | 03 | 1 | DATA-06 | unit | `cargo test -p riftle --lib store::tests::test_round_trip` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/paths.rs` — `#[cfg(test)] mod tests` block covering DATA-07 (portable path logic, using a tempdir)
- [ ] `src-tauri/src/db.rs` — `#[cfg(test)] mod tests` block covering DATA-02 (schema) and DATA-03 (init_db_connection, upsert_app, get_all_apps, increment_launch_count)
- [ ] `src-tauri/src/store.rs` — `#[cfg(test)] mod tests` block covering DATA-05 (Settings Default impl) and DATA-06 (serde round-trip)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| DB created at correct portable path on launch | DATA-01 | Requires AppHandle + filesystem; integration smoke | Run `pnpm tauri dev`, place `riftle-launcher.portable` next to exe, verify `./data/launcher.db` created |
| DB created at correct installed path on launch | DATA-01 | Requires AppHandle + filesystem; integration smoke | Run `pnpm tauri dev` without portable marker, verify `%APPDATA%/riftle-launcher/launcher.db` created |
| settings.json created on first run | DATA-04 | Requires live AppHandle; store plugin needs real app context | Run `pnpm tauri dev`, verify `settings.json` appears at correct data dir path |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 2s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
