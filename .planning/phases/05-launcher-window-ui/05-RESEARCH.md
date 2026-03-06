# Phase 5: Launcher Window UI - Research

**Researched:** 2026-03-06
**Domain:** Vue 3 + Tauri v2 frontend — frameless UI, virtualised list, keyboard navigation, window management
**Confidence:** HIGH (core patterns), MEDIUM (setSize animation path)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Visual Style**
- Background: Dark gradient — #242427 (input area top) fading to #1c1c1e (base) to ~#181818 (bottom). No glass/blur.
- Selection highlight: Blue accent background on selected row (~#0A84FF or #1E6FD9 at ~20% opacity)
- Border: 1px semi-transparent white border (#ffffff25) on launcher window edges
- Input/results separator: 1px divider line (#ffffff18) between search input and result list
- Text: App name #f0f0f0 (unselected), #ffffff (selected); path line #888–#999 (dimmed)
- Theme: Always dark in Phase 5 — no system theme detection
- Fonts: Inter (bundled via Vite) for all text; JetBrains Mono (bundled) for path lines specifically

**Result Row Density**
- Icon size: 32×32px
- Row height: ~48px
- Path line: Selected row only when `show_path` is true; unselected rows never show path
- Path text: JetBrains Mono, ~12px, #888, truncated with ellipsis
- Admin badge: Shield icon + "Admin" label on right side of selected row when Ctrl+Shift held; floats in right margin (no layout shift)
- Search input area height: 52–56px
- Search input font: Inter, 18–20px

**Search Input Design**
- Magnifier icon: Flat monochrome SVG, right-aligned in input area
- Input background: Blends into launcher background (no separate border/pill)
- Placeholder: "Search apps, or > for system commands…"
- Standard text cursor; input is bare/borderless

**Transitions & Animation**
- Three configurable modes from Settings `animation` field (default `"slide"`):
  - `"instant"` — no animation
  - `"fade"` — opacity 0→1 / 1→0 (~120ms)
  - `"slide"` — slide down + fade in on show, reverse on hide (~180ms) (default)
- Phase 5 implements all 3 modes; reads `animation` setting; Phase 8 exposes picker in UI
- Result list update: instant replace on each keystroke
- Keyboard navigation: instant highlight change
- Window height: animated resize as result count changes (150ms ease-out)

**System Commands Display**
- System command results use ⚙️ icon (from `system_command.png`, Phase 4)
- No path line for system commands regardless of `show_path`
- Same row density and selection style as app results

**Empty/Loading States**
- No query: empty list, window shows input area only (minimum height)
- Icon loading: show generic icon placeholder; swap when `{data_dir}/icons/{app_id}.png` ready
- No results: empty list, window collapses to input-only height
- No spinner/skeleton rows

### Claude's Discretion
- Exact blue accent hex (~#0A84FF / #1E6FD9)
- Gradient stop values and easing
- Inter and JetBrains Mono weight choices (400/500 for names, 400 for paths)
- Virtualisation approach (vue-virtual-scroller, @tanstack/virtual, or manual)
- Exact Tauri window resize mechanism (setSize or CSS height transition)
- Icon image loading/caching strategy (img src via convertFileSrc or asset protocol)

### Deferred Ideas (OUT OF SCOPE)
- Animation mode picker UI (instant/fade/slide) → Phase 8 Appearance section
- `animation` field addition to Settings struct → can be done in Phase 5 (Rust side) or deferred to Phase 8 (note for planner)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LWND-01 | Frameless floating window, centered on primary monitor, always-on-top, no taskbar entry | Already configured in tauri.conf.json; Phase 5 adds show/position logic |
| LWND-02 | Fixed width 640px; height grows with result count (min: input only, max: input + 8 rows) | `setSize(new LogicalSize(640, h))` from `@tauri-apps/api/window`; see setSize pitfall |
| LWND-03 | Search input autofocused when window appears; cleared when summoned via hotkey | `inputRef.value.focus()` after `show()` call + reactive `query` ref reset |
| LWND-04 | ↑/↓ navigate list (wraps); Enter launches selected; Escape hides window | `@keydown` on `<input>` + `selectedIndex` ref with modulo wrap |
| LWND-05 | Ctrl+Shift+Enter triggers elevated launch | `event.ctrlKey && event.shiftKey && event.key === 'Enter'` |
| LWND-06 | Window auto-hides on focus loss | `getCurrentWindow().onFocusChanged()` listener; hide when `focused === false` |
| LWND-07 | Each result row: app icon (32×32) · app name | `<img>` with `convertFileSrc(iconPath)` + asset protocol enabled |
| LWND-08 | Selected row shows full executable path below name when `show_path` is true | Conditional `<p class="path-line">` inside result row template |
| LWND-09 | [Admin] badge in selected row hint area when Ctrl+Shift is held | Reactive `adminMode` computed from modifier key state |
| LWND-10 | Result list virtualised for performance | `vue-virtual-scroller@next` — `RecycleScroller` with `itemSize: 48` |
| LWND-11 | Placeholder: "Search apps, or > for system commands…" when no query | Native HTML `placeholder` attribute on `<input>` |
| LWND-12 | System command results render with ⚙️ icon and no path line | `result.kind === 'system'` guard on path line render |
</phase_requirements>

---

## Summary

Phase 5 is a pure Vue 3 frontend phase — no new Rust commands needed. The launcher window is already declared in `tauri.conf.json` as frameless, `skipTaskbar: true`, `alwaysOnTop: true`. The phase builds a single large `App.vue` component (replacing the Phase 1 placeholder) that wires: search input → `invoke('search')` → virtualised result list → keyboard navigation → window management via Tauri's JS API.

The critical technical decisions for the planner are: (1) font bundling via `@fontsource` packages (simplest self-hosted approach for Vite), (2) `vue-virtual-scroller@next` using `RecycleScroller` for the 50-item max list, (3) `convertFileSrc()` from `@tauri-apps/api/core` with `assetProtocol` enabled in `tauri.conf.json` for icon images, and (4) `getCurrentWindow().setSize(new LogicalSize(640, h))` for dynamic height — but with awareness of a historical setSize bug on frameless windows (fixed in Tauri versions later than 2.1–2.2; project is on 2.10.3 which should be clean).

The `animation` field (slide/fade/instant) needs to be added to the Rust `Settings` struct in `store.rs` and exposed via a `get_settings` Tauri command. The planner must decide whether this Rust-side work is Wave 0 of Phase 5 or deferred to Phase 8.

**Primary recommendation:** Build a single-file `App.vue` using Vue 3 Composition API with `ref`/`computed`/`watch` — no Pinia, no component library (established pattern). Wire all Tauri calls (search, window, settings) directly in the component setup. Use `@fontsource/inter` + `@fontsource/jetbrains-mono` npm packages for bundled fonts.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Vue 3 | ^3.5.13 (already installed) | Reactive UI framework | Project choice, already in package.json |
| `@tauri-apps/api` | ^2 (already installed) | Tauri JS bridge — invoke, window ops | Official Tauri v2 JS API |
| `vue-virtual-scroller` | `@next` (~2.x for Vue 3) | Virtualised list rendering | Established Vue 3 virtual scroll library; RecycleScroller is fixed-height performant |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@fontsource/inter` | latest | Self-hosted Inter font via Vite | Bundled fonts — no network, no CDN dependency |
| `@fontsource/jetbrains-mono` | latest | Self-hosted JetBrains Mono font | Path line monospace font |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `vue-virtual-scroller@next` | `@tanstack/virtual` (headless) | TanStack is more flexible but requires more wiring; RecycleScroller is simpler for fixed-height rows |
| `vue-virtual-scroller@next` | Manual windowing (sliceWindow) | At 50 max items, manual is feasible; however virtualiser handles edge cases (scroll events, focus, keyboard) cleanly |
| `@fontsource/inter` | CDN Google Fonts | CDN requires network; Tauri apps run offline — fontsource bundles into dist |
| `convertFileSrc` + asset protocol | Base64 encode icons on Rust side | Base64 bloats IPC calls; convertFileSrc is the standard Tauri pattern for file paths |

**Installation (new packages only):**
```bash
pnpm add vue-virtual-scroller@next @fontsource/inter @fontsource/jetbrains-mono
```

---

## Architecture Patterns

### Recommended Project Structure

```
src/
├── App.vue              # Single component — entire launcher UI (replace Phase 1 placeholder)
├── assets/
│   └── magnifier.svg    # Flat monochrome SVG magnifier icon (create in Phase 5)
├── main.ts              # Unchanged — createApp(App).mount("#app")
└── vite-env.d.ts        # Unchanged
```

All launcher state lives in `App.vue` `<script setup>` — consistent with established "no Pinia" pattern.

### Pattern 1: Tauri Window Operations

**What:** All window show/hide/resize calls use `getCurrentWindow()` from `@tauri-apps/api/window`.
**When to use:** Any interaction that changes window visibility or size.

```typescript
// Source: https://v2.tauri.app/reference/javascript/api/namespacewindow/
import { getCurrentWindow } from '@tauri-apps/api/window'
import { LogicalSize } from '@tauri-apps/api/dpi'

const appWindow = getCurrentWindow()

// Hide on Escape or focus loss
await appWindow.hide()

// Dynamic height resize (input height + result rows)
const INPUT_HEIGHT = 56   // px — search input area
const ROW_HEIGHT   = 48   // px — each result row
const MAX_ROWS     = 8
const BORDER       = 2    // 1px top + 1px bottom

function computeWindowHeight(resultCount: number): number {
  const rows = Math.min(resultCount, MAX_ROWS)
  return INPUT_HEIGHT + BORDER + (rows > 0 ? rows * ROW_HEIGHT : 0)
}

// Call after results change
await appWindow.setSize(new LogicalSize(640, computeWindowHeight(results.value.length)))
```

### Pattern 2: Focus Change — Auto-Hide on Blur

**What:** Subscribe to `onFocusChanged` in `onMounted`; unlisten in `onUnmounted`.
**When to use:** LWND-06 auto-hide requirement.

```typescript
// Source: https://v2.tauri.app/reference/javascript/api/namespacewindow/
import { onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

let unlistenFocus: (() => void) | null = null

onMounted(async () => {
  unlistenFocus = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
    if (!focused) {
      getCurrentWindow().hide()
    }
  })
})

onUnmounted(() => {
  unlistenFocus?.()
})
```

**Pitfall:** `onFocusChanged` returns a `Promise<UnlistenFn>`. Store the unlisten fn and call it in `onUnmounted` to prevent memory leaks.

### Pattern 3: Icon Loading with convertFileSrc + Asset Protocol

**What:** Convert `{data_dir}/icons/{app_id}.png` paths to webview-loadable URLs.
**When to use:** Every result row `<img>` src.

```typescript
// Source: https://v2.tauri.app/reference/javascript/api/namespacecore/
import { convertFileSrc } from '@tauri-apps/api/core'

// In template:
// <img :src="getIconUrl(result.icon_path)" :alt="result.name" width="32" height="32" />

function getIconUrl(iconPath: string): string {
  // icon_path is a filename (e.g. "notepad.png") relative to data_dir/icons/
  // Full absolute path must be constructed from settings or a passed data_dir
  return convertFileSrc(fullIconPath)
}
```

**Required tauri.conf.json change:**
```json
"security": {
  "csp": "default-src 'self' ipc: http://ipc.localhost; img-src 'self' asset: http://asset.localhost",
  "assetProtocol": {
    "enable": true,
    "scope": ["**"]
  }
}
```

**Alternative simpler approach:** Expose `data_dir` path via a Tauri command `get_data_dir()` so the frontend can construct full icon paths. This avoids hardcoding paths in Vue.

### Pattern 4: RecycleScroller for Virtualised List

**What:** `vue-virtual-scroller`'s `RecycleScroller` renders only visible rows, recycling DOM nodes.
**When to use:** LWND-10 virtualisation requirement.

```vue
<!-- Source: https://github.com/Akryum/vue-virtual-scroller README -->
<template>
  <RecycleScroller
    class="result-list"
    :items="results"
    :item-size="48"
    key-field="id"
    v-slot="{ item, index }"
  >
    <div
      class="result-row"
      :class="{ selected: index === selectedIndex }"
      @mousedown.prevent="launchItem(item)"
      @mousemove="selectedIndex = index"
    >
      <img :src="getIconUrl(item.icon_path)" width="32" height="32" />
      <div class="result-text">
        <span class="app-name">{{ item.name }}</span>
        <span
          v-if="index === selectedIndex && showPath && item.kind !== 'system'"
          class="path-line"
        >{{ item.path }}</span>
      </div>
      <span v-if="index === selectedIndex && adminMode" class="admin-badge">
        🛡 Admin
      </span>
    </div>
  </RecycleScroller>
</template>

<script setup lang="ts">
import { RecycleScroller } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'
</script>
```

**Critical CSS requirement:**
```css
.result-list {
  height: calc(var(--result-count) * 48px);  /* or computed inline style */
  max-height: 384px;  /* 8 * 48px */
  overflow-y: auto;
}
.result-row {
  height: 48px;
  display: flex;
  align-items: center;
}
```

### Pattern 5: Keyboard Navigation

**What:** `@keydown` on `<input>` handles all keys; `selectedIndex` ref tracks selection.
**When to use:** LWND-04, LWND-05.

```typescript
// Source: standard Vue 3 Composition API pattern
import { ref, computed } from 'vue'

const selectedIndex = ref(0)
const results = ref<SearchResult[]>([])

function onKeyDown(e: KeyboardEvent) {
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value + 1) % results.value.length
      break
    case 'ArrowUp':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value - 1 + results.value.length) % results.value.length
      break
    case 'Enter':
      e.preventDefault()
      if (e.ctrlKey && e.shiftKey) {
        launchElevated(results.value[selectedIndex.value])
      } else {
        launch(results.value[selectedIndex.value])
      }
      break
    case 'Escape':
      e.preventDefault()
      getCurrentWindow().hide()
      break
  }
}

// Admin badge visibility: Ctrl+Shift held
const adminMode = ref(false)
function onKeyUp(e: KeyboardEvent) {
  adminMode.value = e.ctrlKey && e.shiftKey
}
// Also update on keydown:
// adminMode.value = e.ctrlKey && e.shiftKey  (in onKeyDown before switch)
```

### Pattern 6: Font Bundling with @fontsource

**What:** Import font packages in `main.ts` or `App.vue`; Vite bundles font files into `dist/`.
**When to use:** LWND design spec — Inter + JetBrains Mono bundled, no CDN.

```typescript
// In main.ts (or App.vue <script setup>)
import '@fontsource/inter/400.css'   // weight 400
import '@fontsource/inter/500.css'   // weight 500
import '@fontsource/jetbrains-mono/400.css'
```

```css
/* In App.vue <style> */
.app-name  { font-family: 'Inter', sans-serif; font-weight: 500; }
.path-line { font-family: 'JetBrains Mono', monospace; font-weight: 400; font-size: 12px; }
```

### Pattern 7: Window Slide/Fade Animation

**What:** Show/hide animations via CSS class toggling + Tauri `show()`/`hide()`.
**When to use:** LWND `animation` setting modes.

```vue
<template>
  <div id="app" :class="['launcher', `anim-${animationMode}`, { visible: isVisible }]">
    ...
  </div>
</template>
```

```css
/* Base — hidden state */
#app { opacity: 0; transform: translateY(-6px); }

/* Fade mode */
.anim-fade { transition: opacity 120ms ease; }
.anim-fade.visible { opacity: 1; }

/* Slide mode (default) */
.anim-slide { transition: opacity 180ms ease, transform 180ms ease; }
.anim-slide.visible { opacity: 1; transform: translateY(0); }

/* Instant mode */
.anim-instant { transition: none; }
.anim-instant.visible { opacity: 1; transform: translateY(0); }
```

**Coordination with Tauri:** Call `appWindow.show()` first (makes window visible), then set `isVisible = true` on `nextTick` to trigger CSS transition.

For hide: set `isVisible = false`, wait for transition duration, then call `appWindow.hide()`.

### Anti-Patterns to Avoid

- **Calling `appWindow.hide()` inside `onFocusChanged` immediately without a guard:** If a UAC dialog appears or a child window opens, the blur fires and hides the launcher. Add a `isHiding` guard flag or debounce.
- **Using `document.addEventListener('keydown')` globally instead of `@keydown` on `<input>`:** The input should capture keys; global listeners conflict with normal browser behavior.
- **Forgetting `e.preventDefault()` on Arrow keys:** Without it, the browser scrolls the page instead of navigating the list.
- **Not unlistening `onFocusChanged`:** Memory leak if component remounts.
- **Setting `RecycleScroller` height via JavaScript before CSS is applied:** Results in zero-height scroller. Always set explicit CSS height.
- **Using `mousedown` + `preventDefault` instead of `click` on result rows:** `click` fires after `mouseup`; clicking a row fires blur on input (triggering hide) before `click` fires. Use `@mousedown.prevent` to intercept before blur propagates.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Virtualised list | Custom windowing with v-for slice | `vue-virtual-scroller@next` RecycleScroller | Handles scroll position, item recycling, ResizeObserver, keyboard scroll-into-view |
| Self-hosted fonts | Manual @font-face + static file copying | `@fontsource/inter` + `@fontsource/jetbrains-mono` | Automatic Vite bundling; correct font-display, unicode-range subsets included |
| File path to URL conversion | String manipulation or custom Rust endpoint | `convertFileSrc()` from `@tauri-apps/api/core` | Official Tauri asset protocol; handles OS path separators, encoding |

**Key insight:** At 50 max items RecycleScroller is technically overkill, but it eliminates scroll synchronisation bugs and keeps the pattern consistent with future list growth.

---

## Common Pitfalls

### Pitfall 1: setSize on Frameless Windows (Historical Bug — Monitor Before Use)

**What goes wrong:** In Tauri v2.1.x–v2.2.x, `Window.setSize()` silently failed when `decorations: false` was set. The window would not resize.
**Why it happens:** tao (Tauri's windowing layer) had a regression in that version range.
**Status:** Issues #12076 and #12168 are closed; the fix was merged before the v2.10.x series. Project is on Tauri 2.10.3 — the fix should be present.
**How to avoid:** Verify height resize works in first smoke test. If `setSize` fails, fallback: CSS-only approach — let the webview content drive a fixed-height container with `overflow: hidden`, and only call `setSize` on the first show to set initial height.
**Warning signs:** Window stays at initial height (60px from tauri.conf.json) regardless of result count.

### Pitfall 2: Focus Loss Auto-Hide Fires During Elevated Launch UAC Prompt

**What goes wrong:** When `launchElevated()` is called, Windows shows a UAC dialog. This steals focus from the launcher, firing `onFocusChanged(focused: false)` and hiding the window before the user sees the UAC prompt. The UAC prompt is correctly displayed anyway, but the launcher closes immediately.
**Why it happens:** UAC prompts are a separate OS process with their own focus chain.
**How to avoid:** Set a `launchInProgress` flag before calling `launch_elevated`, suppress auto-hide while it's true. Clear flag after a short delay (500ms) or on window show event.

### Pitfall 3: mousedown on Result Row Triggers input blur → Auto-Hide Before Launch

**What goes wrong:** User clicks a result row. Browser fires `mousedown` on the row → `blur` on the input → `onFocusChanged` fires with `focused: false` → window hides → `click` event never fires.
**Why it happens:** `blur` fires on `mousedown`, before `mouseup`/`click`.
**How to avoid:** Bind result row clicks to `@mousedown.prevent`. The `.prevent` stops default blur propagation on mousedown. This is the correct pattern for custom dropdowns/launchers.

### Pitfall 4: selectedIndex Out of Bounds After Query Change

**What goes wrong:** User navigates to row 5 of a 10-result list, then types more characters reducing results to 3 items. `selectedIndex` is still 5, causing index-out-of-bounds on the results array.
**Why it happens:** `selectedIndex` is not reset when `results` changes.
**How to avoid:** `watch(results, () => { selectedIndex.value = 0 })` — reset to 0 on every result change.

### Pitfall 5: RecycleScroller CSS Height Must Be Explicit

**What goes wrong:** `RecycleScroller` renders an empty div if the scroller has no defined height.
**Why it happens:** The library uses an internal `ResizeObserver` on the container; without explicit height it sees 0 and renders nothing.
**How to avoid:** Always set the scroller's CSS height explicitly, either via inline style binding `:style="{ height: listHeight + 'px' }"` or a CSS class. `listHeight = Math.min(results.length, 8) * 48`.

### Pitfall 6: Asset Protocol CSP Not Configured

**What goes wrong:** Icons appear as broken images. Console shows CSP violation: `img-src` does not allow `asset://` scheme.
**Why it happens:** Tauri v2 requires explicit `assetProtocol` configuration and CSP update.
**How to avoid:** Add to `tauri.conf.json` (see Pattern 3 above). Also enable `assetProtocol.enable: true` and set `scope: ["**"]` or scoped to data dir.

### Pitfall 7: @mousedown.prevent Breaks Text Selection in Input

**What goes wrong:** If `.prevent` is applied too broadly (e.g., on the whole result list container), it prevents the user from clicking in the search input to position the cursor.
**Why it happens:** `preventDefault` on mousedown cancels the default focus action.
**How to avoid:** Apply `@mousedown.prevent` only to individual result rows, not to the input or the outer container.

---

## Code Examples

Verified patterns from official sources and established Vue 3 practices:

### Full App.vue Skeleton

```typescript
// Source: Vue 3 Composition API + Tauri v2 documented patterns
<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { LogicalSize } from '@tauri-apps/api/dpi'
import { RecycleScroller } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'
import '@fontsource/inter/400.css'
import '@fontsource/inter/500.css'
import '@fontsource/jetbrains-mono/400.css'

interface SearchResult {
  id: string
  name: string
  icon_path: string
  path: string
  kind: string
}

const query        = ref('')
const results      = ref<SearchResult[]>([])
const selectedIndex = ref(0)
const adminMode    = ref(false)
const showPath     = ref(false)
const animMode     = ref<'instant' | 'fade' | 'slide'>('slide')
const dataDir      = ref('')
const isVisible    = ref(false)
const inputRef     = ref<HTMLInputElement | null>(null)

let unlistenFocus: (() => void) | null = null
let launchInProgress = false

const listHeight = computed(() =>
  Math.min(results.value.length, 8) * 48
)

watch(query, async (q) => {
  results.value = q ? await invoke<SearchResult[]>('search', { query: q }) : []
  selectedIndex.value = 0
  await updateWindowHeight()
})

watch(results, () => {
  selectedIndex.value = 0
})

async function updateWindowHeight() {
  const h = 56 + 2 + listHeight.value  // input + border + rows
  await getCurrentWindow().setSize(new LogicalSize(640, Math.max(h, 58)))
}

function getIconUrl(iconPath: string): string {
  const fullPath = `${dataDir.value}/icons/${iconPath}`
  return convertFileSrc(fullPath)
}

function onKeyDown(e: KeyboardEvent) {
  adminMode.value = e.ctrlKey && e.shiftKey
  if (!results.value.length) return
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value + 1) % results.value.length
      break
    case 'ArrowUp':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value - 1 + results.value.length) % results.value.length
      break
    case 'Enter':
      e.preventDefault()
      const item = results.value[selectedIndex.value]
      if (!item) break
      if (e.ctrlKey && e.shiftKey) {
        launchElevated(item)
      } else if (item.kind === 'system') {
        runSystemCommand(item)
      } else {
        launch(item)
      }
      break
    case 'Escape':
      e.preventDefault()
      hideWindow()
      break
  }
}

function onKeyUp(e: KeyboardEvent) {
  adminMode.value = e.ctrlKey && e.shiftKey
}

async function launch(item: SearchResult) {
  launchInProgress = true
  await invoke('launch', { id: item.id }).catch(console.error)
  hideWindow()
  launchInProgress = false
}

async function launchElevated(item: SearchResult) {
  launchInProgress = true
  await invoke('launch_elevated', { id: item.id }).catch(console.error)
  setTimeout(() => { launchInProgress = false }, 500)
  hideWindow()
}

async function runSystemCommand(item: SearchResult) {
  launchInProgress = true
  await invoke('run_system_command', { cmd: item.id }).catch(console.error)
  hideWindow()
  launchInProgress = false
}

async function hideWindow() {
  isVisible.value = false
  const delay = animMode.value === 'slide' ? 180 : animMode.value === 'fade' ? 120 : 0
  setTimeout(() => getCurrentWindow().hide(), delay)
}

onMounted(async () => {
  // Load settings
  try {
    const settings = await invoke<{ show_path: boolean; animation: string; data_dir: string }>('get_settings_ui')
    showPath.value = settings.show_path
    animMode.value = (settings.animation ?? 'slide') as typeof animMode.value
    dataDir.value  = settings.data_dir
  } catch { /* use defaults */ }

  // Focus input
  await nextTick()
  inputRef.value?.focus()
  isVisible.value = true

  // Auto-hide on focus loss
  unlistenFocus = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
    if (!focused && !launchInProgress) {
      hideWindow()
    }
  })
})

onUnmounted(() => {
  unlistenFocus?.()
})
</script>
```

### Settings Integration — get_settings_ui Tauri Command

The `animation` field needs to be added to `store.rs` `Settings` struct. A `get_settings_ui` Tauri command (or reusing a generic `get_settings`) must expose: `show_path`, `animation`, and `data_dir` (so frontend can build icon paths).

**Rust stub for `get_settings_ui` in `commands.rs`:**
```rust
#[tauri::command]
pub fn get_settings_ui(
    app: tauri::AppHandle,
    data_dir: tauri::State<std::path::PathBuf>,
) -> serde_json::Value {
    let settings = crate::store::get_settings(&app, &data_dir);
    serde_json::json!({
        "show_path": settings.show_path,
        "animation": settings.animation.as_deref().unwrap_or("slide"),
        "data_dir": data_dir.to_string_lossy(),
    })
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `appWindow` from `@tauri-apps/api/window` | `getCurrentWindow()` from `@tauri-apps/api/window` | Tauri v2.0 | Different import — `appWindow` is v1 pattern |
| `tauri.convertFileSrc()` | `convertFileSrc()` from `@tauri-apps/api/core` | Tauri v2.0 | Moved to core namespace |
| `vue-virtual-scroller` (v1) | `vue-virtual-scroller@next` (Vue 3 compatible) | Vue 3 migration | Must use @next tag for Vue 3; without it installs Vue 2 version |
| `@tauri-apps/api/tauri` (invoke) | `@tauri-apps/api/core` (invoke) | Tauri v2.0 | Renamed module; project already uses core correctly |

**Deprecated/outdated:**
- `import { appWindow } from '@tauri-apps/api/window'`: v1 pattern; v2 uses `getCurrentWindow()`
- `LogicalSize` from `@tauri-apps/api/window`: moved to `@tauri-apps/api/dpi` in v2

---

## Open Questions

1. **`animation` field in Settings struct — Phase 5 or Phase 8?**
   - What we know: CONTEXT.md deferred the Phase 8 UI picker; but Phase 5 reads the `animation` setting
   - What's unclear: Whether to add the Rust-side `animation: String` field to `store.rs` in Phase 5 Wave 0, or hardcode `"slide"` default and defer
   - Recommendation: Add `animation` field to `store.rs` in Phase 5 Wave 0 (alongside get_settings_ui command) — minimal Rust change, avoids two-phase Rust edit later

2. **`get_settings_ui` vs reusing/extending `get_settings` command**
   - What we know: No Tauri commands currently expose settings to the frontend; Phase 8 will build full settings UI
   - What's unclear: Should Phase 5 add a lean `get_settings_ui` command or a more generic `get_settings` command?
   - Recommendation: Add generic `get_settings` Tauri command in Phase 5 that returns the full `Settings` struct as JSON; Phase 8 reuses it

3. **Icon path construction — data_dir via command vs embedded in search results**
   - What we know: `search()` returns `icon_path` as a filename only (e.g., `"notepad.png"`); frontend needs full path for `convertFileSrc`
   - What's unclear: Whether to return absolute `icon_path` from Rust or expose `data_dir` separately
   - Recommendation: Add `data_dir` to the `get_settings` response (simplest); or modify `search()` to return absolute icon paths. Former is cleaner.

---

## Validation Architecture

> `nyquist_validation: true` in `.planning/config.json` — section included.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | None currently installed for frontend; Rust uses `cargo test` |
| Config file | None — Wave 0 must install vitest + @vue/test-utils if frontend tests are required |
| Quick run command | `pnpm test` (after vitest configured) or `cargo test` (Rust only) |
| Full suite command | `cargo test && pnpm test` |

**Assessment:** Phase 5 is pure Vue 3 frontend. The Rust changes (store.rs `animation` field + `get_settings` command) are minimal and well-covered by existing unit test patterns. Frontend keyboard navigation and state logic can be unit-tested with vitest + @vue/test-utils, but the project has no frontend test infrastructure yet. Given the complexity (keyboard nav, focus events, Tauri API calls), the planner should decide whether to install vitest in Wave 0 or rely on manual smoke testing.

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LWND-01 | Frameless window config correct | manual-only | N/A — visual/config verification | N/A |
| LWND-02 | Height grows with result count (max 8 rows) | unit | `pnpm test -- LauncherHeight` | ❌ Wave 0 |
| LWND-03 | Input autofocused and cleared on show | manual-only | N/A — Tauri focus API not unit-testable | N/A |
| LWND-04 | ↑/↓ wraps; Enter launches; Escape hides | unit | `pnpm test -- KeyboardNav` | ❌ Wave 0 |
| LWND-05 | Ctrl+Shift+Enter elevated launch | unit | `pnpm test -- KeyboardNav` | ❌ Wave 0 |
| LWND-06 | Auto-hide on focus loss | manual-only | N/A — Tauri event not unit-testable | N/A |
| LWND-07 | Icon displayed per result row | manual-only | N/A — visual, asset protocol | N/A |
| LWND-08 | Path shown for selected row when show_path true | unit | `pnpm test -- PathLine` | ❌ Wave 0 |
| LWND-09 | Admin badge when Ctrl+Shift held | unit | `pnpm test -- AdminBadge` | ❌ Wave 0 |
| LWND-10 | Virtualised list renders without lag | manual-only | N/A — performance visual test | N/A |
| LWND-11 | Placeholder text present | unit | `pnpm test -- Placeholder` | ❌ Wave 0 |
| LWND-12 | System commands: ⚙️ icon, no path line | unit | `pnpm test -- SystemCommand` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test` (Rust changes only)
- **Per wave merge:** `cargo test` + manual smoke test of launcher window
- **Phase gate:** Full `cargo test` green + manual UAT checklist before `/gsd:verify-work`

### Wave 0 Gaps

**Frontend test infrastructure (optional — planner decides):**
- [ ] `src/__tests__/LauncherLogic.test.ts` — covers LWND-02, LWND-04, LWND-05, LWND-08, LWND-09, LWND-11, LWND-12
- [ ] `vitest.config.ts` + `@vue/test-utils` install: `pnpm add -D vitest @vue/test-utils @vitejs/plugin-vue`

**Note:** Given that most testable logic (keyboard nav, conditional rendering) lives in a single Vue component with Tauri API dependencies, mocking `@tauri-apps/api/core` and `@tauri-apps/api/window` adds non-trivial setup cost. The planner may reasonably elect for manual smoke tests only and add vitest in Phase 8 when Settings Window also needs frontend tests. Mark LWND-02/04/05/08/09/11/12 as manual-smoke if vitest is deferred.

---

## Sources

### Primary (HIGH confidence)
- `https://v2.tauri.app/reference/javascript/api/namespacewindow/` — getCurrentWindow, hide, show, setSize, onFocusChanged, LogicalSize
- `https://v2.tauri.app/reference/javascript/api/namespacecore/` — convertFileSrc, invoke
- `https://github.com/tauri-apps/tauri/discussions/11498` — asset protocol configuration, CSP for img-src, scope setup
- `https://raw.githubusercontent.com/Akryum/vue-virtual-scroller/master/packages/vue-virtual-scroller/README.md` — RecycleScroller API, props, setup
- `https://fontsource.org/fonts/inter/install` — @fontsource/inter install + Vite usage

### Secondary (MEDIUM confidence)
- `https://github.com/tauri-apps/tauri/releases` — confirmed Tauri 2.10.3 is current (released 2026-03-04)
- `https://github.com/tauri-apps/tauri/issues/12076` — setSize + decorations:false bug confirmed closed/fixed
- `https://github.com/tauri-apps/tauri/issues/13633` — blur event reliability issue with tray windows; resolved by removing unstable feature flags

### Tertiary (LOW confidence — flag for validation)
- `onFocusChanged` second-show reliability: bug #13633 was tray-specific and resolved; standard launcher window (not tray) should be unaffected — LOW confidence until smoke-tested

---

## Metadata

**Confidence breakdown:**
- Standard stack (Vue 3, Tauri JS API, vue-virtual-scroller): HIGH — all verified via official docs and README
- Architecture (setSize, convertFileSrc, RecycleScroller patterns): HIGH — all code examples from official sources
- setSize on Tauri 2.10.3 frameless windows: MEDIUM — bug was fixed before 2.10.x but not explicitly confirmed in 2.10.3 release notes
- onFocusChanged reliability for non-tray launcher window: MEDIUM — known tray bug is resolved; standard window behavior expected reliable
- Pitfalls (mousedown/blur, selectedIndex OOB, CSP): HIGH — well-known frontend patterns

**Research date:** 2026-03-06
**Valid until:** 2026-05-01 (30 days for stable; Tauri 2.x moves quickly — recheck if upgrading beyond 2.10.x)
