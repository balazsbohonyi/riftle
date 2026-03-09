# Testing Patterns (Quality Mapping)

## Scope
- This document maps current testing setup, patterns, and major gaps.
- Evidence is drawn from `src-tauri/src/*.rs`, package scripts, and project config.

## Test Execution Setup
- Primary automated tests are Rust unit tests run with Cargo.
- Evidence: command guidance in `CLAUDE.md` (`cd src-tauri && cargo test`).
- Frontend build gate is type-check + bundle, not behavioral testing.
- Evidence: `package.json` script `build: vue-tsc --noEmit && vite build`.
- TypeScript strictness acts as static quality guard.
- Evidence: `tsconfig.json` has `strict`, `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`.
- Rust test-only dependency is declared explicitly.
- Evidence: `tempfile` in `[dev-dependencies]` in `src-tauri/Cargo.toml`.

## Rust Test Distribution
- Database layer has unit tests for schema/init/upsert/query/update behavior.
- Evidence: `src-tauri/src/db.rs` test module (`test_schema_init`, `test_upsert_update_preserves_launch_count`, etc.).
- Settings model has serialization/default tests.
- Evidence: `src-tauri/src/store.rs` tests (`test_settings_defaults`, `test_partial_json_fills_defaults`).
- Search logic has broad ranking and system-command tests.
- Evidence: `src-tauri/src/search.rs` tests (`test_search_prefix_beats_fuzzy`, `test_search_capped_at_50`, system-result tests).
- Indexer has unit tests for crawl/exclude/prune/hash/bootstrap/concurrency guards.
- Evidence: `src-tauri/src/indexer.rs` tests (`test_crawl_discovers_exe`, `test_prune_stale`, `test_atomic_guard_prevents_double_index`).
- Path resolution has basic portable/installed branch tests.
- Evidence: `src-tauri/src/paths.rs` tests.
- Command helper conversion has small pure-function tests.
- Evidence: `src-tauri/src/commands.rs` tests (`test_to_wide_null_*`).

## Testing Style Patterns
- Test modules are colocated with implementation under `#[cfg(test)] mod tests`.
- Evidence: all tested Rust modules above follow this pattern.
- Fixtures are lightweight helper functions in the same module.
- Evidence: `make_app` in `src-tauri/src/search.rs`, `sample_app` in `src-tauri/src/db.rs`, `temp_dir_with_exe` in `src-tauri/src/indexer.rs`.
- Assertions focus on behavior, ranking order, and invariants over snapshots.
- Evidence: ordering assertions in `src-tauri/src/search.rs`; stale-pruning invariants in `src-tauri/src/indexer.rs`.
- In-memory SQLite is preferred for speed and determinism.
- Evidence: `Connection::open_in_memory()` helpers in `src-tauri/src/db.rs` and `src-tauri/src/indexer.rs`.
- Some tests intentionally accept timing variability by asserting non-crash behavior only.
- Evidence: `test_timer_fires` in `src-tauri/src/indexer.rs`.

## Platform/Integration Constraints
- Windows shell shortcut behavior is not fully automated in CI tests.
- Evidence: `#[ignore]` tests in `src-tauri/src/indexer.rs` for `.lnk` scenarios.
- Win32 API operations are mostly untested directly.
- Evidence: no tests in `src-tauri/src/system_commands.rs` and `src-tauri/src/hotkey.rs`.
- Tauri command integration (invoke boundary + managed state wiring) lacks end-to-end automated tests.
- Evidence: no integration test harness under `src-tauri/tests/` and no frontend E2E config.

## Frontend Testing Status
- No dedicated frontend unit test framework is configured.
- Evidence: `package.json` has no `test` script and no Vitest/Jest/Cypress/Playwright dependency.
- No component tests exist for launcher or settings behaviors.
- Evidence: absence of `*.spec.ts`, `*.test.ts`, `tests/`, `__tests__/` under `src/`.
- Runtime behavior relies on manual validation plus static typing.
- Evidence: interactive logic is concentrated in `src/App.vue` and `src/Settings.vue` without automated tests.

## Coverage Gaps (High Impact)
- Hotkey registration fallback and toggle behavior are not directly unit/integration tested.
- Evidence: production logic in `src-tauri/src/hotkey.rs` has no `mod tests`.
- System command dispatch safety and command construction are not test-covered.
- Evidence: `src-tauri/src/system_commands.rs` has no tests.
- Reindex command uses default settings in spawned path, which risks behavioral mismatch.
- Evidence: `reindex` in `src-tauri/src/indexer.rs` constructs `Settings::default()` during manual reindex thread.
- Error-path behavior (DB lock poisoning, failed command spawn, failed Win32 calls) is weakly tested.
- Evidence: many branches log-and-continue (`eprintln!`) without direct failure-case assertions.
- Frontend keyboard navigation, focus-loss auto-hide, and context-menu sizing regressions are unprotected.
- Evidence: logic-heavy handlers in `src/App.vue` have no automated test harness.

## Practical Next Test Priorities
- Add Rust integration tests for command-level IPC behavior (launch/search/settings/update_hotkey) with a minimal app harness.
- Add focused unit tests around `hotkey::register` fallback semantics using abstractions or mockable wrappers.
- Add Windows-conditional integration tests for `.lnk` resolution fixtures to replace ignored cases.
- Add frontend tests for `App.vue` keyboard navigation and `Settings.vue` settings propagation using Vitest + Vue Test Utils.
- Add at least one smoke E2E flow (show launcher -> search -> launch command) for regression coverage.
