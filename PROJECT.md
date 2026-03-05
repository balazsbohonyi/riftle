# Launcher — Minimal Windows App Launcher

## Vision

A keyboard-first, minimal application launcher for Windows built with Tauri v2 (Rust + Vue 3 + TypeScript). Users summon it with a global hotkey, type to fuzzy-search installed apps, and launch with Enter — entirely without touching the mouse. Inspired by Flow Launcher but intentionally smaller in scope, with a clean architecture designed for future extensibility.

## Stack

- **Runtime:** Tauri v2
- **Backend:** Rust
- **Frontend:** Vue 3 + TypeScript + Vite
- **Package manager:** pnpm
- **Database:** SQLite via `rusqlite` (bundled)
- **Platform:** Windows only (Win10 1803+, Win11)

## Key Design Principles

- Sub-100ms hotkey-to-visible response time
- Frameless floating window, no taskbar entry, no tray icon
- All data paths are portable-aware (installer vs. portable mode)
- Single IPC boundary: Rust handles all OS interactions, Vue handles all UI
- Settings are persisted via `tauri-plugin-store` and applied reactively where possible

## Project Structure

```
src-tauri/
├── main.rs                — bootstrap, window management
├── hotkey.rs              — global shortcut registration
├── indexer.rs             — Windows path crawl + user-defined paths
├── search.rs              — fuzzy search over SQLite index
├── system_commands.rs     — lock / shutdown / restart / sleep
├── commands.rs            — Tauri #[command] IPC handlers
├── db.rs                  — SQLite schema & queries
└── store.rs               — settings persistence (portable-aware)

src/
├── launcher/
│   ├── Launcher.vue       — main window root
│   ├── SearchInput.vue    — controlled input + keyboard nav
│   ├── ResultList.vue     — virtualised result rows
│   └── ContextMenu.vue    — right-click HTML overlay
└── settings/
    └── Settings.vue       — settings window root + tabs
```

## IPC Commands

| Command | Description |
|---|---|
| `search(query)` | Returns ranked Result[] (apps + system cmds if > prefix) |
| `launch(id)` | Normal app launch |
| `launch_elevated(id)` | Launch via ShellExecuteW runas |
| `run_system_command(cmd)` | lock / shutdown / restart / sleep |
| `get_settings()` | Returns full settings object |
| `set_settings(patch)` | Merges and persists settings |
| `reindex()` | Triggers manual re-index |
| `open_settings_window()` | Creates / focuses settings window |
| `quit()` | Exits the Tauri process |

## Rust Crates

```toml
tauri = { version = "2", features = [...] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-autostart = "2"
tauri-plugin-store = "2"
rusqlite = { version = "0.31", features = ["bundled"] }
walkdir = "2"
notify = "6"
nucleo = "0.5"
windows-sys = { version = "0.52", features = [
  "Win32_UI_Shell",
  "Win32_System_Shutdown",
  "Win32_System_Power",
  "Win32_System_RemoteDesktop"
] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```
