# UWP Application Discovery

## Objective
Enable discovery and launching of Windows Universal Platform (UWP) and Appx applications (e.g., Calculator, Windows Terminal, Clock) within Riftle.

## Background
Standard Win32 indexing (crawling the Start Menu and Desktop) often misses UWP apps because they do not always have traditional `.lnk` shortcuts in the filesystem. Additionally, UWP icons are not easily extracted via traditional `ExtractIconExW` calls as they are stored within Appx packages.

## Strategy: Native Shell API
Instead of scanning the filesystem or querying the Windows Package Manager (which includes many background libraries), we utilize the **Native Shell API** to enumerate the virtual `shell:AppsFolder` (`FOLDERID_AppsFolder`).

### Benefits
- **Accuracy**: This folder mirrors exactly what the user sees in the Windows Start Menu.
- **Performance**: It is a virtual folder maintained by the OS; querying it is significantly faster than a full disk scan.
- **Icon Quality**: It provides access to high-resolution assets via specialized Shell interfaces.

## Implementation Details

### 1. Enumeration (`indexer.rs`)
- **API**: `SHGetKnownFolderItem` with `FOLDERID_AppsFolder`.
- **Iteration**: Uses `IEnumShellItems` to walk the contents of the virtual folder.
- **Metadata**:
    - **Display Name**: Retrieved via `SIGDN_NORMALDISPLAY`.
    - **AUMID**: The "Application User Model ID" is extracted from the property store using `PKEY_AppUserModel_ID`.

### 2. Icon Extraction Refactor
A major refactor was performed to support both File-based icons and UWP-based bitmaps:
- **`IconSource` Enum**: Created to distinguish between `File(PathBuf)` and `Uwp(String)`.
- **GDI Refactor**: Extracted the core GDI-to-PNG logic from `icon_png_from_hicon` into a shared `icon_png_from_hbitmap(hbitmap: isize)`.
- **UWP Icons**: Uses `IShellItemImageFactory::GetImage` to retrieve a high-quality `HBITMAP` for UWP entries.

### 3. Database & Deduplication (Fallback Strategy)
- **Storage**: UWP apps are stored with an ID prefix `uwp:<AUMID>` and a path `shell:AppsFolder\<AUMID>`.
- **Priority (Inverted)**: Standard sources (Start Menu, Desktop, Additional) are processed **first**. `AppsFolder` is processed **last** as a fallback.
- **Deduplication**: Riftle tracks `seen_names`. If an app is already found via a standard shortcut (like a Steam game `.url` or a Win32 `.lnk`), the UWP entry is skipped. This ensures that custom icons parsed from shortcuts are preserved, while pure UWP apps (Calculator, Terminal) are still successfully discovered.
- **Impact**: This resolves issues where Steam/Epic games would appear with missing/generic icons because the UWP crawler claimed them first.

### 4. Launching (`commands.rs`)
By storing the path as `shell:AppsFolder\<AUMID>`, the existing `launch` command works without modification. `ShellExecuteW` natively understands these Shell paths and opens the corresponding UWP application.

## Technical Hurdles
- **Type Mismatches**: Resolved mismatches between the `windows` crate (using raw pointers) and `windows-sys` (using `isize`) by applying specific casts in GDI calls.
- **Dependencies**: Added `Win32_Storage_EnhancedStorage` and `Win32_Graphics_Gdi` features to `Cargo.toml`.

## Verification
- **Cargo Check**: Verified that the implementation is syntactically and type-correct.
- **Manual Launching**: Confirmed that `shell:AppsFolder\` paths are correctly handled by the Windows shell.
- **Icon Extraction**: Verified that the new bitmap-to-PNG flow handles transparency and sizing correctly for modern UWP assets.
