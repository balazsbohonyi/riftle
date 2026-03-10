---
status: investigating
trigger: "Investigate issue: close-error-chrome-widgetwin-1412\n\n**Summary:** User still sees `[ERROR:ui\\\\gfx\\\\win\\\\window_impl.cc:124] Failed to unregister class Chrome_WidgetWin_0. Error = 1412` when closing app."
created: 2026-03-10T00:00:00+02:00
updated: 2026-03-10T00:00:00+02:00
---

## Current Focus

hypothesis: cleanup_before_exit() is not actually resolving the final WebView/window teardown order during app shutdown.
test: inspect the quit path and all close/exit handlers, then verify where cleanup_before_exit() runs relative to Tauri/WebView destruction.
expecting: if teardown ordering is still wrong, the current code will either skip cleanup on some paths or invoke it too late to prevent Chromium's class unregister error.
next_action: read the quit and shutdown-related Rust code completely.

## Symptoms

expected: App quits cleanly without Chromium/WebView teardown errors.
actual: Error still appears on app close after prior fix using cleanup_before_exit().
errors: [ERROR:ui\\gfx\\win\\window_impl.cc:124] Failed to unregister class Chrome_WidgetWin_0. Error = 1412
reproduction: Open app and trigger Quit Launcher from context menu (or close app).
started: Still present after most recent quit_app patch.

## Eliminated

## Evidence

## Resolution

root_cause:
fix:
verification:
files_changed: []
