---
phase: quick-7
plan: 01
subsystem: frontend-settings
tags: [dropdown, keyboard-navigation, component-extraction, vue]
dependency_graph:
  requires: []
  provides: [src/components/Dropdown.vue]
  affects: [src/Settings.vue]
tech_stack:
  added: []
  patterns: [self-contained-component, v-model, keydown-capture]
key_files:
  created:
    - src/components/Dropdown.vue
  modified:
    - src/Settings.vue
decisions:
  - Dropdown manages its own open/closed state internally — no external openDropdown ref in Settings.vue
  - keydown on root div with preventDefault+stopPropagation for arrow keys when open — prevents Settings window scroll
  - hover, keyboard-highlighted, and selected states all resolve to var(--color-accent)+white (no fallback gray)
  - outside-click handled per-instance via onMounted/onUnmounted document listener with rootEl contains() check
metrics:
  duration: 2min
  completed: 2026-03-08
---

# Quick Task 7: Extract Custom Dropdown into src/components/ Summary

**One-liner:** Reusable Dropdown.vue with self-managed state, accent-color highlight for all states (hover/keyboard/selected), and arrow-key capture that prevents Settings window scroll.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create src/components/Dropdown.vue | 475235b | src/components/Dropdown.vue |
| 2 | Update Settings.vue to use Dropdown component | bb0f62d | src/Settings.vue |

## What Was Built

**Dropdown.vue** (`src/components/Dropdown.vue`):
- Self-contained component with `modelValue` + `options` props and `update:modelValue` emit
- `isOpen` and `highlightedIndex` refs managed internally
- `@keydown` on root div captures ArrowUp/ArrowDown/Enter/Escape with `preventDefault`+`stopPropagation` when open — eliminates Settings window scroll bug
- `highlightedIndex` resets to current value's index on open, -1 on close
- Outside-click handled via `onMounted`/`onUnmounted` document listener checking `rootEl.contains(e.target)`
- All three highlight states (`:hover`, `.highlighted`, `.selected`) use `var(--color-accent)` + `#ffffff` — consistent with context menu and result row convention

**Settings.vue** changes:
- Imported `Dropdown` and replaced both inline dropdown blocks with `<Dropdown v-model="..." @update:modelValue="..."/>`
- Removed: `openDropdown` ref, `toggleDropdown()`, `closeAllDropdowns()`, `onDocumentClick()`, document event listener in `onMounted`/`onUnmounted`
- Removed: all `.custom-select*` CSS rules (63 lines of CSS deleted)
- `onIntervalChange` and `onThemeChange` handlers unchanged — v-model syncs the value before the handler fires

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check

- [x] `src/components/Dropdown.vue` exists
- [x] `src/Settings.vue` has zero `.custom-select*` CSS
- [x] `src/Settings.vue` has zero inline dropdown markup
- [x] `pnpm build` exits 0 with no TypeScript errors
- [x] Commits 475235b and bb0f62d exist

## Self-Check: PASSED
