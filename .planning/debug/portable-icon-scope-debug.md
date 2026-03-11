# portable-icon-scope-debug

## Symptom

Portable-mode search results show broken icons even though installed/dev mode icons render normally.

## Reproduction

1. Create `src-tauri/target/debug/riftle-launcher.portable`.
2. Run `pnpm tauri dev`.
3. Search for apps with custom icons.
4. Observe broken image placeholders in the launcher.

## Findings

- Portable mode stores icons under `src-tauri/target/debug/data/icons`.
- That directory exists and contains hashed `.png` files.
- `src-tauri/tauri.conf.json` scopes asset access to `$DATA/riftle-launcher/icons/**` and `$EXE/../data/icons/**`.
- Tauri's `$EXE` token resolves to the executable's parent directory, not the executable file path.
- In dev portable mode, `$EXE` therefore resolves around `src-tauri/target/debug`, so `$EXE/../data/icons/**` points at `src-tauri/target/data/icons/**`.
- `src-tauri/target/data/icons` does not exist.

## Root Cause

The portable asset-protocol scope misinterprets `$EXE`. The config assumes `$EXE` is the executable file and adds `..`, but Tauri resolves `$EXE` to the executable directory. That makes the portable scope one level too high, so asset URLs for real icons fall outside the allowed scope and the webview blocks them.

## Evidence

- Real portable icon directory: `D:\develop\projects\riftle\src-tauri\target\debug\data\icons`
- Non-existent scoped directory: `D:\develop\projects\riftle\src-tauri\target\data\icons`
- Config line: `"$EXE/../data/icons/**"`

## Fix Direction

Change the portable asset scope from `$EXE/../data/icons/**` to `$EXE/data/icons/**`, then re-run the portable smoke test.
