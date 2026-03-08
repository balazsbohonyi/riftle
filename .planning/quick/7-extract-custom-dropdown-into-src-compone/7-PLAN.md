---
phase: quick-7
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/components/Dropdown.vue
  - src/Settings.vue
autonomous: true
requirements: []

must_haves:
  truths:
    - "Dropdown.vue exists as a standalone reusable component in src/components/"
    - "Settings.vue uses Dropdown via v-model for both 'theme' and 'reindex_interval' dropdowns"
    - "Arrow Up/Down keys move highlight between options without scrolling the Settings window"
    - "Enter selects the highlighted option and closes the dropdown"
    - "Escape closes the dropdown without changing selection"
    - "Hovered option shows var(--color-accent) background + white text"
    - "Keyboard-highlighted option shows var(--color-accent) background + white text"
    - "Currently selected value option shows accent background when dropdown is closed (in trigger)"
  artifacts:
    - path: "src/components/Dropdown.vue"
      provides: "Reusable dropdown with keyboard navigation and accent highlight"
      exports: ["modelValue", "options"]
    - path: "src/Settings.vue"
      provides: "Settings UI using Dropdown component via v-model"
  key_links:
    - from: "src/Settings.vue"
      to: "src/components/Dropdown.vue"
      via: "v-model binding on modelValue prop"
    - from: "Dropdown.vue keydown handler"
      to: "e.preventDefault() + e.stopPropagation()"
      via: "ArrowUp/ArrowDown/Enter/Escape inside open dropdown"
---

<objective>
Extract the inlined custom dropdown from Settings.vue into a reusable Dropdown.vue component, fix arrow key navigation (currently scrolls Settings window instead of cycling options), and apply accent color to all highlighted states (keyboard + hover).

Purpose: Eliminate code duplication between the two dropdown instances, fix a keyboard UX bug, and ensure consistent accent-color highlight across hover and keyboard navigation.
Output: src/components/Dropdown.vue (new), src/Settings.vue (updated to use it)
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/Settings.vue
@src/components/ui/Toggle.vue

<interfaces>
<!-- Current inline dropdown markup in Settings.vue (two instances: 'interval' and 'theme') -->
<!-- Props to implement: modelValue (any), options: Array<{ value: any, label: string }> -->
<!-- Emits: update:modelValue -->
<!-- Settings.vue currently manages openDropdown ref and toggleDropdown/closeAllDropdowns at the page level -->
<!-- After extraction, each Dropdown instance manages its own open/closed state internally -->

Existing styling tokens (from Settings.vue <style scoped>):
  --color-accent, --color-bg-darker, --color-border, --color-text, --color-text-muted
  --spacing-xs, --spacing-sm, --font-size-sm, --font-sans, --radius-sm, --duration-fast
  var(--color-bg-hover, rgba(255,255,255,0.06))  ← current non-accent hover (to be replaced)
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create src/components/Dropdown.vue</name>
  <files>src/components/Dropdown.vue</files>
  <action>
Create a new file src/components/Dropdown.vue as a self-contained dropdown component. The component manages its own open/closed state (no external openDropdown ref needed).

Props:
  - modelValue: any (required) — currently selected value
  - options: Array<{ value: any; label: string }> (required)

Emits: 'update:modelValue'

Internal refs:
  - isOpen: ref(false)
  - highlightedIndex: ref(-1) — resets to index of current modelValue when dropdown opens; -1 when closed

Template structure (mirror existing markup exactly):
  - Root element: div.custom-select with :class="{ open: isOpen }"
  - Trigger: button.custom-select-trigger @click.stop="toggleDropdown" @keydown="onTriggerKeydown"
    - span showing label for current modelValue (computed from options array)
    - span.custom-select-arrow with &#9660;
  - Dropdown list: div.custom-select-dropdown v-if="isOpen" @keydown.stop
    - div.custom-select-option v-for each option, :key="opt.value"
      - :class="{ selected: opt.value === modelValue, highlighted: index === highlightedIndex }"
      - @click.stop="selectOption(opt.value)"
      - @mouseenter="highlightedIndex = index"

Keyboard handling — attach @keydown on the ROOT div (not just trigger) and call e.preventDefault() + e.stopPropagation() for ArrowUp/ArrowDown/Enter/Escape when isOpen is true (this prevents Settings window scroll):
  - ArrowDown: if closed → open + set highlightedIndex to current value index; if open → increment highlightedIndex (clamp to options.length - 1)
  - ArrowUp: if open → decrement highlightedIndex (clamp to 0)
  - Enter: if open → selectOption(options[highlightedIndex].value) if highlightedIndex >= 0, then close
  - Escape: close dropdown, reset highlightedIndex to -1

selectOption(value): emit('update:modelValue', value); isOpen = false; highlightedIndex = -1

toggleDropdown(): if opening, set highlightedIndex to index of current modelValue (or 0 if not found); toggle isOpen

Close on outside click: use onMounted/onUnmounted to add/remove a document click listener that calls isOpen = false when click target is outside the root element (use a template ref on root div, check !rootEl.value?.contains(e.target)).

Styling — copy ALL existing .custom-select* rules from Settings.vue into a <style scoped> block, then:
  - Replace .custom-select-option:hover rule: background: var(--color-accent); color: #ffffff
  - Add .custom-select-option.highlighted rule: background: var(--color-accent); color: #ffffff
  - Keep .custom-select-option.selected rule as-is (background: var(--color-accent); color: #ffffff)
  - Remove .custom-select-option.selected:hover (redundant — hover rule now also uses accent)
  Note: selected + highlighted + hover all resolve to the same accent/white combination — consistent with context menu and result row convention from MEMORY.md.

The trigger button inherits base button styles from Settings.vue's unscoped `select, button` rule — the component's own styles only need the custom-select-specific rules.
  </action>
  <verify>
    <automated>cd D:/develop/projects/riftle && pnpm build 2>&1 | tail -20</automated>
  </verify>
  <done>src/components/Dropdown.vue exists, builds without TypeScript errors, exports modelValue/options contract</done>
</task>

<task type="auto">
  <name>Task 2: Update Settings.vue to use Dropdown component</name>
  <files>src/Settings.vue</files>
  <action>
Replace both inline dropdown blocks in Settings.vue with the new Dropdown component:

1. Add import at top of <script setup>:
   import Dropdown from './components/Dropdown.vue'

2. Remove the following from Settings.vue (now owned by Dropdown.vue):
   - openDropdown ref
   - toggleDropdown() function
   - closeAllDropdowns() function
   - onDocumentClick() function
   - document.addEventListener('click', onDocumentClick) in onMounted
   - document.removeEventListener('click', onDocumentClick) in onUnmounted
   - All .custom-select* CSS rules in <style scoped>

3. Replace the Re-index interval dropdown block (lines ~233-251) with:
   <Dropdown
     :options="[{ value: 5, label: '5 min' }, { value: 15, label: '15 min' }, { value: 30, label: '30 min' }, { value: 60, label: '60 min' }, { value: 0, label: 'Manual only' }]"
     v-model="settings.reindex_interval"
     @update:modelValue="onIntervalChange"
   />

4. Replace the Theme dropdown block (lines ~260-278) with:
   <Dropdown
     :options="[{ value: 'system', label: 'System' }, { value: 'light', label: 'Light' }, { value: 'dark', label: 'Dark' }]"
     v-model="settings.theme"
     @update:modelValue="onThemeChange"
   />
   Note: onThemeChange already reads settings.value.theme internally — the v-model binding keeps it in sync before the handler fires, so no change to onThemeChange needed.

Keep all other Settings.vue logic, styling, and structure intact. The onKeyDown handler for Escape (closeWindow) remains at the window level — do not remove it.
  </action>
  <verify>
    <automated>cd D:/develop/projects/riftle && pnpm build 2>&1 | tail -20</automated>
  </verify>
  <done>Settings.vue compiles cleanly with no inline dropdown markup. Both dropdowns render via Dropdown component. Arrow keys navigate options without scrolling Settings window. Hover and keyboard highlight both show accent color.</done>
</task>

</tasks>

<verification>
After both tasks:
1. `pnpm build` exits 0 with no TypeScript errors
2. Open settings window in dev: both dropdowns render and open correctly
3. Press ArrowDown/ArrowUp with a dropdown open — highlight moves between options, Settings window does NOT scroll
4. Press Enter on a highlighted option — value updates and dropdown closes
5. Press Escape with dropdown open — dropdown closes, selection unchanged
6. Hover an option — accent background + white text
7. Keyboard-highlighted option — accent background + white text
8. Trigger button shows the current label of the selected value
</verification>

<success_criteria>
- src/components/Dropdown.vue exists and is the single source of dropdown logic
- Settings.vue has zero .custom-select* CSS and zero inline dropdown markup
- Arrow key navigation captured inside dropdown (no Settings window scroll)
- All highlight states (selected, keyboard, hover) use var(--color-accent) + #ffffff per project convention
- `pnpm build` passes
</success_criteria>

<output>
After completion, create .planning/quick/7-extract-custom-dropdown-into-src-compone/7-SUMMARY.md
</output>
