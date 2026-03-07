---
phase: quick
plan: 2
type: execute
wave: 1
depends_on: []
files_modified:
  - src/App.vue
  - src-tauri/tauri.conf.json
autonomous: true
requirements: []
must_haves:
  truths:
    - "When the launcher window appears via hotkey or dev mode, the search input has focus immediately"
    - "User can type into the search field without clicking"
    - "The blinking text cursor is visible in the search input"
  artifacts:
    - path: "src/App.vue"
      provides: "Updated showWindow and onMounted with setFocus + input.focus() after win.show()"
  key_links:
    - from: "src/App.vue showWindow()"
      to: "getCurrentWindow().setFocus()"
      via: "called immediately after win.show()"
      pattern: "win\\.setFocus"
---

<objective>
Fix the launcher window so the search input receives focus when the window appears — no mouse click required.

Purpose: The core promise of a keyboard launcher is that it is fully keyboard-driven from the moment it appears. Without input focus, users are forced to click before they can type, breaking the flow.
Output: Updated App.vue where `showWindow()` calls `win.setFocus()` after `win.show()`, and `onMounted` re-focuses the input element after the window is visible. The `"focus": false` config entry in tauri.conf.json is changed to `true` (or removed) so the OS respects the window's focus request on first show.
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@D:\develop\projects\riftle\.planning\STATE.md
@D:\develop\projects\riftle\src\App.vue
@D:\develop\projects\riftle\src-tauri\tauri.conf.json
</context>

<tasks>

<task type="auto">
  <name>Task 1: Fix focus acquisition in showWindow and onMounted</name>
  <files>src/App.vue, src-tauri/tauri.conf.json</files>
  <action>
Two changes are required:

**1. tauri.conf.json — change `"focus": false` to `"focus": true` on the launcher window.**

The `"focus": false` config tells the OS to show the window *without* stealing focus. This is the root cause: even if the frontend calls `inputRef.value?.focus()`, the OS window never activates so keystrokes go to whatever was focused before. Change:

```json
"focus": false
```
to:
```json
"focus": true
```

on the launcher window entry only (not settings).

**2. src/App.vue — update `showWindow()` to call `win.setFocus()` after `win.show()`.**

The current `showWindow()` only calls `win.show()`. After showing, explicitly call `win.setFocus()` so the OS activates the window (brings it to the foreground and captures keyboard input). Then, in `onMounted`, move the `inputRef.value?.focus()` call to AFTER `showWindow()` returns, so the DOM focus is set on the now-active window.

Current `onMounted` order (problematic):
```
1. inputRef.value?.focus()   ← focuses input in a hidden window
2. isVisible.value = true
3. showWindow()              ← shows window, but focus goes back to OS
```

New `onMounted` order:
```
1. isVisible.value = true
2. showWindow()              ← shows window AND steals OS focus
3. await nextTick()
4. inputRef.value?.focus()   ← now focuses input in the active window
```

Update `showWindow()` in `src/App.vue`:
```typescript
async function showWindow() {
  if (!isTauriContext.value) return
  try {
    const win = getCurrentWindow()
    await win.show()
    await win.setFocus()
  } catch (e) {
    console.error('[App] showWindow failed:', e)
  }
}
```

Update the relevant section of `onMounted` (the launcher-window block):
```typescript
// Show the Tauri window only for the launcher
if (isTauriContext.value) {
  const win = getCurrentWindow()
  if (win.label === 'launcher') {
    await showWindow()
    await nextTick()
    inputRef.value?.focus()
  }
} else {
  // Browser dev mode: just focus the input directly
  inputRef.value?.focus()
}
```

Remove the earlier standalone `inputRef.value?.focus()` call that runs before `showWindow()` — it is now replaced by the post-show call above.

Also remove or update the surrounding comment that references the early focus call.
  </action>
  <verify>
    <automated>cd D:\develop\projects\riftle && pnpm build 2>&1 | tail -5</automated>
  </verify>
  <done>
Type-check passes with no errors. When running `pnpm tauri dev`, the launcher window appears with a blinking cursor in the search input — no mouse click needed. Typing immediately produces search results.
  </done>
</task>

</tasks>

<verification>
1. `pnpm build` (vue-tsc --noEmit + vite build) passes with no TypeScript errors
2. `pnpm tauri dev` starts without Rust compile errors
3. On launch, the window appears and the input immediately has focus (cursor visible, typing works)
4. Auto-hide on focus loss (clicking outside) still works — the `onFocusChanged` listener is unchanged
5. The settings window is unaffected (`"focus": false` on launcher was the only config change, settings window entry is untouched)
</verification>

<success_criteria>
- Launcher window appears with the text cursor blinking in the search input
- User can type an app name immediately after the launcher appears — no mouse interaction required
- pnpm build passes with zero errors
- Auto-hide-on-focus-loss behavior is preserved
</success_criteria>

<output>
After completion, create `.planning/quick/2-when-the-launcher-appears-it-does-not-ha/2-SUMMARY.md` following the summary template.
</output>
