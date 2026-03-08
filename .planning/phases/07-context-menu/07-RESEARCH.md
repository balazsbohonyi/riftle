# Phase 7: Context Menu - Research

**Researched:** 2026-03-08
**Domain:** Vue 3 overlay / Tauri v2 Rust command
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Escape key behavior**
- First Escape press closes the context menu only (menu dismissed, launcher stays visible)
- Second Escape press (or Escape when menu is not open) hides the launcher — existing behavior unchanged
- After menu is dismissed via Escape, focus returns to the search input automatically

**Menu visual style**
- Same dark background as launcher (#1c1c1e / #242427 gradient), same 1px rgba(255,255,255,0.15) border, same 9px border-radius
- Text-only menu items — no icons next to labels
- No separator between "Settings" and "Quit Launcher" — two items, clean and minimal
- Hover state: rgba(10, 132, 255, 0.18) blue highlight (same as .result-row.selected)
- Menu is an absolutely-positioned HTML overlay div inside App.vue, positioned at cursor coordinates on right-click

**Right-click trigger scope**
- Right-click on the launcher background and search area opens the context menu
- Right-click on result rows (`.result-row`) does nothing — silently ignored
- Native browser context menu suppressed everywhere via `preventDefault()` on the `contextmenu` event at the root element

**Focus loss and dismissal**
- Click-outside (anywhere outside the menu overlay div, but still inside the launcher) closes the menu
- OS focus loss closes menu AND hides launcher — same as existing auto-hide behavior, no special-casing
- Menu state resets when the launcher is hidden

**Quit Launcher action**
- New `quit_app` Tauri command in Rust (`commands.rs`) that calls `app_handle.exit(0)`
- Frontend invokes `invoke('quit_app')` when "Quit Launcher" is clicked

### Claude's Discretion
- Exact menu item padding and font size (consistent with Inter / ~13–14px)
- Exact menu width (min-width to fit longest item label)
- z-index layering within App.vue
- Exact backdrop/click-outside implementation approach (backdrop div vs. document listener)

### Deferred Ideas (OUT OF SCOPE)
- Per-result context menu (right-click a result row for "Run as Admin", "Open file location") — future phase
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| MENU-01 | Right-click on launcher shows custom HTML Vue overlay, absolutely positioned at cursor | contextmenu event + clientX/clientY positioning pattern documented below |
| MENU-02 | v1 menu items: Settings (opens/focuses settings window) · Quit Launcher (exits process) | quit_app Rust command pattern; open_settings_window stub invoke pattern |
| MENU-03 | Menu dismisses on click-outside or Escape | backdrop div pattern or document mousedown listener; Escape key integration with existing onKeyDown |
</phase_requirements>

---

## Summary

Phase 7 is a minimal, self-contained addition: one new `ref<boolean>` toggling an absolutely-positioned overlay div in `App.vue`, and one new Rust function (`quit_app`) in `commands.rs`. There are no new dependencies, no new Tauri plugins, no new Rust crates. All patterns already exist in the codebase — this phase applies them in a new context.

The implementation fits entirely into two files: `src/App.vue` (template additions, style additions, new state refs, updated `onKeyDown`) and `src-tauri/src/commands.rs` (one new public function). A one-line addition to the `invoke_handler!` macro in `lib.rs` registers the new command.

The CONTEXT.md decisions are exhaustive and leave no open questions. Every implementation detail has been decided. Research here validates those decisions against Tauri v2 and Vue 3 APIs and records the exact code patterns the planner should use.

**Primary recommendation:** Implement as a pure Vue overlay with no third-party context menu library. All decisions are locked. Execute directly against the patterns below.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Vue 3 (existing) | 3.x | Reactive overlay state, template rendering | Already in project |
| @tauri-apps/api/core (existing) | 2.x | `invoke('quit_app')` call | Already in project |

### No New Dependencies
This phase requires zero new npm packages and zero new Rust crates. The implementation is pure Vue template + CSS + one Rust function.

---

## Architecture Patterns

### Recommended Structure

No new files required. Changes go into two existing files:

```
src/
└── App.vue          — add: menuVisible ref, menuX/menuY refs, onContextMenu handler,
                       closeMenu(), onKeyDown update (Escape branch), template overlay div,
                       CSS for .context-menu and .menu-item
src-tauri/src/
└── commands.rs      — add: quit_app() function
└── lib.rs           — add: crate::commands::quit_app to invoke_handler!
```

### Pattern 1: Overlay State (existing ref<boolean> pattern)

**What:** Use `ref<boolean>` for menu visibility, two `ref<number>` for position. Same pattern as `isVisible`, `adminMode`, etc. already in App.vue.

**When to use:** Any conditional overlay in the existing single-component architecture.

```typescript
// Source: existing App.vue ref pattern
const menuVisible = ref(false)
const menuX       = ref(0)
const menuY       = ref(0)
```

### Pattern 2: contextmenu Event Interception

**What:** `@contextmenu.prevent` on the root `.launcher-app` element suppresses the native OS menu everywhere. A separate handler checks the event target to decide whether to open the custom menu.

**When to use:** Root-level suppression is always correct for a launcher window that owns all right-click behavior.

```typescript
// Source: CONTEXT.md integration points + standard Vue 3 event handling
function onContextMenu(e: MouseEvent) {
  // Ignore right-click on result rows — reserved for future per-result menu
  if ((e.target as HTMLElement).closest('.result-row')) return
  menuX.value = e.clientX
  menuY.value = e.clientY
  menuVisible.value = true
}
```

Template binding:
```html
<div class="launcher-app" ... @contextmenu.prevent="onContextMenu">
```

Note: `@contextmenu.prevent` calls `e.preventDefault()` before calling the handler function. The handler receives the event with coordinates already available. No extra `e.preventDefault()` call needed inside the handler.

### Pattern 3: Absolute Positioning from clientX/clientY

**What:** The overlay div is `position: absolute` inside `.launcher-app` (which is `position: relative` or already fills the viewport). `left` and `top` are bound via inline style to `menuX` / `menuY`.

**Clamping:** The window is fixed 500px wide. The menu width should be clamped so it does not overflow. A simple approach: cap `left` at `500 - menuWidth`. At Claude's discretion for exact implementation.

```html
<div
  v-if="menuVisible"
  class="context-menu"
  :style="{ left: menuX + 'px', top: menuY + 'px' }"
>
  <div class="menu-item" @mousedown.prevent="openSettings">Settings</div>
  <div class="menu-item" @mousedown.prevent="quitApp">Quit Launcher</div>
</div>
```

Use `@mousedown.prevent` (not `@click`) on menu items to prevent the launcher from losing focus before the action fires. This mirrors the existing `.result-row @mousedown.prevent` pattern.

### Pattern 4: Click-Outside via Backdrop Div (recommended)

**What:** A transparent full-area backdrop div sits behind the menu. A `mousedown` on the backdrop closes the menu. This is simpler and more reliable than a `document.addEventListener` approach because it does not require cleanup and cannot conflict with the existing `onFocusChanged` listener.

```html
<!-- Backdrop sits behind menu, closes on click -->
<div
  v-if="menuVisible"
  class="menu-backdrop"
  @mousedown.prevent="closeMenu"
></div>
<!-- Menu sits above backdrop -->
<div v-if="menuVisible" class="context-menu" ...>
  ...
</div>
```

```css
.menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 99;           /* above launcher content, below menu */
}

.context-menu {
  position: absolute;
  z-index: 100;          /* above backdrop */
  /* visual styles below */
}
```

`position: fixed; inset: 0` makes the backdrop cover the entire window area regardless of scroll. `@mousedown.prevent` prevents focus from shifting.

Alternative approach (document listener): attach a `mousedown` listener to `document` in `onMounted` that calls `closeMenu()` if the click target is outside the `.context-menu` div. Requires `removeEventListener` in `onUnmounted`. More complex, not preferred given the single-component architecture.

### Pattern 5: Escape Key Integration

**What:** The existing `onKeyDown` function already handles `Escape` by calling `hideWindow()`. This needs to be updated: if the menu is visible, first Escape closes the menu and returns focus to input; second Escape falls through to `hideWindow()`.

```typescript
// Source: modified from existing App.vue onKeyDown
if (e.key === 'Escape') {
  e.preventDefault()
  if (menuVisible.value) {
    closeMenu()          // dismiss menu, return focus to input
    inputRef.value?.focus()
    return               // do NOT hide the launcher
  }
  hideWindow()           // existing behavior when menu is not open
  return
}
```

`closeMenu()` is a simple helper:
```typescript
function closeMenu() {
  menuVisible.value = false
}
```

### Pattern 6: Menu State Reset on Hide

**What:** `menuVisible` must be reset to `false` whenever the launcher is hidden. This ensures the menu is never visible when the launcher reappears via hotkey.

```typescript
// Source: existing App.vue hideWindow pattern
async function hideWindow() {
  menuVisible.value = false   // add this line
  isVisible.value = false
  // ... rest of existing hideWindow body unchanged
}
```

Also reset in the `launcher-show` event listener (which already resets `isVisible`, `results`, `query`):
```typescript
unlistenShow = await listen('launcher-show', async () => {
  menuVisible.value = false   // add this line
  isVisible.value = false
  // ... rest unchanged
})
```

### Pattern 7: quit_app Rust Command

**What:** One new public function in `commands.rs`. Calls `app_handle.exit(0)` which triggers Tauri's `RunEvent::ExitRequested` and `RunEvent::Exit` cleanup before terminating the process.

```rust
// Source: Tauri v2 docs.rs AppHandle / community examples verified via WebSearch
#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}
```

Registration in `lib.rs` `invoke_handler!` — add one line:
```rust
crate::commands::quit_app,
```

Note: `app.exit(0)` is the correct Tauri v2 method. It is documented on `tauri::AppHandle`. No plugin needed — `tauri-plugin-process` is optional and not required when `AppHandle::exit` is available directly. (Confidence: MEDIUM — WebSearch confirmed, official docs.rs build was unavailable but the Tauri v2 official docs page at v2.tauri.app/plugin/process confirms the method exists on AppHandle.)

### Pattern 8: Settings Stub (Phase 8 forward-compatibility)

**What:** The "Settings" menu item calls `invoke('open_settings_window')`. This command does not exist yet (it is implemented in Phase 8). The invoke call should be wrapped in `.catch(console.error)` so the failure is silent during Phase 7.

```typescript
async function openSettings() {
  closeMenu()
  await invoke('open_settings_window').catch(console.error)
}
```

This matches the CONTEXT.md note: "Phase 7 can wire the call (it will be a no-op stub until Phase 8)."

### Pattern 9: Menu Visual CSS

**What:** CSS for the menu overlay consistent with existing launcher aesthetic. Values locked by CONTEXT.md; padding/font-size/width are at Claude's discretion.

```css
.context-menu {
  position: absolute;
  background: linear-gradient(180deg, #242427 0%, #1c1c1e 40%, #181818 100%);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 9px;
  min-width: 160px;        /* discretion: fits "Quit Launcher" label */
  padding: 4px 0;
  z-index: 100;
  overflow: hidden;
}

.menu-item {
  font-family: 'Inter', sans-serif;
  font-size: 13px;         /* discretion: consistent with app-name 14px */
  font-weight: 400;
  color: #f0f0f0;
  padding: 8px 14px;       /* discretion */
  cursor: pointer;
  user-select: none;
}

.menu-item:hover {
  background: rgba(10, 132, 255, 0.18);  /* locked: same as .result-row.selected */
  color: #ffffff;
}
```

### Anti-Patterns to Avoid

- **Using a third-party context menu library:** No dependency needed; the overlay is 20 lines of HTML + 30 lines of CSS.
- **Using `@click` instead of `@mousedown` on menu items:** `@click` fires after `mousedown` + `mouseup`. If the user mousedowns on a menu item and the focus-loss event fires first, the menu closes before `@click` fires. Use `@mousedown.prevent` (same pattern as `.result-row`).
- **Using `@contextmenu.prevent.stop` on result rows:** CONTEXT.md specifies using `@contextmenu.stop` on `.result-row` to prevent the event from bubbling to the root handler. The root handler already has `.prevent` which calls `preventDefault()` globally. Stopping on the row prevents `onContextMenu` from being called.
- **Not resetting menu state on `hideWindow()`:** If `menuVisible` is not reset in `hideWindow()`, the menu will reappear the next time the launcher is shown.
- **Calling `app_handle.exit()` on the main thread from a synchronous context:** In the Tauri command handler, `app.exit(0)` is called from a spawned command thread, which is safe. Do not call it from the main thread setup closure.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Process exit | Custom win32 ExitProcess call | `app.exit(0)` on AppHandle | Tauri runs cleanup hooks; raw ExitProcess skips them |
| Click-outside detection | Complex DOM tree walking | Backdrop div with mousedown handler | Simpler, no cleanup needed, no conflicts |
| Context menu library | Third-party npm package | Pure HTML div + CSS | Zero deps; design is locked; 50 lines total |

---

## Common Pitfalls

### Pitfall 1: mousedown vs click on menu items
**What goes wrong:** Menu item `@click` handler never fires because `mousedown` on the item causes focus to shift away from the launcher window, triggering `onFocusChanged` → `hideWindow()` which destroys the menu before `click` fires.
**Why it happens:** `click` = `mousedown` + `mouseup` on the same element. Focus change fires on `mousedown`.
**How to avoid:** Use `@mousedown.prevent` on menu items (same as `.result-row`). The `.prevent` stops the default browser focus-shift behavior.
**Warning signs:** Menu items appear clickable but nothing happens; hideWindow fires on every right-click attempt.

### Pitfall 2: Backdrop prevents launcher focus loss auto-hide
**What goes wrong:** The backdrop div intercepts `mousedown` events that should dismiss the menu AND allow other elements to receive focus.
**Why it happens:** `@mousedown.prevent` on the backdrop calls `preventDefault()`, which stops focus from shifting to the clicked element.
**How to avoid:** This is actually desired behavior for the backdrop — click-outside should close the menu without doing anything else. The OS focus-loss auto-hide is triggered only when the user clicks completely outside the Tauri window (not within the launcher). Backdrop clicks are inside the launcher; they should only close the menu.

### Pitfall 3: Menu appears at wrong position after window resize
**What goes wrong:** `clientX`/`clientY` coordinates are relative to the browser viewport (the WebView), not the OS screen. Since the launcher window adjusts its height dynamically, coordinates from `clientX`/`clientY` correctly map to the Vue overlay without conversion.
**How to avoid:** Use `clientX`/`clientY` directly as `left`/`top` in the overlay style. Do NOT use `screenX`/`screenY` (those are OS screen coordinates, useless for CSS positioning).
**Warning signs:** Menu appears offset from where the user right-clicked.

### Pitfall 4: z-index stacking order
**What goes wrong:** Menu appears behind the result list's `RecycleScroller` content.
**Why it happens:** Stacking contexts in CSS; `RecycleScroller` may create its own stacking context.
**How to avoid:** Assign `z-index: 100` to `.context-menu` and `z-index: 99` to `.menu-backdrop`. The `.launcher-app` root div should have `position: relative` (already established by the existing layout — verify during implementation).
**Warning signs:** Menu items are invisible or partially obscured by result rows.

### Pitfall 5: Escape key swallowed by input
**What goes wrong:** The search `<input>` may intercept `keydown` events before they reach the root handler, depending on event propagation.
**Why it happens:** The `@keydown` handler is on the `<input>` element. Escape pressed while the menu is open needs to be caught there, since the input has focus.
**How to avoid:** The `onKeyDown` function is already bound to the search input via `@keydown="onKeyDown"`. The Escape branch in `onKeyDown` must check `menuVisible.value` first. This is correct because focus returns to the input after menu open (or the input retains focus since `.prevent` on backdrop/menu-items prevents focus shift).
**Warning signs:** Escape hides the entire launcher when the menu is open, instead of just dismissing the menu.

---

## Code Examples

Verified patterns from official sources and existing codebase:

### Rust quit_app command (commands.rs addition)
```rust
// Source: Tauri v2 AppHandle::exit — confirmed via v2.tauri.app docs and WebSearch
#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}
```

### lib.rs invoke_handler addition (one line)
```rust
// Add to the existing generate_handler! list in lib.rs
crate::commands::quit_app,
```

### Vue: full menu state block (script setup addition)
```typescript
// Menu state
const menuVisible = ref(false)
const menuX       = ref(0)
const menuY       = ref(0)

function closeMenu() {
  menuVisible.value = false
}

function onContextMenu(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('.result-row')) return
  menuX.value = e.clientX
  menuY.value = e.clientY
  menuVisible.value = true
}

async function openSettings() {
  closeMenu()
  await invoke('open_settings_window').catch(console.error)
}

async function quitApp() {
  closeMenu()
  await invoke('quit_app').catch(console.error)
}
```

### Vue: template additions (inside .launcher-app div)
```html
<!-- Backdrop: click-outside closes menu -->
<div
  v-if="menuVisible"
  class="menu-backdrop"
  @mousedown.prevent="closeMenu"
></div>

<!-- Context menu overlay -->
<div
  v-if="menuVisible"
  class="context-menu"
  :style="{ left: menuX + 'px', top: menuY + 'px' }"
>
  <div class="menu-item" @mousedown.prevent="openSettings">Settings</div>
  <div class="menu-item" @mousedown.prevent="quitApp">Quit Launcher</div>
</div>
```

### Vue: root element contextmenu binding
```html
<!-- Add @contextmenu.prevent="onContextMenu" to existing root div -->
<div class="launcher-app" :class="[`anim-${animMode}`, { visible: isVisible }]"
     @contextmenu.prevent="onContextMenu">
```

### Vue: result-row contextmenu stop (prevents menu from opening on row right-click)
```html
<!-- Add @contextmenu.stop to existing result-row div -->
<div
  class="result-row"
  :class="{ selected: active && index === selectedIndex }"
  @mousedown.prevent="launchItem(item)"
  @mousemove="active && (selectedIndex = index)"
  @contextmenu.stop
>
```

Note: `@contextmenu.stop` without a handler is valid Vue — it calls `stopPropagation()` to prevent the event from reaching `.launcher-app`'s `onContextMenu` handler.

### Vue: updated onKeyDown Escape branch
```typescript
if (e.key === 'Escape') {
  e.preventDefault()
  if (menuVisible.value) {
    closeMenu()
    inputRef.value?.focus()
    return
  }
  hideWindow()
  return
}
```

### Vue: hideWindow menu state reset (add one line)
```typescript
async function hideWindow() {
  menuVisible.value = false   // NEW: reset menu on hide
  isVisible.value = false
  // ... rest of existing hideWindow body unchanged
}
```

### Vue: launcher-show listener menu state reset (add one line)
```typescript
unlistenShow = await listen('launcher-show', async () => {
  menuVisible.value = false   // NEW: reset menu on launcher show
  isVisible.value = false
  // ... rest unchanged
})
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `tauri::process::exit()` (v1) | `AppHandle::exit(0)` (v2) | Tauri v2 release | Must use AppHandle method, not process crate |
| `tauri-plugin-process` (optional) | `AppHandle::exit(0)` built-in | Tauri v2 | No plugin needed for basic exit |

---

## Open Questions

1. **`position: relative` on `.launcher-app`**
   - What we know: `.context-menu` uses `position: absolute` which requires an ancestor with `position: relative` (or `absolute`/`fixed`) to be the positioning context.
   - What's unclear: `.launcher-app` currently has no explicit `position` declaration in its CSS (only `width: 100%`, `height: 100%`, background, border-radius).
   - Recommendation: Add `position: relative` to `.launcher-app` CSS rule during implementation. This is a one-line safe addition with no visual impact.

2. **Menu overflow clamping**
   - What we know: Window is 500px wide. Menu appears at `clientX`. If right-clicked near the right edge, menu would overflow.
   - What's unclear: Whether overflow clamping is required or if the window's `overflow: hidden` (currently commented out in `.launcher-app`) handles it.
   - Recommendation: Apply simple clamping in `onContextMenu`: `menuX.value = Math.min(e.clientX, 500 - 170)` (assuming ~170px menu width). Adjust constant once menu width is determined during implementation.

---

## Sources

### Primary (HIGH confidence)
- Existing `src/App.vue` — all Vue patterns extracted from live codebase
- Existing `src-tauri/src/commands.rs` — Rust command pattern (`#[tauri::command] pub fn`, `tauri::AppHandle` parameter)
- Existing `src-tauri/src/lib.rs` — `invoke_handler!` registration pattern
- `07-CONTEXT.md` — all implementation decisions locked by user

### Secondary (MEDIUM confidence)
- [Tauri v2 Process Plugin docs](https://v2.tauri.app/plugin/process/) — confirms `AppHandle::exit()` exists and is the standard quit mechanism
- [Tauri AppHandle docs.rs](https://docs.rs/tauri/latest/tauri/struct.AppHandle.html) — attempted; build failed for 2.10.3; method confirmed via community sources
- [WebSearch: Tauri v2 quit_app command examples](https://github.com/tauri-apps/tauri/discussions/3555) — multiple community examples confirm `app_handle.exit(0)` pattern

### Tertiary (LOW confidence)
- None — all claims are verified against the existing codebase or official Tauri v2 sources.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; verified against existing codebase
- Architecture: HIGH — all patterns are already proven in the existing single-component design
- Pitfalls: HIGH — derived from existing code patterns (mousedown vs click) and CSS fundamentals (z-index, positioning)
- Rust quit_app: MEDIUM — `AppHandle::exit(0)` confirmed via official docs and community examples; docs.rs build unavailable for 2.10.3

**Research date:** 2026-03-08
**Valid until:** 2026-09-08 (stable APIs; Tauri v2 + Vue 3 are stable)
