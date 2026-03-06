---
status: testing
phase: 03-indexer
source: 03-01-SUMMARY.md, 03-02-SUMMARY.md, 03-03-SUMMARY.md, 03-04-SUMMARY.md, 03-05-SUMMARY.md
started: 2026-03-06T14:00:00Z
updated: 2026-03-06T14:00:00Z
---

# Phase 3: Indexer — User Acceptance Test

> Validate that the Windows application indexer crawls paths, resolves shortcuts, extracts icons, and keeps the index fresh through background synchronization.

---

## Current Test

number: 1
name: Startup index population
expected: |
  Launch the app. Check the SQLite database (`launcher.db`):
  - **Installed mode:** %APPDATA%\riftle-launcher\launcher.db
  - **Portable mode:** ./data/launcher.db (relative to exe)

  The apps table should contain entries from:
  - Start Menu Programs (both APPDATA and PROGRAMDATA user\Microsoft\Windows\Start Menu\Programs)
  - Desktop (both user USERPROFILE\Desktop and Public\Desktop)
  - PATH directories (resolved .exe files)
  - Any user-defined additional_paths from settings

  Verify: At least 10-20 apps found, names match installed applications (e.g., Chrome, Edge, VSCode, etc. if installed).
awaiting: user response

---

## Tests

### 1. Startup index population
expected: SQLite database populated with apps from Start Menu, Desktop, PATH, and configured paths on startup. Database located at %APPDATA%\riftle-launcher\launcher.db (installed) or ./data/launcher.db (portable).
result: [pending]

### 2. .LNK shortcut resolution
expected: Start Menu shortcuts (.lnk files) resolve to their target executables. Database shows the actual .exe path, not the .lnk path
result: [pending]

### 3. Icon extraction and caching
expected: Icons are extracted from indexed apps and saved as PNG files in {data_dir}/icons/. First launch shows generic icon placeholder, icons update as extraction completes
result: [pending]

### 4. Generic icon fallback
expected: Apps without extractable icons show a generic app icon placeholder (bundled in the binary, copied to icons/generic.png on startup)
result: [pending]

### 5. Concurrent index prevention
expected: If a manual reindex is triggered while the timer is firing, only one index runs. The second trigger is silently dropped (no error, no stall)
result: [pending]

### 6. Stale app removal
expected: If an app is uninstalled (e.g., a Start Menu shortcut deleted or .exe removed), the next full index removes it from the database
result: [pending]

### 7. Background timer re-index
expected: After the initial index, a background timer fires every 15 minutes (configurable via settings.reindex_interval). New apps installed between timer fires will be picked up on the next cycle
result: [pending]

### 8. Filesystem watcher
expected: When a file is added to the Windows Start Menu directory, the watcher detects the change within ~500ms and triggers an incremental re-index. New app appears in the index without waiting for the 15-minute timer
result: [pending]

### 9. Excluded paths handling
expected: If a directory is added to the excluded_paths setting, apps in that directory are skipped during indexing. Previously indexed apps from that path are removed on next full index
result: [pending]

---

## Summary

total: 9
passed: 0
issues: 0
pending: 9
skipped: 0

---

## Gaps

[none yet]
