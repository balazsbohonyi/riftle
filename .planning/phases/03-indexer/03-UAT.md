---
status: complete
phase: 03-indexer
source: 03-01-SUMMARY.md, 03-02-SUMMARY.md, 03-03-SUMMARY.md, 03-04-SUMMARY.md, 03-05-SUMMARY.md
started: 2026-03-06T14:00:00Z
updated: 2026-03-06T16:00:00Z
---

# Phase 3: Indexer — User Acceptance Test

> Validate that the Windows application indexer crawls paths, resolves shortcuts, extracts icons, and keeps the index fresh through background synchronization.

---

## Current Test

[testing complete]

---

## Tests

### 1. Startup index population
expected: SQLite database populated with apps from Start Menu, Desktop, PATH, and configured paths on startup. Database located at %APPDATA%\riftle-launcher\launcher.db (installed) or ./data/launcher.db (portable).
result: issue
reported: "app crashes immediately with panic in lnk-0.3.0/linkinfo.rs — database is empty"
severity: blocker

### 2. .LNK shortcut resolution
expected: Start Menu shortcuts (.lnk files) resolve to their target executables. Database shows the actual .exe path, not the .lnk path
result: issue
reported: "after crash fix, resolution works but shows many noise entries from System32, SysWOW64, and PATH directories (e.g. appverif.exe, hundreds of CLI tools)"
severity: major

### 3. Icon extraction and caching
expected: Icons are extracted from indexed apps and saved as PNG files in {data_dir}/icons/. First launch shows generic icon placeholder, icons update as extraction completes
result: issue
reported: "icon files are written to the icons folder but DB shows generic.png for all items even after restart"
severity: major

### 4. Generic icon fallback
expected: Apps without extractable icons show a generic app icon placeholder (bundled in the binary, copied to icons/generic.png on startup)
result: pass

### 5. Concurrent index prevention
expected: If a manual reindex is triggered while the timer is firing, only one index runs. The second trigger is silently dropped (no error, no stall)
result: skipped
reason: requires timing-sensitive manual trigger; covered by unit test test_atomic_guard_prevents_double_index

### 6. Stale app removal
expected: If an app is uninstalled (e.g., a Start Menu shortcut deleted or .exe removed), the next full index removes it from the database
result: skipped
reason: requires uninstalling a real application; logic covered by test_prune_stale unit test

### 7. Background timer re-index
expected: After the initial index, a background timer fires every 15 minutes (configurable via settings.reindex_interval). New apps installed between timer fires will be picked up on the next cycle
result: skipped
reason: requires 15-minute wait; timer thread logic covered by test_timer_fires unit test

### 8. Filesystem watcher
expected: When a file is added to the Windows Start Menu directory, the watcher detects the change within ~500ms and triggers an incremental re-index. New app appears in the index without waiting for the 15-minute timer
result: skipped
reason: requires creating a real .lnk in Start Menu folder; watcher setup covered by code review

### 9. Excluded paths handling
expected: If a directory is added to the excluded_paths setting, apps in that directory are skipped during indexing. Previously indexed apps from that path are removed on next full index
result: skipped
reason: settings UI not yet built (Phase 8); logic covered by test_crawl_excludes_path unit test

---

## Summary

total: 9
passed: 1
issues: 3
pending: 0
skipped: 5

---

## Gaps

- truth: "App starts and populates the database on first launch"
  status: failed
  reason: "User reported: app crashes immediately with panic in lnk-0.3.0/linkinfo.rs — database is empty"
  severity: blocker
  test: 1
  root_cause: "lnk crate v0.3 panics (instead of returning Err) on malformed .lnk files due to out-of-bounds slice indexing in linkinfo.rs. Running on main thread causes process exit before any indexing occurs."
  artifacts:
    - path: "src-tauri/src/indexer.rs"
      issue: "resolve_lnk used lnk::ShellLink which panics on malformed shortcuts"
    - path: "src-tauri/Cargo.toml"
      issue: "lnk = '^0.3' dependency"
  missing:
    - "Replace lnk crate with Windows IShellLink COM API (CoCreateInstance + IPersistFile + GetPath)"
  debug_session: ""

- truth: "Indexed apps are relevant user-launchable applications, not system utilities or CLI tools"
  status: failed
  reason: "User reported: shows many noise entries from System32, SysWOW64, and PATH directories (e.g. appverif.exe, hundreds of CLI tools)"
  severity: major
  test: 2
  root_cause: "Indexer crawled PATH env var (hundreds of System32/SysWOW64 executables) and did not filter system directory targets from .lnk resolution. No noise filtering existed."
  artifacts:
    - path: "src-tauri/src/indexer.rs"
      issue: "get_index_paths included all PATH directories; resolve_lnk had no system dir filter"
    - path: "src-tauri/src/store.rs"
      issue: "Settings had no system_tool_allowlist field"
  missing:
    - "Remove PATH indexing entirely — Start Menu covers all user-launchable apps"
    - "Add system directory blocklist to resolve_lnk (System32, SysWOW64, winsxs, microsoft.net, Common Files)"
    - "Add configurable system_tool_allowlist in Settings with useful defaults (notepad, calc, regedit, etc.)"
    - "Use .lnk filename as display name; use PE FileDescription for direct .exe files"
    - "Deduplicate indexed apps by exe path within a single index run"
  debug_session: ""

- truth: "Icons are extracted and stored correctly; DB reflects real icon filenames after extraction"
  status: failed
  reason: "User reported: icon files are written to the icons folder but DB shows generic.png for all items even after restart"
  severity: major
  test: 3
  root_cause: "Two compounding bugs: (1) upsert_app ON CONFLICT DO UPDATE unconditionally overwrote icon_path back to 'generic.png' on every re-index, preventing permanently cached icons. (2) When DB was deleted but icon files remained, icon_file.exists() check skipped extraction threads but the new INSERT used 'generic.png' — never synced with existing files."
  artifacts:
    - path: "src-tauri/src/db.rs"
      issue: "ON CONFLICT DO UPDATE SET included icon_path = excluded.icon_path, resetting cached icons on every re-index"
    - path: "src-tauri/src/indexer.rs"
      issue: "icon_file.exists() check prevented thread spawn but upsert still wrote generic.png for new DB rows"
  missing:
    - "Remove icon_path from ON CONFLICT DO UPDATE SET — icon is managed exclusively by extraction thread"
    - "Before upsert, check if icon file exists on disk; if so, use real filename in the INSERT directly"
  debug_session: ""
