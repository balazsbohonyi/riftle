## Unreleased — In development

- **Multi-monitor support.** Launcher and Settings now appear on the primary monitor by default. Optionally show the launcher on the monitor where the mouse cursor is. Per-monitor DPI handling was tightened so the launcher sizes and positions correctly after display changes.
- **Settings sidebar.** The Settings window is now organized with a left sidebar for General, Appearance, Search, and Shortcuts. General opens by default, the global shortcut controls now live at the top of General, and each section stays focused instead of scrolling through one long settings page.
- **Smoother launcher reveal.** Reduced the visible flash when showing the launcher, especially around monitor positioning and sizing updates.
- **Improved selection contrast.** Highlighted results and selected search text are now easier to read in both light and dark themes.
- **Refined visual polish.** The launcher, context menu, and settings window now have a calmer, more cohesive look with better contrast and depth.


## 1.0.0-beta.1 - 2025-05-26

### Launching apps

- **App launcher.** Press a global hotkey, type part of an app name, and press Enter to launch it. No mouse required. The launcher appears quickly - indexing runs in the background so it never blocks at startup.
- **Smart search ranking.** Typing the start of an app name or its initials ranks it higher than a loose fuzzy match. Frequently launched and recently launched apps break ties.
- **App icons in results.** Each result shows the app's own icon, extracted automatically during indexing.
- **UWP and Microsoft Store app support.** Riftle indexes and can launch apps installed from the Microsoft Store alongside traditional desktop apps. Apps that use custom protocol links, such as Steam games, are also handled correctly.
- **Launch as Administrator.** Hold Ctrl+Shift+Enter to launch the selected app with elevated permissions.
- **System commands.** Type `>` to reach lock, shutdown, restart, sleep, and hibernate directly from the launcher.
- **Search state preserved.** The launcher remembers your last query when you bring it back up.

### Custom shortcuts

- **Named shortcuts.** Add shortcuts to folders, files, scripts, and app launches with arguments. By default, shortcuts are scored and mixed in with app results - you can pin them above all apps in Settings.
- **File shortcut parameters.** Point a shortcut at an executable and supply command-line arguments, useful for launching the same app with different projects or profiles.

### Settings

- **Settings window.** A dedicated settings window covers the global hotkey, appearance, search paths, re-index controls, and custom shortcuts.
- **Configurable global hotkey.** Change the key combination that opens Riftle in Settings. The default is Ctrl+Space. If another app already owns the chosen shortcut, Riftle falls back to the default automatically and lets you know.
- **Appearance settings.** Choose between light, dark, and system themes. Toggle whether the file path appears under each search result.
- **Search path control.** Add extra folders for Riftle to scan, and exclude specific folders from indexing.
- **Re-index controls.** Set how often Riftle refreshes its app index in the background, or trigger a manual re-index at any time.
- **Launch at startup.** Optionally start Riftle with Windows. Disabled automatically in portable mode.
- **Sound on open.** An optional sound plays when the launcher appears.

### Window and system integration

- **System tray icon.** Riftle lives in the system tray. Single-click to toggle the launcher, right-click for the tray menu with Settings and Quit.
- **Right-click context menu.** Right-click the launcher to reach Settings and Quit without touching the tray.
- **Single instance.** Opening Riftle a second time focuses the existing window instead of starting a duplicate.
- **Portable mode.** Place a `riftle-launcher.portable` file next to the app and all data stays in a local `data` folder beside it, nothing written to AppData.
- **Drop shadow.** A subtle shadow sits behind the launcher window.

