---
phase: quick-6
plan: 01
type: execute
wave: 1
depends_on: []
files_modified: [src/Settings.vue]
autonomous: true
requirements: []

must_haves:
  truths:
    - "Theme dropdown shows selected value with accent background and white text"
    - "Re-index interval dropdown shows selected value with accent background and white text"
    - "Hovered (non-selected) option shows a subtle highlight, not accent"
    - "Clicking outside an open dropdown closes it"
    - "Selecting an option closes the dropdown and triggers the same save logic as before"
    - "The old option:checked CSS rule is removed"
  artifacts:
    - path: "src/Settings.vue"
      provides: "Custom dropdown replacing both native <select> elements"
      contains: "CustomDropdown"
  key_links:
    - from: "CustomDropdown (theme)"
      to: "onThemeChange()"
      via: "v-model + @change watcher or direct call on option select"
    - from: "CustomDropdown (reindex_interval)"
      to: "onIntervalChange()"
      via: "direct call passing numeric value on option select"
---

<objective>
Replace both native `<select>` elements in Settings.vue (Theme and Re-index interval) with a custom dropdown built from divs, giving full control over selected-vs-hover styling that Chromium's native `<select>` cannot provide.

Purpose: The native `<select>` in WebView2 does not allow independent styling of the selected option vs. a hovered option — `option:checked` also matches hover, making it impossible to show a distinct hover state. A custom dropdown solves this.

Output: Settings.vue with an inline CustomDropdown implementation (no new file), `option:checked` CSS rule removed, both dropdowns visually consistent with existing Settings styling.
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md

Key styling conventions from project memory:
- Selection highlight: solid `var(--color-accent)` background, `#ffffff` text
- Hover (non-selected): subtle, NOT accent — use `var(--color-bg-hover)` or a slightly lighter shade
- Inputs match: `background: var(--color-bg-darker)`, `border: 1px solid var(--color-border)`, `border-radius: var(--radius-sm)`, `padding: var(--spacing-xs) var(--spacing-sm)`
- Focus ring: `border-color: var(--color-accent)`
- Font: `var(--font-sans)`, `var(--font-size-sm)`
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement CustomDropdown and replace both selects</name>
  <files>src/Settings.vue</files>
  <action>
In `src/Settings.vue`, implement a self-contained custom dropdown inline (no separate file). Follow these exact steps:

**1. Add reactive state for open dropdowns** (in `<script setup>`):

```ts
const openDropdown = ref<string | null>(null)

function toggleDropdown(id: string) {
  openDropdown.value = openDropdown.value === id ? null : id
}

function closeAllDropdowns() {
  openDropdown.value = null
}
```

**2. Add a click-outside handler** that closes dropdowns when clicking anywhere outside `.custom-select`:

```ts
function onDocumentClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.custom-select')) {
    closeAllDropdowns()
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown)
  document.addEventListener('click', onDocumentClick)
  // ... existing onMounted body ...
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
  document.removeEventListener('click', onDocumentClick)
})
```

**3. Update `onIntervalChange`** — it currently reads from `(e.target as HTMLSelectElement).value`. Replace it with a version that accepts the value directly:

```ts
async function onIntervalChange(val: number) {
  settings.value.reindex_interval = val
  await saveSettings()
  await emitTo('launcher', 'settings-changed', { reindex_interval: val }).catch(console.error)
}
```

**4. Replace the Theme `<select>` in `<template>`**:

Old:
```html
<select v-model="settings.theme" @change="onThemeChange">
  <option value="system">System</option>
  <option value="light">Light</option>
  <option value="dark">Dark</option>
</select>
```

New:
```html
<div class="custom-select" :class="{ open: openDropdown === 'theme' }">
  <button
    type="button"
    class="custom-select-trigger"
    @click.stop="toggleDropdown('theme')"
  >
    <span>{{ { system: 'System', light: 'Light', dark: 'Dark' }[settings.theme] }}</span>
    <span class="custom-select-arrow">&#9660;</span>
  </button>
  <div class="custom-select-dropdown" v-if="openDropdown === 'theme'">
    <div
      v-for="opt in [{ value: 'system', label: 'System' }, { value: 'light', label: 'Light' }, { value: 'dark', label: 'Dark' }]"
      :key="opt.value"
      class="custom-select-option"
      :class="{ selected: settings.theme === opt.value }"
      @click.stop="settings.theme = opt.value; onThemeChange(); closeAllDropdowns()"
    >{{ opt.label }}</div>
  </div>
</div>
```

**5. Replace the Re-index interval `<select>` in `<template>`**:

Old:
```html
<select :value="settings.reindex_interval" @change="onIntervalChange">
  <option value="5">5 min</option>
  <option value="15">15 min</option>
  <option value="30">30 min</option>
  <option value="60">60 min</option>
  <option value="0">Manual only</option>
</select>
```

New:
```html
<div class="custom-select" :class="{ open: openDropdown === 'interval' }">
  <button
    type="button"
    class="custom-select-trigger"
    @click.stop="toggleDropdown('interval')"
  >
    <span>{{ { 5: '5 min', 15: '15 min', 30: '30 min', 60: '60 min', 0: 'Manual only' }[settings.reindex_interval] }}</span>
    <span class="custom-select-arrow">&#9660;</span>
  </button>
  <div class="custom-select-dropdown" v-if="openDropdown === 'interval'">
    <div
      v-for="opt in [{ value: 5, label: '5 min' }, { value: 15, label: '15 min' }, { value: 30, label: '30 min' }, { value: 60, label: '60 min' }, { value: 0, label: 'Manual only' }]"
      :key="opt.value"
      class="custom-select-option"
      :class="{ selected: settings.reindex_interval === opt.value }"
      @click.stop="onIntervalChange(opt.value); closeAllDropdowns()"
    >{{ opt.label }}</div>
  </div>
</div>
```

**6. Remove the `option:checked` CSS rule** (lines 312–315 in current file):

```css
/* DELETE this entire block: */
option:checked {
  background: var(--color-accent);
  color: #ffffff;
}
```

**7. Add custom dropdown CSS** in `<style scoped>` (do NOT add to the unscoped `<style>` block):

```css
/* Custom dropdown */
.custom-select {
  position: relative;
  display: inline-block;
  min-width: 120px;
}

.custom-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-sm);
  width: 100%;
  /* inherit button base styles from existing select, button rule */
}

.custom-select-arrow {
  font-size: 10px;
  opacity: 0.6;
  pointer-events: none;
  transition: transform var(--duration-fast);
}

.custom-select.open .custom-select-arrow {
  transform: rotate(180deg);
}

.custom-select-dropdown {
  position: absolute;
  top: calc(100% + 2px);
  left: 0;
  right: 0;
  background: var(--color-bg-darker);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  overflow: hidden;
  z-index: 100;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.custom-select-option {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: var(--font-size-sm);
  font-family: var(--font-sans);
  cursor: pointer;
  color: var(--color-text);
  transition: background var(--duration-fast);
}

.custom-select-option:hover {
  background: var(--color-bg-hover, rgba(255, 255, 255, 0.06));
}

.custom-select-option.selected {
  background: var(--color-accent);
  color: #ffffff;
}

.custom-select-option.selected:hover {
  background: var(--color-accent);
  color: #ffffff;
}
```

Note: The existing `select, button { ... }` rule already covers `.custom-select-trigger` appearance because `.custom-select-trigger` is a `button`. No need to duplicate those styles.

Note on `--color-bg-hover`: if this CSS variable is not defined in the project's design tokens, use `rgba(255, 255, 255, 0.06)` as the fallback directly in the `:hover` rule (as shown above with the `var()` fallback syntax). Do NOT add a new CSS variable definition.
  </action>
  <verify>
    <automated>cd D:/develop/projects/riftle && pnpm build 2>&1 | tail -20</automated>
  </verify>
  <done>
- `pnpm build` (vue-tsc + vite) exits 0 with no TypeScript errors
- Both `&lt;select&gt;` elements are gone from Settings.vue template
- `option:checked` CSS rule is gone from Settings.vue
- Custom dropdown CSS and script logic is present in Settings.vue
- No new files created
  </done>
</task>

</tasks>

<verification>
Run `pnpm build` from the project root. It must exit 0. The build output confirms TypeScript types are satisfied and the Vue template compiles without errors.

Visual spot-check (optional, not a blocking checkpoint): Open settings in dev mode, verify theme and interval dropdowns open on click, close on outside click, show accent on selected, show subtle hover on non-selected.
</verification>

<success_criteria>
- Both native `<select>` elements replaced with custom div-based dropdowns
- Selected option: `var(--color-accent)` background, `#ffffff` text
- Hovered non-selected option: subtle background, clearly NOT accent
- Click outside closes the dropdown
- All existing save/emit logic preserved (onThemeChange, onIntervalChange)
- `option:checked` CSS rule removed
- `pnpm build` passes with no errors
</success_criteria>

<output>
After completion, create `.planning/quick/6-replace-native-select-elements-in-settin/6-SUMMARY.md` with what was done, files changed, and any decisions made.
</output>
