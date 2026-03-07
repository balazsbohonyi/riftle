---
status: diagnosed
trigger: "Admin badge only appears when using CTRL+SHIFT and hovering over selected item"
created: 2026-03-07T00:00:00Z
updated: 2026-03-07T00:00:00Z
---

## Current Focus

hypothesis: Admin badge v-if condition is gated on runtime keyboard state (adminMode) and row selection rather than a per-result elevation property
test: read badge v-if in App.vue, read SearchResult struct in search.rs
expecting: badge has no per-result field — it is driven entirely by transient keyboard state
next_action: COMPLETE — root cause confirmed

## Symptoms

expected: Admin badge visible on all results that require admin elevation, always, without special keyboard interaction
actual: Badge only appears when CTRL+SHIFT is held AND cursor is hovering over the selected item
errors: none — behaviour is wrong by design implementation, not a runtime error
reproduction: Search for any app. Badge never shown. Hold CTRL+SHIFT and hover the selected row — badge appears.
started: Since Phase 05 UI implementation

## Eliminated

- hypothesis: Badge hidden by CSS display:none
  evidence: .admin-badge has no display:none rule; it is removed from DOM entirely via v-if
  timestamp: 2026-03-07

- hypothesis: Backend does not populate an elevation field
  evidence: SearchResult struct in search.rs genuinely has no requires_elevation / admin field — so back-end IS missing data, but that is a separate structural gap from the v-if logic bug
  timestamp: 2026-03-07

## Evidence

- timestamp: 2026-03-07
  checked: src/App.vue line 307
  found: |
    v-if="index === selectedIndex && adminMode"
  implication: Badge is visible ONLY when the row is the selected row AND adminMode is true. adminMode is a global reactive ref, not a per-result property.

- timestamp: 2026-03-07
  checked: src/App.vue lines 111-113 and 145-147 (onKeyDown / onKeyUp)
  found: |
    adminMode.value = e.ctrlKey && e.shiftKey   // set on every keydown
    adminMode.value = e.ctrlKey && e.shiftKey   // cleared on every keyup
  implication: adminMode is a transient, ephemeral flag. It is true only while CTRL+SHIFT is physically held. The moment the user releases either key, adminMode resets to false and the badge disappears.

- timestamp: 2026-03-07
  checked: src/App.vue lines 10-16 (TypeScript interface SearchResult)
  found: |
    interface SearchResult {
      id: string; name: string; icon_path: string; path: string; kind: string
    }
  implication: The front-end type has no requires_elevation or admin field. The badge has no data source to test per-result elevation status.

- timestamp: 2026-03-07
  checked: src-tauri/src/search.rs lines 20-26 (Rust SearchResult struct)
  found: |
    pub struct SearchResult {
      pub id: String; pub name: String; pub icon_path: String;
      pub path: String; pub kind: String;
    }
  implication: The Rust struct is the source of truth serialised to the front end. It has no requires_elevation / admin field. The back end cannot signal to the UI whether an app needs elevation.

## Resolution

root_cause: >
  The admin badge v-if condition (src/App.vue:307) is "index === selectedIndex && adminMode",
  where adminMode is a transient global flag driven by the keyboard state of the *current keydown
  event*, not a stable per-result property. Because adminMode is false whenever CTRL+SHIFT is
  not actively held, the badge disappears as soon as the user releases those keys — and it
  never appears on non-selected rows at all. Additionally, neither the Rust SearchResult struct
  nor the TypeScript interface carries a requires_elevation field, so there is no per-result
  data available to drive persistent badge visibility.

fix: NOT APPLIED (diagnose-only mode)

verification: NOT APPLIED

files_changed: []
