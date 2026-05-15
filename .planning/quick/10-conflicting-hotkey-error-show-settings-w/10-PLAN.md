---
phase: 10-conflicting-hotkey
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/lib.rs
  - src/Settings.vue
autonomous: true
requirements: [HOTKEY-CONFLICT-01]

must_haves:
  truths:
    - "When the configured hotkey is already taken at startup, Settings opens automatically"
    - "An error message appears below the hotkey field explaining the conflict"
    - "The hotkey section is scrolled into view so the user sees the error immediately"
    - "The error is cleared once the user successfully sets a new hotkey"
  artifacts:
    - path: "src-tauri/src/lib.rs"
      provides: "Emits hotkey-conflict event to settings window and auto-opens Settings on startup failure"
    - path: "src/Settings.vue"
      provides: "Listens for hotkey-conflict event, sets hotkeyError, scrolls hotkey section into view"
  key_links:
    - from: "src-tauri/src/lib.rs"
      to: "src/Settings.vue"
      via: "hotkey-conflict event emitted to 'settings' window"
      pattern: "emit.*hotkey-conflict"
---

<objective>
When hotkey registration fails at startup (e.g. Alt+Space taken by Espanso), show the Settings window automatically with an error message below the hotkey field and scroll it into view. The error clears once a working hotkey is saved.

Purpose: Silent failures leave the user confused — no global shortcut works and there is no indication why.
Output: Settings auto-opens on startup conflict with a clear, actionable error inline below the hotkey field.
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@src-tauri/src/hotkey.rs
@src-tauri/src/lib.rs
@src/Settings.vue
</context>

<tasks>

<task type="auto">
  <name>Task 1: Detect startup hotkey conflict in lib.rs and open Settings with event</name>
  <files>src-tauri/src/lib.rs</files>
  <action>
In lib.rs, after the hotkey registration block (lines ~265-270), detect when a conflict/fallback occurred and trigger the Settings window with an error notification.

Current code (around line 265):
```rust
let actual_hotkey = crate::hotkey::register(app.handle(), &settings.hotkey);
if actual_hotkey != settings.hotkey {
    let mut updated = settings.clone();
    updated.hotkey = actual_hotkey;
    crate::store::set_settings(app.handle(), &data_dir, &updated);
}
```

Extend this block to also detect whether the fallback was triggered because of a conflict with the originally requested key. The `register()` function falls back when the requested key fails — so `actual_hotkey != settings.hotkey` means registration of `settings.hotkey` failed.

Add a new `hotkey_conflict` flag that captures the originally-requested hotkey when a fallback occurred:

```rust
let actual_hotkey = crate::hotkey::register(app.handle(), &settings.hotkey);
let hotkey_conflict = if actual_hotkey != settings.hotkey {
    let failed_key = settings.hotkey.clone();
    let mut updated = settings.clone();
    updated.hotkey = actual_hotkey.clone();
    crate::store::set_settings(app.handle(), &data_dir, &updated);
    Some(failed_key)
} else {
    None
};
```

Then, AFTER the full setup closure (after all managed state, tray, and DWM setup are done — i.e., just before `Ok(())`), open the settings window and emit the event if a conflict was detected:

```rust
if let Some(failed_key) = hotkey_conflict {
    // Show the settings window so the user can rebind immediately
    if let Err(e) = show_settings_window(app.handle()) {
        eprintln!("[startup] could not open settings for hotkey conflict: {}", e);
    }
    // Emit event to settings window — it will display the error inline
    if let Some(settings_win) = app.get_webview_window("settings") {
        let _ = settings_win.emit("hotkey-conflict", &failed_key);
    }
}
```

Place this block just before the final `Ok(())` at the end of the setup closure. The `hotkey_conflict` variable must be declared inside `#[cfg(desktop)]` since `hotkey::register` is desktop-only. To keep things tidy, wrap both the registration block and the conflict show block inside `#[cfg(desktop)]`.

Note: `show_settings_window` already calls `win.center()` on first open, `win.show()`, and `win.set_focus()` — no additional work needed there.
  </action>
  <verify>
    <automated>cd src-tauri && cargo build 2>&amp;1 | tail -20</automated>
  </verify>
  <done>cargo build succeeds with no errors. The conflict detection block compiles cleanly. The hotkey_conflict variable is in scope at the point where settings window is shown.</done>
</task>

<task type="auto">
  <name>Task 2: Handle hotkey-conflict event in Settings.vue with error display and scroll</name>
  <files>src/Settings.vue</files>
  <action>
In Settings.vue, add a listener for the `hotkey-conflict` Tauri event. When received, set `hotkeyError` with an informative message and scroll the hotkey section into view.

**Step 1 — Add a ref for the hotkey section element.**

In the `<script setup>` section, add after the existing refs (around line 51):
```typescript
const hotkeyRowRef = ref<HTMLElement | null>(null)
```

**Step 2 — Add event listener setup in onMounted.**

Add a `hotkey-conflict` event listener inside the `isTauriContext` guard in `onMounted`, AFTER the existing settings load block. Add at the end of the `if (!isTauriContext.value) return` block:

```typescript
// Listen for startup hotkey conflict (emitted by lib.rs when configured key is already taken)
const { listen } = await import('@tauri-apps/api/event')
const unlistenConflict = await listen<string>('hotkey-conflict', (event) => {
  const failedKey = event.payload
  hotkeyError.value = `'${failedKey}' is already in use by another app — please set a different hotkey`
  // Scroll hotkey section into view after DOM updates
  nextTick(() => {
    hotkeyRowRef.value?.scrollIntoView({ behavior: 'smooth', block: 'center' })
  })
})
// Store unlisten for cleanup
unlistenConflictRef = unlistenConflict
```

**Step 3 — Add unlisten ref and clean up in onUnmounted.**

Add at the top of the script (near the `settingsScrollTimer` variable declaration):
```typescript
let unlistenConflictRef: (() => void) | undefined
```

In `onUnmounted`, add:
```typescript
unlistenConflictRef?.()
```

**Step 4 — Wire the template ref to the hotkey Row.**

In the `<template>`, add `ref="hotkeyRowRef"` to the `<Row label="Global shortcut">` element:
```html
<Row label="Global shortcut" ref="hotkeyRowRef">
```

**Step 5 — Verify hotkeyError is cleared on successful hotkey change.**

The existing `onHotkeyChange` already sets `hotkeyError.value = null` at the top of the function — this handles clearing the conflict error too. No change needed there.

**Implementation note:** The `listen` import is dynamic (inside the mounted handler) to stay consistent with the project's pattern of dynamic Tauri API imports (see PathList.vue and the autostart import in Settings.vue). The `unlistenConflict` function must be called on unmount to avoid memory leaks.
  </action>
  <verify>
    <automated>cd D:/develop/projects/riftle && pnpm build 2>&amp;1 | tail -20</automated>
  </verify>
  <done>pnpm build (vue-tsc + vite) completes with no TypeScript errors. The hotkeyRowRef, unlistenConflictRef, and hotkey-conflict listener compile cleanly.</done>
</task>

</tasks>

<verification>
1. `cd src-tauri && cargo test` — all existing tests pass
2. `cd D:/develop/projects/riftle && pnpm build` — TypeScript and Vite build succeed
3. Manual smoke test: temporarily change the configured hotkey in settings.json to a key known to be taken (e.g. set it to a key blocked by another app), restart the app — Settings window should open automatically with the error message visible below the hotkey field
</verification>

<success_criteria>
- On startup with a conflicting hotkey: Settings window opens automatically, error message reads "'[key]' is already in use by another app — please set a different hotkey", hotkey section is visible (scrolled into view)
- Setting a new working hotkey clears the error (hotkeyError becomes null via existing onHotkeyChange logic)
- No regression: normal startup with a working hotkey does not open Settings
- cargo build and pnpm build both pass
</success_criteria>

<output>
After completion, create `.planning/quick/10-conflicting-hotkey-error-show-settings-w/10-SUMMARY.md`
</output>
