# Riftle

A keyboard-first, minimal application launcher for Windows built with Tauri v2 (Rust + Vue 3 + TypeScript). Summon it with a global hotkey, type to fuzzy-search installed apps, and launch with Enter — entirely without touching the mouse.

Inspired by Flow Launcher but intentionally smaller in scope, with a clean architecture designed for future extensibility.

## Features

- **Sub-100ms response** from hotkey to visible window
- **Fuzzy search** over installed apps with MRU-weighted ranking
- **Keyboard navigation** — arrow keys, Enter, Escape
- **Elevated launch** via Ctrl+Shift+Enter (UAC runas)
- **System commands** — lock, shutdown, restart, sleep via `>` prefix
- **Right-click context menu** — Settings and Quit
- **Frameless floating window** — no taskbar entry, no tray icon
- **Configurable global hotkey** with live rebinding
- **Portable mode** — stores all data in `./data/` next to the exe
- **Appearance settings** — theme, opacity, path display toggle

## Stack

| Layer | Technology |
|---|---|
| Runtime | Tauri v2 |
| Backend | Rust |
| Frontend | Vue 3 + TypeScript + Vite |
| Package manager | pnpm |
| Database | SQLite via `rusqlite` (bundled) |
| Platform | Windows 10 1803+, Windows 11 |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- WebView2 runtime (pre-installed on Windows 11; auto-bootstrapped on Windows 10)

### Development

```sh
pnpm install
pnpm tauri dev
```

### Build

```sh
pnpm tauri build
```

Produces NSIS and MSI installer artifacts in `src-tauri/target/release/bundle/`.

### Portable Build

Copy the release exe alongside a `launcher.portable` marker file. All settings and the app index will be stored in `./data/` relative to the exe.

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

## Usage

| Action | Shortcut |
|---|---|
| Show / hide launcher | Configurable (default: Alt+Space) |
| Navigate results | Arrow Up / Down |
| Launch selected | Enter |
| Launch as Administrator | Ctrl+Shift+Enter |
| Dismiss | Escape |
| System commands | Type `>` prefix (e.g. `> shutdown`) |
| Context menu | Right-click |

## Settings

Open the Settings window from the right-click context menu or by searching for "Settings". Available sections:

- **General** — autostart with Windows (hidden in portable mode)
- **Hotkey** — live rebind the global shortcut
- **Search** — additional/excluded paths, manual reindex trigger
- **Appearance** — theme, opacity, show/hide path under selected result

## License

MIT
