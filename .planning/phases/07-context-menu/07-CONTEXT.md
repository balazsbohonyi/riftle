# Phase 7: Context Menu - Context

**Gathered:** 2026-03-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a right-click context menu as a custom Vue HTML overlay on the launcher window with two actions: Settings (opens/focuses settings window) and Quit Launcher (exits the process). All frontend work plus one new Rust command (`quit_app`). No other new Rust commands.

</domain>

<decisions>
## Implementation Decisions

### Escape key behavior
- First Escape press closes the context menu only (menu dismissed, launcher stays visible)
- Second Escape press (or Escape when menu is not open) hides the launcher — existing behavior unchanged
- After menu is dismissed via Escape, focus returns to the search input automatically

### Menu visual style
- Same dark background as launcher (#1c1c1e / #242427 gradient), same 1px rgba(255,255,255,0.15) border, same 9px border-radius — looks like a native part of the launcher
- Text-only menu items — no icons next to labels
- No separator between "Settings" and "Quit Launcher" — two items, clean and minimal
- Hover state: same rgba(10, 132, 255, 0.18) blue highlight used for result row selection — consistent with existing selection style
- Menu is an absolutely-positioned HTML overlay div inside App.vue, positioned at cursor coordinates on right-click

### Right-click trigger scope
- Right-click on the launcher background and search area opens the context menu
- Right-click on result rows (`.result-row`) does nothing — silently ignored (reserved for future per-result context menu)
- Native browser context menu suppressed everywhere on the launcher via `preventDefault()` on the `contextmenu` event at the root element

### Focus loss and dismissal
- Click-outside (anywhere outside the menu overlay div, but still inside the launcher) closes the menu — implemented via a document `mousedown` listener or transparent backdrop
- OS focus loss (e.g. alt-tab while menu open) closes menu AND hides launcher — same as existing auto-hide behavior, no special-casing for menu state
- Menu state resets when the launcher is hidden (so menu is never visible when launcher re-appears via hotkey)

### Quit Launcher action
- New `quit_app` Tauri command in Rust (`commands.rs`) that calls `app_handle.exit(0)`
- Frontend invokes `invoke('quit_app')` when "Quit Launcher" is clicked

### Claude's Discretion
- Exact menu item padding and font size (consistent with Inter / ~13–14px)
- Exact menu width (min-width to fit longest item label)
- z-index layering within App.vue
- Exact backdrop/click-outside implementation approach (backdrop div vs. document listener)

</decisions>

<specifics>
## Specific Ideas

- No specific references — keep the menu as minimal and consistent with the existing launcher aesthetic as possible

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `App.vue` — entire launcher UI; context menu is added as a new conditional overlay div inside the existing template
- `.result-row.selected` style — `rgba(10, 132, 255, 0.18)` blue highlight reused for menu item hover
- `hideWindow()` function — already handles animation and focus; menu state should reset when this is called

### Established Patterns
- Conditional rendering via `v-if` on a `ref<boolean>` — same pattern as other toggle states in App.vue
- `launchInProgress` flag pattern — menu open state should similarly suppress auto-hide side effects if needed
- `invoke()` from `@tauri-apps/api/core` — used for all Tauri commands; `quit_app` follows the same pattern
- `onFocusChanged` listener already wired — no changes needed; focus loss auto-hides launcher (which resets menu state implicitly)

### Integration Points
- `@contextmenu.prevent` on `.launcher-app` root — suppress native menu everywhere
- `@contextmenu.stop` on `.result-row` — prevent bubbling to root handler (so result rows don't open the menu)
- `invoke('quit_app')` — new command, needs to be added to `commands.rs` and registered in `invoke_handler` in `lib.rs`
- `invoke('open_settings_window')` — Phase 8 implements this command; Phase 7 can wire the call (it will be a no-op stub until Phase 8)
- Menu position: captured from `MouseEvent.clientX / clientY` on the contextmenu event; clamped to stay within window bounds

</code_context>

<deferred>
## Deferred Ideas

- Per-result context menu (right-click a result row for "Run as Admin", "Open file location") — future phase

</deferred>

---

*Phase: 07-context-menu*
*Context gathered: 2026-03-08*
