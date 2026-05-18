# Riftle - Dev Environment Setup

This document is for contributors and local development. For the user-facing overview, see the main README sections for [features](../README.md#what-riftle-does), [basic use](../README.md#basic-use), [shortcuts](../README.md#shortcuts), and [settings](../README.md#settings).

## Technology Stack

| Layer | Technology |
|---|---|
| Runtime | Tauri v2 |
| Backend | Rust |
| Frontend | Vue 3 + TypeScript + Vite |
| Package manager | pnpm |
| Database | SQLite via `rusqlite` bundled |
| Platform | Windows 10 1803+, Windows 11 |

## Project Structure

```text
src/                    - Vue 3 frontend
|-- App.vue             - launcher UI: search input, result list, keyboard navigation, context menu
|-- main.ts             - launcher Vue app mount
|-- Settings.vue        - settings window: General, Hotkey, Appearance, Search, Shortcuts
|-- settings-main.ts    - settings Vue app mount for the multi-page build
|-- assets/             - frontend assets
|-- components/         - shared Vue components
|-- components/ui/      - settings UI primitives
`-- styles/tokens.css   - CSS design tokens

src-tauri/src/          - Rust backend
|-- lib.rs              - app entry point, startup sequence, plugin registration
|-- main.rs             - binary entry point
|-- paths.rs            - data directory resolution for portable vs installed mode
|-- db.rs               - SQLite schema and queries
|-- store.rs            - settings persistence
|-- indexer.rs          - Windows app crawling, background re-indexing, file watching
|-- search.rs           - fuzzy, prefix, acronym, system command, and shortcut search
|-- shortcuts.rs        - shortcut display names, IDs, and validation
|-- hotkey.rs           - global shortcut registration and updates
|-- commands.rs         - Tauri command handlers for launch, shortcuts, settings, quit
`-- system_commands.rs  - lock, shutdown, restart, sleep
```

## Quick Commands

```powershell
# Install dependencies
pnpm install

# Run the frontend only
pnpm dev

# Run the full Tauri app in development mode
pnpm tauri dev

# Type-check and build the frontend
pnpm build

# Run Rust tests
cd src-tauri
cargo test

# Build release installers
pnpm tauri build
```

Release installer artifacts are written under `src-tauri/target/release/bundle/`.

## Portable Development Build

Create a `riftle-launcher.portable` marker file next to the built executable. Settings, extracted icons, and index data will be stored in a local `data` directory beside the executable.

## Local Environment Setup

### 1. VS Code

Download and install VS Code from https://code.visualstudio.com/

### Extensions

Install all of the following from the VS Code Marketplace:

#### Rust & Tauri

| Extension | Link | Purpose |
|---|---|---|
| **rust-analyzer** | [marketplace](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) | Rust LSP: completions, type hints, inline errors |
| **Even Better TOML** | [marketplace](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml) | `Cargo.toml` syntax highlighting and validation |
| **CodeLLDB** | [marketplace](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) | Rust debugger for the Tauri backend process |
| **Tauri** | [marketplace](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) | `tauri.conf.json` schema and Tauri command snippets |

#### Vue & TypeScript

| Extension | Link | Purpose |
|---|---|---|
| **Vue - Official** (Volar) | [marketplace](https://marketplace.visualstudio.com/items?itemName=Vue.volar) | Vue 3 and TypeScript support |

#### Code Quality

| Extension | Link | Purpose |
|---|---|---|
| **ESLint** | [marketplace](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint) | Linting for TypeScript and Vue |
| **Prettier** | [marketplace](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) | Code formatting |

#### Optional but Recommended

| Extension | Link | Purpose |
|---|---|---|
| **Error Lens** | [marketplace](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens) | Shows Rust and TypeScript errors inline next to the code |
| **GitLens** | [marketplace](https://marketplace.visualstudio.com/items?itemName=eamodio.gitlens) | Enhanced Git history and blame |

### 2. Microsoft C++ Build Tools

Required by Rust's MSVC toolchain on Windows.

1. Download from https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. Run the installer.
3. Select **Desktop development with C++**. This installs the MSVC compiler and Windows SDK.
4. Complete installation and restart your machine.

If you already have Visual Studio 2019 or 2022 installed with the C++ workload, you can skip this step.

### 3. Rust

Install Rust via `rustup`, the official Rust toolchain manager.

1. Download the installer from https://rustup.rs/
2. Run it and follow the prompts.
3. When asked about the toolchain, make sure **MSVC** is selected as the host triple, for example `x86_64-pc-windows-msvc`. Do not use GNU.
4. After installation, restart your terminal and verify:

```powershell
rustc --version
cargo --version
```

#### Keep Rust Updated

```powershell
rustup update
```

### 4. Node.js

Use the LTS release. The recommended approach is via `winget`, which is built into Windows 10/11:

```powershell
winget install OpenJS.NodeJS.LTS
```

Or download the installer directly from https://nodejs.org/

Verify:

```powershell
node --version
npm --version
```

### 5. pnpm

Install pnpm with npm:

```powershell
npm install -g pnpm@latest-10
```

Verify:

```powershell
pnpm --version
```

### 6. WebView2 Runtime

Tauri uses Microsoft Edge WebView2 to render the frontend.

On Windows 10 version 1803+ and Windows 11, WebView2 is usually pre-installed. If it is missing, download the Evergreen Bootstrapper from https://developer.microsoft.com/en-us/microsoft-edge/webview2/

### 7. WiX Toolset

Required only when building `.msi` installer packages. It is not needed for development or NSIS builds.

Install via winget:

```powershell
winget install WiXToolset.WiXToolset
```

Or download from https://wixtoolset.org/

NSIS `.exe` installer builds work with `pnpm tauri build` without WiX.

### 8. App Icons

Riftle uses the Tauri CLI to generate icons from a single source image.

1. Place a high-resolution 1024x1024 PNG in the project root.
2. The current source file is expected to be named `app-icon.png`.
3. Regenerate all icon formats in `src-tauri/icons/`:

```powershell
pnpm tauri icon ./app-icon.png
cp src-tauri/icons/32x32.png src-tauri/icons/generic.png
cp src-tauri/icons/32x32.png src-tauri/icons/system_command.png
```

#### Troubleshooting Stale Icons

If `pnpm tauri dev` still shows the old icon, try the following:

1. Clean the Rust cache so Tauri embeds the regenerated icons:

```powershell
cd src-tauri
cargo clean -p riftle
cd ..
pnpm tauri dev
```

2. If the files in `src-tauri/icons/` look correct but Windows still shows the old icon, change the `version` in `src-tauri/tauri.conf.json` slightly to force Windows to refresh its cache.
