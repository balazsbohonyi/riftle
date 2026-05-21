# Quick Task 11 Summary

## Completed

- Added `pin_shortcuts_to_top` to settings, defaulting to `false`.
- Added `shortcut_launches` SQLite persistence for shortcut launch counts.
- Increment shortcut launch counts only after successful shortcut launches.
- Changed search so unpinned shortcuts rank with app results using the same match tier, fuzzy score, and launch-count tiebreaking.
- Kept `>` system-command search isolated from shortcuts and apps.
- Added the `Pin shortcuts to top` toggle under Appearance.

## Verification

- `cd src-tauri && cargo test` passed: 97 tests.
- `pnpm.cmd build` passed.
- `graphify update .` completed.

## Notes

- PowerShell blocked `pnpm build` through `pnpm.ps1`; `pnpm.cmd build` was used instead.
- Commit not created from this worktree because unrelated Rust file diffs are present.
