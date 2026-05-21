# Quick Task 11 Plan

## Goal

Make shortcut result ordering configurable from Settings.

## Tasks

- done: Capture behavior decisions in `11-CONTEXT.md`.
- done: Add `pin_shortcuts_to_top` to persisted settings with default `false`.
- done: Add shortcut launch counts keyed by existing shortcut ids.
- done: Rank shortcuts through the same search scoring path as apps when unpinned.
- done: Preserve pinned behavior behind the new setting, while ranking shortcuts inside the pinned block.
- done: Add the Appearance toggle in `Settings.vue`.
- done: Verify with `cargo test`, `pnpm.cmd build`, and `graphify update .`.

## Verification

- `cd src-tauri && cargo test`
- `pnpm.cmd build`
- `graphify update .`
