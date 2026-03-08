---
phase: quick-4
plan: 4
type: execute
wave: 1
depends_on: []
files_modified: [src/App.vue]
autonomous: true
requirements: []
must_haves:
  truths:
    - "The bottom border of the launcher is visible at all times (empty state, with results)"
    - "The bottom border does not appear/disappear when the context menu is opened"
  artifacts:
    - path: "src/App.vue"
      provides: "Window height includes bottom padding so border renders inside transparent area"
  key_links:
    - from: "updateWindowHeight()"
      to: "getCurrentWindow().setSize()"
      via: "BOTTOM_PAD constant added to all height calculations"
      pattern: "BOTTOM_PAD"
---

<objective>
Fix the invisible bottom border on the launcher window.

Purpose: The bottom 1px CSS border is clipped because the OS window is sized exactly to the content height, placing the border pixel flush at the transparent window edge where WebView2 does not render it reliably. Adding a small constant bottom padding (8px) to every setSize() call gives the border pixel room inside the transparent area.

Output: src/App.vue with consistent BOTTOM_PAD applied to all four height calculation sites.
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/App.vue
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add BOTTOM_PAD constant and apply to all height calculations</name>
  <files>src/App.vue</files>
  <action>
    At the top of the <script setup> section, after the existing constants, add:

    ```ts
    const BOTTOM_PAD = 8 // px — extra transparent space below border so WebView2 renders the bottom 1px border
    ```

    Then update every setSize() height calculation in App.vue to include BOTTOM_PAD:

    1. `updateWindowHeight()` (line ~99):
       Change: `const h = Math.max(56 + listHeight.value, 56)`
       To:     `const h = Math.max(56 + listHeight.value, 56) + BOTTOM_PAD`

    2. `watch(menuVisible)` restore handler (line ~74):
       Change: `const h = Math.max(56 + listHeight.value, 56)`
       To:     `const h = Math.max(56 + listHeight.value, 56) + BOTTOM_PAD`

    3. `onContextMenu()` overflow calculation (line ~205):
       Change: `const contentH = Math.max(56 + listHeight.value, 56)`
       To:     `const contentH = Math.max(56 + listHeight.value, 56) + BOTTOM_PAD`

    4. `launcher-show` listener reset (line ~341):
       Change: `await getCurrentWindow().setSize(new LogicalSize(500, 56))`
       To:     `await getCurrentWindow().setSize(new LogicalSize(500, 56 + BOTTOM_PAD))`

    Do NOT change the CSS. Do NOT change listHeight computation. Do NOT add any margin/padding to .launcher-app CSS — the extra space is transparent OS window area below the component, not visible content.
  </action>
  <verify>
    Run `pnpm build` (vue-tsc --noEmit passes). Then in `pnpm tauri dev`: summon the launcher with Alt+Space, confirm the bottom border is visible in empty state and with results.
  </verify>
  <done>
    Bottom border of .launcher-app is visible at all times. The border does not appear/disappear when right-clicking to open the context menu. `pnpm build` passes with no type errors.
  </done>
</task>

</tasks>

<verification>
Manual check in `pnpm tauri dev`:
1. Summon launcher (Alt+Space) — bottom border visible in empty state
2. Type a query — bottom border visible with results list
3. Right-click — context menu opens, bottom border is still visible (not newly appearing)
4. Close context menu — border still present
</verification>

<success_criteria>
- `pnpm build` exits 0 (no type errors)
- Bottom border is consistently visible in all launcher states
- BOTTOM_PAD = 8 constant defined once and referenced in all four height calculation sites
</success_criteria>

<output>
After completion, create `.planning/quick/4-the-bottom-border-of-the-launcher-is-not/4-SUMMARY.md`
</output>
