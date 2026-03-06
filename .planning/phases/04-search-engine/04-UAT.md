---
status: complete
phase: 04-search-engine
source: [04-01-SUMMARY.md, 04-02-SUMMARY.md, 04-03-SUMMARY.md]
started: 2026-03-06T18:30:00Z
updated: 2026-03-06T18:35:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Test suite passes
expected: Run `cargo test` inside `src-tauri/`. Expect: 34 passed, 0 failed, 2 ignored.
result: pass

### 2. App starts without errors
expected: Run `npm run tauri dev` (or however you start the app). The window opens normally with no crash or startup error in the terminal. No panics related to search index initialization or system_command.png.
result: pass

### 3. Empty query returns empty list
expected: With the app running, open devtools console and run:
  `window.__TAURI__.core.invoke('search', { query: '' })`
  Expect: `[]` (empty array — empty query guard returns nothing)
result: skipped
reason: No visible window — launcher hides until hotkey triggered; frontend not wired until Phase 5

### 4. Fuzzy search returns ranked results
expected: In devtools console run:
  `window.__TAURI__.core.invoke('search', { query: 'ch' })`
  Expect: an array of app objects with fields `id`, `name`, `icon_path`, `path`, `kind`. Results should be sorted — prefix matches (names starting with "ch") appear before fuzzy matches. No crash.
result: skipped
reason: No visible window — requires Phase 5 launcher window UI

### 5. System command prefix routes correctly
expected: In devtools console run:
  `window.__TAURI__.core.invoke('search', { query: '>shut' })`
  Expect: array containing the Shutdown system command (`kind: "system"`, `icon_path: "system_command.png"`). Should NOT contain regular app results.
result: skipped
reason: No visible window — requires Phase 5 launcher window UI

### 6. Index survives reindex
expected: In devtools console run:
  `await window.__TAURI__.core.invoke('reindex')`
  Then immediately:
  `window.__TAURI__.core.invoke('search', { query: 'ch' })`
  Expect: search still returns results after rebuild — index was swapped atomically, not cleared.
result: skipped
reason: No visible window — requires Phase 5 launcher window UI

## Summary

total: 6
passed: 2
issues: 0
pending: 0
skipped: 4

## Gaps

[none yet]
