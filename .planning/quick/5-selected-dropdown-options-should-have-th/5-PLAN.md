---
phase: quick-5
plan: 5
type: execute
wave: 1
depends_on: []
files_modified:
  - src/Settings.vue
autonomous: true
requirements: []
must_haves:
  truths:
    - "The currently selected option in each dropdown shows accent blue background and white text"
    - "Non-selected options in the open dropdown list retain the dark background and normal text"
  artifacts:
    - path: "src/Settings.vue"
      provides: "option:checked CSS rule"
      contains: "option:checked"
  key_links:
    - from: "src/Settings.vue CSS"
      to: "select > option elements"
      via: "option:checked selector"
      pattern: "option:checked"
---

<objective>
Style the selected option in Settings.vue dropdowns to use the project accent color (blue background, white text) instead of the OS-default gray highlight.

Purpose: Visual consistency — selected state matches the selection highlight used everywhere else in the app (result rows, context menu hover all use var(--color-accent) + #ffffff).
Output: Updated Settings.vue with option:checked CSS rule.
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@src/Settings.vue
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add option:checked CSS rule to Settings.vue</name>
  <files>src/Settings.vue</files>
  <action>
    In the `<style>` block of src/Settings.vue, add an `option:checked` rule immediately after the existing `select, button { ... }` block (after line ~304):

    ```css
    option:checked {
      background: var(--color-accent);
      color: #ffffff;
    }
    ```

    This targets the currently selected option element inside an open native dropdown. `option:checked` is the standard CSS pseudo-class for the selected option and is supported in Chromium (which Tauri's WebView uses). No JavaScript or component changes needed.

    Do NOT use `option:selected` (invalid pseudo-class) or `:focus` variants. Do NOT add `appearance: none` to the select — that would break the native dropdown arrow and open behavior on Windows.
  </action>
  <verify>pnpm build</verify>
  <done>Build passes. Opening either dropdown in the Settings window shows the selected option highlighted with the accent blue background (#0A84FF) and white text instead of the OS gray.</done>
</task>

</tasks>

<verification>
Run `pnpm build` — must complete with no TypeScript or Vite errors.
Manual check: open Settings window, open the Theme or Re-index interval dropdown, confirm selected option shows blue background + white text.
</verification>

<success_criteria>
Selected dropdown options display var(--color-accent) background and #ffffff text, matching the selection highlight style used for result rows and context menu items throughout the app.
</success_criteria>

<output>
After completion, create `.planning/quick/5-selected-dropdown-options-should-have-th/5-SUMMARY.md`
</output>
