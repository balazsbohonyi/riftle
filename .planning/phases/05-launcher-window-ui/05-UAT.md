---
status: diagnosed
phase: 05-launcher-window-ui
source: 05-01-SUMMARY.md, 05-02-SUMMARY.md
started: 2026-03-07T00:00:00Z
updated: 2026-03-07T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Launcher Window Displays
expected: Launcher window appears with dark gradient background, search input with right-aligned magnifier icon, and empty result list area. Window is 58px tall (minimum).
result: pass

### 2. Search Input Accepts Typed Characters
expected: Click search input and type characters. Text appears in the input field and persists as you type.
result: pass

### 3. Search Results Display with Icons and Names
expected: Type a search query. Results list appears below with app icons on the left, app names in the middle, and optional path display. Each row is 48px tall and scrollable if more than 8 results.
result: pass
note: tested with 5 results

### 4. Window Height Resizes Dynamically
expected: Search results change the window height: 0 results = 58px, 1 result = 106px, 8 results = 440px (max). Height updates smoothly when typing or results change.
result: pass

### 5. Window Animates on Resize
expected: When window height changes, it animates smoothly. Default animation is "slide" (180ms duration). Can see window expand/contract with motion, not instant jumps.
result: issue
reported: "I can't determine if the height changes smoothly, maybe the animation it's too fast"
severity: minor

### 6. App Icons Load from Asset Protocol
expected: Icons next to app names display correctly from the application's data directory. Icons render as small 32x32 pixel images with correct app branding.
result: pass

### 7. Path Display Shows App Location
expected: Below each app name, a gray path-line shows the full file path to the executable. Path text is styled in monospace font (JetBrains Mono) and smaller than the app name.
result: pass

### 8. Admin Badge Appears for Admin Apps
expected: Some search results have an "admin" badge visible. This indicates apps that need admin elevation. Badge appears consistently for known admin-required apps.
result: issue
reported: "the badge only appears when I'm using CTRL+SHIFT and hover over the selected item"
severity: major

### 9. Virtual Scrolling Works for Large Result Sets
expected: Search for a term that returns 20+ results. Window shows max 8 rows (440px). Scroll down to see remaining results. Scrolling is smooth and fast (virtualized list, not all items rendered).
result: pass
note: selection highlight sometimes not visible on certain items while scrolling

### 10. Window Hides on Focus Loss
expected: Launcher window is visible and active. Click outside the window to another app. Launcher window hides automatically. Click the hotkey to show it again.
result: pass

## Summary

total: 10
passed: 8
issues: 2
pending: 0
skipped: 0

## Gaps

- truth: "Window expand/contract animation is visible with motion, not instant jumps (slide 180ms)"
  status: failed
  reason: "User reported: I can't determine if the height changes smoothly, maybe the animation it's too fast"
  severity: minor
  test: 5
  root_cause: "setSize() is called immediately with no delay or CSS height transition, so the OS window resizes atomically with no visible animation; the 180ms transition only covers opacity/transform for show/hide, not dimensional resize"
  artifacts:
    - path: "src/App.vue"
      lines: "69-78"
      issue: "setSize() called with no delay or interpolation"
    - path: "src/App.vue"
      lines: "40-48"
      issue: "watcher calls updateWindowHeight() immediately after results arrive"
    - path: "src/App.vue"
      lines: "361-368"
      issue: "CSS transition covers opacity/transform only — no height transition"
    - path: "src/App.vue"
      lines: "270-278"
      issue: ".result-list inline height style has no transition property"
  missing:
    - "Add CSS height transition to .result-list wrapper"
    - "Sequence setSize() to fire after CSS animation completes (~180ms delay)"
  debug_session: ".planning/debug/window-expand-animation-imperceptible.md"

- truth: "Admin badge is visible on results that require admin elevation, consistently without special interaction"
  status: failed
  reason: "User reported: the badge only appears when I'm using CTRL+SHIFT and hover over the selected item"
  severity: major
  test: 8
  root_cause: "Badge v-if is gated on adminMode (a transient keyboard ref) AND selectedIndex match — badge only shows while CTRL+SHIFT is held on the selected row; there is no per-result requires_elevation field in SearchResult"
  artifacts:
    - path: "src/App.vue"
      line: 307
      issue: "v-if=\"index === selectedIndex && adminMode\" — badge gated on keyboard state, not per-result flag"
    - path: "src/App.vue"
      line: 112
      issue: "adminMode.value = e.ctrlKey && e.shiftKey — set transiently on keydown"
    - path: "src/App.vue"
      line: 146
      issue: "adminMode.value cleared on keyup — badge disappears when keys released"
    - path: "src-tauri/src/search.rs"
      lines: "20-26"
      issue: "SearchResult struct has no requires_elevation field"
  missing:
    - "Add requires_elevation: bool to SearchResult struct in src-tauri/src/search.rs"
    - "Mirror requires_elevation in TypeScript SearchResult interface in src/App.vue"
    - "Change badge v-if to item.requires_elevation (decouple from adminMode keyboard state)"
  debug_session: ".planning/debug/admin-badge-visibility.md"

- truth: "Selected item highlight is consistently visible while scrolling through virtual list"
  status: failed
  reason: "User reported: sometimes the selection is not visible for some items (during virtual scroll)"
  severity: minor
  test: 9
  root_cause: "RecycleScroller DOM nodes mid-recycle have active=false; :class binding evaluates selectedIndex against a stale slot index during the recycle transition, and @mousemove fires during scroll with stale index data transiently corrupting selectedIndex"
  artifacts:
    - path: "src/App.vue"
      lines: "278-279"
      issue: "active not destructured from slot scope — recycle state ignored"
    - path: "src/App.vue"
      line: 282
      issue: ":class=\"{ selected: index === selectedIndex }\" — applied even when view is mid-recycle (active=false)"
    - path: "src/App.vue"
      line: 283
      issue: "@mousemove=\"selectedIndex = index\" — fires during scroll with stale index, corrupts selectedIndex"
  missing:
    - "Destructure active from v-slot scope: v-slot=\"{ item, index, active }\""
    - "Gate :class on active: :class=\"{ selected: active && index === selectedIndex }\""
    - "Guard @mousemove: @mousemove=\"active && (selectedIndex = index)\""
  debug_session: ".planning/debug/virtual-scroll-selection-highlight.md"
