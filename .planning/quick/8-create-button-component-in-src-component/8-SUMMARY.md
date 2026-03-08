---
phase: quick-8
plan: 01
subsystem: frontend/ui
tags: [component, button, refactor, settings, vue]
dependency_graph:
  requires: []
  provides: [src/components/ui/Button.vue]
  affects: [src/Settings.vue, src/components/ui/PathList.vue]
tech_stack:
  added: []
  patterns: [scoped-component, inheritAttrs-false, variant-prop]
key_files:
  created:
    - src/components/ui/Button.vue
  modified:
    - src/Settings.vue
    - src/components/ui/PathList.vue
decisions:
  - inheritAttrs-false with v-bind="$attrs" ensures click handlers and other attrs are forwarded cleanly to the native button element
  - default variant CSS matches the former .add-btn rules exactly (bg-darker, border, muted text) so PathList.vue render is pixel-identical after replacement
  - Removed generic "select, button" CSS fallback block in Settings.vue — it was the only unscoped base rule; settings-close and reset-link each have explicit overrides that are unaffected
metrics:
  duration: 5min
  completed: "2026-03-09"
  tasks: 2
  files: 3
---

# Phase quick-8 Plan 01: Button Component Summary

**One-liner:** Reusable Button.vue with default/accent variants replaces inline buttons in Settings.vue and PathList.vue.

## What Was Built

A single-file `Button.vue` component in `src/components/ui/` with two variants:

- `default` — bg-darker background, 1px border, muted text; accent border + full text on hover
- `accent` — accent background, white text, no border; opacity fade on hover

Props: `variant` (`default` | `accent`, default `'default'`) and `type` (`button` | `submit` | `reset`, default `'button'`). Uses `inheritAttrs: false` + `v-bind="$attrs"` so click handlers, disabled, and any future attrs bind to the native `<button>` element.

## Changes Made

**src/components/ui/Button.vue** (created)
- Full scoped SFC with `.btn`, `.btn--default`, `.btn--accent` classes
- All styles use CSS custom properties matching the existing design token system

**src/Settings.vue**
- Added `import Button from './components/ui/Button.vue'`
- Re-index Now row: `<button type="button" @click="onReindexNow">` replaced with `<Button variant="accent" @click="onReindexNow">`
- Removed `select, button { ... }` and `select:focus, button:focus { ... }` fallback rule blocks from `<style scoped>` — Button.vue owns those styles now; `reset-link` and `settings-close` have their own explicit rules and are unaffected

**src/components/ui/PathList.vue**
- Added `import Button from './Button.vue'`
- Add Folder: `<button class="add-btn" @click="addPath" type="button">` replaced with `<Button variant="default" @click="addPath">`
- Removed `.add-btn` and `.add-btn:hover` rule blocks from `<style scoped>`
- `.remove-btn` and `.remove-btn:hover` left untouched

## Verification

`pnpm build` exits 0 with no TypeScript errors after both tasks.

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check

- [x] `src/components/ui/Button.vue` exists
- [x] `src/Settings.vue` uses `<Button variant="accent">`
- [x] `src/components/ui/PathList.vue` uses `<Button variant="default">`
- [x] Commit bc0f535 — Task 1
- [x] Commit 8ac4290 — Task 2
- [x] `pnpm build` passes after each task

## Self-Check: PASSED
