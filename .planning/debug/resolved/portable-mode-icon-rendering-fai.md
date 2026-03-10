---
status: resolved
trigger: "Investigate the root cause of the portable-mode icon rendering failure for phase 09.3."
created: 2026-03-11T00:00:00+02:00
updated: 2026-03-11T01:35:00+02:00
---

## Current Focus

hypothesis: Portable-mode icon loading broke because phase 09.3 relied on the `$EXE` asset-scope variable, but Tauri maps `$EXE` to `executable_dir()` and does not support that resolver on Windows.
test: Correlate the portable data path, the hardened scope entry, and Tauri's BaseDirectory implementation/docs for `$EXE`.
expecting: If `$EXE` is unavailable on Windows, the portable scope entry never authorizes `<exe_dir>\data\icons\**`, while installed mode still works through `$DATA`.
next_action: Implement a portable-safe loading path that does not depend on Tauri asset scope for app-managed icons.

## Symptoms

expected: Run Riftle with the portable marker flow so icon assets resolve from the portable data directory, then search for several apps. Icons should still render correctly under the narrowed `$EXE/../data/icons/**` asset scope.
actual: User reported: "in portable mode the icons are broken"
errors: none reported
reproduction: Test 2 in `.planning/phases/09.3-asset-protocol-security-hardening/09.3-UAT.md`
started: After Phase 09.3 asset protocol scope narrowing

## Eliminated

- hypothesis: The Rust-side icon filename validator is rejecting legitimate portable icon filenames.
  evidence: `validate_icon_filename()` only checks the bare filename format in `search.rs`; installed/dev mode still renders icons under the same validator, so the regression is not mode-specific validation fallout.
  timestamp: 2026-03-11T00:00:00+02:00

- hypothesis: The frontend builds a different icon path shape in portable mode than in installed mode.
  evidence: `App.vue` always built `data_dir + /icons/ + icon_path`; `get_settings_cmd` returns the resolved `data_dir` directly, so the frontend path construction itself was mode-agnostic.
  timestamp: 2026-03-11T00:00:00+02:00

## Evidence

- timestamp: 2026-03-11T00:00:00+02:00
  checked: `.planning/phases/09.3-asset-protocol-security-hardening/09.3-UAT.md`
  found: Installed/dev icon rendering passed, but portable-mode icon rendering failed after narrowing the asset protocol scope.
  implication: The regression was specific to the portable path/scope interaction, not generic icon extraction or rendering.

- timestamp: 2026-03-11T00:00:00+02:00
  checked: `src-tauri/tauri.conf.json`
  found: `assetProtocol.scope` allowed only `$DATA/riftle-launcher/icons/**` and `$EXE/../data/icons/**`.
  implication: Portable icon loading depended entirely on the `$EXE...` scope entry resolving correctly on Windows.

- timestamp: 2026-03-11T00:00:00+02:00
  checked: `src-tauri/src/paths.rs`
  found: Portable mode data dir is derived from the executable directory itself: `exe_dir.join("data")` when `exe_dir.join("riftle-launcher.portable").exists()`.
  implication: In portable mode the actual icon files live under `<exe_dir>\data\icons\**` and had to be explicitly allowed by the asset scope.

- timestamp: 2026-03-11T00:00:00+02:00
  checked: `C:\Users\Balazs\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\tauri-2.10.3\src\path\desktop.rs`
  found: Tauri documents `executable_dir()` / `executableDir()` as not supported on Windows.
  implication: The `$EXE`-based asset scope entry used for portable mode was not a reliable Windows path anchor.

## Resolution

root_cause: Phase 09.3 hardened `assetProtocol.scope` to `[$DATA/riftle-launcher/icons/**, $EXE/../data/icons/**]`, but portable mode depended on the `$EXE` branch and Tauri 2.10.3 maps `$EXE` to `executable_dir()`, which is documented as not supported on Windows. As a result, the narrowed asset scope no longer authorized portable `<exe_dir>\data\icons\**` paths even though Riftle still writes icons there and the frontend still requested them correctly.
fix: "Serve app-managed icons through a validated Rust command and frontend blob URL cache, then remove the obsolete assetProtocol dependency and scope."
verification: "`cargo test` and `pnpm.cmd build` pass after replacing convertFileSrc() icon reads with get_icon_bytes() and removing assetProtocol scope. Manual installed/dev and portable smoke tests remain pending."
files_changed:
  - src-tauri/src/commands.rs
  - src-tauri/src/lib.rs
  - src/App.vue
  - src-tauri/tauri.conf.json
  - src-tauri/Cargo.toml
  - src-tauri/Cargo.lock
