# Phase 8: Settings Window - Context

**Gathered:** 2026-03-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the full Settings window as a separate, single-instance Tauri window with four sections — General, Hotkey, Search, Appearance — and reactive live updates to the open launcher. Pre-phase styling infrastructure (tokens.css, App.vue refactor, ui primitives) is Wave 0 of this phase.

Keyboard shortcut registration (update_hotkey Rust command) is already implemented from Phase 9. Phase 8 only builds the Settings UI that calls it.

</domain>

<decisions>
## Implementation Decisions

### Pre-phase infrastructure (Wave 0)

- **Tokens**: Create `src/styles/tokens.css` with all CSS custom properties as defined in `.planning/settings-styling.md`. Import once in `src/main.ts` before app mount — available globally to both windows.
- **App.vue refactor**: Replace all hardcoded values in App.vue's `<style>` block with CSS variables. Purely mechanical substitution — no behavior change. Verified by visual smoke test only (no pixel-perfect diff needed).
- **UI primitives**: Build only the primitives actively used in Phase 8 (all 5 end up needed): `Section.vue`, `Row.vue`, `Toggle.vue`, `KeyCapture.vue`, `PathList.vue`. Location: `src/components/ui/`. No new dependencies.
- **Sequencing**: Tokens + primitives = Wave 0; App.vue refactor = Wave 1; Settings.vue = later waves.

### Settings window chrome

- **Frameless**: `decorations: false` in tauri.conf.json for the settings window. No OS title bar.
- **Custom header**: Custom in-page header with `data-tauri-drag-region`. Left side: app icon + "Riftle Settings". Right side: close button (calls `window.close()`).
- **Window configuration**: Normal framed sizing, min 600×400px (per SETT-01) — but frameless (decorations off).

### Settings layout

- **Single scrollable page**: All four sections stacked vertically, separated by section headings + horizontal dividers. Scroll to navigate. No tabs or sidebar.
- **Section order**: General → Hotkey → Search → Appearance (top to bottom).
- **Row pattern**: Each setting row uses `Row.vue` — label on left, control on right. Consistent layout across all sections.

### Reactive propagation

- **Mechanism**: Settings window emits `settings-changed` Tauri event with changed values. `App.vue` listens with `listen('settings-changed')` (same pattern as existing `launcher-show` event).
- **Persistence**: Settings write to disk immediately on every change (no save button, no unsaved state). Each control change calls `set_settings` Tauri command.
- **Which settings trigger live launcher update**:
  - `theme` → launcher applies `data-theme` attribute to root element; CSS token overrides handle theming
  - `opacity` → launcher sets CSS opacity/background-alpha via token or inline style
  - `show_path` → launcher's `showPath` ref updates
  - `reindex_interval` → emits event to restart background timer in Rust (new Rust command or existing mechanism)
- **Live preview**: Launcher updates in real time as user interacts (while dragging opacity slider, while toggling show_path). Not on mouseup/commit — on every input event.

### Theme implementation

- **Options**: System / Light / Dark (all three, per SETT-07). No deferral of Light theme.
- **Mechanism**: Selecting a theme sets `data-theme="light"` or `data-theme="dark"` on the launcher's root element. `tokens.css` includes `[data-theme="light"] { ... }` overrides. System follows `prefers-color-scheme` media query.

### General section

- **Autostart toggle**: In portable mode — visible but disabled (greyed out). Tooltip or adjacent note: "Not available in portable mode". User can see the feature exists; understands why inactive.
- **Autostart toggle**: In installed mode — functional, calls tauri-plugin-autostart.

### Hotkey section

- **Key capture UX**: Click input → shows "Press shortcut…" placeholder → user presses key combination → input displays captured combo (e.g. "Ctrl+Alt+Space") → saves immediately. Escape cancels capture and restores previous value.
- **Takes effect immediately**: Calls `update_hotkey` Tauri command (already implemented in hotkey.rs).

### Search section

- **Additional paths**: `PathList.vue` — add path via folder picker dialog, remove with [-] button. Changing paths triggers re-index.
- **Excluded paths**: Same `PathList.vue` component.
- **Re-index now**: Button triggers `reindex` Tauri command (already implemented in Phase 3).
- **Re-index interval**: Dropdown selector with discrete options: 5 min / 15 min / 30 min / 60 min / Manual only. Default: 15 min (matches `default_reindex_interval` in store.rs). Changing interval takes effect immediately.

### Claude's Discretion

- Exact token values — already defined in `.planning/settings-styling.md`
- Light/Dark theme color palette specifics (needs to be dark-aesthetic consistent)
- Opacity slider range (REQUIREMENTS.md says 0.85–1.0; implementation approach is Claude's call)
- Error state handling for invalid hotkey combinations
- Exact animation/transition for settings window open/close

</decisions>

<specifics>
## Specific Ideas

- Settings-styling.md approach confirmed: CSS custom properties + thin Vue component primitives (Option B). Both windows use tokens from day one.
- "App name + icon on left, close button right" — like `⛰️ Riftle Settings [X]` in the custom header.
- The settings window should feel minimal and consistent with the launcher's dark aesthetic — not generic Windows settings.
- Token naming confirmed: semantic scale (`--color-accent`, `--spacing-md`) with no namespace prefix.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets

- `src/App.vue`: Source of all current hardcoded CSS values — the refactor target. Style block is unscoped (no `scoped` attribute), which must be preserved.
- `src/main.ts`: Entry point for `main.js` window — import `tokens.css` here. The settings window will need its own entry file (`settings-main.ts`) that also benefits from the global import *if* bundled separately, or shares main.ts's globals if Vite handles it.
- Existing Tauri events pattern: `listen('launcher-show', ...)` in App.vue — `listen('settings-changed', ...)` follows the same pattern.

### Established Patterns

- **Tauri commands**: All invocations use `invoke()` from `@tauri-apps/api/core`.
- **Settings struct** (`store.rs`): `hotkey`, `theme`, `opacity`, `show_path`, `autostart`, `additional_paths`, `excluded_paths`, `reindex_interval`, `animation`, `system_tool_allowlist` — all fields already defined with serde defaults.
- **Already implemented commands** Phase 8 UI calls: `update_hotkey` (hotkey.rs), `reindex` (indexer.rs), `get_settings_cmd` and `set_settings_cmd` (store.rs).
- **Unscoped styles**: App.vue uses unscoped `<style>` for RecycleScroller DOM compatibility — Settings.vue can use scoped styles.
- **isTauriContext guard**: Pattern from App.vue — guards all Tauri API calls; Settings.vue should do the same.

### Integration Points

- `src-tauri/src/lib.rs`: Register settings window + `open_settings_window` command (SETT-01, SETT-02).
- `src/App.vue`: Add `listen('settings-changed', ...)` handler; update `showPath`, theme, opacity reactively.
- `tauri.conf.json`: Settings window needs `decorations: false` added (currently not specified).
- `capabilities/default.json`: May need settings window label added to windows array.

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 08-settings-window*
*Context gathered: 2026-03-08*
