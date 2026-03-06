---
phase: 4
slug: search-engine
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-06
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`#[test]`) |
| **Config file** | None — `cargo test` in `src-tauri/` |
| **Quick run command** | `cargo test search` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test search`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 4-W0-01 | 01 | 0 | SRCH-01..05 | build | `cargo build` | ❌ W0 | ⬜ pending |
| 4-01-01 | 01 | 1 | SRCH-01 | unit | `cargo test test_search_empty_returns_empty` | ❌ W0 | ⬜ pending |
| 4-01-02 | 01 | 1 | SRCH-01 | unit | `cargo test test_search_fuzzy_returns_matches` | ❌ W0 | ⬜ pending |
| 4-01-03 | 01 | 1 | SRCH-02 | unit | `cargo test test_search_prefix_beats_fuzzy` | ❌ W0 | ⬜ pending |
| 4-01-04 | 01 | 1 | SRCH-02 | unit | `cargo test test_search_acronym_tier` | ❌ W0 | ⬜ pending |
| 4-01-05 | 01 | 1 | SRCH-02 | unit | `cargo test test_search_mru_tiebreak` | ❌ W0 | ⬜ pending |
| 4-01-06 | 01 | 1 | SRCH-03 | unit | `cargo test test_search_capped_at_50` | ❌ W0 | ⬜ pending |
| 4-02-01 | 02 | 2 | SRCH-04 | unit | `cargo test test_search_system_prefix_all` | ❌ W0 | ⬜ pending |
| 4-02-02 | 02 | 2 | SRCH-04 | unit | `cargo test test_search_system_prefix_filtered` | ❌ W0 | ⬜ pending |
| 4-02-03 | 02 | 2 | SRCH-04 | unit | `cargo test test_search_system_no_space` | ❌ W0 | ⬜ pending |
| 4-02-04 | 02 | 2 | SRCH-04 | unit | `cargo test test_search_system_no_app_mixing` | ❌ W0 | ⬜ pending |
| 4-02-05 | 02 | 2 | SRCH-05 | unit | `cargo test test_system_result_kind` | ❌ W0 | ⬜ pending |
| 4-02-06 | 02 | 2 | SRCH-05 | unit | `cargo test test_system_result_icon` | ❌ W0 | ⬜ pending |
| 4-02-07 | 02 | 2 | SRCH-05 | unit | `cargo test test_system_result_path_empty` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/icons/system_command.png` — placeholder PNG asset (16×16 or 32×32, any valid PNG; real icon at Claude's discretion)
- [ ] `nucleo-matcher = "0.3"` added to `[dependencies]` in `src-tauri/Cargo.toml` (direct dep for unambiguous imports)
- [ ] `src-tauri/src/search.rs` — all 13 test stubs scaffolded in `#[cfg(test)]` block starting RED (`todo!()` or `should_panic`)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `system_command.png` copied to `{data_dir}/icons/` on startup | SRCH-05 | Requires AppHandle and real filesystem | Launch `pnpm tauri dev`, inspect `%APPDATA%\com.riftle.launcher\icons\` for `system_command.png` |
| Nucleo index rebuilds after `reindex()` without blip | SRCH-01 | Requires live Tauri managed state + background threads | Run `pnpm tauri dev`, trigger re-index, confirm search still returns results mid-rebuild |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
