---
phase: 12-fix-multi-monitor-dpi-sizing
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/lib.rs
autonomous: true
requirements: [DPI-FIX-01]

must_haves:
  truths:
    - "Launcher appears at correct physical size when summoned on new monitor (first show after connect/disconnect)"
    - "Launcher is centered horizontally on the target monitor on first show after monitor change"
    - "Hide+reshow is no longer needed to correct sizing"
  artifacts:
    - path: "src-tauri/src/lib.rs"
      provides: "show_positioned_launcher correctly orders window show, size, and position"
      min_lines: 45
  key_links:
    - from: "show_positioned_launcher"
      to: "win.show()"
      via: "must happen before set_size to trigger DPI re-evaluation"
      pattern: "show\\(\\)"
---

<objective>
Fix the multi-monitor DPI sizing bug where the launcher shows with incorrect physical width on first summon after connecting or disconnecting an external monitor with a different DPI scale.

**Root cause:** `show_positioned_launcher` calls `win.set_size(LogicalSize(...))` while the window is still hidden. On Windows, DPI change notifications only fire when a window becomes visible (`show()`). The `LogicalSize → PhysicalSize` mapping on a hidden window uses stale DPI awareness from the previous monitor. After `show()` fires (triggering DPI re-evaluation), subsequent calls work correctly — that's why hide+reshow masks the issue.

**Fix:** Reorder operations so `win.show()` happens FIRST. This triggers DPI re-evaluation, then `set_size()` maps logical sizes to the correct physical dimensions for the target monitor. The window uses fade-in animation with transparency, so no visible flash occurs from the reorder.

**Purpose:** One-shot correct sizing on monitor connect/disconnect — no more hide+reshow workaround.
**Output:** Single change to `src-tauri/src/lib.rs`
</objective>

<execution_context>
@C:/Users/Balazs/.config/opencode/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@src-tauri/src/lib.rs (lines 96-142 — show_positioned_launcher)
</context>

<interfaces>
The `show_positioned_launcher` function in `lib.rs`:

```rust
#[tauri::command]
fn show_positioned_launcher(
    app: tauri::AppHandle,
    window_width: f64,
    window_height: f64,
    anchor_height: f64,
    follow_cursor: bool,
) -> Result<(), String> {
    let win = app
        .get_webview_window("launcher")
        .ok_or_else(|| "launcher window not found".to_string())?;

    win.set_size(tauri::LogicalSize::new(window_width, window_height))
        .map_err(|e| e.to_string())?;

    let monitor = if follow_cursor { ... };
    let monitor = monitor.or_else(|| ...).or_else(|| ...);

    if let Some(monitor) = monitor {
        // ... position calculation using monitor.scale_factor() ...
        win.set_position(tauri::PhysicalPosition::new(x, y))
            .map_err(|e| e.to_string())?;
    } else {
        win.center().map_err(|e| e.to_string())?;
    }

    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}
```

Constants from App.vue (passed from frontend):
- `WINDOW_WIDTH = 564` (CSS pixels, logical)
- `window_height` = launcherWindowHeight(...) (CSS pixels, logical)  
- `anchor_height` = launcherCenterAnchorHeight(...) (CSS pixels, logical)

Monitor scale_factor comes from `monitor.scale_factor()` which is already correct (resolved via monitor chain at runtime). The problem is that `set_size()` runs before `show()` so the window's internal DPI context hasn't updated yet.
</interfaces>

<tasks>

<task type="auto">
  <name>Task 1: Reorder show_positioned_launcher — show() before set_size()/set_position()</name>
  <files>src-tauri/src/lib.rs</files>
  <action>
    In `show_positioned_launcher` (lines 96-142), reorder operations so `win.show()` fires BEFORE `win.set_size()`, monitor resolution, and `win.set_position()`. The new order:

    1. Show window first (triggers DPI re-evaluation on Windows when target monitor has different scale)
    2. Set window size (LogicalSize — now maps correctly because DPI context is current)
    3. Resolve target monitor via chain: cursor → primary → current → center()
    4. Calculate position using resolved monitor's scale_factor
    5. Set window position
    6. Set focus

    **Why this works:** Windows sends `WM_DPICHANGED` when a hidden window becomes visible on a monitor with a different DPI. This updates the window's internal DPI awareness. Subsequent `set_size(LogicalSize(...))` then maps logical pixels to the correct physical size. The fade-in animation (opacity 0→1) prevents any visible flash from the reorder.

    **Why the original order fails:** `set_size()` on a hidden window uses the stale DPI context from the monitor where the HWND was last shown. The logical-to-physical mapping is wrong, producing incorrect physical size. The stale size is what becomes visible on first `show()`.
  </action>
  <verify>
    <automated>cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5</automated>
  </verify>
  <done>
    - `cargo test` passes (all existing tests)
    - `pnpm build` passes (frontend type-check)
    - `show_positioned_launcher` now calls `win.show()` before `win.set_size()`
    - No other behavior changed
  </done>
</task>

</tasks>

<verification>
1. Run `cargo test` from `src-tauri/` — all tests pass (no business logic changed, only operation order)
2. Run `pnpm build` — frontend still type-checks (App.vue unchanged)
3. Visual/functional check (manual): Connect external monitor with different DPI, summon launcher via hotkey — size and position correct on first show. Disconnect external monitor, summon launcher — same. No regression on single-monitor setups.
</verification>

<success_criteria>
- [ ] `cargo test` passes
- [ ] `pnpm build` passes  
- [ ] `show_positioned_launcher` orders operations as: show → set_size → resolve_monitor → set_position → set_focus
- [ ] Launcher no longer shows with wrong size on first summon after monitor connect/disconnect
</success_criteria>

<output>
After completion, create `.planning/quick/12-fix-multi-monitor-dpi-sizing-on-connect-/12-SUMMARY.md`
</output>
