# CONCERNS

## Scope
- Repository scan focused on runtime risk, technical debt, fragility, security posture, and performance hotspots.
- Evidence references point to current source files under `src/` and `src-tauri/`.

## High Risk

1. Reindex path ignores user settings during manual reindex.
- Evidence: `src-tauri/src/indexer.rs` uses `Settings::default()` inside `reindex` (`let settings = crate::store::Settings::default();`).
- Impact: manual reindex can drop `additional_paths`, `excluded_paths`, and `system_tool_allowlist` behavior, creating inconsistent search results.
- Mitigation: load settings from store in `reindex` using managed `data_dir` and `store::get_settings`, then pass those settings to `run_full_index`.

2. Reindex interval updates are not applied to running timer thread.
- Evidence: `start_background_tasks` captures `interval_mins` once; `Settings.vue` emits `settings-changed` with `reindex_interval`, but backend timer never consumes updated interval.
- Impact: UI shows changed interval but backend schedule remains stale until restart.
- Mitigation: store interval in shared atomic/state and recalculate deadline on update, or restart timer worker after settings save.

3. "Manual only" interval (`0`) can trigger continuous periodic indexing.
- Evidence: `Settings.vue` offers `{ value: 0, label: 'Manual only' }`; `indexer.rs` computes `Duration::from_secs(interval_mins as u64 * 60)` and loops every second.
- Impact: with zero interval, deadline is constantly due, causing repeated indexing attempts and unnecessary CPU/IO churn.
- Mitigation: treat `0` as disabled and skip timer-triggered indexing entirely.

4. Settings API contract mismatch can erase allowlist data.
- Evidence: `Settings` includes `system_tool_allowlist` in `src-tauri/src/store.rs`; `get_settings_cmd` does not include this field in returned JSON; `src/Settings.vue` expects and saves `system_tool_allowlist`.
- Impact: saving settings can persist an empty allowlist unintentionally, changing indexing behavior for system tools.
- Mitigation: include `system_tool_allowlist` in `get_settings_cmd` payload and add a regression test for round-trip parity.

5. Overly broad local asset protocol scope.
- Evidence: `src-tauri/tauri.conf.json` sets `assetProtocol.scope` to `["**"]`.
- Impact: expands local file exposure surface if any path-building bug occurs in UI or command responses.
- Mitigation: constrain scope to app-owned data roots (for example, icons directory under resolved data dir) and validate icon filenames server-side.

## Medium Risk

6. Unbounded icon extraction thread creation.
- Evidence: `run_full_index` in `src-tauri/src/indexer.rs` spawns one thread per uncached icon (`std::thread::spawn` inside discovery loop).
- Impact: large app sets can create thread storms, increased memory usage, and context-switch overhead.
- Mitigation: replace per-item spawn with bounded worker pool (e.g., rayon or fixed channel + N workers).

7. Silent state reset patterns hide data corruption and operational problems.
- Evidence: `db::init_db` deletes DB on init failure; `store::get_settings` returns defaults on malformed or load failure.
- Impact: users lose launch history/settings without explicit warning or backup.
- Mitigation: create backup before reset (`launcher.db.bak`, `settings.json.bak`) and emit surfaced warning event/toast.

8. Multiple `.unwrap()` calls in runtime paths can crash process on poisoned locks or window-handle failures.
- Evidence: `src-tauri/src/commands.rs`, `search.rs`, `lib.rs` use `.lock().unwrap()` / `launcher.hwnd().unwrap()`.
- Impact: rare runtime errors become hard crashes instead of recoverable failures.
- Mitigation: convert to fallible handling (`map_err`, `if let`, poison recovery strategy) with structured logging.

9. Path exclusion matching is brittle (normalization and case-sensitivity concerns).
- Evidence: `crawl_dir` uses `path.starts_with(ex)` where `ex` is raw `String` from settings.
- Impact: exclusions may fail for case variants, mixed separators, or unnormalized paths, causing unexpected indexing.
- Mitigation: canonicalize both path and excluded roots (or normalize lowercased absolute path strings) before comparison.

10. Symlink-following recursive crawl can expand unexpectedly.
- Evidence: `walkdir::WalkDir::new(root).follow_links(true)` in `crawl_dir`.
- Impact: potentially traverses very large trees or linked directories beyond user intent, increasing latency.
- Mitigation: add max-depth/per-root guards, skip reparse points by default, and expose an opt-in setting for link following.

11. COM initialization lifecycle is unmanaged in shortcut resolution.
- Evidence: `resolve_lnk` calls `CoInitializeEx` and intentionally avoids `CoUninitialize`.
- Impact: thread-lifetime COM state may accumulate and is hard to reason about under frequent calls.
- Mitigation: isolate COM usage in dedicated worker thread with clear init/uninit lifecycle.

12. Long path support for `.lnk` target resolution may be incomplete.
- Evidence: `resolve_lnk` allocates `MAX_PATH` buffer for `GetPath`.
- Impact: long-path targets may be truncated or skipped.
- Mitigation: use APIs/patterns that support extended-length paths and verify behavior with long-path fixtures.

13. System command execution has no explicit confirmation gate.
- Evidence: `run_system_command` directly spawns `shutdown /s` or `shutdown /r`.
- Impact: accidental invocation has immediate disruptive effect.
- Mitigation: require second-step confirmation for irreversible actions (shutdown/restart), configurable in settings.

14. Hotkey update flow unregisters old hotkey before new one is guaranteed.
- Evidence: `hotkey::update_hotkey` unregisters existing shortcut first, then registers new key.
- Impact: brief periods with no valid hotkey if registration path errors unexpectedly.
- Mitigation: two-phase swap when API allows, or rollback registration on failure.

## Low Risk / Debt

15. Search input triggers backend call and resize on every keystroke without debounce.
- Evidence: `src/App.vue` `watch(query, async ...)` invokes `search` and `updateWindowHeight` each change.
- Impact: unnecessary IPC and window resize churn while typing quickly.
- Mitigation: add short debounce (50-120ms) and skip resize when visible row count is unchanged.

16. Verbose runtime `console.log` in production code paths.
- Evidence: many logs in `src/App.vue` around search, sizing, focus, hide/show.
- Impact: noisy logs, minor overhead, harder diagnostics.
- Mitigation: gate logs behind dev flag or centralized logger with levels.

17. Settings persistence command always returns `Ok(())` even if write fails.
- Evidence: `set_settings_cmd` wraps `set_settings` and always returns success; `set_settings` only logs failures.
- Impact: UI can report successful save when persistence failed.
- Mitigation: make `set_settings` fallible and propagate errors to frontend.

18. Limited automated coverage for critical Windows integration paths.
- Evidence: `.lnk` tests in `indexer.rs` are `#[ignore]`; no frontend tests.
- Impact: regressions in indexing/hotkey/window coordination are likely during refactors.
- Mitigation: add integration smoke tests for `.lnk`, settings round-trip, reindex scheduling, and launcher-show workflow.

19. Encoding/mojibake artifacts in comments and UI strings.
- Evidence: multiple files contain `—` / `…` style corruption markers.
- Impact: maintainability/documentation quality issue and potential display inconsistencies.
- Mitigation: normalize file encoding to UTF-8 and lint for invalid replacement sequences.

20. Rust side emits `settings-changed` fields that frontend does not fully consume.
- Evidence: `Settings.vue` emits `{ reindex_interval }`; `App.vue` listener currently applies only `theme` and `show_path`.
- Impact: unclear contract and stale behavior assumptions.
- Mitigation: define typed shared event contract and either consume or stop emitting unused fields.

## Cross-Cutting Recommendation
- Add a small reliability pass before feature work:
  - Fix settings/reindex contract bugs (items 1-4).
  - Add bounded concurrency for indexing (item 6).
  - Tighten security surface (`assetProtocol.scope`) (item 5).
  - Add regression tests for settings round-trip and manual-only indexing behavior.
