# Phase 1: Project Scaffold & Configuration - Context

**Gathered:** 2026-03-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Configure the existing Tauri v2 skeleton so both application windows (launcher and settings) are declared in tauri.conf.json with correct flags, all required Rust crates are in Cargo.toml, lib.rs is replaced with the real app structure (all plugins registered, no greet placeholder), and `pnpm tauri dev` starts cleanly.

</domain>

<decisions>
## Implementation Decisions

### App Identity
- Product name stays **riftle** (not renamed to "launcher")
- Bundle identifier changes to **com.riftle.launcher**
- Launcher window label: **launcher** (to match IPC design: open_settings_window references 'settings' label)
- Launcher window title: **Riftle** (visible in task manager/accessibility even without title bar)

### Launcher Window Flags (tauri.conf.json)
- Decorations: **false** (frameless)
- Shadow: **true** (system shadow, makes floating window feel grounded)
- Transparent: **true** (enables CSS-driven rounded corners / background effects)
- Resizable: **false** (fixed 640px width; height grows programmatically)
- Skip taskbar: **true** (no taskbar entry)
- Always on top: **true**
- Visible at launch: **false** (window starts hidden; hotkey shows it)
- Position: always centered on primary monitor when shown

### Settings Window Flags (tauri.conf.json)
- Label: **settings**
- Decorations: **true** (normal framed window)
- Min size: 600×400px
- Visible at launch: **false** (starts hidden; opened on demand)
- Skip taskbar: **false**

### lib.rs — Phase 1 Scope
- **Replace wholesale** — delete the greet command entirely, set up real app structure
- **Register ALL plugins** in Phase 1: tauri-plugin-store, tauri-plugin-global-shortcut, tauri-plugin-autostart, tauri-plugin-opener
- Plugins registered even though most won't have command handlers until later phases
- Stub Rust module files (db.rs, store.rs, hotkey.rs, indexer.rs, search.rs, commands.rs, system_commands.rs): **Claude's discretion** — create stubs if it makes later phases cleaner

### Crate Version Strategy
- Tauri plugins (global-shortcut, autostart, store): **exact versions** pinned (e.g. "2.x.y") — plugin APIs must match tauri minor version
- Domain crates (rusqlite, nucleo, walkdir, notify, windows-sys, serde, serde_json): **caret ranges** (e.g. "^0.31") — stable semver, benefits from bug fixes
- Cargo.lock: **committed** to git — this is a binary/app, reproducible builds required

### Vue Frontend — Phase 1 Scope
- Replace the default App.vue with a minimal launcher shell (not the full UI — that's Phase 5)
- The frontend should not throw errors and should load correctly in the launcher window

### Claude's Discretion
- Whether to create stub Rust module files in Phase 1 or leave to per-phase creation
- Exact plugin versions to pin (research current Tauri 2.x ecosystem versions)
- Whether to set up the settings window Vue shell in Phase 1 or leave for Phase 8

</decisions>

<code_context>
## Existing Code Insights

### Current State
- `tauri.conf.json`: single window "riftle" (800×600, no special flags) — needs full rewrite to 2-window config
- `Cargo.toml`: only tauri + tauri-plugin-opener + serde + serde_json — all domain crates missing
- `src-tauri/src/lib.rs`: default greet command scaffold — replace wholesale
- `src/App.vue`: default Tauri+Vue template — replace with launcher shell

### Reusable Assets
- Build pipeline already working: `pnpm dev` / `pnpm build` configured in tauri.conf.json
- TypeScript + Vite config already in place
- src-tauri/build.rs exists (unchanged)

### Integration Points
- tauri.conf.json window labels ("launcher", "settings") are referenced by open_settings_window() IPC command — must match exactly
- Transparent window flag interacts with CSS — frontend needs background set in CSS, not tauri window background

</code_context>

<specifics>
## Specific Ideas

- The tauri window background color should be transparent when transparent: true — background styling belongs entirely in CSS
- Cargo.lock should be in .gitignore currently (Tauri scaffold may ignore it) — verify and fix if needed

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-project-scaffold-configuration*
*Context gathered: 2026-03-05*
