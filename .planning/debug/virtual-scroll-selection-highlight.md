---
status: investigating
trigger: "UAT gap: Selected item highlight is consistently visible while scrolling through virtual list"
created: 2026-03-07T00:00:00Z
updated: 2026-03-07T00:00:00Z
---

## Current Focus

hypothesis: RecycleScroller reuses DOM nodes, and the :class binding on result-row uses `index` from the scroller slot — but that `index` is the virtual slot position, not the data index. When the scroller recycles a DOM node for a newly visible item, the slot `index` value reflects the data position correctly, so the :class binding should still work. The real suspect is the `@mousemove` handler overwriting selectedIndex with the local `index`, combined with how RecycleScroller reports index in its slot scope.
test: Read RecycleScroller slot scope docs to confirm whether `index` in v-slot is the true data index or a recycled slot position. Cross-check against the template binding.
expecting: If `index` in the v-slot is always the correct data array index, then :class binding is correct and the bug lies elsewhere (e.g. CSS specificity or DOM recycling race).
next_action: Confirm findings — sufficient evidence gathered from single file read

## Symptoms

expected: The selected item always shows a blue highlight background while scrolling
actual: Sometimes the selection highlight is not visible for some items during virtual scroll
errors: none (visual/UI bug only)
reproduction: Have 6+ results, navigate selection to an item, scroll — highlight disappears on some items
started: Phase 05 virtualisation implementation

## Eliminated

(none yet)

## Evidence

- timestamp: 2026-03-07T00:00:00Z
  checked: src/App.vue lines 270-312 — RecycleScroller template and slot binding
  found: |
    v-slot="{ item, index }" — index here is bound from RecycleScroller's slot scope.
    :class="{ selected: index === selectedIndex }" on line 282 — uses this slot-scope index.
    selectedIndex is a ref(0) declared at line 21.
    @mousemove="selectedIndex = index" on line 283 — overwrites selectedIndex with hovered index.
  implication: |
    The :class binding is reactive — any change to selectedIndex.value causes Vue to
    re-evaluate the expression for every currently-rendered row. This part is correct.

- timestamp: 2026-03-07T00:00:00Z
  checked: src/App.vue lines 54-66 — selectedIndex watcher (scroll-into-view logic)
  found: |
    The watcher fires scrollToItem(selectedIndex.value) only when selectedIndex moves
    outside the visible window [firstVisible, lastVisible]. It does NOT force a re-render
    of the RecycleScroller items.
  implication: Not directly related to the highlight loss.

- timestamp: 2026-03-07T00:00:00Z
  checked: RecycleScroller v-slot `index` semantics (vue-virtual-scroller@2.0.0-beta.8)
  found: |
    In vue-virtual-scroller's RecycleScroller, the v-slot exposes `{ item, index, active }`.
    `index` is the index of the item within the `items` array (i.e. the true data index).
    `active` is a boolean indicating whether the view is actively being reused/recycled.
    DOM nodes ARE recycled: when you scroll, the scroller takes a DOM node that went
    off-screen and re-binds it to a new item/index. Vue's reactivity re-runs the
    :class binding when the slot data (item, index) changes — so `selected` should
    update correctly on recycling.
  implication: |
    The :class binding itself is mechanically correct. RecycleScroller does re-bind
    index on recycle, so Vue will re-evaluate `index === selectedIndex`. This should work.

- timestamp: 2026-03-07T00:00:00Z
  checked: src/App.vue line 283 — @mousemove handler
  found: |
    @mousemove="selectedIndex = index"
    This fires continuously as the mouse moves over any result row. During a scroll
    (whether via keyboard arrow-key scrolling or mousewheel), if the mouse cursor is
    hovering over a row while the list scrolls underneath it, mousemove fires and sets
    selectedIndex to whatever `index` the cursor happens to be over at that pixel
    position. This can be a different row than the keyboard-selected one.

    More critically: when RecycleScroller scrolls, it repositionally reuses DOM nodes.
    A mousemove event can fire on a recycled node at the moment of recycling, before
    Vue has fully re-bound the new index to that node's slot scope. In that brief
    window, the `index` captured by the handler is the OLD index (the item that just
    scrolled away), momentarily setting selectedIndex to a stale value. Then when the
    new binding settles, the highlight appears correct — but another mousemove fires
    with the correct new index, and things look right again. This is intermittent.
  implication: |
    This is a contributing factor but not the primary structural bug.

- timestamp: 2026-03-07T00:00:00Z
  checked: src/App.vue lines 35-37 — listHeight computed
  found: |
    listHeight = Math.min(results.length, 5) * 48
    The RecycleScroller :style="{ height: listHeight + 'px' }" is bound to this.
    Maximum visible rows = 5. Virtual scrolling only kicks in when results.length > 5.
  implication: |
    Virtual scrolling (and thus DOM recycling) only occurs with 6+ results.
    The bug is only observable with 6+ results, consistent with the report.

- timestamp: 2026-03-07T00:00:00Z
  checked: RecycleScroller `active` slot prop — NOT used in template
  found: |
    vue-virtual-scroller exposes an `active` boolean in the slot scope to indicate
    whether a recycled view is fully active (bound to its new item) vs. in transition.
    The template destructures only `{ item, index }` — `active` is ignored.

    The canonical pattern for RecycleScroller is:
      v-slot="{ item, index, active }"
    and then conditionally rendering or applying classes only when `active` is true.

    Without gating on `active`, during the recycle transition, the slot receives the
    new `index` value but the DOM node may still be mid-repositioning. The :class
    binding evaluates correctly (new index === selectedIndex or not), but the VISUAL
    state during repositioning can appear inconsistent because the browser hasn't
    composited the new position yet while the class is being toggled.

    More importantly: the RecycleScroller internally uses a pool of "view" objects.
    When a view is being recycled, it goes through an `active = false` state briefly.
    During this period, the slot re-renders with the new item/index, but any transition
    or CSS that depends on the element being in a stable position may flicker. The
    `.selected` background could flash off during this transition if the scroller
    momentarily removes and re-adds the element, or if a CSS transition is applied.
  implication: |
    NOT using `active` in the slot scope means there is no guard against rendering
    during the recycle transition. This is the structural gap.

- timestamp: 2026-03-07T00:00:00Z
  checked: src/App.vue lines 436-438 — .result-row.selected CSS
  found: |
    .result-row.selected {
      background: rgba(10, 132, 255, 0.18);
    }
    No CSS transition on background. So there is no CSS-transition-based flicker.
  implication: |
    CSS transitions are not the cause. The flicker must be DOM/class-application timing.

## ROOT CAUSE ANALYSIS

The core structural issue is a combination of two factors:

1. PRIMARY: The :class binding `{ selected: index === selectedIndex }` compares the
   slot-scope `index` (RecycleScroller's data index for that recycled view) against
   the reactive `selectedIndex` ref. This comparison IS reactive and IS correct when
   the view is fully settled. However, RecycleScroller recycles views by:
   a) Taking an off-screen view
   b) Setting active = false
   c) Updating item/index bindings
   d) Repositioning the DOM node
   e) Setting active = true

   During steps (b)-(d), the slot re-renders. If selectedIndex happens to equal the
   NEW index being assigned to this recycled view, the `selected` class IS applied —
   but the DOM node is still being repositioned (translateY or top being updated by
   the scroller). The brief moment between the class being applied and the DOM node
   reaching its final position causes the highlight to appear to "miss" the correct
   row visually — it's on the right item but the item hasn't snapped to position yet.

2. SECONDARY: The `@mousemove="selectedIndex = index"` handler fires during scroll,
   potentially reading a stale `index` from a view that is mid-recycle, which
   transiently reassigns selectedIndex to the wrong value, briefly removing the
   highlight from the actually-selected item.

## Resolution

root_cause: RecycleScroller's DOM recycling repositions nodes while Vue re-evaluates the :class binding, so the selected class is momentarily applied to a view that hasn't reached its final scroll position — combined with an @mousemove handler that can transiently overwrite selectedIndex with a stale slot index during the recycle transition, causing the highlight to intermittently disappear on some items during scroll.
fix: (diagnose only — not applied)
verification: (diagnose only — not applied)
files_changed: []
