# No-Admin Installer + Portable Build

## Summary
- Yes, Riftle can be installed per-user without administrator rights.
- The shield you saw is expected for the `.msi`: current generated WiX output uses `InstallScope="perMachine"`.
- Riftle itself does not appear to require admin rights during install. The only explicit elevation path is runtime-only: `launch_elevated()` uses `ShellExecuteW("runas")` when the user presses `Ctrl+Shift+Enter`.
- Tauri’s NSIS installer supports current-user install mode (`currentUser`) into `%LOCALAPPDATA%`, with metadata in `HKCU`. Official Tauri docs confirm this avoids admin rights: https://v2.tauri.app/distribute/windows-installer/

## Key Changes
- Prefer NSIS `.exe` as the consumer installer:
  - Set `bundle.targets` to `["nsis"]` or document that users should download only `src-tauri/target/release/bundle/nsis/*-setup.exe`.
  - Add explicit config under `bundle.windows.nsis.installMode: "currentUser"` in [tauri.conf.json](d:/develop/projects/riftle/src-tauri/tauri.conf.json:30), even though this is the current default, so the intent is locked.
- Treat MSI as enterprise/admin distribution:
  - Either stop building `.msi` for normal releases, or keep it clearly labeled as admin/per-machine.
  - If no-admin MSI is truly required, use a custom WiX `.wxs` template; Tauri’s built-in `WixConfig` exposes `template`, but no simple `installScope` toggle.
- Add portable packaging:
  - Build release normally, take `src-tauri/target/release/riftle.exe`.
  - Zip it with a sibling marker file named `riftle-launcher.portable`.
  - Include `.github/release/README_portable.txt`, explaining that data is stored in `./data/`, autostart is unavailable in the current portable UI, and WebView2 may need to be installed.
  - Automate this in `.github/workflows/release.yml` so each tagged release uploads `Riftle-<version>-portable-windows-x64.zip` to the draft GitHub Release.

## Admin-Rights Audit
- App data is already user-scoped in installed mode: `%APPDATA%\riftle-launcher`.
- Autostart uses the current-user Windows Run key via `HKCU`, not machine-wide startup.
- Indexing reads Start Menu/Desktop/PATH locations, including all-users locations, but does not need to write there.
- System commands like shutdown/restart may depend on normal Windows user privileges at runtime, but they do not require installer elevation.
- WebView2 is the main packaging caveat: installer bootstrapper may need to install/update WebView2. For strict no-admin portability, either require existing WebView2 or use Tauri `webviewInstallMode: "fixedRuntime"` and bundle the fixed runtime.

## Portable Autostart Discussion
- Windows technically allows autostart for a portable Riftle build without administrator rights.
- Enabling autostart through the current `tauri-plugin-autostart` path would write a current-user registry entry under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- The registry value would point to the portable executable's absolute path at the time the setting is enabled, for example `D:\Tools\Riftle\riftle.exe`.
- If the portable folder is moved, renamed, deleted, or stored on a disconnected removable drive, the registry entry becomes stale. On the next sign-in Windows will try that old path, fail to launch Riftle, and usually continue without a visible error.
- The stale entry is not dangerous, but it leaves residue in the user's profile and may still appear in Task Manager's Startup Apps list until manually removed.
- If portable autostart is allowed, the Settings UI should make the tradeoff explicit with wording like: `Starts Riftle from this folder when you sign in. Disable this before moving or deleting the portable folder.`
- A portable `README_portable.txt` should also mention that enabling startup writes a current-user startup entry and that users should disable it before deleting or moving the portable app.

## Test Plan
- Build NSIS and verify generated `installer.nsi` contains `RequestExecutionLevel user` and installs into `$LOCALAPPDATA\Riftle Launcher`.
- Run the NSIS installer as a non-admin user and confirm no UAC shield appears on Install.
- Confirm installed app launches, registers hotkey, writes settings/db/icons under `%APPDATA%\riftle-launcher`, and autostart toggles without UAC.
- Build portable zip, run from an arbitrary user-writable folder, confirm `./data/` is created next to the exe and no installer/admin prompt appears.
- Confirm the GitHub Release draft contains `Riftle-<version>-portable-windows-x64.zip`.
- Document MSI separately as admin/per-machine unless a custom per-user WiX template is implemented.

## Assumptions
- Normal public distribution should use the NSIS `.exe`, not the MSI.
- MSI remains optional for enterprise/admin installs.
- Portable build may require preinstalled WebView2 unless fixed runtime bundling is added.
