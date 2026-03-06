# Phase 5: Launcher Window UI - Context

**Gathered:** 2026-03-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the complete Vue 3 launcher window: frameless layout with search input, virtualised result list, full keyboard navigation (↑/↓/Enter/Escape/Ctrl+Shift+Enter), conditional path display, admin badge, and auto-hide on focus loss. Pure frontend — no new Rust commands. Consumes the `search()` Tauri command from Phase 4 and will stub `launch()`/`launch_elevated()`/`run_system_command()` calls that Phase 6 implements.

</domain>

<decisions>
## Implementation Decisions

### Visual Style
- **Background:** Dark with subtle top-to-bottom gradient — slightly lighter at top (~#242427 input area) fading to base (#1c1c1e) through results, slightly darker at bottom (~#181818). No glass/blur effect.
- **Selection highlight:** Blue accent background on selected row (e.g., #0A84FF or #1E6FD9 at ~20% opacity, full text contrast maintained)
- **Border:** 1px semi-transparent white border (#ffffff25) on the launcher window edges
- **Input/results separator:** 1px divider line (#ffffff18) between the search input area and result list
- **Text:** App name #f0f0f0 (unselected), #ffffff (selected); path line #888–#999 (dimmed)
- **Theme:** Always dark — no system theme detection in Phase 5. Phase 8 Settings handles theme switching.
- **Fonts:** Inter (bundled via Vite) for all text; JetBrains Mono (bundled) for path lines specifically

### Result Row Density
- **Icon size:** 32×32px — better recognition over 16×16; consistent with modern launchers
- **Row height:** ~48px (accommodates 32×32 icon with vertical padding)
- **Path line:** Appears below app name on the **selected row only** when `show_path` setting is true. Unselected rows show name only regardless of `show_path`.
- **Path text:** JetBrains Mono, ~12px, #888, truncated with ellipsis if overflows
- **Admin badge:** When Ctrl+Shift is held, a shield icon + "Admin" label appears on the **right side** of the selected row in the accent color. Doesn't shift layout — floats in the row's right margin.
- **Search input area height:** 52–56px with generous vertical padding
- **Search input font:** Inter, 18–20px

### Search Input Design
- **Icon:** Flat monochrome SVG magnifier icon, **right-aligned** in the input area (not left)
- **Input background:** Blends into launcher background — no separate border or pill box for the `<input>` element itself. The launcher border and divider define the zones.
- **Placeholder:** "Search apps, or > for system commands…" (from REQUIREMENTS.md LWND-11)
- **Cursor:** Standard text cursor; input is bare/borderless

### Transitions & Animation
- **Window show/hide:** 3 configurable modes, read from Settings (`animation` field to be added to Settings struct):
  - `"instant"` — no animation, window appears/disappears immediately
  - `"fade"` — opacity 0→1 on show, 1→0 on hide (~120ms)
  - `"slide"` — window slides down slightly + fades in on show, reverses on hide (~180ms) **(default)**
  - Phase 5 implements all 3 modes and reads the setting; Phase 8 exposes the picker in the Appearance section
- **Result list update:** Instant replace on each keystroke — no crossfade between result sets
- **Keyboard navigation:** Instant highlight change — no animated cursor sliding
- **Window height:** Animated resize as result count changes (150ms ease-out) from input-only height up to input + 8 rows max

### System Commands Display
- System command results use the ⚙️ icon (from `system_command.png` in Phase 4)
- No path line rendered for system commands regardless of `show_path` setting
- Same row density and selection style as app results

### Empty / Loading States
- **No query entered:** Empty list, window shows input area only (minimum height)
- **Icon loading (async):** Show generic icon placeholder while `{data_dir}/icons/{app_id}.png` is not yet extracted; swap in when ready
- **No results for query:** Empty list (window collapses to input-only height)
- No loading spinner or skeleton rows — instant feedback from cached nucleo index

### Claude's Discretion
- Exact blue accent hex value (around #0A84FF / #1E6FD9)
- Gradient stop values and easing
- Inter and JetBrains Mono weight choices (400/500 for names, 400 for paths)
- Virtualisation approach (vue-virtual-scroller, @tanstack/virtual, or manual implementation)
- Exact Tauri window resize mechanism (setSize or CSS height transition)
- Icon image loading/caching strategy in Vue (img src via convertFileSrc or asset protocol)

</decisions>

<specifics>
## Specific Ideas

- Font pairing: Inter for names (clean, modern), JetBrains Mono for paths (developer-familiar, clear for file paths)
- Magnifier icon right-aligned — less conventional but cleaner left margin for text alignment with result row names

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `App.vue` — minimal placeholder; has `background: #1c1c1e`, `border-radius: 12px`, transparent window wiring. Phase 5 replaces this wholesale with the full launcher UI.
- `search()` Tauri command (Phase 4) — returns `Vec<{ id, name, icon_path, path, kind }>`. `icon_path` is a filename relative to `{data_dir}/icons/`.
- `reindex()` Tauri command (Phase 3) — not directly called by launcher UI but shares managed state

### Established Patterns
- Transparent window with CSS background — Phase 1 confirmed this works; `html/body/#app` all need `height: 100%` chain
- No Pinia / no global state store — simple `ref`/`reactive` in component scope is the established pattern
- `invoke()` from `@tauri-apps/api/core` for Tauri commands
- `appWindow` from `@tauri-apps/api/window` for window operations (hide, focus events)
- No existing component library — all UI is custom CSS + Vue template

### Integration Points
- `invoke('search', { query })` → `Vec<SearchResult>` — result list data source
- `invoke('launch', { id })` — Phase 6 stub (write the call in Phase 5, Phase 6 implements the command)
- `invoke('launch_elevated', { id })` — Phase 6 stub
- `invoke('run_system_command', { cmd })` — Phase 6 stub
- `appWindow.hide()` — called on Escape, focus loss, and after successful launch
- `appWindow.onFocusChanged()` — triggers auto-hide when focus leaves
- `convertFileSrc(path)` from `@tauri-apps/api/core` — converts local file paths to Tauri asset URLs for `<img src>`
- Settings `show_path` and `animation` fields — read via `invoke('get_settings')` or reactive store
- `animation` field does not yet exist in Settings struct — needs to be added to `store.rs` with default `"slide"`

</code_context>

<deferred>
## Deferred Ideas

- Animation mode picker UI (instant / fade / slide) → Phase 8 Appearance section in Settings Window
- `animation` field addition to Settings struct → can be done in Phase 5 (Rust side) or deferred to Phase 8 — note for planner to decide where it fits cleanly

</deferred>

---

*Phase: 05-launcher-window-ui*
*Context gathered: 2026-03-06*
