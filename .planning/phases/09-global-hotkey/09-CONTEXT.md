# Phase 9: Global Hotkey - Context

**Gathered:** 2026-03-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Register the configured global hotkey at startup, implement toggle show/hide behaviour, clear input and focus on show, and expose an `update_hotkey` Tauri command for runtime hotkey changes (Phase 8 will invoke this). No new UI chrome — only the Rust hotkey logic and the minimal App.vue changes to respond to the `launcher-show` event.

</domain>

<decisions>
## Implementation Decisions

### Animation on re-show
- When the hotkey shows the window, the slide/fade animation **replays** — `isVisible` is reset to `false` then back to `true` via the `launcher-show` event handler
- The animation mode setting (slide/fade/instant) governs both the initial show and every subsequent hotkey-triggered show — same logic throughout
- When the hotkey hides the window (toggle off), the hide animation **plays** before `win.hide()` — same behaviour as Escape-triggered hide (consistent dismissal)

### Signal mechanism (show trigger flow)
- Rust emits a `"launcher-show"` event to the launcher window when the hotkey fires and the window is being shown
- Vue listens for `"launcher-show"` via `listen()` from `@tauri-apps/api/event`
- On receiving `"launcher-show"`: reset `isVisible` to `false`, clear `query.value`, wait one `nextTick` (to let CSS reset), then set `isVisible` to `true` and focus `inputRef`
- No event emitted when hotkey hides the window — Vue handles the hide side via the existing `hideWindow()` function

### Window position
- Call `win.center()` on every hotkey-triggered show before emitting `"launcher-show"`
- Guarantees the launcher appears centered on the primary monitor per HKEY-02, regardless of any prior state

### Startup visibility
- Window stays hidden until first hotkey press — no auto-show on startup
- Remove the `showWindow()` call from `onMounted` for the launcher window path in App.vue
- Keep `inputRef.value?.focus()` inside the `!isTauriContext` dev-mode branch — maintains the browser dev workflow

### HKEY-03: update_hotkey command
- Implement `update_hotkey(hotkey: String)` Tauri command in `hotkey.rs` in Phase 9
- Command: deregister old shortcut → register new shortcut → persist new hotkey to settings.json via `store::set_settings()`
- **Trust the caller** — no format validation; the plugin returns an error if the string is invalid, logged to stderr
- Phase 8 Settings window will invoke this command from the UI

### Claude's Discretion
- Exact Rust API call pattern for `tauri_plugin_global_shortcut` (`on_shortcut` vs builder pattern)
- How to coordinate the hotkey-triggered hide with the existing `onFocusChanged` auto-hide (both calling `hide()` on the window is safe — idempotent)
- Whether `update_hotkey` command stores the previous hotkey string in managed state or reads it from settings.json at deregister time

</decisions>

<specifics>
## Specific Ideas

- The toggle logic is straightforward: `if win.is_visible() { hide } else { center → show → set_focus → emit "launcher-show" }`
- `hideWindow()` in Vue already handles animation + `win.hide()` — hotkey-triggered hide should call into this same path (via the existing Rust event or directly from Vue listening to a `"launcher-hide"` event, or by letting `onFocusChanged` handle it naturally)
- `update_hotkey` is a thin wrapper: `global_shortcut.unregister(old) → global_shortcut.on_shortcut(new, handler) → store.set_settings(...)`

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `hotkey.rs` — currently empty (Phase 9 stub); all hotkey logic goes here
- `showWindow()` in App.vue — existing function; Phase 9 removes its call from `onMounted` but keeps the function for potential other uses
- `hideWindow()` in App.vue — existing function with animation support; hotkey-triggered hide should reuse this logic
- `store::get_settings()` / `store::set_settings()` — used by `update_hotkey` to persist the new hotkey value
- `tauri_plugin_global_shortcut` — already registered in `lib.rs` setup callback via `app.handle().plugin(...)`

### Established Patterns
- `#[cfg(desktop)]` guard in `lib.rs` setup — hotkey registration follows this same pattern
- `app.get_webview_window("launcher")` — window access pattern from Phase 6 launch commands
- `win.hide()` and `win.show()` + `win.set_focus()` — window show/hide from Phase 5/6
- `app.emit_to("launcher", event, payload)` or `win.emit(event, payload)` — event emission pattern
- `listen()` from `@tauri-apps/api/event` — Vue-side event listener (not yet used; new in Phase 9)
- Non-fatal errors via `eprintln!` — established Phase 3/6 pattern for background/command failures
- All new Tauri commands registered in `lib.rs` `invoke_handler`

### Integration Points
- `lib.rs` setup: call `crate::hotkey::register(app.handle(), &settings.hotkey)` after search index init
- `lib.rs` invoke_handler: add `crate::hotkey::update_hotkey`
- `App.vue` onMounted: remove `showWindow()` call for launcher window; add `listen("launcher-show", handler)`
- `App.vue` onUnmounted: unlisten the `"launcher-show"` listener alongside the existing focus listener cleanup

</code_context>

<deferred>
## Deferred Ideas

- None — discussion stayed within phase scope. HKEY-03 UI wiring belongs to Phase 8.

</deferred>

---

*Phase: 09-global-hotkey*
*Context gathered: 2026-03-07*
