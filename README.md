<div align="center">
   <img src="./docs/images/riftle-logo.png" alt="Riftle Launcher logo" width="64" height="64">
   <h1>Riftle Launcher</h1>
</div>

Riftle Launcher is a minimal Windows launcher for people who prefer the keyboard. Press the default hotkey, Ctrl+Space, type a few letters, and press Enter to open an app, folder, file, or custom shortcut.

It is inspired by Flow Launcher, but intentionally smaller in scope: fast app search, a quiet floating window, and settings that focus on daily launcher behavior instead of plugin-heavy workflows.

<div align="center">
   <img src="./docs/images/riftle-demo.gif" alt="Riftle demo">
</div>

## Status

> [!NOTE]
> Riftle is currently in beta. Download the latest prerelease from the [Releases](https://github.com/balazsbohonyi/riftle/releases) page.

## What Riftle Does

- Opens from anywhere with a configurable global hotkey.
- Searches installed Windows apps as you type.
- Ranks exact prefixes and acronym-style matches above loose fuzzy matches.
- Learns from usage so frequently launched apps become easier to reach.
- Launches the selected item with Enter.
- Launches apps as Administrator with Ctrl+Shift+Enter.
- Opens system actions such as lock, shutdown, restart, and sleep with the `>` prefix.
- Lets you create named shortcuts for folders, files, apps, and app launches with parameters.
- Keeps shortcuts above normal app results when their names match your query.
- Runs as a single instance, so opening Riftle again focuses the existing launcher instead of starting a duplicate copy.
- Stays available from the Windows system tray.
- Supports light, dark, and system themes.
- Supports portable mode, where settings and the search index live next to the app.

## First Run

When Riftle starts for the first time, it builds a local search index of your installed apps and shortcuts. The launcher window stays out of the way until you summon it with Ctrl+Space or from the tray icon.

The first index pass may take a short moment depending on how many apps and shortcuts are on the machine. After that, Riftle keeps the index fresh in the background based on the re-index interval in Settings.

## Basic Use

| Action | Shortcut |
|---|---|
| Show or hide Riftle | Ctrl+Space by default |
| Toggle Riftle from tray | Single-click the tray icon |
| Open tray menu | Right-click the tray icon |
| Move through results | Arrow Up / Arrow Down |
| Open selected result | Enter |
| Open app as Administrator | Ctrl+Shift+Enter |
| Close launcher | Escape |
| Open Settings | Ctrl+, launcher context menu, or tray menu |
| Open context menu | Right-click the launcher |

The configured hotkey opens Riftle over your current work. Type part of what you want, use the arrow keys if needed, then press Enter. Riftle hides again after launching the item.

The system tray icon is also available while Riftle is running. Double-click it to summon the launcher, single-click it to toggle the launcher, or right-click it to open the tray menu with Settings and Quit Launcher.

The default hotkey is Ctrl+Space. You can change it in Settings.

## Search

Riftle scans the usual Windows app locations automatically, including Start Menu entries and installed app shortcuts. Search is designed for short, fast queries:

- Typing the beginning of a name ranks that app highly.
- Typing initials can match multi-word names.
- Fuzzy matching catches partial or imperfect queries.
- Recently launched and frequently launched items are used as tie-breakers.

System commands are kept separate from normal app search. Type `>` to see them, or type a command name after it, for example:

```text
> lock
> shutdown
> restart
> sleep
```

## What Gets Indexed

Riftle builds its search results from local Windows sources:

- Start Menu shortcuts.
- Desktop shortcuts.
- UWP and Microsoft Store apps.
- Additional folders added in Settings.
- Custom folder and file shortcuts created in Riftle.

Excluded paths in Settings are skipped during indexing. Custom shortcuts are stored separately from the app scan and appear above normal app results when their names match your query.

## Shortcuts

Shortcuts are for things Windows app search does not handle well: project folders, documents, scripts, tools with command-line arguments, or files that should always open through a specific app.

Examples:

- `Work` -> `D:\Work`
- `Notes` -> `D:\Notes\inbox.md`
- `Budget` -> `D:\Finance\budget.xlsx`
- `Admin Tools` -> `C:\Tools\admin.cmd`
- `Project A` -> `Code.exe` with parameters pointing at a project folder

Shortcuts are useful because they give long paths and repetitive launch commands short, memorable names. Instead of browsing through Explorer or remembering a full command, you can type the alias in Riftle and press Enter.

### Creating Shortcuts

1. Open Riftle Settings with Ctrl+, or right-click the launcher and choose Settings.
2. Go to **Shortcuts**.
3. Choose **Directories** for folders or **Files** for files and app launches.
4. Select **Add folder** or **Add file**.
5. Enter the target path.
6. Optionally enter an alias. If you leave it blank, Riftle uses the folder or file name.
7. Save the shortcut.

Shortcut names must be unique. Riftle checks folder shortcuts and file shortcuts together, so two entries cannot both appear as the same search name.

### File Shortcut Parameters

File shortcuts can include parameters when the target is an executable app or script: `.exe`, `.com`, `.bat`, or `.cmd`.

Use this when a file is more reliable if opened through a specific app, or when you want a command-like launcher entry. For example, point the shortcut at an editor executable and put the project path in **Parameters**.

When parameters are used, setting an alias is mandatory. This keeps multiple shortcuts to the same executable easy to tell apart in search results.

## Settings

Open Settings with Ctrl+, from the launcher context menu, from the tray menu, or by searching for Settings.

### General

- **Launch at startup** starts Riftle when Windows starts. This is disabled in portable mode because portable copies should not register themselves with Windows automatically.
- **Play sound on open** controls the small feedback sound when the launcher appears.

### Hotkey

- **Global shortcut** controls the key combination that summons Riftle.
- The default shortcut is Ctrl+Space.
- Changes apply immediately.
- If Windows rejects a shortcut because another app owns it, Riftle falls back to the default shortcut and shows the active value.
- Use **Reset** to return to the default.

### Appearance

- **Theme** can follow the system theme or force light/dark mode.
- **Show path** controls whether Riftle shows the selected result's path under the result name.

### Search

- **Additional paths** adds folders Riftle should scan beyond the standard Windows app locations.
- **Excluded paths** prevents specific folders from being scanned.
- **Re-index interval** controls how often Riftle refreshes the app index in the background. Choose **Manual only** if you want indexing to happen only when requested.
- **Re-index now** refreshes the search index immediately.

### Shortcuts

- **Directories** contains folder shortcuts.
- **Files** contains file, app, and parameterized launch shortcuts.
- Aliases control the search name shown in Riftle.
- Shortcuts appear before normal app results when their names match the query.

## Portable Mode

Portable mode stores Riftle data next to the executable instead of in the normal Windows application data folder. This is useful for trying Riftle without installing it system-wide, or for keeping a self-contained copy on a synced or removable drive.

Create a `riftle-launcher.portable` marker file next to the executable. Riftle will store settings, icons, and index data in a local `data` folder beside the app.

## Privacy and Local Data

Riftle works locally. It indexes local app shortcuts, local app metadata, custom shortcut settings, icons, and launch counts so search can be fast and useful. This data is stored on your machine.

Riftle does not require an account, cloud sync, or an external service to search and launch apps.

## Development

This README is focused on using Riftle. Contributor documentation lives in [docs/DEV-SETUP.md](docs/DEV-SETUP.md):

- [Technology stack](docs/DEV-SETUP.md#technology-stack)
- [Project structure](docs/DEV-SETUP.md#project-structure)
- [Development commands](docs/DEV-SETUP.md#quick-commands)
- [Local environment setup](docs/DEV-SETUP.md#local-environment-setup)

## License

[MIT](LICENSE)
