---
phase: quick-6
plan: 01
subsystem: settings-ui
tags: [custom-dropdown, settings, vue, css]
dependency_graph:
  requires: []
  provides: [custom-dropdown-component]
  affects: [src/Settings.vue]
tech_stack:
  added: []
  patterns: [inline-custom-dropdown, click-outside-handler, vue-conditional-rendering]
key_files:
  created: []
  modified:
    - src/Settings.vue
decisions:
  - Kept `select, button` CSS selector in place (harmless; only `option:checked` was removed)
  - Used `v-if` (not `v-show`) for dropdown panel — panel is destroyed on close, not hidden
  - `closeAllDropdowns()` called after `onIntervalChange()` inline in template (not inside the handler) to keep the handler reusable
metrics:
  duration: 1min
  completed: "2026-03-08"
  tasks: 1
  files_modified: 1
---

# Phase quick-6 Plan 01: Replace Native Select Elements with Custom Dropdowns — Summary

Custom div-based dropdowns replacing both native `<select>` elements in Settings.vue, giving full control over selected-vs-hover styling that WebView2's native `<select>` cannot provide.

## What Was Done

Both native `<select>` elements in `src/Settings.vue` (Theme and Re-index interval) were replaced with inline custom dropdown components. The implementation is fully self-contained in Settings.vue with no new files.

### Script changes

- Added `openDropdown = ref<string | null>(null)` reactive state
- Added `toggleDropdown(id)` and `closeAllDropdowns()` helpers
- Added `onDocumentClick()` click-outside handler registered on `document`
- Updated `onMounted` to also call `document.addEventListener('click', onDocumentClick)`
- Updated `onUnmounted` to also call `document.removeEventListener('click', onDocumentClick)`
- Replaced `onIntervalChange(e: Event)` with `onIntervalChange(val: number)` — no longer reads from `e.target`

### Template changes

- Theme `<select>` replaced with `.custom-select` div containing a trigger `<button>` and a `v-if` dropdown panel with option divs
- Re-index interval `<select>` replaced with the same pattern, numeric option values, `onIntervalChange(opt.value)` called directly

### CSS changes

- Removed `option:checked { background: var(--color-accent); color: #ffffff; }` block
- Added `.custom-select`, `.custom-select-trigger`, `.custom-select-arrow`, `.custom-select-dropdown`, `.custom-select-option` rules to `<style scoped>`
- Selected state: `var(--color-accent)` background, `#ffffff` text
- Hover state (non-selected): `var(--color-bg-hover, rgba(255, 255, 255, 0.06))` — subtle, not accent

## Verification

`pnpm build` (vue-tsc + vite) exits 0 with no TypeScript errors or warnings.

## Commits

| Hash | Description |
|------|-------------|
| 7f97737 | feat(quick-6): replace native select elements with custom dropdowns |

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- `src/Settings.vue` exists and contains `.custom-select-trigger`, `.custom-select-option`, no `<select>` elements, no `option:checked` CSS rule
- Commit `7f97737` verified in git log
