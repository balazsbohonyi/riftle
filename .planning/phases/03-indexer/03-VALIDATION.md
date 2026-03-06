---
phase: 3
slug: indexer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-06
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test (`#[cfg(test)]` + `cargo test`) |
| **Config file** | None — Cargo.toml `[lib]` section |
| **Quick run command** | `cargo test -p riftle --lib indexer` |
| **Full suite command** | `cargo test -p riftle --lib` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p riftle --lib indexer`
- **After every plan wave:** Run `cargo test -p riftle --lib`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 3-W0-01 | W0 | 0 | INDX-01 | unit | `cargo test -p riftle --lib indexer::tests::test_crawl_discovers_exe` | ❌ W0 | ⬜ pending |
| 3-W0-02 | W0 | 0 | INDX-01 | unit | `cargo test -p riftle --lib indexer::tests::test_crawl_discovers_lnk` | ❌ W0 | ⬜ pending |
| 3-W0-03 | W0 | 0 | INDX-02 | unit | `cargo test -p riftle --lib indexer::tests::test_resolve_lnk_valid` | ❌ W0 | ⬜ pending |
| 3-W0-04 | W0 | 0 | INDX-02 | unit | `cargo test -p riftle --lib indexer::tests::test_resolve_lnk_broken` | ❌ W0 | ⬜ pending |
| 3-W0-05 | W0 | 0 | INDX-03 | unit | `cargo test -p riftle --lib indexer::tests::test_prune_stale` | ❌ W0 | ⬜ pending |
| 3-W0-06 | W0 | 0 | INDX-03 | unit | `cargo test -p riftle --lib indexer::tests::test_crawl_excludes_path` | ❌ W0 | ⬜ pending |
| 3-W0-07 | W0 | 0 | INDX-04 | unit | `cargo test -p riftle --lib indexer::tests::test_icon_filename_stable` | ❌ W0 | ⬜ pending |
| 3-W0-08 | W0 | 0 | INDX-04 | unit | `cargo test -p riftle --lib indexer::tests::test_generic_icon_bootstrap` | ❌ W0 | ⬜ pending |
| 3-W0-09 | W0 | 0 | INDX-06 | unit | `cargo test -p riftle --lib indexer::tests::test_timer_fires` | ❌ W0 | ⬜ pending |
| 3-W0-10 | W0 | 0 | INDX-06 | unit | `cargo test -p riftle --lib indexer::tests::test_timer_reset` | ❌ W0 | ⬜ pending |
| 3-W0-11 | W0 | 0 | INDX-07 | unit | `cargo test -p riftle --lib indexer::tests::test_atomic_guard_prevents_double_index` | ❌ W0 | ⬜ pending |
| 3-W0-12 | W0 | 0 | INDX-08 | unit | `cargo test -p riftle --lib` (compile check) | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/indexer.rs` — fill stub with all public function signatures + `#[cfg(test)]` module with test stubs for all 12 test cases above
- [ ] `src-tauri/icons/generic.png` — create 32×32 placeholder PNG (must exist at compile time for `include_bytes!`)
- [ ] Test helper `fn temp_dir_with_exes()` — creates temp directory with dummy .exe and .lnk files for crawl tests
- [ ] Test helper `fn in_memory_db()` — reuse pattern from db.rs tests (`Connection::open_in_memory()`)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Async icon extraction — icon_path starts as generic, updates after extraction | INDX-05 | Requires real Windows executables with embedded icons on disk; cannot mock HICON | Launch app, open DB browser, confirm icon_path column updates from "generic.png" to app-specific png after indexing |
| Filesystem watcher debounce — incremental re-index fires within ~500ms of Start Menu change | INDX-07 | Requires running process and actual filesystem events | Start app, copy a .lnk file to Start Menu user folder, verify in DB within 1s that new app appears |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
