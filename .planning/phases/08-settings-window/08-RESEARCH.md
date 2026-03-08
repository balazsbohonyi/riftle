# Phase 8: Settings Window - Research

**Researched:** 2026-03-08
**Domain:** Tauri v2 multi-window management, Vue 3 SFC composition, CSS custom properties, inter-window event communication
**Confidence:** HIGH

## Summary

Phase 8 builds the Settings window for Riftle — the first time the app has two simultaneously active UI surfaces. The core challenges are: (1) correctly wiring a single-instance secondary Tauri window that opens from the launcher, (2) propagating setting changes from Settings.vue back to App.vue in real time via Tauri events, and (3) establishing the CSS token foundation (tokens.css + UI primitives) that both windows share from this point forward.

The good news is that nearly every piece of infrastructure required already exists: the settings window is declared in `tauri.conf.json` with label `settings`, already listed in `capabilities/default.json`, and the `update_hotkey`, `reindex`, `get_settings_cmd`, and `set_settings` commands are all implemented. The `open_settings_window` Tauri command stub is the only Rust work needed. The frontend work is the bulk of this phase — tokens.css, App.vue refactor, five UI primitive components, and Settings.vue itself.

The approach is CSS custom properties + thin Vue component primitives (Option B from settings-styling.md), with reactive propagation via `emit('settings-changed', payload)` from the settings window and `listen('settings-changed', handler)` in App.vue. No third-party component libraries, no Tailwind — consistent with the project's dependency-light philosophy.

**Primary recommendation:** Wave 0 = tokens.css + primitives; Wave 1 = App.vue CSS refactor; Wave 2 = Settings.vue + Rust `open_settings_window` command; Wave 3 = reactive listener in App.vue + live preview; Wave 4 = final wiring and Ctrl+, keyboard shortcut.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Pre-phase infrastructure (Wave 0)**
- Tokens: Create `src/styles/tokens.css` with all CSS custom properties as defined in `.planning/settings-styling.md`. Import once in `src/main.ts` before app mount — available globally to both windows.
- App.vue refactor: Replace all hardcoded values in App.vue's `<style>` block with CSS variables. Purely mechanical substitution — no behavior change. Verified by visual smoke test only (no pixel-perfect diff needed).
- UI primitives: Build only the primitives actively used in Phase 8 (all 5 end up needed): `Section.vue`, `Row.vue`, `Toggle.vue`, `KeyCapture.vue`, `PathList.vue`. Location: `src/components/ui/`. No new dependencies.
- Sequencing: Tokens + primitives = Wave 0; App.vue refactor = Wave 1; Settings.vue = later waves.

**Settings window chrome**
- Frameless: `decorations: false` in tauri.conf.json for the settings window. No OS title bar.
- Custom header: Custom in-page header with `data-tauri-drag-region`. Left side: app icon + "Riftle Settings". Right side: close button (calls `window.close()`).
- Window configuration: Normal framed sizing, min 600x400px (per SETT-01) — but frameless (decorations off).

**Settings layout**
- Single scrollable page: All four sections stacked vertically, separated by section headings + horizontal dividers. Scroll to navigate. No tabs or sidebar.
- Section order: General → Hotkey → Search → Appearance (top to bottom).
- Row pattern: Each setting row uses `Row.vue` — label on left, control on right. Consistent layout across all sections.

**Reactive propagation**
- Mechanism: Settings window emits `settings-changed` Tauri event with changed values. `App.vue` listens with `listen('settings-changed')` (same pattern as existing `launcher-show` event).
- Persistence: Settings write to disk immediately on every change (no save button, no unsaved state). Each control change calls `set_settings` Tauri command.
- Which settings trigger live launcher update:
  - `theme` → launcher applies `data-theme` attribute to root element; CSS token overrides handle theming
  - `opacity` → launcher sets CSS opacity/background-alpha via token or inline style
  - `show_path` → launcher's `showPath` ref updates
  - `reindex_interval` → emits event to restart background timer in Rust (new Rust command or existing mechanism)
- Live preview: Launcher updates in real time as user interacts (while dragging opacity slider, while toggling show_path). Not on mouseup/commit — on every input event.

**Theme implementation**
- Options: System / Light / Dark (all three, per SETT-07). No deferral of Light theme.
- Mechanism: Selecting a theme sets `data-theme="light"` or `data-theme="dark"` on the launcher's root element. `tokens.css` includes `[data-theme="light"] { ... }` overrides. System follows `prefers-color-scheme` media query.

**General section**
- Autostart toggle: In portable mode — visible but disabled (greyed out). Tooltip or adjacent note: "Not available in portable mode". User can see the feature exists; understands why inactive.
- Autostart toggle: In installed mode — functional, calls tauri-plugin-autostart.

**Hotkey section**
- Key capture UX: Click input → shows "Press shortcut..." placeholder → user presses key combination → input displays captured combo (e.g. "Ctrl+Alt+Space") → saves immediately. Escape cancels capture and restores previous value.
- Takes effect immediately: Calls `update_hotkey` Tauri command (already implemented in hotkey.rs).

**Search section**
- Additional paths: `PathList.vue` — add path via folder picker dialog, remove with [-] button. Changing paths triggers re-index.
- Excluded paths: Same `PathList.vue` component.
- Re-index now: Button triggers `reindex` Tauri command (already implemented in Phase 3).
- Re-index interval: Dropdown selector with discrete options: 5 min / 15 min / 30 min / 60 min / Manual only. Default: 15 min (matches `default_reindex_interval` in store.rs). Changing interval takes effect immediately.

### Claude's Discretion
- Exact token values — already defined in `.planning/settings-styling.md`
- Light/Dark theme color palette specifics (needs to be dark-aesthetic consistent)
- Opacity slider range (REQUIREMENTS.md says 0.85–1.0; implementation approach is Claude's call)
- Error state handling for invalid hotkey combinations
- Exact animation/transition for settings window open/close

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SETT-01 | Separate Tauri window (label: settings), normal framed, min size 600x400px | Window already declared in tauri.conf.json; need to flip `decorations: true` → `false` and add drag region |
| SETT-02 | Single-instance: open_settings_window() focuses existing window if already open | `app.get_webview_window("settings")` + `win.is_visible()` / `win.show()` + `win.set_focus()` pattern |
| SETT-03 | Accessible via context menu → Settings and Ctrl+, when launcher focused | `open_settings_window` Tauri command already called by `openSettings()` in App.vue; add Ctrl+, to onKeyDown |
| SETT-04 | General section: launch at startup toggle (disabled/hidden in portable mode) | `tauri-plugin-autostart` already registered; portable mode detection via `get_settings_cmd` response |
| SETT-05 | Hotkey section: key-capture input to rebind; takes effect immediately | `update_hotkey` command already implemented in hotkey.rs; `KeyCapture.vue` primitive handles capture UX |
| SETT-06 | Search section: add/remove additional paths, excluded paths, Re-index now, interval selector | `reindex` command implemented; folder picker via `@tauri-apps/plugin-dialog`; `set_settings` for persistence |
| SETT-07 | Appearance section: theme/opacity/show_path — all reactive on open launcher | `listen('settings-changed')` in App.vue; `data-theme` attribute + CSS token overrides for theming |
</phase_requirements>

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri v2 | 2.10.3 (pinned) | Multi-window management, IPC commands, event system | Already in project |
| Vue 3 | ^3.x | Settings.vue SFC, reactive state, composition API | Already in project |
| `@tauri-apps/api` | current | `invoke`, `listen`, `emit`, `getCurrentWindow`, `WebviewWindow` | Already in project |
| `tauri-plugin-autostart` | 2.5.1 (pinned) | Autostart toggle in General section | Already registered in lib.rs |
| `@tauri-apps/plugin-dialog` | current | Folder picker for PathList.vue | Needs confirming — see note |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@fontsource/inter` | already installed | Typography in settings window | Same fonts as launcher |
| `@fontsource/jetbrains-mono` | already installed | Monospace for path display | Path rows in PathList.vue |

### Note on @tauri-apps/plugin-dialog
The folder picker for `PathList.vue` (SETT-06) requires a Tauri dialog API. The project already has `tauri-plugin-opener` but not `tauri-plugin-dialog`. The `open()` function from `@tauri-apps/plugin-dialog` with `directory: true` is the standard approach for folder selection in Tauri v2.

**Action required for Wave 0 plan:** Add `tauri-plugin-dialog` to `Cargo.toml` and `package.json`, register it in `lib.rs`, and add permissions to `capabilities/default.json`. Alternatively, implement with a workaround using a text input for path entry. Given the locked decision ("add path via folder picker dialog"), plugin-dialog is needed.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `@tauri-apps/plugin-dialog` | Plain `<input>` for path typing | No OS folder browser UX, but avoids a new dep |
| CSS custom properties | Tailwind | Tailwind rejected — doesn't match dark aesthetic, adds PostCSS complexity |
| Thin Vue primitives | shadcn-vue / PrimeVue | Component libraries rejected — ship own visual identity, heavy override burden |

**Installation (if plugin-dialog is added):**
```bash
# Rust side — add to Cargo.toml
tauri-plugin-dialog = "2"

# JS side
pnpm add @tauri-apps/plugin-dialog
```

---

## Architecture Patterns

### Recommended Project Structure
```
src/
├── styles/
│   └── tokens.css              # CSS custom properties — imported once in main.ts
├── components/
│   └── ui/
│       ├── Section.vue         # Section wrapper: heading + divider + slot
│       ├── Row.vue             # Label-left / control-right layout row
│       ├── Toggle.vue          # Accessible on/off toggle
│       ├── KeyCapture.vue      # Hotkey capture input (SETT-05)
│       └── PathList.vue        # Add/remove folder path list (SETT-06)
├── App.vue                     # Launcher window — listens for settings-changed
├── Settings.vue                # Settings window root component
├── main.ts                     # Launcher entry — imports tokens.css
└── settings-main.ts            # Settings window entry — imports tokens.css + mounts Settings.vue

src-tauri/src/
├── lib.rs                      # Add open_settings_window command + invoke_handler registration
└── store.rs                    # Add set_settings_cmd Tauri command

settings.html                   # Second Vite entry point for settings window
vite.config.ts                  # Add multi-page build (input: { main: index.html, settings: settings.html })
```

### Pattern 1: Tauri Multi-Window with Single Instance Guard

**What:** The `open_settings_window` Rust command checks if the settings window is already visible before showing it. If visible: bring to focus. If hidden: show and focus.

**When to use:** Any time a secondary window must be single-instance.

**Example:**
```rust
// In lib.rs or a dedicated settings.rs module
#[tauri::command]
pub fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    match app.get_webview_window("settings") {
        Some(win) => {
            // Window exists — bring to front regardless of visible state
            win.show().map_err(|e| e.to_string())?;
            win.set_focus().map_err(|e| e.to_string())?;
            Ok(())
        }
        None => Err("settings window not found".into()),
    }
}
```

**Note:** Since the settings window is declared in `tauri.conf.json` (not created dynamically), `get_webview_window("settings")` will always return `Some`. The single-instance check is simply `win.show()` + `win.set_focus()` — calling `show()` on an already-visible window is idempotent in Tauri v2.

### Pattern 2: Tauri Multi-Page Vite Build

**What:** Vite multi-page setup to serve two separate HTML entry points — one for the launcher window (`/index.html` → `src/main.ts`) and one for the settings window (`/settings.html` → `src/settings-main.ts`).

**When to use:** When Tauri has two WebviewWindows that need separate Vue app instances and separate component trees.

**Example:**
```typescript
// vite.config.ts
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";

export default defineConfig({
  plugins: [vue()],
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        settings: resolve(__dirname, "settings.html"),
      },
    },
  },
  // ... existing server config
});
```

```html
<!-- settings.html -->
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>Riftle Settings</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/settings-main.ts"></script>
  </body>
</html>
```

**Tauri dev server note:** In dev mode, Tauri uses a single dev URL (`http://localhost:1420`). The settings window needs to point to its own HTML file. In `tauri.conf.json`, the settings window's `url` field must be set to the settings page path. In dev mode: `http://localhost:1420/settings.html`. In production: the bundled `settings.html` from `dist/`.

**tauri.conf.json settings window url:**
```json
{
  "label": "settings",
  "url": "settings.html",
  ...
}
```

### Pattern 3: Cross-Window Reactive Event Propagation

**What:** Settings window emits `settings-changed` event; launcher window listens and applies changes.

**When to use:** Any time a setting change in Settings.vue must immediately affect App.vue state.

**Example (Settings.vue — emitter):**
```typescript
import { emit } from '@tauri-apps/api/event'

async function onThemeChange(newTheme: string) {
  // 1. Persist to disk
  await invoke('set_settings_cmd', { key: 'theme', value: newTheme })
  // 2. Propagate to launcher
  await emit('settings-changed', { theme: newTheme })
}
```

**Example (App.vue — listener, in onMounted):**
```typescript
import { listen } from '@tauri-apps/api/event'

// After existing unlistenShow setup:
unlistenSettings = await listen<Partial<SettingsPayload>>('settings-changed', ({ payload }) => {
  if (payload.theme !== undefined) {
    applyTheme(payload.theme)
  }
  if (payload.opacity !== undefined) {
    // apply opacity
  }
  if (payload.show_path !== undefined) {
    showPath.value = payload.show_path
  }
})
```

**Note:** `emit()` from `@tauri-apps/api/event` broadcasts to ALL windows in the app, including the emitting window itself. Use `emitTo('launcher', ...)` to target only the launcher window and avoid self-handling in Settings.vue.

### Pattern 4: Theme Application via data-theme Attribute

**What:** Apply theme by setting `data-theme` attribute on the launcher's root element. CSS token overrides cascade from `tokens.css`.

**When to use:** System/Light/Dark theme switching.

**Example (App.vue):**
```typescript
function applyTheme(theme: string) {
  const root = document.documentElement
  if (theme === 'system') {
    root.removeAttribute('data-theme')
    // tokens.css handles system via @media (prefers-color-scheme)
  } else {
    root.setAttribute('data-theme', theme) // 'light' or 'dark'
  }
}
```

**tokens.css light theme override block:**
```css
[data-theme="light"] {
  --color-bg:           #f0f0f0;
  --color-bg-lighter:   #ffffff;
  --color-bg-darker:    #e0e0e0;
  --color-text:         #1c1c1e;
  --color-text-muted:   #666;
  --color-text-dim:     #aaaaaa;
  --color-border:       rgba(0, 0, 0, 0.12);
  --color-divider:      rgba(0, 0, 0, 0.08);
}

@media (prefers-color-scheme: light) {
  :root:not([data-theme="dark"]) {
    /* same overrides as [data-theme="light"] for system mode */
  }
}
```

### Pattern 5: KeyCapture Component

**What:** An input that enters "capture mode" on click, records the next keydown, and formats modifiers + key into a string like `"Ctrl+Alt+Space"`.

**When to use:** SETT-05 hotkey rebinding UI.

**Example (KeyCapture.vue):**
```typescript
const capturing = ref(false)
const displayValue = ref(props.modelValue)

function startCapture() {
  capturing.value = true
  displayValue.value = 'Press shortcut...'
}

function onKeyDown(e: KeyboardEvent) {
  if (!capturing.value) return
  e.preventDefault()

  if (e.key === 'Escape') {
    // Cancel: restore previous value
    capturing.value = false
    displayValue.value = props.modelValue
    return
  }

  // Build hotkey string: modifiers first, then key
  const mods: string[] = []
  if (e.ctrlKey)  mods.push('Ctrl')
  if (e.altKey)   mods.push('Alt')
  if (e.shiftKey) mods.push('Shift')
  if (e.metaKey)  mods.push('Meta')

  // Ignore bare modifier keypresses (wait for non-modifier key)
  const modKeys = ['Control', 'Alt', 'Shift', 'Meta']
  if (modKeys.includes(e.key)) return

  const keyName = e.code === 'Space' ? 'Space' : e.key
  const hotkey = [...mods, keyName].join('+')

  capturing.value = false
  displayValue.value = hotkey
  emit('update:modelValue', hotkey)
  emit('change', hotkey)  // triggers immediate save + update_hotkey call
}
```

**Key insight:** `tauri-plugin-global-shortcut` expects hotkey strings in the format `"Modifier+Key"` (e.g., `"Alt+Space"`, `"Ctrl+Shift+Enter"`). The `KeyCapture` component must produce exactly this format. The `e.key` value maps naturally for most keys; `e.code === 'Space'` special-cases the space bar since `e.key` returns `" "` for space.

### Pattern 6: Autostart Toggle with Portable Mode Detection

**What:** The General section toggle calls `tauri-plugin-autostart` in installed mode. In portable mode, the toggle is visible but disabled with explanatory text.

**When to use:** SETT-04.

**Example (Settings.vue):**
```typescript
import { invoke } from '@tauri-apps/api/core'
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'

const settings = ref<Settings | null>(null)
const isPortable = ref(false)

onMounted(async () => {
  const s = await invoke<SettingsResponse>('get_settings_cmd')
  // get_settings_cmd currently returns: show_path, animation, data_dir, hotkey, theme, opacity
  // Need to also return: autostart, additional_paths, excluded_paths, reindex_interval, is_portable
  settings.value = { ...s }
  isPortable.value = s.is_portable  // NEW field needed in get_settings_cmd
})

async function onAutostartToggle(enabled: boolean) {
  if (isPortable.value) return  // guard: should be disabled in UI already
  if (enabled) {
    await enable()
  } else {
    await disable()
  }
  await invoke('set_settings_cmd', { ... })
}
```

**Important gap:** `get_settings_cmd` currently returns only 6 fields: `show_path`, `animation`, `data_dir`, `hotkey`, `theme`, `opacity`. The Settings UI needs all settings fields plus an `is_portable` flag. `set_settings_cmd` (distinct from `set_settings` internal function) does not yet exist as a Tauri command. Both need to be created.

### Anti-Patterns to Avoid

- **Opening settings window with WebviewWindowBuilder in every call:** Always use `get_webview_window("settings")` + `show/set_focus`. Never create a new WebviewWindow dynamically — the window is already declared in `tauri.conf.json`.
- **Using `emit()` without window targeting:** `emit()` broadcasts to all windows including the emitting window. Use `emitTo('launcher', 'settings-changed', payload)` to avoid Settings.vue receiving its own events.
- **Blocking the main thread in Tauri command with heavy I/O:** `open_settings_window` is a simple window management command; keep it synchronous and fast. The `reindex` command is already async.
- **Using scoped styles in Settings.vue for layout tokens:** Token CSS variables only work cross-component with unscoped styles or on the `:root` element. Keep settings-specific layout in `<style scoped>` but keep token application as `var(--token-name)` references — those work fine in scoped styles.
- **Re-implementing folder dialog with `<input type="file">`:** WebView-rendered `<input type="file">` behaves differently in Tauri (may open browser dialog instead of native OS dialog). Use `@tauri-apps/plugin-dialog` `open({ directory: true })` for the folder picker.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Folder picker dialog | Custom path input + manual validation | `@tauri-apps/plugin-dialog` `open({ directory: true })` | Native OS folder browser UX; handles path validation, Unicode paths, Windows path separators |
| Hotkey format parsing | Custom key string parser | Browser `KeyboardEvent` properties (`e.key`, `e.code`, `e.ctrlKey`, etc.) | Already correct format; just compose into "Modifier+Key" string |
| Global shortcut registration | Manual Windows API calls | `tauri-plugin-global-shortcut` `update_hotkey` command (already implemented) | Already handles deregister + register; persists to settings.json |
| Autostart management | Registry writes / startup folder manipulation | `tauri-plugin-autostart` (already registered) | Handles both `HKCU\Run` and `HKLM\Run`; cross-version safe |
| Settings persistence | Custom JSON file I/O | `store::set_settings()` via `set_settings_cmd` Tauri command | Uses tauri-plugin-store with atomic writes |
| Window dragging in frameless window | Custom drag logic | `data-tauri-drag-region` attribute on header element | Native OS drag behavior; single attribute |

**Key insight:** The project has a strong principle of using Tauri plugins rather than raw Windows API calls for OS-level features. This phase adds `@tauri-apps/plugin-dialog` as the one new dependency — everything else reuses existing infrastructure.

---

## Common Pitfalls

### Pitfall 1: Settings Window URL in Dev vs Production

**What goes wrong:** In dev mode, `tauri.conf.json` currently points both windows to `http://localhost:1420` (the single Vite dev server). The settings window will render the launcher's `App.vue` instead of `Settings.vue`.

**Why it happens:** Tauri v2's single-page dev server serves only one entry point by default. The multi-page Vite build (with a separate `settings.html`) is only effective after `pnpm tauri build`.

**How to avoid:** Add a `url` field to the settings window in `tauri.conf.json` pointing to `settings.html`. For dev mode: `"http://localhost:1420/settings.html"`. Vite's dev server serves any `.html` file under the project root by name. With the multi-page `rollupOptions.input` config in `vite.config.ts`, Vite will also serve `/settings.html` during dev.

**Warning signs:** Settings window shows search input and results list instead of settings UI.

### Pitfall 2: get_settings_cmd Missing Fields for Settings.vue

**What goes wrong:** `get_settings_cmd` in `store.rs` currently only returns 6 fields: `show_path`, `animation`, `data_dir`, `hotkey`, `theme`, `opacity`. Settings.vue needs ALL settings fields plus `is_portable`.

**Why it happens:** `get_settings_cmd` was written in Phase 5 to serve App.vue's needs only.

**How to avoid:** Extend `get_settings_cmd` to return the full `Settings` struct plus `is_portable` detection. Alternatively, create a new `get_full_settings_cmd` that Settings.vue uses. Since `Settings` already derives `Serialize`, `serde_json::json!(settings)` already captures all fields — just add `is_portable` detection via `paths::data_dir()` comparison.

**Warning signs:** Settings.vue has `undefined` values for autostart, additional_paths, excluded_paths, or reindex_interval.

### Pitfall 3: set_settings_cmd Does Not Exist Yet

**What goes wrong:** `store.rs` has `get_settings_cmd` (Tauri command) and `set_settings` (internal function), but there is no `set_settings_cmd` Tauri command that the frontend can invoke.

**Why it happens:** The CONTEXT.md notes "calls `set_settings` Tauri command" but looking at the code, `set_settings` is an internal Rust function, not a `#[tauri::command]`. The frontend cannot call it directly.

**How to avoid:** Add `set_settings_cmd` as a `#[tauri::command]` in `store.rs`. It should accept a full `Settings` struct (or a partial patch), call `set_settings()` internally, and register it in `lib.rs`'s `invoke_handler`.

**Warning signs:** `invoke('set_settings_cmd', ...)` throws "command not found" error in the console.

### Pitfall 4: emit() vs emitTo() Cross-Window Events

**What goes wrong:** Using `emit('settings-changed', payload)` from Settings.vue sends the event to ALL windows, including Settings.vue itself. If Settings.vue has a `listen('settings-changed')` handler (e.g., for self-consistency), it will receive its own events and may cause update loops.

**Why it happens:** Tauri v2's `emit()` is application-wide broadcast.

**How to avoid:** Use `emitTo('launcher', 'settings-changed', payload)` from Settings.vue to target only the launcher window. This is available from `@tauri-apps/api/event`.

**Warning signs:** Double updates or console logs showing Settings.vue processing its own emitted events.

### Pitfall 5: decorations: false Requires Custom Window Chrome

**What goes wrong:** Changing `decorations: false` for the settings window removes the OS title bar — meaning no title, no minimize/maximize/close buttons, and the window is not draggable by default.

**Why it happens:** This is the locked decision from CONTEXT.md (frameless settings window with custom header). Currently `tauri.conf.json` has `"decorations": true` for the settings window — this must be changed.

**How to avoid:** Add a custom header in Settings.vue with `data-tauri-drag-region` attribute for drag support. Include a close button that calls `getCurrentWindow().close()`. The header should show the app icon and "Riftle Settings" text per the CONTEXT.md specifics.

**Warning signs:** Settings window appears but cannot be dragged, has no title, no way to close.

### Pitfall 6: tauri-plugin-dialog Not Yet in Project

**What goes wrong:** `PathList.vue` calls `open({ directory: true })` from `@tauri-apps/plugin-dialog`, but the plugin is not installed (not in Cargo.toml, package.json, or capabilities).

**Why it happens:** The project only has `tauri-plugin-opener`, not `tauri-plugin-dialog`.

**How to avoid:** Add `tauri-plugin-dialog = "2"` to Cargo.toml, `@tauri-apps/plugin-dialog` to package.json, `app.handle().plugin(tauri_plugin_dialog::init())` to lib.rs setup, and `"dialog:default"` to capabilities/default.json.

**Warning signs:** `import { open } from '@tauri-apps/plugin-dialog'` throws module not found, or `invoke` calls for dialog commands fail.

### Pitfall 7: Ctrl+, Intercepted Before Reaching Vue

**What goes wrong:** Adding `Ctrl+,` as a keyboard shortcut in the launcher to open settings (SETT-03). If a global shortcut or browser focus behavior intercepts Comma, it may not reach the `onKeyDown` handler.

**Why it happens:** Comma is a standard character key — usually not intercepted. But if the search input has focus, the `,` character would be typed into the query instead of triggering the shortcut. The handler must check for `e.ctrlKey` first and call `e.preventDefault()`.

**How to avoid:** In `onKeyDown`, add an early check: `if (e.key === ',' && e.ctrlKey) { e.preventDefault(); openSettings(); return; }`.

**Warning signs:** Pressing Ctrl+, types a comma in the search box instead of opening settings.

### Pitfall 8: Background Timer Restart on reindex_interval Change

**What goes wrong:** When the user changes `reindex_interval` via the dropdown, the in-memory background timer does not update — only the persisted setting changes. The timer continues running at the old interval.

**Why it happens:** `start_background_tasks` in `indexer.rs` reads the interval at startup and creates a timer thread with that fixed interval. There is no mechanism to change the running timer's interval.

**How to avoid:** The CONTEXT.md notes "emits event to restart background timer in Rust (new Rust command or existing mechanism)". The existing `reindex` command plus the `timer_tx` Sender (`Arc<Mutex<Sender>>` in managed state) provides a mechanism. A new `set_reindex_interval_cmd` command could read the new interval, persist it via `set_settings`, then restart the background timer by dropping the old timer thread and spawning a new one. Alternatively, the simpler approach: on app restart, the persisted setting takes effect. Evaluate during planning whether live timer restart is actually needed for a good UX (the interval change is not critical to be immediate).

---

## Code Examples

Verified patterns from existing codebase:

### Existing: listen() pattern in App.vue (mirrors settings-changed)
```typescript
// Source: src/App.vue, lines 302-319
unlistenShow = await listen('launcher-show', async () => {
  menuVisible.value = false
  isVisible.value = false
  results.value = []
  query.value = ''
  await getCurrentWindow().setSize(new LogicalSize(500, 56)).catch(console.error)
  await getCurrentWindow().center().catch(console.error)
  await nextTick()
  isVisible.value = true
  await nextTick()
  inputRef.value?.focus()
})
```

### Existing: invoke() for settings (App.vue onMounted)
```typescript
// Source: src/App.vue, lines 253-266
const settings = await invoke<{
  show_path: boolean
  animation: string
  data_dir: string
}>('get_settings_cmd')
showPath.value = settings.show_path
animMode.value = (settings.animation ?? 'slide') as typeof animMode.value
dataDir.value  = settings.data_dir
```

### Existing: open_settings_window call (App.vue)
```typescript
// Source: src/App.vue, line 200
async function openSettings() {
  closeMenu()
  await invoke('open_settings_window').catch(console.error)
}
```
**Note:** `open_settings_window` is called by App.vue but NOT yet registered in `lib.rs` `invoke_handler`. This is a gap to fill in Phase 8.

### Existing: settings window already declared in tauri.conf.json
```json
// Source: src-tauri/tauri.conf.json, lines 29-38
{
  "label": "settings",
  "title": "Riftle Settings",
  "width": 800,
  "height": 600,
  "minWidth": 600,
  "minHeight": 400,
  "decorations": true,
  "visible": false
}
```
**Action required:** Change `"decorations": true` to `"decorations": false`. Add `"url": "settings.html"` for multi-page routing.

### Existing: update_hotkey command (hotkey.rs)
```rust
// Source: src-tauri/src/hotkey.rs, lines 44-65
#[tauri::command]
pub fn update_hotkey(
    app: tauri::AppHandle,
    hotkey: String,
    data_dir: tauri::State<std::path::PathBuf>,
) -> Result<(), String> {
    let mut settings = crate::store::get_settings(&app, &data_dir);
    app.global_shortcut()
        .unregister(settings.hotkey.as_str())
        .unwrap_or_else(|e| eprintln!("[hotkey] unregister failed: {}", e));
    crate::hotkey::register(&app, &hotkey);
    settings.hotkey = hotkey;
    crate::store::set_settings(&app, &data_dir, &settings);
    Ok(())
}
```

### Existing: capabilities/default.json already covers settings window
```json
// Source: src-tauri/capabilities/default.json
{
  "windows": ["launcher", "settings"],
  "permissions": [
    "core:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-size",
    ...
  ]
}
```
**Action required:** Add `"dialog:default"` (or `"dialog:allow-open"`) when tauri-plugin-dialog is added.

### New: open_settings_window Rust command (to be added)
```rust
// In lib.rs or new settings.rs module
#[tauri::command]
pub fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    let win = app.get_webview_window("settings")
        .ok_or_else(|| "settings window not found".to_string())?;
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}
```

### New: set_settings_cmd Rust command (to be added in store.rs)
```rust
#[tauri::command]
pub fn set_settings_cmd(
    app: tauri::AppHandle,
    data_dir: tauri::State<std::path::PathBuf>,
    settings: Settings,  // full struct from frontend JSON
) -> Result<(), String> {
    set_settings(&app, &data_dir, &settings);
    Ok(())
}
```

### New: tokens.css root variables (from settings-styling.md)
```css
/* Source: .planning/settings-styling.md */
:root {
  --color-bg:           #1c1c1e;
  --color-bg-lighter:   #242427;
  --color-bg-darker:    #181818;
  --color-text:         #f0f0f0;
  --color-text-muted:   #888;
  --color-text-dim:     #555558;
  --color-accent:       #0A84FF;
  --color-selection-bg: rgba(10, 132, 255, 0.18);
  --color-border:       rgba(255, 255, 255, 0.15);
  --color-divider:      rgba(255, 255, 255, 0.094);
  --font-sans:          'Inter', sans-serif;
  --font-mono:          'JetBrains Mono', monospace;
  --font-size-xl:       18px;
  --font-size-base:     14px;
  --font-size-sm:       13px;
  --font-size-xs:       11px;
  --spacing-xs:         4px;
  --spacing-sm:         8px;
  --spacing-md:         12px;
  --spacing-lg:         16px;
  --radius:             9px;
  --radius-sm:          4px;
  --duration-fast:      120ms;
  --duration-normal:    180ms;
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hardcoded color/spacing values in App.vue | CSS custom properties in tokens.css | Phase 8 Wave 0 | Both windows share design language; theming enabled |
| Single window (launcher only) | Two windows: launcher + settings | Phase 8 | Requires Vite multi-page build + cross-window events |
| get_settings_cmd returns 6 fields for App.vue only | get_settings_cmd returns all fields + is_portable | Phase 8 | Settings.vue can render all sections correctly |
| No set_settings Tauri command | set_settings_cmd Tauri command | Phase 8 | Settings.vue can persist changes from frontend |

**Deprecated/outdated:**
- `open_settings_window` in App.vue calls `invoke('open_settings_window')` — this command was wired in App.vue during Phase 7 but the Rust handler does not yet exist. Phase 8 must register it in `lib.rs` invoke_handler.

---

## Open Questions

1. **reindex_interval live update mechanism**
   - What we know: The background timer in `indexer.rs` is started once with a fixed interval; `timer_tx` is in managed state as `Arc<Mutex<Sender<()>>>` but this only resets the timer (not changes its interval)
   - What's unclear: Whether live timer interval change is needed for good UX, or whether "takes effect on next app restart" is acceptable
   - Recommendation: Implement as "save setting, restart background task thread with new interval." Since `timer_tx` Sender is in managed state, the simplest approach is to send a signal to kill the old timer and spawn a new one with the updated interval. Alternatively, accept that the change takes effect on restart for MVP — this is a low-visibility UX tradeoff.

2. **settings-main.ts and tokens.css double-import**
   - What we know: `src/main.ts` will import `tokens.css` once. `src/settings-main.ts` is a separate entry point for the settings window.
   - What's unclear: Whether both entry points import `tokens.css` independently (correct), or whether there's a shared mechanism
   - Recommendation: Both `main.ts` and `settings-main.ts` independently import `tokens.css`. Vite bundles them separately; both windows get the tokens. This is the standard Vite multi-page pattern.

3. **settings.html URL in dev mode**
   - What we know: `tauri.conf.json` currently has no `url` field for the settings window; it inherits the default `devUrl` from the build config (`http://localhost:1420`)
   - What's unclear: Whether Tauri v2 requires an explicit `url` field in the window config or inherits from `build.devUrl`
   - Recommendation: Add explicit `"url": "settings.html"` to the settings window config. During dev, Vite serves `settings.html` from the project root at `http://localhost:1420/settings.html`. This is the standard multi-page Tauri pattern.

---

## Sources

### Primary (HIGH confidence)
- Direct inspection of `src-tauri/tauri.conf.json` — confirmed settings window config, decorations, URL
- Direct inspection of `src-tauri/capabilities/default.json` — confirmed both windows covered, missing dialog permission
- Direct inspection of `src-tauri/src/lib.rs` — confirmed invoke_handler gaps (open_settings_window not registered)
- Direct inspection of `src-tauri/src/store.rs` — confirmed get_settings_cmd returns limited fields, no set_settings_cmd command exists
- Direct inspection of `src-tauri/src/hotkey.rs` — confirmed update_hotkey fully implemented
- Direct inspection of `src/App.vue` — confirmed openSettings() calls invoke('open_settings_window'); listen() pattern for launcher-show
- Direct inspection of `.planning/settings-styling.md` — confirmed full token values and Option B decision

### Secondary (MEDIUM confidence)
- Tauri v2 multi-window patterns: `get_webview_window()` + `show()` + `set_focus()` — inferred from existing hotkey.rs and Phase 6 patterns in codebase; consistent with standard Tauri v2 documentation patterns
- Vite multi-page build with `rollupOptions.input` — standard Vite documentation pattern; consistent with existing `vite.config.ts` structure

### Tertiary (LOW confidence)
- `@tauri-apps/plugin-dialog` API for `open({ directory: true })` — standard Tauri plugin pattern; not verified against Context7 but consistent with official Tauri v2 plugin conventions

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified directly from existing codebase files
- Architecture: HIGH — patterns extracted from existing working code + locked CONTEXT.md decisions
- Pitfalls: HIGH — identified from direct code gaps (missing commands, missing decorations change, missing url field)

**Research date:** 2026-03-08
**Valid until:** 2026-04-08 (Tauri v2 stable; no fast-moving parts in this phase's dependencies)
