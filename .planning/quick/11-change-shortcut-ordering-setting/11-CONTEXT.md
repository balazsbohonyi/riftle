# Quick Task 11: Change Shortcut Ordering Setting - Context

**Gathered:** 2026-05-21
**Status:** Ready for implementation

<domain>
## Task Boundary

Add a persisted setting that controls whether user shortcuts are pinned above normal app search results.

</domain>

<decisions>
## Implementation Decisions

### Default Behavior
- Persist the new setting as `pin_shortcuts_to_top`.
- Default is `false`.
- When off, shortcuts rank in the same search result ordering model as apps.

### Ranking
- Shortcuts should use the same prefix/acronym/fuzzy match tiers as apps.
- Launch count is a tiebreaker in the same model as app launch counts.
- Shortcut launches need their own persisted launch counts.

### Pinned Mode
- When `pin_shortcuts_to_top` is `true`, shortcuts appear before app results.
- Pinned shortcuts still use ranked ordering within the shortcut block.
- Pinned shortcuts still consume the 50-result cap before app results.

### Counts
- Shortcut launch count increments only after a successful shortcut launch.
- Shortcut identity is path-based: directory path for directory shortcuts, file path plus parameters for file shortcuts. Alias renames keep count.

### System Commands
- `>` system-command searches remain isolated and never include shortcuts or apps.

### Settings UI
- Add the setting under Appearance.
- Label the toggle `Pin shortcuts to top`.

</decisions>

<specifics>
## Specific Ideas

Implementation should be small and aligned with the existing settings/search architecture:
- Add the new field to `Settings`.
- Persist counts where launch counts for searchable entities can be queried by `search`.
- Update Rust unit tests around shortcut ordering and settings compatibility.
- Add the toggle to `Settings.vue`.

</specifics>
