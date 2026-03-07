---
status: diagnosed
trigger: "Window expand/contract animation is visible with motion, not instant jumps (slide 180ms)"
created: 2026-03-07T00:00:00Z
updated: 2026-03-07T00:00:00Z
---

## Current Focus

hypothesis: setSize() is called synchronously inside updateWindowHeight() with no delay — the OS window resize fires immediately, before any CSS transition has time to animate the HTML content expanding to fill the new space
test: trace execution order: query watcher fires -> updateWindowHeight() called -> setSize() awaited -> window grows -> CSS .result-list height style changes at same frame
expecting: setSize() and listHeight style update are effectively simultaneous, making the OS-level expansion appear instant even though a CSS opacity/transform transition is active
next_action: DIAGNOSED — diagnosis only mode, no fix to apply

## Symptoms

expected: Window visibly slides/expands smoothly (180ms) as results appear or disappear
actual: Height change appears to happen instantly (or too fast to perceive as animation)
errors: none — behaviour is functional but the transition is imperceptible
reproduction: Type a query in the launcher -> results appear -> observe window height change
started: Phase 05 implementation

## Eliminated

- hypothesis: CSS transition missing entirely on .launcher-app
  evidence: .anim-slide { transition: opacity 180ms ease, transform 180ms ease; } is present at line 364. Transition IS defined.
  timestamp: 2026-03-07

- hypothesis: animMode not set to 'slide' by default
  evidence: Line 24 initialises animMode as 'slide'. Settings load at onMounted overrides with loaded value (line 197). Default is correct.
  timestamp: 2026-03-07

- hypothesis: CSS transition applied to wrong element
  evidence: Transition is on .launcher-app (the root div) but it only transitions opacity and transform — NOT height. The height is controlled by Tauri OS window size, not by CSS height property on the div.
  timestamp: 2026-03-07

## Evidence

- timestamp: 2026-03-07
  checked: App.vue lines 361-368 — CSS animation mode rules
  found: |
    .anim-slide { transition: opacity 180ms ease, transform 180ms ease; }
    .anim-slide.visible { opacity: 1; transform: translateY(0); }
    No height transition is declared anywhere.
  implication: The CSS transition only covers show/hide animation (opacity + translateY). It does NOT animate the window height dimension.

- timestamp: 2026-03-07
  checked: App.vue lines 69-78 — updateWindowHeight()
  found: |
    async function updateWindowHeight() {
      const h = Math.max(56 + listHeight.value, 56)
      await getCurrentWindow().setSize(new LogicalSize(500, h)).catch(console.error)
    }
  implication: setSize() is called directly and awaited. The OS window resize is instant — Tauri provides no built-in easing for setSize(). The DOM listHeight style update (:style="{ height: listHeight + 'px' }") and the OS resize both happen within the same microtask queue flush.

- timestamp: 2026-03-07
  checked: App.vue line 46 — watcher call site
  found: |
    watch(query, async (q) => {
      results.value = q.trim() ? await invoke(...) : []
      selectedIndex.value = 0
      await updateWindowHeight()   // <-- immediate resize after results set
    })
  implication: There is zero deliberate delay between results arriving and setSize() being called. The window height jumps atomically with the results being rendered — no animation frame is given to interpolate.

- timestamp: 2026-03-07
  checked: App.vue lines 35-37 — listHeight computed
  found: |
    const listHeight = computed(() => Math.min(results.value.length, 5) * 48)
  implication: Truth spec says Math.max(56 + 2 + Math.min(results.length, 8) * 48, 58) with cap at 8 rows. Actual code uses Math.min(results, 5) * 48 with max cap at 5 rows and no +2px divider. Minor discrepancy but not the animation root cause.

- timestamp: 2026-03-07
  checked: App.vue lines 246, 270-278 — template binding for list height
  found: |
    <RecycleScroller :style="{ height: listHeight + 'px' }">
    No CSS transition on .result-list height. No transition on any container height.
  implication: The inner CSS height changes instantly (no transition property). The outer OS window resize via setSize() also changes instantly. There is nothing interpolating size over 180ms.

- timestamp: 2026-03-07
  checked: App.vue lines 361-365 — what .anim-slide actually animates
  found: Only opacity and transform (translateY -6px -> 0) on window open/close.
  implication: The 180ms duration was designed for the reveal animation (window appearing from above), NOT for the expand/contract on result count change. The expand/contract has no animation path at all.

## Resolution

root_cause: |
  updateWindowHeight() calls Tauri setSize() synchronously (no delay, no easing), so the OS window
  resizes instantly. There is no CSS height transition on any element, and Tauri's setSize() has no
  built-in interpolation. The 180ms "slide" mode only animates opacity+transform on show/hide — it
  has no effect on height changes when results appear or disappear.

fix: NOT APPLIED (diagnose-only mode)

verification: NOT APPLIED (diagnose-only mode)

files_changed: []
