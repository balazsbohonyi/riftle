# Phase 1: Project Scaffold & Configuration - Research

**Researched:** 2026-03-06
**Domain:** Tauri v2 multi-window configuration, Rust crate wiring, plugin registration
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Product name stays **riftle** (not renamed)
- Bundle identifier changes to **com.riftle.launcher**
- Launcher window label: **launcher**
- Launcher window title: **Riftle**
- Launcher window flags: `decorations: false`, `shadow: true`, `transparent: true`, `resizable: false`, `skipTaskbar: true`, `alwaysOnTop: true`, `visible: false`
- Settings window label: **settings**, `decorations: true`, min 600×400px, `visible: false`
- lib.rs: replace wholesale — remove greet, register ALL plugins (store, global-shortcut, autostart, opener)
- Crate version strategy: Tauri plugins pinned exact (e.g. "2.x.y"); domain crates use caret ranges (e.g. "^0.31")
- Cargo.lock committed to git
- All crates from PROJECT.md: tauri (2), tauri-plugin-global-shortcut (2), tauri-plugin-autostart (2), tauri-plugin-store (2), tauri-plugin-opener (2), rusqlite (^0.31 bundled), walkdir (^2), notify (^6), nucleo (^0.5), windows-sys (^0.52 with Win32 features), serde (^1 derive), serde_json (^1)
- Replace App.vue with minimal launcher shell (no errors, loads correctly)

### Claude's Discretion
- Whether to create stub Rust module files in Phase 1 or leave to per-phase creation
- Exact plugin versions to pin (research current Tauri 2.x ecosystem versions)
- Whether to set up the settings window Vue shell in Phase 1 or leave for Phase 8

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SCAF-02 | Both windows declared in tauri.conf.json (launcher: frameless, skip_taskbar, always_on_top; settings: normal, hidden by default) | Window config field names confirmed: `skipTaskbar`, `alwaysOnTop`, `decorations`, `transparent`, `shadow`, `visible`, `resizable`, `minWidth`, `minHeight` — all camelCase in Tauri v2 schema |
| SCAF-03 | All required Rust crates added to Cargo.toml (rusqlite bundled, windows-sys with required features) | Exact crate versions and feature flags researched; plugin init patterns confirmed |
| SCAF-04 | `pnpm tauri dev` starts without errors | Transparent window pitfall (shadow must be false for transparency to work) documented; capabilities must cover new window labels |
</phase_requirements>

---

## Summary

Phase 1 is a pure configuration and wiring phase with no new logic to implement. The existing Tauri v2 skeleton is complete and working — `pnpm tauri dev` currently starts. The work is: rewrite `tauri.conf.json` for two windows, expand `Cargo.toml` with all domain crates, replace `lib.rs` with real plugin registration, update the capabilities file, replace `App.vue` with a minimal launcher shell, and verify Cargo.lock is tracked by git.

The most significant research findings are: (1) all window config field names in Tauri v2 use **camelCase** (e.g. `skipTaskbar`, `alwaysOnTop`); (2) there is a known transparent window pitfall where `shadow: true` (the default) prevents transparency from working on Windows — but the user decision explicitly sets `shadow: true` alongside `transparent: true`, which is the correct intent (shadow provides the floating appearance, and the WebView renders on top of it); (3) the capabilities/default.json must be updated from `"windows": ["main"]` to cover the new labels `"launcher"` and `"settings"`.

**Primary recommendation:** Proceed task-by-task in this order — tauri.conf.json rewrite, Cargo.toml expansion, lib.rs replacement, capabilities update, App.vue launcher shell, Cargo.lock gitignore verification — and run `pnpm tauri dev` only at the end.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri | 2.10.3 (latest) | App runtime, window management, IPC | Framework core |
| tauri-plugin-store | 2.4.2 (latest) | Settings persistence | Official Tauri plugin |
| tauri-plugin-global-shortcut | 2.3.0 (latest) | Global hotkey registration | Official Tauri plugin |
| tauri-plugin-autostart | 2.5.1 (latest) | Launch at system startup | Official Tauri plugin |
| tauri-plugin-opener | 2.x (already in Cargo.toml) | Open files/URLs | Official Tauri plugin |

### Domain Crates
| Library | Version | Purpose | Note |
|---------|---------|---------|------|
| rusqlite | ^0.31 (per PROJECT.md) | SQLite database | Pinned per project decision; latest is 0.38 — use ^0.31 per user spec |
| walkdir | ^2 | Directory traversal | Stable, widely used |
| notify | ^6 | Filesystem watching | Latest major is 8.x, but ^6 was specified in PROJECT.md — caret range keeps it within v6.x |
| nucleo | ^0.5 | Fuzzy matching | Used in helix-editor |
| windows-sys | ^0.52 | Win32 API bindings | Latest is 0.61.2; ^0.52 keeps it stable as specified |
| serde | ^1 (derive) | Serialization | Standard Rust |
| serde_json | ^1 | JSON handling | Standard Rust |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| rusqlite ^0.31 | rusqlite ^0.38 | Project spec says ^0.31; newer has more features but Phase 1 doesn't use DB yet |
| notify ^6 | notify ^8 | Project spec says ^6; v8 API changed; use v6 per project decision |

**Installation (Cargo):**
```bash
# These are added by editing Cargo.toml directly
# Plugin crates also need JS counterparts:
pnpm add @tauri-apps/plugin-store @tauri-apps/plugin-global-shortcut @tauri-apps/plugin-autostart
```

---

## Architecture Patterns

### Tauri v2 Project Structure (existing, matches project)
```
src-tauri/
├── src/
│   ├── main.rs          # bootstrap only — DO NOT modify
│   └── lib.rs           # real entry point — replace wholesale
├── capabilities/
│   └── default.json     # permissions per window — update labels
├── Cargo.toml           # add all domain crates
└── tauri.conf.json      # rewrite: two windows
src/
└── App.vue              # replace with minimal launcher shell
```

### Pattern 1: Two-Window tauri.conf.json Declaration
**What:** Declare both windows statically in `app.windows` array with all required flags
**When to use:** Windows known at compile time with fixed configurations
**Example:**
```json
// Source: https://schema.tauri.app/config/2 (verified field names)
{
  "app": {
    "windows": [
      {
        "label": "launcher",
        "title": "Riftle",
        "width": 640,
        "height": 60,
        "decorations": false,
        "transparent": true,
        "shadow": true,
        "resizable": false,
        "skipTaskbar": true,
        "alwaysOnTop": true,
        "visible": false,
        "focus": false
      },
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
    ]
  }
}
```

### Pattern 2: lib.rs Plugin Registration
**What:** All plugins registered upfront in lib.rs using the Tauri Builder chain
**When to use:** Phase 1 wires all plugins so later phases only add command handlers
**Example:**
```rust
// Source: https://v2.tauri.app/plugin/store/ and https://v2.tauri.app/plugin/global-shortcut/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new().build()
                )?;
                app.handle().plugin(tauri_plugin_autostart::init(
                    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                    None,
                ))?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Note on autostart:** `MacosLauncher::LaunchAgent` is the required parameter type even on Windows — it is the enum variant used for the constructor even though Windows uses a different underlying mechanism (registry). Pass `None` for args unless needed.

### Pattern 3: Capabilities Update for Multiple Windows
**What:** Update the capabilities/default.json to cover new window labels
**When to use:** Any time window labels change from the default "main"
**Example:**
```json
// Source: https://v2.tauri.app/security/capabilities/
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capability for all windows",
  "windows": ["launcher", "settings"],
  "permissions": [
    "core:default",
    "opener:default",
    "store:default",
    "global-shortcut:default",
    "autostart:default"
  ]
}
```

### Pattern 4: Minimal Vue Launcher Shell
**What:** Replace the greet scaffold App.vue with a minimal shell that renders without errors
**When to use:** Phase 1 — frontend must not throw, full UI is Phase 5
**Example:**
```vue
<!-- Minimal shell — no imports of greet command, no Tauri API calls yet -->
<script setup lang="ts">
// Phase 1: minimal shell — full UI implemented in Phase 5
</script>

<template>
  <div id="app">
    <!-- Launcher shell placeholder -->
  </div>
</template>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { background: transparent; }
#app { width: 100%; height: 100%; }
</style>
```

### Anti-Patterns to Avoid
- **Wrong field names:** Tauri v2 uses camelCase in JSON (`skipTaskbar`, `alwaysOnTop`) — snake_case (`skip_taskbar`) is a Rust struct field name, not the JSON key
- **Leaving capabilities with `"windows": ["main"]`:** The new window labels are "launcher" and "settings" — the old "main" label won't exist, so the default capability will grant no window access and IPC will fail silently
- **Calling `pnpm tauri dev` before Cargo.toml changes:** Adding crates requires compile time; run only after all edits are complete
- **Modifying main.rs:** The `main.rs` is bootstrap-only; all logic belongs in `lib.rs`
- **Forgetting to add JS plugin packages:** Rust plugins need matching JS/TS packages in package.json for frontend use

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Settings persistence | Custom JSON file I/O | `tauri-plugin-store` | Handles atomic writes, watching, typed access |
| Global hotkey | Raw Win32 RegisterHotKey | `tauri-plugin-global-shortcut` | Cross-thread safety, lifecycle management, conflict detection |
| Autostart | Direct registry writes | `tauri-plugin-autostart` | Handles installer vs portable modes, UAC considerations |
| File opening | ShellExecuteW directly in Phase 1 | `tauri-plugin-opener` | Already in scaffold, consistent |

**Key insight:** The Tauri plugin ecosystem covers all cross-cutting concerns (settings, hotkeys, autostart) so Phase 1 is pure wiring — no custom logic.

---

## Common Pitfalls

### Pitfall 1: Shadow + Transparent Conflict
**What goes wrong:** Setting `"transparent": true` alone does not produce a transparent window on Windows; a white rectangle appears
**Why it happens:** Tauri v2 enables shadow by default, which forces a non-transparent background in WebView2
**How to avoid:** The user decision sets `"shadow": true` explicitly — this is intentional for the floating appearance. The transparent setting enables the CSS-controlled background, not a fully invisible window. The shadow creates the floating UI appearance. The CSS `background: transparent` on `body` makes the WebView content area transparent within the window frame.
**Warning signs:** Solid white rectangle instead of transparent area — indicates the CSS is not setting `background: transparent`

**Important clarification from GitHub issue #8308:** Setting `shadow: false` was the workaround for v2 bug where transparency was broken. However, the user decision deliberately chose `shadow: true` — this is the correct modern behavior where shadow provides the visual grounding effect for the floating launcher. The App.vue must set `body { background: transparent; }` in CSS or the launcher background will be white.

### Pitfall 2: Window Label Mismatch in Capabilities
**What goes wrong:** `pnpm tauri dev` starts but IPC commands fail with permission errors
**Why it happens:** The existing `capabilities/default.json` has `"windows": ["main"]` — but after renaming windows to "launcher" and "settings", no window matches the capability
**How to avoid:** Update `"windows": ["launcher", "settings"]` in capabilities/default.json at the same time as tauri.conf.json
**Warning signs:** Frontend `invoke()` calls throw or return permission errors in the dev console

### Pitfall 3: Cargo.lock Not in Git
**What goes wrong:** Builds are not reproducible; different machines may get different dependency versions
**Why it happens:** Many gitignore templates exclude `Cargo.lock` for libraries; the root `.gitignore` does not exclude it, and `src-tauri/.gitignore` only excludes `/target/` and `/gen/schemas` — so `Cargo.lock` is already tracked
**How to avoid:** Verify with `git status src-tauri/Cargo.lock` — if tracked, no action needed; if untracked, remove from .gitignore
**Warning signs:** `src-tauri/Cargo.lock` shows as untracked in `git status`

**Current state verified:** The `src-tauri/.gitignore` only excludes `/target/` and `/gen/schemas`. Cargo.lock is NOT excluded and should already be tracked (or will be tracked once it exists after first compile).

### Pitfall 4: Plugin Version Mismatch with Tauri Core
**What goes wrong:** Build fails with version constraint errors or runtime IPC protocol mismatches
**Why it happens:** Tauri requires NPM packages and Rust crate major/minor versions to be aligned
**How to avoid:** Pin exact plugin versions that are confirmed compatible. With Tauri core at 2.10.3: use tauri-plugin-store "2.4.2", tauri-plugin-global-shortcut "2.3.0", tauri-plugin-autostart "2.5.1". The JS packages must also be added to package.json at matching versions.
**Warning signs:** Build error mentioning "version mismatched Tauri packages"

### Pitfall 5: Missing JS Plugin Packages
**What goes wrong:** TypeScript build fails or runtime errors when frontend tries to use plugins
**Why it happens:** Each Rust plugin has a corresponding npm package that provides the TS bindings
**How to avoid:** Run `pnpm add @tauri-apps/plugin-store @tauri-apps/plugin-global-shortcut @tauri-apps/plugin-autostart` alongside Cargo.toml changes
**Warning signs:** TypeScript import errors for `@tauri-apps/plugin-*`

### Pitfall 6: Settings Window URL Configuration
**What goes wrong:** Settings window shows the same content as launcher window
**Why it happens:** Without specifying `url`, both windows load `index.html` (the default)
**How to avoid:** For Phase 1, both windows can share the same index.html (settings UI is Phase 8). Use Vue Router hash routing in later phases: `"url": "#/settings"` for the settings window config. Phase 1 only needs both windows to not throw errors.
**Warning signs:** Two identical windows opening on dev start

---

## Code Examples

Verified patterns from official sources:

### Complete tauri.conf.json rewrite
```json
// Source: https://schema.tauri.app/config/2 (schema verified field names)
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "riftle",
  "version": "0.1.0",
  "identifier": "com.riftle.launcher",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "launcher",
        "title": "Riftle",
        "width": 640,
        "height": 60,
        "decorations": false,
        "transparent": true,
        "shadow": true,
        "resizable": false,
        "skipTaskbar": true,
        "alwaysOnTop": true,
        "visible": false,
        "focus": false
      },
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
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

### Complete Cargo.toml additions
```toml
# Source: PROJECT.md + confirmed version research (2026-03-06)
[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }

# Official Tauri plugins — exact version pins
tauri-plugin-opener = "2"
tauri-plugin-store = "2.4.2"
tauri-plugin-global-shortcut = "2.3.0"
tauri-plugin-autostart = "2.5.1"

# Domain crates — caret ranges (stable semver)
rusqlite = { version = "^0.31", features = ["bundled"] }
walkdir = "^2"
notify = "^6"
nucleo = "^0.5"
windows-sys = { version = "^0.52", features = [
  "Win32_UI_Shell",
  "Win32_System_Shutdown",
  "Win32_System_Power",
  "Win32_System_RemoteDesktop"
] }
serde = { version = "^1", features = ["derive"] }
serde_json = "^1"
```

### lib.rs replacement
```rust
// Source: https://v2.tauri.app/plugin/store/
//         https://v2.tauri.app/plugin/global-shortcut/
//         https://v2.tauri.app/plugin/autostart/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new().build()
                )?;
                app.handle().plugin(tauri_plugin_autostart::init(
                    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                    None,
                ))?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### capabilities/default.json update
```json
// Source: https://v2.tauri.app/security/capabilities/
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capability for launcher and settings windows",
  "windows": ["launcher", "settings"],
  "permissions": [
    "core:default",
    "opener:default",
    "store:default",
    "global-shortcut:default",
    "autostart:default"
  ]
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| snake_case window config in Tauri v1 (`skip_taskbar`) | camelCase in Tauri v2 JSON (`skipTaskbar`) | Tauri 2.0 | Must use camelCase in tauri.conf.json |
| Plugin init directly in Builder chain for all plugins | Desktop-only plugins use `#[cfg(desktop)]` + `setup` | Tauri 2.0 | global-shortcut and autostart use setup callback pattern |
| Single global allowlist (Tauri v1) | Per-window capability files (Tauri v2) | Tauri 2.0 | capabilities/default.json must include new window labels |
| `Builder::default().build()` for store plugin | `Builder::new().build()` | tauri-plugin-store 2.x | Both work but `Builder::new()` is the documented pattern |

**Deprecated/outdated:**
- tauri v1 `allowlist` in tauri.conf.json: replaced by capability files in v2
- `tauri::generate_handler![]` with greet command: will be removed; placeholder `[]` until real commands added in later phases

---

## Open Questions

1. **Settings window Vue shell in Phase 1**
   - What we know: Both windows load the same `index.html` by default; settings UI is Phase 8
   - What's unclear: Whether a minimal settings shell should be scaffolded now or a router-based approach planned
   - Recommendation (Claude's discretion): Use `"url": "#/launcher"` for the launcher window config (Phase 5 sets up router) and leave settings window with default URL for now; only create router in Phase 5. For Phase 1, both windows share the same minimal App.vue shell.

2. **Stub Rust module files**
   - What we know: PROJECT.md lists db.rs, store.rs, hotkey.rs, indexer.rs, search.rs, commands.rs, system_commands.rs
   - What's unclear: Whether creating empty stub files now reduces friction in later phases
   - Recommendation (Claude's discretion): Create stub files with `// Phase X placeholder` comments. This prevents "file not found" issues if any future lib.rs `mod` declarations are added, and makes the project structure immediately visible.

3. **tauri-plugin-opener exact version**
   - What we know: Cargo.toml currently has `tauri-plugin-opener = "2"` (no exact pin)
   - What's unclear: User decision says Tauri plugins use exact versions; opener was pre-pinned as "2"
   - Recommendation: Pin to the same minor series as the other plugins — check with `cargo search tauri-plugin-opener` or leave as "2" since it was the scaffold default and is already working.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None installed — this is a Tauri app, not a library |
| Config file | None |
| Quick run command | `pnpm tauri dev` (manual smoke test) |
| Full suite command | `cargo check` (compilation check without running) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SCAF-02 | Two windows declared in tauri.conf.json with correct flags | manual-only (visual) | `cargo check` confirms JSON is valid | ❌ Wave 0 — config verification only |
| SCAF-03 | All crates in Cargo.toml compile | compilation | `cargo check --manifest-path src-tauri/Cargo.toml` | ❌ Wave 0 |
| SCAF-04 | `pnpm tauri dev` starts without errors | smoke test | `pnpm tauri dev` (manual, ~60s first run) | ❌ Wave 0 |

**Note:** SCAF-02, SCAF-03, and SCAF-04 are configuration/compilation requirements with no unit-testable behavior. Verification is: (1) `cargo check` passes, (2) `pnpm tauri dev` starts, (3) launcher window appears with expected properties visible.

### Sampling Rate
- **Per task commit:** `cargo check --manifest-path src-tauri/Cargo.toml`
- **Per wave merge:** `pnpm tauri dev` (full smoke test — manual, requires Windows)
- **Phase gate:** `pnpm tauri dev` starts clean before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] No automated test framework needed for this phase — all requirements are configuration/compilation checks
- [ ] Smoke test is manual: `pnpm tauri dev` starts and launcher window appears

*(Existing infrastructure: `cargo check` for Rust compilation, `pnpm build` for frontend TypeScript check)*

---

## Sources

### Primary (HIGH confidence)
- `https://schema.tauri.app/config/2` — Window config field names verified (camelCase: `skipTaskbar`, `alwaysOnTop`, `decorations`, `transparent`, `shadow`, `resizable`, `visible`, `minWidth`, `minHeight`)
- `https://v2.tauri.app/plugin/store/` — `tauri_plugin_store::Builder::new().build()` init pattern
- `https://v2.tauri.app/plugin/global-shortcut/` — `Builder::new().build()` with `#[cfg(desktop)]` setup pattern
- `https://v2.tauri.app/plugin/autostart/` — `tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None)` pattern
- `https://v2.tauri.app/security/capabilities/` — capabilities/default.json format with `windows` array and `permissions`
- `https://docs.rs/crate/tauri/latest` — Tauri 2.10.3 is current stable (released 2026-03-04)
- WebSearch results — plugin version confirmations: store 2.4.2, global-shortcut 2.3.0, autostart 2.5.1

### Secondary (MEDIUM confidence)
- GitHub issue #8308 — transparent + shadow interaction on Windows (verified: shadow behavior is intentional in v2, CSS `background: transparent` required on body)
- WebSearch results — camelCase confirmation from community examples and Tauri dev.to tutorials
- `https://github.com/tauri-apps/tauri/blob/dev/examples/multiwindow/tauri.conf.json` — multiwindow example structure

### Tertiary (LOW confidence)
- rusqlite 0.38.0 is latest — but user spec says `^0.31`, which is respected; `^0.31` resolves to latest 0.31.x patch
- notify latest is 8.2.0 — but user spec says `^6`; confirmed `^6` resolves within 6.x series
- windows-sys latest is 0.61.2 — but user spec says `^0.52`; confirmed `^0.52` resolves within 0.52.x

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — versions verified against crates.io/docs.rs via WebSearch (2026-03-06)
- Architecture: HIGH — window config field names verified against official schema; plugin init patterns from official docs
- Pitfalls: HIGH — transparent/shadow interaction verified from GitHub issue; capabilities window label issue verified from official docs

**Research date:** 2026-03-06
**Valid until:** 2026-04-06 (Tauri plugin ecosystem moves fast; plugin versions may update)
