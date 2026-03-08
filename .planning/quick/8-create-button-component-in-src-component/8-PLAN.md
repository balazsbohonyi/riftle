---
phase: quick-8
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/components/ui/Button.vue
  - src/Settings.vue
  - src/components/ui/PathList.vue
autonomous: true
requirements: []
must_haves:
  truths:
    - "Re-index button renders with accent background and white text"
    - "Add Folder button renders with bg-darker background, border, and muted text matching current look"
    - "All replaced buttons behave identically to the original (click events fire, type=button prevents form submit)"
    - "reset-link button and settings-close button are unchanged"
    - "remove-btn in PathList is unchanged"
  artifacts:
    - path: "src/components/ui/Button.vue"
      provides: "Reusable button with default and accent variants"
      exports: ["variant prop (default | accent)", "type prop (default: button)"]
  key_links:
    - from: "src/Settings.vue"
      to: "src/components/ui/Button.vue"
      via: "import Button + <Button variant='accent'>"
    - from: "src/components/ui/PathList.vue"
      to: "src/components/ui/Button.vue"
      via: "import Button + <Button variant='default'>"
---

<objective>
Create a reusable Button.vue component with `default` and `accent` variants, then replace the Re-index button in Settings.vue and the Add Folder button in PathList.vue with it.

Purpose: Unify button styling behind a single component so variant changes propagate everywhere from one source.
Output: src/components/ui/Button.vue, updated Settings.vue, updated PathList.vue
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/Settings.vue
@src/components/ui/PathList.vue
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create Button.vue with default and accent variants</name>
  <files>src/components/ui/Button.vue</files>
  <action>
Create `src/components/ui/Button.vue` as a scoped single-file component:

Props:
- `variant`: `'default' | 'accent'` — default value `'default'`
- `type`: `'button' | 'submit' | 'reset'` — default value `'button'`

Template: a single `<button>` that binds `:type="type"` and forwards all other attrs/listeners via `v-bind="$attrs"` (use `inheritAttrs: false` so the root element receives them cleanly).

Styles (scoped):
```
.btn — shared base
  font-family: var(--font-sans)
  font-size: var(--font-size-sm)
  border-radius: var(--radius-sm)
  padding: var(--spacing-xs) var(--spacing-sm)
  cursor: pointer
  transition: opacity var(--duration-fast), border-color var(--duration-fast)

.btn:focus
  outline: none

.btn--default
  background: var(--color-bg-darker)
  border: 1px solid var(--color-border)
  color: var(--color-text-muted)

.btn--default:hover
  border-color: var(--color-accent)
  color: var(--color-text)

.btn--accent
  background: var(--color-accent)
  border: none
  color: #ffffff

.btn--accent:hover
  opacity: 0.88

.btn--accent:focus
  outline: none
```

Bind classes: `:class="['btn', `btn--${variant}`]"` on the button element.

The `default` variant intentionally matches PathList's current `.add-btn` styling (bg-darker background, border, muted text, accent border on hover). The `accent` variant is for Re-index Now — accent background, white text, no border.
  </action>
  <verify>File exists at src/components/ui/Button.vue and `pnpm build` passes (vue-tsc noEmit)</verify>
  <done>Button.vue exports two visually distinct variants with correct CSS token usage; type-checks clean</done>
</task>

<task type="auto">
  <name>Task 2: Replace buttons in Settings.vue and PathList.vue</name>
  <files>src/Settings.vue, src/components/ui/PathList.vue</files>
  <action>
**Settings.vue changes:**

1. Add `import Button from './components/ui/Button.vue'` to the `<script setup>` imports block.

2. In the template, replace the Re-index Now row button:
   BEFORE: `<button type="button" @click="onReindexNow">{{ reindexButtonText }}</button>`
   AFTER:  `<Button variant="accent" @click="onReindexNow">{{ reindexButtonText }}</Button>`

3. In `<style scoped>`, remove the `select, button { ... }` and `select:focus, button:focus { ... }` base rule blocks. These were the generic button fallback. After replacement, `settings-close` and `reset-link` each have their own explicit style blocks that override everything, so removing the base rule is safe. Leave the `.reset-link`, `.settings-close`, `.hotkey-row` rules intact.

**PathList.vue changes:**

1. Add `import Button from './Button.vue'` to the `<script setup>` imports block.

2. In the template, replace the Add Folder button:
   BEFORE: `<button class="add-btn" @click="addPath" type="button">+ Add folder</button>`
   AFTER:  `<Button variant="default" @click="addPath">+ Add folder</Button>`

3. In `<style scoped>`, remove the `.add-btn` and `.add-btn:hover` rule blocks entirely (the Button component owns that styling now).

4. Leave `.remove-btn` and `.remove-btn:hover` untouched — that button stays as a plain `<button>`.
  </action>
  <verify>`pnpm build` passes with no TypeScript errors; visually confirm Re-index button is accent blue with white text and Add Folder button looks identical to before (bg-darker, border, muted text)</verify>
  <done>Both files use Button component; removed dead CSS; no regressions on reset-link, settings-close, or remove-btn</done>
</task>

</tasks>

<verification>
Run `pnpm build` — must exit 0 with no TS errors.
Visual spot-check in `pnpm tauri dev`: open Settings, confirm Re-index button has accent background + white text; Add Folder buttons have the default bordered look; Reset (hotkey) and × (close) buttons are unaffected.
</verification>

<success_criteria>
- Button.vue exists with default and accent variants
- Re-index Now button in Settings.vue uses `<Button variant="accent">`
- Add Folder button in PathList.vue uses `<Button variant="default">`
- remove-btn, reset-link, settings-close unchanged
- `pnpm build` exits 0
</success_criteria>

<output>
After completion, create `.planning/quick/8-create-button-component-in-src-component/8-SUMMARY.md`
</output>
