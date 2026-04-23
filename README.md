# Riftle

A keyboard-first, minimal application launcher for Windows built with Tauri v2 (Rust + Vue 3 + TypeScript). Summon it with a global hotkey, type to fuzzy-search installed apps, and launch with Enter — entirely without touching the mouse.

Inspired by Flow Launcher but intentionally smaller in scope, with a clean architecture designed for future extensibility.

<div align="center">
   <img src="./docs/images/riftle-demo.gif" alt="Chrono Weave">
</div>

## Table of Contents

[Status](#status)</br>
[Features](#features)</br>
[Stack](#stack)</br>
[Getting Started](#getting-started)</br>
&emsp;[Environment Setup](#environment-setup)</br>
&emsp;[Run in Development](#run-in-development)</br>
&emsp;[Build](#build)</br>
&emsp;[Portable Build](#portable-build)</br>
[Project Structure](#project-structure)</br>
[Usage](#usage)</br>
[Settings](#settings)</br>
[License](#license)

## Status

> **Work in progress — no stable release yet.**
>
> Riftle is under active development. Core functionality (hotkey, fuzzy search, launch, settings) works, but the project has not reached a tagged release. Expect breaking changes between commits and incomplete or missing features.

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

### Environment Setup

Install the required tools before running the project:

- [Rust](https://rustup.rs/) (stable, MSVC toolchain)
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- WebView2 runtime (pre-installed on Windows 11; auto-bootstrapped on Windows 10)

For a complete step-by-step walkthrough — including VS Code extensions, build tools, and optional WiX for MSI builds — see [docs/DEV-SETUP.md](docs/DEV-SETUP.md).

### Run in Development

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
src/                    — Vue 3 frontend
├── App.vue             — launcher UI (search input, result list, keyboard nav, context menu)
├── main.ts             — launcher Vue app mount
├── Settings.vue        — settings window (General, Hotkey, Search, Appearance)
├── settings-main.ts    — settings Vue app mount (multi-page build entry)
├── assets/
├── components/ui/      — settings UI primitives  (Toggle, KeyCapture, PathList, …)
├── styles/tokens.css   — CSS design tokens (colors, spacing, typography)

src-tauri/src/          — Rust backend
├── lib.rs              — app entry point, startup sequence, plugin registration
├── main.rs             — binary entry point
├── paths.rs            — data directory resolution (portable vs installed mode)
├── db.rs               — SQLite schema & queries
├── store.rs            — settings persistence (portable-aware)
├── indexer.rs          — Windows path crawl + user-defined paths + background re-index
├── search.rs           — Nucleo fuzzy/prefix/acronym search with MRU ranking
├── hotkey.rs           — global shortcut registration + update_hotkey command
├── commands.rs         — Tauri #[command] IPC handlers (launch, launch_elevated, quit_app)
├── system_commands.rs  — lock / shutdown / restart / sleep
```

## Usage

| Action | Shortcut |
|---|---|
| Show / hide launcher | Configurable (default: Shift+Enter) |
| Navigate results | Arrow Up / Down |
| Launch selected | Enter |
| Launch as Administrator | Shift+Enter |
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

[MIT](LICENSE)
