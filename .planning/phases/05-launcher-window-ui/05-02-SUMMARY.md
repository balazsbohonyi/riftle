---
phase: 05-launcher-window-ui
plan: 02
subsystem: ui
tags: [vue3, composition-api, vue-virtual-scroller, RecycleScroller, fontsource, inter, jetbrains-mono, tauri, convertFileSrc, asset-protocol, animation]

# Dependency graph
requires:
  - phase: 05-01
    provides: get_settings_cmd Tauri command returning show_path, animation, data_dir; Settings struct with animation field
  - phase: 04-search-engine
    provides: search() Tauri command returning Vec<SearchResult> with id, name, icon_path, path, kind
provides:
  - Full launcher Vue 3 UI with search input, virtualised result list, window resize, and animation support
  - RecycleScroller-based result list with 48px rows (max 8 visible), icon images via asset:// protocol
  - Dynamic window height formula: 56px input + 2px border + n*48px rows (58px min, 440px max)
  - Animation modes: slide (180ms), fade (120ms), instant — controlled by Settings.animation
  - Tauri protocol-asset feature enabled; CSP updated for asset:// and ipc:// origins
  - get_settings_cmd registered in invoke_handler
affects: [06-launch-actions, 07-context-menu, 05-03]

# Tech tracking
tech-stack:
  added:
    - vue-virtual-scroller@2.0.0-beta.8 (RecycleScroller virtual list)
    - "@fontsource/inter@5.2.8 (Inter font 400+500 weights)"
    - "@fontsource/jetbrains-mono@5.2.8 (JetBrains Mono 400 weight)"
  patterns:
    - Vue 3 Composition API with <script setup lang="ts"> for all launcher components
    - watch(query) pattern for reactive search invocation
    - computed listHeight drives both RecycleScroller :style height and window resize
    - getIconUrl() constructs absolute path from dataDir + 'icons/' + icon_path filename then convertFileSrc()
    - Auto-hide on focus loss via getCurrentWindow().onFocusChanged listener
    - unlistenFocus pattern for cleanup in onUnmounted

key-files:
  created:
    - src/assets/magnifier.svg
  modified:
    - src/App.vue
    - src/main.ts
    - src/vite-env.d.ts
    - package.json
    - pnpm-lock.yaml
    - src-tauri/Cargo.toml
    - src-tauri/Cargo.lock
    - src-tauri/src/lib.rs
    - src-tauri/tauri.conf.json

key-decisions:
  - "vue-virtual-scroller@next (2.0.0-beta.8) has no TypeScript declarations — shimmed in vite-env.d.ts with declare module pattern"
  - "Style block is unscoped (no scoped attribute) so RecycleScroller internal DOM elements receive CSS rules correctly"
  - "html/body background changed from #1c1c1e opaque to transparent — #app gradient is the sole background"
  - "Tauri protocol-asset feature added to Cargo.toml to enable convertFileSrc() asset:// URL scheme"
  - "CSP updated to allow asset: http://asset.localhost for icon image loading via assetProtocol"
  - "Launch commands (launch, launch_elevated, run_system_command) are stubs wrapped in .catch(console.error) — Phase 6 implements Rust side"
  - "launchInProgress flag prevents auto-hide on focus loss during launch sequence"

patterns-established:
  - "App.vue keyboard handler: onKeyDown sets adminMode from ctrlKey+shiftKey, dispatches arrow/enter/escape"
  - "Window height computation: Math.max(56 + 2 + Math.min(results.length, 8) * 48, 58)"
  - "Icon path construction: dataDir + sep + 'icons' + sep + iconPath (sep detected from dataDir backslash)"

requirements-completed: [LWND-01, LWND-02, LWND-03, LWND-07, LWND-10, LWND-11]

# Metrics
duration: 5min
completed: 2026-03-06
---

# Phase 05 Plan 02: Launcher Window UI Summary

**Vue 3 launcher UI with RecycleScroller virtual list, Inter/JetBrains Mono fonts, asset:// icon loading, and dynamic 58–440px window resize driven by search result count**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-06T20:52:33Z
- **Completed:** 2026-03-06T20:57:18Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Complete launcher UI built in App.vue: dark gradient background, search input with right-aligned magnifier, RecycleScroller virtualised result list, icon images, app names, path-line conditionals, admin badge
- Dynamic window height resizes on every search result change (58px with no results, up to 440px for 8 rows)
- Inter font for input and result names; JetBrains Mono for path-line; all loaded via @fontsource packages in main.ts
- Tauri Rust side wired: protocol-asset feature, get_settings_cmd in invoke_handler, CSP updated for asset:// origin

## Task Commits

Each task was committed atomically:

1. **Task 1: Install npm dependencies and create magnifier SVG asset** - `1f528d8` (chore)
2. **Task 2: Build App.vue — layout, search input, result list, window resize** - `e5ba290` (feat)

## Files Created/Modified

- `src/App.vue` - Full launcher UI: search input, RecycleScroller result list, window resize, animation classes, keyboard handler, icon loading
- `src/assets/magnifier.svg` - Flat monochrome SVG magnifier icon (18x18, stroke #888888)
- `src/main.ts` - Added three @fontsource CSS imports (Inter 400, Inter 500, JetBrains Mono 400)
- `src/vite-env.d.ts` - Added declare module for vue-virtual-scroller (RecycleScroller, DynamicScroller, DynamicScrollerItem)
- `package.json` - Added vue-virtual-scroller@next, @fontsource/inter, @fontsource/jetbrains-mono
- `pnpm-lock.yaml` - Lockfile updated
- `src-tauri/Cargo.toml` - Added protocol-asset feature to tauri dependency
- `src-tauri/Cargo.lock` - Lockfile updated (http-range crate added)
- `src-tauri/src/lib.rs` - Registered get_settings_cmd in invoke_handler
- `src-tauri/tauri.conf.json` - Updated CSP and added assetProtocol config for icon loading

## Decisions Made

- **vue-virtual-scroller types shim:** Package ships no TypeScript declarations. Added `declare module 'vue-virtual-scroller'` in `vite-env.d.ts` with `RecycleScroller` as `DefineComponent` — resolves TS7016 build error (Rule 3 auto-fix).
- **Unscoped style block:** `<style>` (not `<style scoped>`) required so CSS targets RecycleScroller's internal DOM elements correctly.
- **Transparent html/body:** Background changed from `#1c1c1e` to `transparent` — the `#app` gradient is now the sole visual background layer, matching Phase 1's transparent Tauri window design.
- **protocol-asset + CSP:** `convertFileSrc()` requires Tauri's asset protocol feature and matching CSP. Both were missing and added as part of this task.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added vue-virtual-scroller TypeScript type declarations**
- **Found during:** Task 2 (pnpm build)
- **Issue:** `vue-virtual-scroller@2.0.0-beta.8` ships no `.d.ts` files — TypeScript build failed with TS7016 "implicitly has 'any' type"
- **Fix:** Added `declare module 'vue-virtual-scroller'` with RecycleScroller, DynamicScroller, and DynamicScrollerItem component type shims in `src/vite-env.d.ts`
- **Files modified:** `src/vite-env.d.ts`
- **Verification:** `pnpm build` succeeds with no TypeScript errors
- **Committed in:** `e5ba290` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking build error)
**Impact on plan:** Necessary for TypeScript compilation. No scope creep.

## Issues Encountered

None — build passed cleanly after adding the type declaration shim.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- App.vue complete with all UI components for Phase 03 (keyboard navigation, focus management)
- Launch stubs in place — Phase 6 will implement the Rust-side launch/launch_elevated commands
- Animation infrastructure (slide/fade/instant CSS classes) ready for Phase 03 wiring
- RecycleScroller active and rendering; keyboard arrow navigation hooks are present but Phase 03 adds scroll-into-view logic

---
*Phase: 05-launcher-window-ui*
*Completed: 2026-03-06*

## Self-Check: PASSED

- src/App.vue: FOUND
- src/assets/magnifier.svg: FOUND
- src/main.ts: FOUND
- .planning/phases/05-launcher-window-ui/05-02-SUMMARY.md: FOUND
- Commit 1f528d8 (Task 1): FOUND
- Commit e5ba290 (Task 2): FOUND
