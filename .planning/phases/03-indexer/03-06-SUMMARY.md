# Phase 3 — Gap Fix Summary (03-06)

> Fixes for 3 blockers/majors discovered during UAT. All changes are in the indexer and data layer.

---

## Accomplishments

### 1. Replaced lnk crate with Windows IShellLink COM API
- **Problem:** `lnk` v0.3 panicked on malformed `.lnk` files (out-of-bounds slice in `linkinfo.rs`), crashing the app on the main thread before any indexing occurred.
- **Fix:** Removed `lnk` dependency entirely. `resolve_lnk` now uses `CoCreateInstance(ShellLink)` + `IPersistFile::Load` + `IShellLinkW::GetPath` — the native Windows way to resolve shortcuts. Returns `None` on any failure; cannot panic.
- **Files:** `src-tauri/src/indexer.rs`, `src-tauri/Cargo.toml`

### 2. Removed PATH indexing; added system directory filtering
- **Problem:** Indexer crawled all PATH directories (System32, SysWOW64, hundreds of CLI tools), producing hundreds of irrelevant entries. Start Menu `.lnk` files also resolved to system executables (e.g. `appverif.exe`).
- **Fix:**
  - Dropped PATH crawling entirely — Start Menu and Desktop cover all user-launchable apps. Users can add CLI tool directories via `additional_paths`.
  - Added system directory blocklist in `resolve_lnk`: skips targets in `System32`, `SysWOW64`, `winsxs`, `microsoft.net`, `Program Files\Common Files`, `Program Files (x86)\Common Files`.
  - Added `system_tool_allowlist` field to `Settings` (default: notepad, calc, regedit, mspaint, cmd, powershell, taskmgr, mstsc, and ~15 other useful tools). Tools in the allowlist bypass the blocklist. List is user-editable in `settings.json`.
- **Files:** `src-tauri/src/indexer.rs`, `src-tauri/src/store.rs`

### 3. Better app display names
- **Problem:** Apps showed raw exe stem (e.g. `chrome`) instead of human-readable names (e.g. `Google Chrome`).
- **Fix:**
  - For `.lnk`-sourced apps: use the `.lnk` filename as display name (e.g. `Google Chrome.lnk` → `"Google Chrome"`).
  - For direct `.exe` files: read `FileDescription` from the PE VERSIONINFO resource via `GetFileVersionInfoW` + `VerQueryValueW`. Falls back to exe stem if not present.
  - `make_app_record` updated to accept an optional `display_name` override.
- **Files:** `src-tauri/src/indexer.rs`, `src-tauri/Cargo.toml`

### 4. Fixed icon DB sync bugs (two compounding issues)
- **Problem:** All apps showed `generic.png` in the DB even though icon files were present on disk.
- **Root cause A:** `upsert_app` ON CONFLICT DO UPDATE unconditionally reset `icon_path` to `"generic.png"` on every re-index, overwriting previously cached icon filenames.
- **Root cause B:** When the DB was deleted but icon files remained on disk, `icon_file.exists()` was true so no extraction thread spawned — but the new INSERT used `"generic.png"` with no mechanism to sync with existing files.
- **Fix A:** Removed `icon_path` from the `ON CONFLICT DO UPDATE SET` clause. Icon path is now managed exclusively by the extraction thread after first write.
- **Fix B:** Before upserting, check if the icon file already exists on disk. If yes, set `icon_path` to the real filename in the INSERT directly. If no, insert with `"generic.png"` and spawn extraction thread as before.
- **Files:** `src-tauri/src/db.rs`, `src-tauri/src/indexer.rs`

### 5. Path-based deduplication within index run
- **Problem:** The same exe could be discovered multiple times in one run (e.g. via both user and all-users Start Menu shortcuts), causing redundant DB writes and redundant icon extraction threads.
- **Fix:** `discovered_ids: HashSet<String>` (already present for prune tracking) is now checked via `insert()` return value before processing each app. First occurrence wins; subsequent duplicates are skipped entirely.
- **Files:** `src-tauri/src/indexer.rs`

---

## Modified Files

| File | Change |
|------|--------|
| `src-tauri/Cargo.toml` | Removed `lnk`; added `windows = 0.58` (COM/IShellLink); added `Win32_Storage_FileSystem` to `windows-sys` features |
| `src-tauri/src/indexer.rs` | `resolve_lnk` rewritten with IShellLink; PATH crawl removed; system dir filter + allowlist; deduplication; icon cache sync; `get_file_description`; `make_app_record` display name param |
| `src-tauri/src/db.rs` | `upsert_app` ON CONFLICT no longer overwrites `icon_path` |
| `src-tauri/src/store.rs` | `Settings` + `system_tool_allowlist` field with rich default list |

---

## User-Facing Changes

- App no longer crashes on startup with malformed shortcuts
- Index contains only user-launchable apps — no CLI tools, no system utilities
- App names show human-readable labels (`Google Chrome`, `Microsoft Edge`, etc.)
- Icons display correctly and persist across restarts and re-indexes
- Useful Windows tools (Notepad, Calculator, regedit, etc.) remain accessible; list is configurable
