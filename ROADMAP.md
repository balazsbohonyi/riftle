# Roadmap

## Milestone 1 — v1 (Current)

### Phase 1 · Project Scaffold & Configuration
Bootstrap the Tauri v2 + Vue 3 + TypeScript project using `pnpm create tauri-app .`. Configure both application windows (`launcher` and `settings`) in `tauri.conf.json` with the correct flags (frameless, skip_taskbar, always_on_top for launcher; normal framed for settings). Add all required Rust crates to `Cargo.toml` and verify `pnpm tauri dev` starts cleanly.

**Requirements:** R01

---

### Phase 2 · Data Layer
Implement the SQLite database schema and settings persistence layer. Build the portable-mode detection logic — a single `app_data_dir()` helper that all other modules use to resolve file paths. Wire up `tauri-plugin-store` for settings with a typed `Settings` struct and sensible defaults.

**Requirements:** R02, R03, R04

---

### Phase 3 · Indexer
Build the Windows application indexer: crawl Start Menu, Desktop, and PATH locations plus user-defined additional paths; resolve `.lnk` shortcuts to their target executables; extract app icons via `ExtractIconEx` asynchronously; persist results to SQLite. Add a background re-index timer and a `notify`-based filesystem watcher for live updates.

**Requirements:** R05, R06, R07

---

### Phase 4 · Search Engine
Implement the `search` Tauri command using `nucleo` for fuzzy matching. Apply MRU-weighted ranking (match score + launch frequency). Handle the `>` prefix to surface system command results instead of app results.

**Requirements:** R08, R09

---

### Phase 5 · Launcher Window UI
Build the Vue 3 launcher window: frameless floating layout, autofocused search input, virtualised result list, keyboard navigation (↑↓ Enter Escape), conditional path display for the selected row, `[Admin]` badge on Ctrl+Shift hold, system command hint in the placeholder, and auto-hide on focus loss.

**Requirements:** R10, R11, R12

---

### Phase 6 · Launch Actions
Implement all three Tauri launch commands in Rust: normal launch, elevated launch via `ShellExecuteW runas`, and system commands (lock / shutdown / restart / sleep) via `windows-sys`. All actions hide the launcher window after execution. UAC cancellation is silently absorbed.

**Requirements:** R13

---

### Phase 7 · Context Menu
Add the right-click context menu as a custom Vue HTML overlay. Wire up Settings (open/focus settings window) and Quit Launcher (exit process). Handle dismiss-on-click-outside and Escape.

**Requirements:** R14

---

### Phase 8 · Settings Window
Build the full Settings window as a separate single-instance Tauri window. Implement all four sections: General (autostart, hidden in portable mode), Hotkey (live rebind), Search (additional/excluded paths, reindex), and Appearance (theme, opacity, show_path toggle). All appearance changes must propagate reactively to the open launcher.

**Requirements:** R15, R16

---

### Phase 9 · Global Hotkey
Register the configurable global hotkey via `tauri-plugin-global-shortcut`. Implement toggle behaviour (show/hide launcher), input clear-and-focus on show, and live hotkey rebinding when changed in Settings.

**Requirements:** R17

---

### Phase 10 · Packaging & Distribution
Configure `tauri build` to produce NSIS and MSI installer artifacts. Document the portable build process (raw exe + `launcher.portable` marker zip). Verify installers work on a clean Windows machine, WebView2 bootstrapper triggers correctly, and the portable build stores data in `./data/` as expected.

**Requirements:** R18, R19
