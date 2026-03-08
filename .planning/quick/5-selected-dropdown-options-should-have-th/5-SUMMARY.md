---
phase: quick-5
plan: 5
subsystem: frontend/settings
tags: [css, settings, dropdown, ui-polish]
dependency_graph:
  requires: []
  provides: [option:checked CSS rule in Settings.vue]
  affects: [src/Settings.vue]
tech_stack:
  added: []
  patterns: [option:checked pseudo-class for native select styling in Chromium/Tauri WebView]
key_files:
  created: []
  modified:
    - src/Settings.vue
decisions:
  - Used option:checked pseudo-class (Chromium-supported) rather than JS-driven custom dropdown to keep native behavior intact
metrics:
  duration: 3min
  completed: 2026-03-08
---

# Quick Task 5: Style Selected Dropdown Options with Accent Color — Summary

**One-liner:** Added `option:checked` CSS rule to Settings.vue so the selected option in open native dropdowns shows `var(--color-accent)` background with white text, matching the selection highlight used for result rows and context menus.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add option:checked CSS rule to Settings.vue | d72f4fe | src/Settings.vue |

## Changes Made

### src/Settings.vue

Added five lines immediately after the `select:focus, button:focus` block in the scoped style section:

```css
option:checked {
  background: var(--color-accent);
  color: #ffffff;
}
```

`option:checked` is the standard CSS pseudo-class for the currently-selected option element within an open native `<select>` dropdown. It is supported by Chromium (Tauri's WebView) and applies to both the Theme and Re-index interval dropdowns in the Settings window.

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- [x] src/Settings.vue modified and contains `option:checked` rule
- [x] Commit d72f4fe exists
- [x] Build passed with no TypeScript or Vite errors
