# Settings Window Styling Strategy

## Why This Document Exists

Phase 8 introduces a dedicated Settings window — the first time Riftle has more than one UI surface. Before writing any code, we needed to decide how styling would be managed across both the launcher (`App.vue`) and the new settings window (`Settings.vue`), and how that approach would hold up as the settings window grows with more sections and UI element types over time.

The core problem: `App.vue` currently hardcodes every design value (colors, fonts, spacing, radii) inline in its `<style>` block — no CSS custom properties, no shared foundation. A second window built the same way would inevitably drift visually unless there was a disciplined, structural mechanism for sharing the design language.

We also evaluated whether to use a utility framework (Tailwind) or a pre-built component library, and concluded both were wrong for this app: Tailwind's defaults don't match Riftle's specific dark aesthetic and add build complexity, while component libraries ship opinionated visual identities that would require heavy overrides to match the existing look and feel.

The chosen approach — **CSS custom properties (design tokens) + thin Vue component primitives** — keeps Riftle dependency-light, gives us a single source of truth for every design value, and enforces consistency structurally rather than by convention alone.

---

## Context

Phase 8 introduces a separate Tauri window (label: `settings`) with at least 4 sections:
- **General** — startup toggle
- **Hotkey** — key-capture input
- **Search** — path list, re-index button, interval selector
- **Appearance** — theme picker, opacity slider, show_path toggle

Currently `App.vue` has **no CSS custom properties** — all design values are hardcoded inline. The settings window needs to stay visually consistent with the launcher now and as it grows.

## Current Design Tokens (hardcoded in App.vue)

```
Background gradient: #242427 → #1c1c1e → #181818
Text primary:        #f0f0f0
Text muted:          #888 / #555558
Accent:              #0A84FF (iOS blue)
Border:              rgba(255,255,255,0.15)
Selection bg:        rgba(10,132,255,0.18)
Font:                Inter (400,500) + JetBrains Mono (400)
Border-radius:       9px (window), 4px (icons)
```

## Options

### Option A — CSS Custom Properties (Design Tokens) Only
Extract all hardcoded values to `src/styles/tokens.css`. Both App.vue and Settings.vue import it.
Each SFC still owns its own layout/component styles inline.

- ✅ Zero new dependencies
- ✅ Enables future theming (light/dark) via `[data-theme="light"]` overrides
- ✅ All values change in one place
- ⚠️  No built-in UI primitives — each SFC author needs to follow conventions
- ⚠️  Still possible to drift if people write ad-hoc styles

### Option B — CSS Custom Properties + Vue Component Primitives
Same as A, but also create small Vue SFCs for reusable UI elements:
`Section.vue`, `Row.vue`, `Toggle.vue`, etc.

- ✅ Consistency enforced structurally, not just by convention
- ✅ Adding a new settings section = reusing the same building blocks
- ✅ Accessibility can be baked into each primitive once
- ✅ Easy to add new element types later (slider, path list, dropdown)
- ⚠️  Small upfront cost to create the primitives
- ✅ No new dependencies

### Option C — Tailwind CSS
Add Tailwind; style everything with utility classes.

- ✅ Design system baked in (spacing scale, color palette)
- ❌ Tailwind's default palette doesn't match this app's specific dark aesthetic
- ❌ All custom colors/fonts still need to be configured in `tailwind.config`
- ❌ Adds build complexity (PostCSS pipeline)
- ❌ Verbose HTML; clashes with the minimal SFC approach
- ❌ Overkill for a focused utility app

### Option D — Component Library (shadcn-vue / Radix Vue / PrimeVue)
Use a pre-built Vue component library.

- ✅ Free accessible components out of the box
- ❌ All of them ship with their own visual identity — heavy customization required to match this app
- ❌ Large bundle size; significant dependency maintenance burden
- ❌ Harder to maintain the minimal, native-feel aesthetic

## Recommendation: Option B

**CSS custom properties + thin Vue component primitives.**

Reasoning:
1. The app is intentionally minimal and dependency-light — keep it that way
2. Option A alone relies on discipline; Option B enforces consistency structurally
3. The component primitives would be small (20-40 lines each) and purpose-built
4. The tokens layer enables theming later (SETT-07 requires System/Light/Dark theme)
5. New sections are added by composing the same primitives — no style guessing

## Decisions (confirmed with user)

| Question | Answer |
|---|---|
| Tokens scope | **Both windows** — App.vue refactored to use tokens too. Single source of truth from day one. |
| Component location | **`src/components/ui/`** — generic primitives, reusable beyond settings. |
| Token naming | **Semantic scale** — `--color-accent`, `--spacing-md`, etc. (no namespace prefix) |

## Implementation Plan

### Step 1: Create `src/styles/tokens.css`
Extract all hardcoded values from `App.vue` into CSS custom properties on `:root`.

```css
:root {
  /* Backgrounds */
  --color-bg:           #1c1c1e;
  --color-bg-lighter:   #242427;
  --color-bg-darker:    #181818;

  /* Text */
  --color-text:         #f0f0f0;
  --color-text-muted:   #888;
  --color-text-dim:     #555558;

  /* Accent & interactive */
  --color-accent:       #0A84FF;
  --color-selection-bg: rgba(10, 132, 255, 0.18);

  /* Borders */
  --color-border:       rgba(255, 255, 255, 0.15);
  --color-divider:      rgba(255, 255, 255, 0.094);  /* #ffffff18 */

  /* Typography */
  --font-sans:          'Inter', sans-serif;
  --font-mono:          'JetBrains Mono', monospace;
  --font-size-xl:       18px;
  --font-size-base:     14px;
  --font-size-sm:       13px;
  --font-size-xs:       11px;

  /* Spacing */
  --spacing-xs:         4px;
  --spacing-sm:         8px;
  --spacing-md:         12px;
  --spacing-lg:         16px;

  /* Shape */
  --radius:             9px;
  --radius-sm:          4px;

  /* Animation */
  --duration-fast:      120ms;
  --duration-normal:    180ms;
}
```

Import in `src/main.ts` (once, before app mount) — available globally to both windows.

### Step 2: Refactor `App.vue` to use tokens
Replace every hardcoded value in `App.vue`'s `<style>` block with the corresponding CSS variable. No behavior change, purely mechanical substitution.

### Step 3: Create `src/components/ui/` primitives for Phase 8

| Component | Purpose |
|---|---|
| `Section.vue` | Section wrapper: heading + horizontal divider + slot for rows |
| `Row.vue` | Label on left, control slot on right (consistent label+control layout) |
| `Toggle.vue` | Accessible on/off toggle (replaces native checkbox visually) |
| `KeyCapture.vue` | Hotkey capture input (SETT-05) |
| `PathList.vue` | Add/remove folder paths with folder picker (SETT-06) |

Additional primitives added as needed when new sections arrive.

### Step 4: Build `Settings.vue`
New root component for the settings window. Imports and composes the primitives. No inline magic values — all styling via tokens + component classes.

## File Changes Summary

| File | Action |
|---|---|
| `src/styles/tokens.css` | **Create** — all design tokens |
| `src/main.ts` | **Edit** — import tokens.css |
| `src/App.vue` | **Edit** — replace hardcoded values with CSS vars |
| `src/components/ui/Section.vue` | **Create** |
| `src/components/ui/Row.vue` | **Create** |
| `src/components/ui/Toggle.vue` | **Create** |
| `src/components/ui/KeyCapture.vue` | **Create** |
| `src/components/ui/PathList.vue` | **Create** |
| `src/Settings.vue` | **Create** — settings window root |
| `src-tauri/src/lib.rs` | **Edit** — register settings window + open_settings_window command |

## Verification

1. `pnpm build` — no TypeScript errors, no missing imports
2. Visual check: launcher appearance unchanged after App.vue refactor
3. Open settings window — all sections render with consistent styling
4. Toggle, key capture, path list all functional
5. Appearance changes (theme/opacity) reflect live in the open launcher
