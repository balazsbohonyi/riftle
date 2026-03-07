---
phase: 05-launcher-window-ui
verified: 2026-03-07T01:35:00Z
status: gaps_found
score: 9/12 must-haves verified
re_verification:
  previous_status: gaps_found
  previous_score: "not tracked (initial gap-closure pass)"
  gaps_closed:
    - "Admin badge decoupled from transient keyboard state — now uses item.requires_elevation (LWND-09 architecture fixed)"
    - "CSS height transition added to .result-list — window resize has visible animation (LWND-02 CSS)"
    - "RecycleScroller active prop gating — selection highlight consistent during virtual scroll (LWND-10)"
  gaps_remaining:
    - "LWND-02: Max visible rows is 5 (not 8 as per spec); window width is 500px (not 640px as per spec)"
    - "LWND-09: Badge never shown at runtime — all requires_elevation values hardcoded false; requirement says 'when Ctrl+Shift is held' but impl is data-driven"
    - "LWND-04/05/06: Implemented in code but REQUIREMENTS.md still marks them Pending — documentation gap"
gaps:
  - truth: "Window height grows with result count (min: input only, max: input + 8 rows = 440px)"
    status: failed
    reason: "listHeight computed uses Math.min(results.length, 5) capping at 5 rows (240px DOM height), not 8 rows (384px). setSize() also uses width 500 not 640. LWND-02 specifies '640px wide; max input + 8 rows'."
    artifacts:
      - path: "src/App.vue"
        issue: "Line 37: Math.min(results.value.length, 5) — capped at 5 rows, not 8 as per LWND-02"
      - path: "src/App.vue"
        issue: "Line 82: LogicalSize(500, h) — width is 500px, not 640px as per spec"
      - path: "src-tauri/tauri.conf.json"
        issue: "Line 17: 'width': 500 — window declared as 500px wide, not 640px"
    missing:
      - "Change Math.min(results.value.length, 5) to Math.min(results.value.length, 8) in listHeight computed"
      - "Change LogicalSize(500, h) to LogicalSize(640, h) in updateWindowHeight (or LogicalSize(500, h) if 500 is the intentional design width)"
      - "Update tauri.conf.json launcher window width to match (640 or 500 — pick one and be consistent)"
      - "Update scrollerRef scroll logic: line 57 uses visibleRows = 5, should match MAX_ROWS"
  - truth: "Admin badge appears on applicable rows (LWND-09 requirement: when Ctrl+Shift is held)"
    status: failed
    reason: "Badge v-if is item.requires_elevation. All search results return requires_elevation: false (hardcoded at both SearchResult construction sites in search.rs). The badge is therefore never rendered at runtime. Additionally, LWND-09 requirement specifies 'when Ctrl+Shift is held' but the implementation is data-driven — a behavioral mismatch with the written requirement."
    artifacts:
      - path: "src-tauri/src/search.rs"
        issue: "Line 106: requires_elevation: false — hardcoded for all app results in score_and_rank"
      - path: "src-tauri/src/search.rs"
        issue: "Line 154: requires_elevation: false — hardcoded for all system command results"
      - path: "src/App.vue"
        issue: "Line 312: v-if='item.requires_elevation' — badge never shown since all values are false"
    missing:
      - "Clarify intent: should the badge show when Ctrl+Shift is held (original LWND-09 spec) or only for requires_elevation apps (plan 04's redesign)?"
      - "If data-driven: Phase 6/8 must set requires_elevation: true for apps with manifest requiresAdministrator — currently none do, so badge is dead code"
      - "If keyboard-driven: restore v-if='index === selectedIndex && adminMode' (and update REQUIREMENTS.md to reflect the data-driven redesign intent)"
  - truth: "REQUIREMENTS.md accurately reflects implementation status for LWND-04, LWND-05, LWND-06"
    status: failed
    reason: "REQUIREMENTS.md marks LWND-04, LWND-05, LWND-06 as Pending. All three are fully implemented in App.vue: ArrowUp/Down/Enter/Escape keyboard navigation, Ctrl+Shift+Enter elevated launch invocation, and onFocusChanged auto-hide. This is a documentation-only gap, not a code gap — but it misrepresents phase completion state."
    artifacts:
      - path: ".planning/REQUIREMENTS.md"
        issue: "Lines 49-51, 154-156: LWND-04/05/06 marked [ ] Pending but code is complete"
    missing:
      - "Update REQUIREMENTS.md: mark LWND-04, LWND-05, LWND-06 as [x] Complete with Phase 5 in traceability table"

human_verification:
  - test: "Confirm intended window dimensions (640px vs 500px)"
    expected: "Either 640px (per LWND-02 spec) or 500px (current impl) declared as the intended design width. If 500px was an intentional UAT-informed redesign, update REQUIREMENTS.md LWND-02 and plan docs."
    why_human: "Cannot determine from code alone whether 500px/5-row cap was a deliberate design adjustment or a bug — UAT Test 4 passed which hints user accepted 5-row behavior"
  - test: "Verify window height animation is perceptible at 180ms"
    expected: "When query produces results, the window expands visibly with a smooth motion. The CSS height transition on .result-list plays before setSize() fires."
    why_human: "UAT Test 5 reported ambiguity ('maybe animation is too fast'). The fix (CSS transition + deferred setSize) is coded correctly but perceptibility is a human judgment call."
  - test: "Verify icon images load for real indexed apps"
    expected: "App icons appear as 32x32 PNGs next to app names. No broken image icons. DevTools shows no CSP errors for asset:// URLs."
    why_human: "Requires running pnpm tauri dev with real indexed apps. Asset protocol wiring is correctly coded but runtime depends on correct dataDir path resolution."
  - test: "Verify keyboard navigation behaves correctly end-to-end"
    expected: "ArrowDown selects next; ArrowUp wraps to last; ArrowDown from last wraps to first. Enter invokes launch stub. Ctrl+Shift+Enter invokes launch_elevated stub. Escape hides window."
    why_human: "All code is present and wired. Runtime verification requires the Tauri app to be running — keyboard events cannot be simulated programmatically from this context."
---

# Phase 5: Launcher Window UI Verification Report

**Phase Goal:** Build the launcher window UI — dark gradient background, search input, virtualised result list with icons and dynamic window resize
**Verified:** 2026-03-07T01:35:00Z
**Status:** gaps_found
**Re-verification:** Yes — gap-closure pass after plans 05-04 (admin badge) and 05-05 (animation + selection highlight)

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | Launcher renders dark gradient background with search input | VERIFIED | App.vue line 347: `linear-gradient(180deg, #242427 0%, #1c1c1e 40%, #181818 100%)` |
| 2 | Search input autofocused; placeholder correct | VERIFIED | Line 216: `inputRef.value?.focus()`; line 264: placeholder matches spec |
| 3 | RecycleScroller renders virtualised result list | VERIFIED | Lines 275-317: RecycleScroller with item-size=48, key-field="id" |
| 4 | Window height grows with results (min/max range) | FAILED | Math.min caps at 5 rows (240px), not 8 rows (384px) per LWND-02; width is 500px not 640px |
| 5 | Icons load via convertFileSrc + asset protocol | VERIFIED | Lines 3, 86-93: convertFileSrc wired; tauri.conf.json assetProtocol.enable=true; CSP img-src includes asset: |
| 6 | Keyboard navigation wired (ArrowUp/Down, Enter, Escape) | VERIFIED | Lines 127-146: all key cases handled with correct wrap logic |
| 7 | Ctrl+Shift+Enter triggers elevated launch | VERIFIED | Line 140: `if (e.ctrlKey && e.shiftKey) { launchElevated(item) }` |
| 8 | Auto-hide on focus loss via onFocusChanged | VERIFIED | Lines 234-240: `onFocusChanged` with launchInProgress guard |
| 9 | Admin badge (LWND-09) — architecture correct but badge never visible | FAILED | Badge v-if="item.requires_elevation" is correct; but requires_elevation is always false (hardcoded in search.rs) — badge is dead code at runtime |
| 10 | Path line on selected row when show_path=true, not for system commands | VERIFIED | Line 305: `v-if="index === selectedIndex && showPath && item.kind !== 'system'"` |
| 11 | System commands render with system_command.png icon, no path line | VERIFIED | Lines 148-156 (search.rs): system results use icon_path="system_command.png"; path-line conditional excludes kind=system |
| 12 | Window resize animation visible (CSS transition + deferred setSize) | VERIFIED (code) | Line 427: `transition: height 180ms ease`; lines 78-81: setSize deferred by animMode; HUMAN needed for perceptibility |
| 13 | RecycleScroller selection highlight consistent via active guard | VERIFIED | Lines 283, 287, 289: v-slot destructures active; :class and @mousemove gated on active |
| 14 | get_settings_cmd returns animation, show_path, data_dir to frontend | VERIFIED | store.rs lines 137-151: command defined; lib.rs line 116: registered in invoke_handler |
| 15 | Inter font renders in search input | VERIFIED | main.ts lines 3-5: fontsource imports; App.vue line 388: font-family Inter |
| 16 | REQUIREMENTS.md accurately reflects implementation status | FAILED | LWND-04/05/06 marked Pending but fully implemented |

**Score:** 9/12 must-haves verified (3 gaps)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/App.vue` | Full launcher UI, min 150 lines | VERIFIED | 498 lines; complete UI with all sections |
| `src/assets/magnifier.svg` | Flat monochrome SVG magnifier | VERIFIED | Exists; valid SVG with circle + line |
| `src/main.ts` | Font imports from @fontsource | VERIFIED | 3 fontsource imports present |
| `src-tauri/src/store.rs` | Settings struct with animation; get_settings_cmd | VERIFIED | animation field at line 37; get_settings_cmd at lines 137-151 |
| `src-tauri/src/lib.rs` | get_settings_cmd in invoke_handler | VERIFIED | Line 116: `crate::store::get_settings_cmd` |
| `src-tauri/tauri.conf.json` | assetProtocol enabled; CSP img-src asset: | VERIFIED | Lines 41-45: assetProtocol.enable=true; CSP includes asset: |
| `src-tauri/src/search.rs` | SearchResult with requires_elevation: bool | VERIFIED | Line 26: `pub requires_elevation: bool`; populated at all sites |
| `node_modules/vue-virtual-scroller` | npm package installed | VERIFIED | package.json + node_modules confirmed |
| `node_modules/@fontsource/inter` | npm package installed | VERIFIED | Confirmed in node_modules |
| `node_modules/@fontsource/jetbrains-mono` | npm package installed | VERIFIED | Confirmed in node_modules |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| App.vue `watch(query)` | `invoke('search')` | query watcher calls invoke, sets results ref | WIRED | Lines 41-48: watcher invokes search, selectedIndex reset |
| App.vue `updateWindowHeight()` | `getCurrentWindow().setSize()` | called after query watcher; deferred by animMode | WIRED | Lines 70-83: delay then setSize(500, h) |
| App.vue `img` tags | `convertFileSrc(fullPath)` | getIconUrl() builds path from dataDir + icons/ + icon_path | WIRED | Lines 86-94: convertFileSrc called with absolute path |
| App.vue `onFocusChanged` | `hideWindow()` | listener checks !launchInProgress before hiding | WIRED | Lines 234-240: guard present |
| App.vue `RecycleScroller v-slot` | `.result-row :class selected` | active prop guards class during DOM recycle | WIRED | Lines 283, 287: active destructured and used |
| `onMounted` | `invoke('get_settings_cmd')` | loads show_path, animation, data_dir into refs | WIRED | Lines 194-208: conditional on isTauriContext |
| `score_and_rank` SearchResult | App.vue badge `v-if` | requires_elevation field serialized via Tauri JSON | WIRED (structurally) | Field exists and is in interface; badge logic correct but always false |
| `src/App.vue` badge | `item.requires_elevation` | badge span v-if driven by stable per-result flag | WIRED (never fires) | v-if="item.requires_elevation" at line 312 — correct architecture but always false |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| LWND-01 | 05-02, 05-03 | Frameless, centered, always-on-top, no taskbar | SATISFIED | tauri.conf.json: center=true, decorations=false, skipTaskbar=true, alwaysOnTop=true |
| LWND-02 | 05-02, 05-05 | Fixed width 640px; height grows with results (max 8 rows) | PARTIAL | Height transition wired; but width is 500px (not 640px) and max is 5 rows (not 8) |
| LWND-03 | 05-02 | Input autofocused on window appear | SATISFIED | Line 216: inputRef.value?.focus() in onMounted |
| LWND-04 | 05-02, 05-03 | Arrow key navigation, Enter launches, Escape hides | SATISFIED (code) | Lines 127-148: all key handlers implemented; REQUIREMENTS.md still shows Pending |
| LWND-05 | 05-02, 05-03 | Ctrl+Shift+Enter elevated launch | SATISFIED (code) | Line 140-141: ctrlKey+shiftKey check calls launchElevated; REQUIREMENTS.md still shows Pending |
| LWND-06 | 05-02, 05-03 | Auto-hide on focus loss | SATISFIED (code) | Lines 234-240: onFocusChanged with launchInProgress guard; REQUIREMENTS.md still shows Pending |
| LWND-07 | 05-01, 05-02 | 32x32 icons per result row | SATISFIED | Lines 291-298: img.app-icon with width=32 height=32; getIconUrl via convertFileSrc |
| LWND-08 | 05-02 | Path line on selected row when show_path=true | SATISFIED | Line 305: v-if conditions correct (selectedIndex + showPath + kind != system) |
| LWND-09 | 05-04 | [Admin] badge when Ctrl+Shift held | PARTIAL | Architecture: badge uses item.requires_elevation (correct design direction). Gap: all requires_elevation values are false — badge dead code. Requirement text says "when Ctrl+Shift is held" not "when item.requires_elevation" — behavioral mismatch. |
| LWND-10 | 05-02, 05-05 | Result list virtualised | SATISFIED | RecycleScroller active guard wired; consistent highlight during scroll |
| LWND-11 | 05-02 | Correct placeholder text | SATISFIED | Line 264: "Search apps, or > for system commands…" exact match |
| LWND-12 | 05-02 | System commands with gear icon, no path line | SATISFIED | search.rs: icon_path="system_command.png"; App.vue: path-line excludes kind=system |

**Orphaned Requirements:** None. All LWND-01 through LWND-12 are claimed by plans in this phase.

**Documentation Gap:** LWND-04, LWND-05, LWND-06 are implemented in code but REQUIREMENTS.md still marks them `[ ] Pending`. This needs to be updated to `[x] Complete`.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|---------|--------|
| `src/App.vue` | 45, 76, 92 | Multiple `console.log` statements in production paths | Info | Debug noise; no functional impact |
| `src/App.vue` | 37 | `Math.min(results.value.length, 5)` — 5-row cap contradicts LWND-02 spec (8 rows) | Blocker | Window never reaches 440px max height; window height formula diverges from spec |
| `src/App.vue` | 82 | `new LogicalSize(500, h)` — 500px width contradicts LWND-02 spec (640px) | Blocker | Window is 140px narrower than spec |
| `src-tauri/src/search.rs` | 106, 154 | `requires_elevation: false` hardcoded at every construction site | Warning | Admin badge is dead code until Phase 6/8 — but this is expected/documented in plan 04 |
| `.planning/REQUIREMENTS.md` | 49-51, 154-156 | LWND-04/05/06 marked Pending but implemented | Warning | Misleading project state tracking |

---

### Human Verification Required

#### 1. Window Dimensions (Design Intent Clarification)

**Test:** Open tauri.conf.json and App.vue side by side. Confirm whether 500px width and 5-row cap are deliberate design adjustments from the 640px/8-row spec.
**Expected:** A design decision is documented — either the spec is updated to 500px/5-rows, or the code is corrected to 640px/8-rows.
**Why human:** UAT Test 4 (window height resizes dynamically) was reported as "pass" which suggests the user may have accepted the 5-row behavior. Cannot determine from code whether 500/5 was intentional.

#### 2. Window Height Animation Perceptibility

**Test:** Run `pnpm tauri dev`. Type a query to produce results. Observe whether the window expands with visible motion or appears to jump.
**Expected:** Window visibly expands/contracts smoothly over ~180ms as results appear/disappear.
**Why human:** UAT Test 5 was ambiguous ("maybe the animation is too fast"). The fix (CSS transition + deferred setSize) is correctly coded. Perceptibility is a human judgment.

#### 3. Admin Badge Runtime Behavior

**Test:** The badge v-if is now item.requires_elevation (always false). To test the badge works at all: temporarily change one `requires_elevation: false` in search.rs to `true` for a known app, rebuild, and verify the badge appears on that result row.
**Expected:** Badge text "[Admin]" in blue appears on the right side of any row where requires_elevation is true.
**Why human:** All requires_elevation values are false at runtime; cannot verify badge renders without modifying test data.

#### 4. Icon Images Load at Runtime

**Test:** Run `pnpm tauri dev` with indexed apps. Search for a known app (e.g., "notepad"). Verify the 32x32 app icon renders alongside the app name.
**Expected:** Icons display as .png images, no broken image placeholders. DevTools Network shows asset:// requests succeeding.
**Why human:** Asset protocol is correctly configured; runtime correctness depends on dataDir path resolution and actual icon files existing.

---

### Gaps Summary

**3 gaps block full verification of the phase goal:**

**Gap 1 — Window dimensions diverge from LWND-02 spec (blocker if spec is authoritative):**
The code uses 500px width and 5-row max instead of the specified 640px width and 8-row max. The window height formula `Math.max(56 + listHeight, 56)` where `listHeight = Math.min(results.length, 5) * 48` produces max 296px (not 440px). Additionally `setSize(500, h)` not `setSize(640, h)`. Both tauri.conf.json and App.vue are internally consistent at 500px/5-rows, suggesting a deliberate design decision — but it contradicts LWND-02. This needs explicit confirmation from the project owner: either update the requirement or fix the code.

**Gap 2 — Admin badge is never visible at runtime (LWND-09 partial):**
Plan 04 correctly decoupled the badge from transient keyboard state by introducing `requires_elevation: bool` on SearchResult. The architecture is sound. However, since all construction sites set `requires_elevation: false`, the badge is dead code. Furthermore, the LWND-09 requirement text says "when Ctrl+Shift is held" — not "when item.requires_elevation". This creates a behavioral mismatch: the original requirement described keyboard-triggered visibility; the implementation is data-triggered. Phase 6 or 8 must set `requires_elevation: true` for apps with elevation manifests to make this functional, OR the requirement must be updated to reflect the data-driven design decision from plan 04.

**Gap 3 — REQUIREMENTS.md documentation out of sync (documentation gap):**
LWND-04, LWND-05, and LWND-06 are fully implemented in App.vue (keyboard navigation, elevated launch, auto-hide) but remain marked `[ ] Pending` in REQUIREMENTS.md. This misrepresents the actual phase completion state.

---

### Gap Closure Verification (Plans 05-04 and 05-05)

The three specific gap-closure items requested for this pass are verified:

| Gap | Fix Applied | Code Evidence | Status |
|-----|-------------|---------------|--------|
| LWND-09: Admin badge data-driven via item.requires_elevation | Plan 05-04 | App.vue line 312: `v-if="item.requires_elevation"`; search.rs line 26: field exists | CLOSED (architecture) |
| LWND-02: Window resize animation via CSS height transition + deferred setSize() | Plan 05-05 | App.vue line 427: `transition: height 180ms ease`; line 78: animMode delay; line 82: setSize after delay | CLOSED (code) |
| LWND-10: RecycleScroller selection highlight via active prop guard | Plan 05-05 | App.vue line 283: `v-slot="{ item, index, active }"`; line 287: `active &&`; line 289: `active &&` | CLOSED |

All three gap-closure targets are confirmed implemented in the codebase. The residual gaps above (window dimensions and badge never firing) are pre-existing design decisions or scope items for Phase 6/8, not regressions from the gap-closure work.

---

_Verified: 2026-03-07T01:35:00Z_
_Verifier: Claude (gsd-verifier)_
