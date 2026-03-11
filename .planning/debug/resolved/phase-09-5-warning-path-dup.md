---
status: resolved
trigger: "Investigate the Phase 09.5 UAT gap below and find root cause only."
created: 2026-03-12T00:00:00Z
updated: 2026-03-12T01:41:08Z
---

## Current Focus

hypothesis: The recovery banner path is duplicated because both backend recovery warning constructors embed the backup path in `message`, while `App.vue` also renders `warning.backup_path` as a separate line styled with the mono/dim tokens.
test: Compare all backend warning constructors against the launcher banner template and warning CSS.
expecting: Recovery warnings will contain the path once in `message` and a second time in `backup_path`, and the second rendered copy will use `--font-mono` and `--color-text-dim`.
next_action: Record the confirmed root cause for the UAT gap report.

## Symptoms

expected: When a backend recovery warning is present, the launcher shows a visible inline warning banner on open. Dismissing it removes the banner cleanly without leaving broken spacing or blocking search interaction.
actual: The warning displays the path to the backup file twice. The second copy uses a JetBrains-style monospace font and gray color; keep just one backup path and make it lighter, almost white.
errors: None reported.
reproduction: Test 2 in UAT.
started: Discovered during UAT.

## Eliminated

## Evidence

- timestamp: 2026-03-12T00:06:00Z
  checked: src-tauri/src/lib.rs startup_db_warning
  found: The DB recovery warning message formats the backup path directly into `message` with `A backup was saved to {path}`, and also sets `backup_path` to the same file path.
  implication: A DB recovery payload already contains one visible path in prose before the frontend renders anything else.

- timestamp: 2026-03-12T00:07:00Z
  checked: src-tauri/src/store.rs load_settings_outcome recovery warning
  found: The settings recovery warning uses the same pattern, interpolating the backup path into `message` and separately populating `backup_path`.
  implication: The duplication is systemic across backend recovery warnings, not limited to one warning kind.

- timestamp: 2026-03-12T00:08:00Z
  checked: src/App.vue warning banner template
  found: The banner always renders `warning.message` and then conditionally renders `warning.backup_path` in a separate `<span class="warning-path">`.
  implication: Any recovery warning whose message already contains the path will display the path twice.

- timestamp: 2026-03-12T00:09:00Z
  checked: src/App.vue warning styles and src/styles/tokens.css
  found: `.warning-path` uses `font-family: var(--font-mono)` and `color: var(--color-text-dim)`, while tokens define `--font-mono: 'JetBrains Mono', monospace` and `--color-text-dim: #555558` in dark theme.
  implication: The second copy appearing in JetBrains-style mono and gray is expected from the current CSS, not a rendering artifact.

## Resolution

root_cause: Backend recovery warnings carry the backup path twice by design. Both Rust constructors place the path inside the human-readable `message` and also populate `backup_path`, while the launcher template renders both fields. The duplicate line uses the dedicated `.warning-path` style, which is configured as JetBrains Mono with dim gray text.
fix:
verification:
files_changed: []
