---
phase: 13-eliminate-flash-when-showing-launcher
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/lib.rs
autonomous: true
requirements: [DPI-FIX-02]

must_haves:
  truths:
    - "Launcher shows at correct physical size on first frame — no stale-size flash"
    - "Launcher is centered horizontally on target monitor on first show after DPI change"
  artifacts:
    - path: "src-tauri/src/lib.rs"
      provides: "show_positioned_launcher with correct show-before-set-size ordering using PhysicalSize"
  key_links:
    - from: "show_positioned_launcher"
      to: "monitor.scale_factor()"
      via: "resolved before show() to compute PhysicalSize"
      pattern: "scale_factor"
---

<objective>
Eliminate the split-second stale-size flash when the launcher appears on a monitor with a different DPI. The current fix (Task 12) reorders `show()` before `set_size()` — correct size after flash. This fix ensures the window is correctly sized BEFORE it's shown.

**Root cause of flash:** `show()` reveals the window at its stale cached geometry from the previous monitor. Then `set_size(LogicalSize)` resizes correctly — but the user sees the wrong size for one frame.

**Fix:** Resolve the target monitor FIRST, compute physical pixel dimensions using the resolved monitor's `scale_factor()`, and call `set_size(PhysicalSize)` before `show()`. `PhysicalSize` bypasses DPI conversion entirely — the window is at the correct pixel dimensions before it's ever visible.

Purpose: Zero-frame correct sizing on DPI changes.
Output: Single change to `src-tauri/src/lib.rs`
</objective>

<execution_context>
@C:/Users/Balazs/.config/opencode/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@src-tauri/src/lib.rs (lines 96-148 — show_positioned_launcher)

Current code after Task 12:
```rust
fn show_positioned_launcher(...) -> Result<(), String> {
    let win = app.get_webview_window("launcher")...;

    win.show().map_err(|e| e.to_string())?;         // FLASH: shows at stale size

    win.set_size(tauri::LogicalSize::new(window_width, window_height))...;

    let monitor = if follow_cursor { ... cursor ... } else { None };
    let monitor = monitor.or_else(|| primary).or_else(|| current);

    if let Some(monitor) = monitor {
        let scale_factor = monitor.scale_factor();
        // ... position math using scale_factor ...
        win.set_position(PhysicalPosition::new(x, y))?;
    } else {
        win.center()?;
    }

    win.set_focus()?;
    Ok(())
}
```

New approach (move monitor resolution before show, compute physical size):
1. Resolve target monitor FIRST (cursor → primary → current)
2. Get scale_factor from resolved monitor
3. `set_size(PhysicalSize)` — bypasses stale DPI, window sized correctly
4. `show()` — window visible at correct size, NO FLASH
5. Position using same physical coordinates
6. `set_focus()`
</context>

<tasks>

<task type="auto">
  <name>Task 1: Move monitor resolution before show(), use PhysicalSize for set_size()</name>
  <files>src-tauri/src/lib.rs</files>
  <action>
    Rewrite `show_positioned_launcher` (lines 96-148) to resolve the target monitor BEFORE showing the window, compute physical pixel dimensions using the monitor's `scale_factor()`, and pass `PhysicalSize` to `set_size()`. This eliminates the stale-size flash because `PhysicalSize` bypasses DPI conversion.

    New order:
    1. Resolve target monitor: cursor (if follow_cursor) → primary → current
    2. Get scale_factor from resolved monitor (default 1.0)
    3. `win.set_size(PhysicalSize::new(pw, ph))` — correct size, no DPI conversion needed
    4. `win.show()` — window appears at the CORRECT size (no flash)
    5. Position using physical coordinates (same scale_factor as sizing)
    6. `win.set_focus()`

    Replace the function body with:

    ```rust
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

        // Resolve target monitor first: cursor → primary → current
        let monitor = if follow_cursor {
            win.cursor_position()
                .ok()
                .and_then(|pos| win.monitor_from_point(pos.x, pos.y).ok().flatten())
        } else {
            None
        };
        let monitor = monitor
            .or_else(|| win.primary_monitor().ok().flatten())
            .or_else(|| win.current_monitor().ok().flatten());

        // Get scale factor from the resolved monitor — use 1.0 fallback
        let scale_factor = monitor
            .as_ref()
            .map(|m| m.scale_factor())
            .unwrap_or(1.0);

        // Set size using PhysicalSize BEFORE show() — PhysicalSize bypasses stale DPI
        // conversion, so the window is correctly sized before it's ever visible.
        // This eliminates the one-frame flash of stale geometry.
        win.set_size(tauri::PhysicalSize::new(
            (window_width * scale_factor).round() as u32,
            (window_height * scale_factor).round() as u32,
        ))
        .map_err(|e| e.to_string())?;

        // Show window — now at correct physical size, no flash
        win.show().map_err(|e| e.to_string())?;

        // Position using same resolved monitor and scale factor
        if let Some(monitor) = monitor {
            let work_area = monitor.work_area();
            let physical_width = window_width * scale_factor;
            let physical_anchor_height = anchor_height * scale_factor;
            let x = work_area.position.x
                + ((work_area.size.width as f64 - physical_width) / 2.0).round() as i32;
            let y = work_area.position.y
                + ((work_area.size.height as f64 - physical_anchor_height) / 2.0).round() as i32;

            win.set_position(tauri::PhysicalPosition::new(x, y))
                .map_err(|e| e.to_string())?;
        } else {
            win.center().map_err(|e| e.to_string())?;
        }

        win.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    }
    ```

    Key differences from current code:
    - Monitor resolution moved BEFORE `show()`
    - `set_size()` uses `PhysicalSize` (computed from logical × scale_factor) instead of `LogicalSize`
    - `show()` happens AFTER `set_size()` — window at correct dimensions from frame 1
    - `scale_factor` stored in local variable, reused for both sizing and positioning
  </action>
  <verify>
    <automated>cd src-tauri && cargo test 2>&1 | tail -5</automated>
  </verify>
  <done>
    - `cargo test` passes (all existing tests)
    - `pnpm build` passes
    - Monitor resolution happens before `win.show()`
    - `set_size()` uses `PhysicalSize` computed from logical × scale_factor
    - No flash on first show after DPI change
  </done>
</task>

</tasks>

<verification>
1. `cargo test` — all tests pass (101/101)
2. `pnpm build` — no errors
3. Manual: Connect monitor with different DPI, summon launcher — correct size from frame 1
</verification>

<success_criteria>
- [ ] `cargo test` passes
- [ ] `pnpm build` passes
- [ ] `show_positioned_launcher` resolves monitor, computes physical size, calls `set_size(PhysicalSize)`, THEN calls `show()`
- [ ] No stale-size flash on monitor DPI changes
</success_criteria>

<output>
After completion, create `.planning/quick/13-eliminate-flash-when-showing-launcher-on/13-SUMMARY.md`
</output>
