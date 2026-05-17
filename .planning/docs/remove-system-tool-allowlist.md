# System Tool Allowlist Removal & System32 Blocklist Analysis

## Background & Discussion

**Initial Investigation:**
We set out to investigate the `system_tool_allowlist` configuration to determine its current relevance now that Riftle natively supports UWP application discovery (which includes modern and legacy system tools like Calculator, Notepad, etc.).

**How the Allowlist Was Being Used:**
1. It was saved to `settings.json` via the Rust backend (`src-tauri/src/store.rs`) and managed by the frontend payload (`src/Settings.vue`).
2. During indexing, it acted as an **exception list** for a hardcoded security filter. Specifically, if the indexer found a `.lnk` shortcut (e.g., on the Desktop) that pointed into `C:\Windows\System32\` (or `syswow64`, `winsxs`, etc.), the indexer would completely block it unless the executable name (e.g., `cmd.exe`) was in the `system_tool_allowlist`.

**UWP Cross-Referencing Results:**
We queried the local `launcher.db` to see if the tools in the default allowlist were already natively discovered via UWP:
*   **Missing (Deprecated/Removed in Windows 11):** `wordpad.exe`, `write.exe`, `optionalfeatures.exe`
*   **Discovered but under AUMIDs/GUIDs:** `notepad.exe`, `mspaint.exe`, `calc.exe`, `snippingtool.exe`, `taskmgr.exe`, `wmplayer.exe`, `mstsc.exe`, `resmon.exe`, `perfmon.exe`, `eventvwr.exe`, `compmgmt.exe`
*   **Discovered literally:** `cmd.exe`, `powershell.exe`, `regedit.exe`, etc.

**The Flaw in the Blocklist:**
Because the indexer no longer crawls the `PATH` environment variable (which used to cause floods of CLI tools), it only encounters `System32` binaries if the user explicitly creates a `.lnk` shortcut to them in an indexed directory (like the Desktop). Actively blocking the resolution of a user's intentional shortcut simply because of its destination path is overly aggressive. Therefore, the entire `System32` blocklist inside the `.lnk` resolver is fundamentally flawed and unnecessary.

**Conclusion:**
By removing the `System32` blocklist, the indexer will happily resolve and index any manual shortcuts the user creates. Consequently, the `system_tool_allowlist` (which only existed to bypass that blocklist) becomes obsolete and can be fully removed from the codebase.

---

## Implementation Plan

### 1. Update Indexer Logic (`src-tauri/src/indexer.rs`)
*   **Remove the Blocklist:** In `resolve_lnk()`, remove the block of code that checks if the `normalized` target path contains `\windows\system32\`, `\windows\syswow64\`, `\windows\winsxs\`, etc., and returns `None`.
*   **Remove Allowlist Parameter:** Remove the `allowlist` parameter from the `resolve_lnk` function signature and the `LnkQuery` struct.
*   **Update Crawl Signature:** Remove the `allowlist: &[String]` parameter from `crawl_dir()`.
*   **Update Call Sites:**
    *   In `run_full_index()`, remove `&settings.system_tool_allowlist` from the `crawl_dir()` call.
    *   In `spawn_com_worker()`, stop passing `query.allowlist` into `resolve_lnk()`.
    *   In `crawl_dir()` where `LnkQuery` is constructed, remove the `allowlist` field assignment.
*   **Fix Tests:** Update the test stubs in `indexer.rs` to reflect the updated function signatures.

### 2. Update Settings Contract (`src-tauri/src/store.rs`)
*   **Remove Field:** Remove the `system_tool_allowlist` field (and its `#[serde(default)]` attribute) from the `Settings` struct.
*   **Remove Defaults:** Delete the `default_system_tool_allowlist()` helper function. Update `Settings::default()` to omit this field.
*   **Update Payload:** Remove `"system_tool_allowlist": settings.system_tool_allowlist` from the `serde_json::json!({...})` macro in `get_settings_cmd`.
*   **Update Unit Tests:**
    *   Remove `test_system_tool_allowlist_survives_serde_round_trip`.
    *   Update/remove `test_get_settings_cmd_json_includes_allowlist_field` so it no longer expects `system_tool_allowlist` in the JSON shape.
    *   Update `shortcut_old_settings_json_defaults_to_empty_arrays` and `shortcut_get_settings_cmd_json_shape_includes_arrays` to omit the allowlist field.

### 3. Update Frontend State (`src/Settings.vue`)
*   **Clean Interfaces:** Remove `system_tool_allowlist: string[]` from the `SettingsData` and `SettingsResponse` interfaces.
*   **Clean Defaults:** Remove `system_tool_allowlist: []` from the `settings.value` ref initialization.
*   **Clean Payload:** Remove the `system_tool_allowlist: response.system_tool_allowlist` assignment in the `onMounted` backend response handler.

### 4. Verification & Testing
1.  **Compilation:** Run `cargo check` in `src-tauri` and `npm run build`.
2.  **Unit Tests:** Run `cargo test` in `src-tauri`.
3.  **Manual UAT:** Create a shortcut to `C:\Windows\System32\regedit.exe` on the Desktop. Trigger a manual re-index. Search for "regedit" in Riftle. Ensure the manual shortcut is successfully discovered and resolved.
