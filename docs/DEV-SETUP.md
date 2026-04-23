# Riftle — Dev Environment Setup

## 1. VS Code

Download and install VS Code from https://code.visualstudio.com/

### Extensions

Install all of the following from the VS Code Marketplace:

#### Rust & Tauri

| Extension | Link | Purpose |
|---|---|---|
| **rust-analyzer** | [marketplace](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) | Rust LSP — completions, type hints, inline errors. Non-negotiable. |
| **Even Better TOML** | [marketplace](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml) | `Cargo.toml` syntax highlighting + validation |
| **CodeLLDB** | [marketplace](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) | Rust debugger — attach to the Tauri backend process |
| **Tauri** | [marketplace](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) | `tauri.conf.json` schema + Tauri command snippets |

#### Vue & TypeScript

| Extension | Link | Purpose |
|---|---|---|
| **Vue - Official** (Volar) | [marketplace](https://marketplace.visualstudio.com/items?itemName=Vue.volar) | Vue 3 + TypeScript support. Replaces the old Vetur. |

#### Code Quality

| Extension | Link | Purpose |
|---|---|---|
| **ESLint** | [marketplace](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint) | Linting for TypeScript + Vue |
| **Prettier** | [marketplace](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) | Code formatting |

#### Optional but Recommended

| Extension | Link | Purpose |
|---|---|---|
| **Error Lens** | [marketplace](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens) | Shows Rust/TS errors inline next to the code — pairs great with rust-analyzer |
| **GitLens** | [marketplace](https://marketplace.visualstudio.com/items?itemName=eamodio.gitlens) | Enhanced Git history and blame |

---

## 2. Microsoft C++ Build Tools

Required by Rust's MSVC toolchain on Windows.

1. Download from https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. Run the installer
3. Select **"Desktop development with C++"** workload — this pulls in the MSVC compiler and Windows SDK
4. Complete installation and **restart your machine**

> If you already have Visual Studio 2019 or 2022 installed with the C++ workload, you can skip this step.

---

## 3. Rust

Install Rust via `rustup` — the official Rust toolchain manager.

1. Download the installer from https://rustup.rs/
2. Run it and follow the prompts
3. When asked about the toolchain, make sure **MSVC** is selected as the host triple (e.g. `x86_64-pc-windows-msvc`) — **not** GNU
4. After installation, restart your terminal and verify:

```powershell
rustc --version
cargo --version
```

### Keep Rust updated

```powershell
rustup update
```

---

## 4. Node.js

Use the LTS release. The recommended approach is via `winget` (built into Windows 10/11):

```powershell
winget install OpenJS.NodeJS.LTS
```

Or download the installer directly from https://nodejs.org/

Verify:

```powershell
node --version
npm --version
```

---

## 5. pnpm

Tauri projects work best with pnpm. Enable it via Node's built-in `corepack`:

```powershell
npm install -g pnpm@latest-10
```

Verify:

```powershell
pnpm --version
```

---

---

## 6. WebView2 Runtime

Tauri uses Microsoft Edge WebView2 to render the frontend.

**On Windows 10 (version 1803+) and Windows 11 it is pre-installed** — you likely don't need to do anything.

If for some reason it's missing, download the Evergreen Bootstrapper from:
https://developer.microsoft.com/en-us/microsoft-edge/webview2/

---

## 7. WiX Toolset *(MSI builds only)*

Required only when building `.msi` installer packages. Not needed for development or NSIS builds.

Install via winget:

```powershell
winget install WiXToolset.WiXToolset
```

Or download from https://wixtoolset.org/

> **Tip:** You can skip this entirely during early development. NSIS (`.exe` installer) builds work out of the box with `pnpm tauri build` without WiX.

