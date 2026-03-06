---
phase: quick-1
plan: 1
type: execute
wave: 1
depends_on: []
files_modified:
  - .planning/phases/02-data-layer/02-01-PLAN.md
  - .planning/phases/02-data-layer/02-01-SUMMARY.md
  - .planning/phases/02-data-layer/02-03-PLAN.md
  - .planning/phases/02-data-layer/02-RESEARCH.md
  - .planning/phases/02-data-layer/02-VALIDATION.md
  - .planning/phases/02-data-layer/02-VERIFICATION.md
autonomous: true
requirements: []

must_haves:
  truths:
    - "All Phase 1 and Phase 2 planning docs reference %APPDATA%\\riftle-launcher\\ (not %APPDATA%\\com.riftle.launcher\\) for the installed-mode data path"
    - "All Phase 2 planning docs reference riftle-launcher.portable as the portable marker filename (not launcher.portable)"
  artifacts:
    - path: ".planning/phases/02-data-layer/02-VERIFICATION.md"
      provides: "Updated installed-mode path and portable marker name"
    - path: ".planning/phases/02-data-layer/02-RESEARCH.md"
      provides: "Updated APPDATA path references"
  key_links:
    - from: ".planning/phases/02-data-layer/02-VERIFICATION.md"
      to: "src-tauri/src/paths.rs"
      via: "Paths documented in VERIFICATION match actual implementation"
      pattern: "riftle-launcher"
---

<objective>
Update Phase 1 and Phase 2 GSD planning documents to reflect the path changes made during Phase 3 implementation:

1. Installed-mode APPDATA path: `%APPDATA%\com.riftle.launcher\` → `%APPDATA%\riftle-launcher\`
2. Portable marker filename: `launcher.portable` → `riftle-launcher.portable`

Purpose: The actual implementation in `src-tauri/src/paths.rs` uses hardcoded `%APPDATA%\riftle-launcher\` (via `std::env::var("APPDATA") + "riftle-launcher"`) and the marker file is `riftle-launcher.portable`. Earlier phase planning docs still reference the old names, creating a misleading mismatch between docs and code.

Note: The bundle identifier `com.riftle.launcher` in `tauri.conf.json` is unchanged and correct — do NOT replace that string. Only filesystem path references need updating.

Output: Phase 1 and Phase 2 planning files with consistent, accurate path references.
</objective>

<execution_context>
@C:/Users/Balazs/.claude/get-shit-done/workflows/execute-plan.md
@C:/Users/Balazs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md

Key facts from actual implementation (src-tauri/src/paths.rs):
- Installed mode: `PathBuf::from(appdata).join("riftle-launcher")` — i.e., `%APPDATA%\riftle-launcher\`
- Portable marker: `exe_dir.join("riftle-launcher.portable")` — i.e., `riftle-launcher.portable`
- The Tauri bundle identifier `com.riftle.launcher` is NOT used for the data directory (hardcoded path was chosen for discoverability)
</context>

<tasks>

<task type="auto">
  <name>Task 1: Update Phase 2 planning files — APPDATA path and portable marker</name>
  <files>
    .planning/phases/02-data-layer/02-01-PLAN.md
    .planning/phases/02-data-layer/02-01-SUMMARY.md
    .planning/phases/02-data-layer/02-03-PLAN.md
    .planning/phases/02-data-layer/02-RESEARCH.md
    .planning/phases/02-data-layer/02-VALIDATION.md
    .planning/phases/02-data-layer/02-VERIFICATION.md
  </files>
  <action>
In each file listed, make two targeted substitutions:

**Substitution A — APPDATA path:**
Replace all occurrences of `com.riftle.launcher` that appear as a filesystem path component (i.e., in contexts like `%APPDATA%\com.riftle.launcher`, `%APPDATA%/com.riftle.launcher`, `app_data_dir()` return value descriptions, warning sign examples). Replace with `riftle-launcher`.

Do NOT replace `com.riftle.launcher` where it is the bundle identifier in tauri.conf.json context (e.g., `"identifier": "com.riftle.launcher"` or discussions of the Tauri bundle identifier string itself). In practice: any occurrence in a path string like `%APPDATA%\com.riftle.launcher\` should change; occurrences in sentences like "identifier changed to com.riftle.launcher" or "bundle identifier is com.riftle.launcher" should stay.

Specific occurrences to update per file:

`02-01-PLAN.md`:
- Line 18: `%APPDATA%/com.riftle.launcher/` → `%APPDATA%/riftle-launcher/`
- Line 42: `%APPDATA%\com.riftle.launcher\` → `%APPDATA%\riftle-launcher\`
- Line 98: `// returns %APPDATA%\com.riftle.launcher\` → `// returns %APPDATA%\riftle-launcher\`
- Line 109: `launcher.portable` → `riftle-launcher.portable`
- Line 118: `launcher.portable` → `riftle-launcher.portable`
- Line 128: `/// Installed mode: %APPDATA%\com.riftle.launcher\` → `/// Installed mode: %APPDATA%\riftle-launcher\`
- Line 129: `/// Installed mode: %APPDATA%\com.riftle.launcher\ (via Tauri path resolver).` → update to reflect hardcoded path: `/// Installed mode: %APPDATA%\riftle-launcher\ (hardcoded via APPDATA env var + "riftle-launcher").`
- Lines 143, 167, 171, 174, 193, 195: `launcher.portable` → `riftle-launcher.portable`
- Line 265: `launcher.portable` → `riftle-launcher.portable`

`02-01-SUMMARY.md`:
- Line 19: `launcher.portable marker` → `riftle-launcher.portable marker`
- Line 25: `launcher.portable marker file` → `riftle-launcher.portable marker file`
- Line 57: `%APPDATA%/com.riftle.launcher/` → `%APPDATA%/riftle-launcher/`
- Line 70: `launcher.portable` → `riftle-launcher.portable`
- Line 92: `launcher.portable` → `riftle-launcher.portable`

`02-03-PLAN.md`:
- Line 188: `%APPDATA%\com.riftle.launcher\settings.json` → `%APPDATA%\riftle-launcher\settings.json`

`02-RESEARCH.md`:
- Line 30: `launcher.portable` → `riftle-launcher.portable`
- Line 56: `launcher.portable` → `riftle-launcher.portable`
- Line 63: `launcher.portable` → `riftle-launcher.portable`
- Line 90: `%APPDATA%\com.riftle.launcher\` (in the return value description) → `%APPDATA%\riftle-launcher\`; also update the surrounding sentence to note the actual implementation uses hardcoded path, not app_data_dir()
- Line 135: `launcher.portable` → `riftle-launcher.portable`
- Line 139: `// Uses Tauri path resolver — returns %APPDATA%\com.riftle.launcher\` → update comment to reflect actual implementation: `// Hardcoded via APPDATA env var — returns %APPDATA%\riftle-launcher\`
- Line 332: `%APPDATA%\com.riftle.launcher\settings.json` → `%APPDATA%\riftle-launcher\settings.json`
- Line 335: `%APPDATA%\com.riftle.launcher\` and `launcher.portable` → update both

`02-VALIDATION.md`:
- Line 68: `%APPDATA%/com.riftle.launcher/riftle.db` → `%APPDATA%/riftle-launcher/launcher.db` (note: also fix filename from riftle.db to launcher.db if it says riftle.db)

`02-VERIFICATION.md`:
- Line 9: `launcher.portable` → `riftle-launcher.portable`
- Line 11: `%APPDATA%/com.riftle.launcher/launcher.db` → `%APPDATA%/riftle-launcher/launcher.db`
- Line 12: `launcher.portable` and `%APPDATA%\\com.riftle.launcher\\` → update both
- Line 37: `launcher.portable` → `riftle-launcher.portable`
- Line 38: `%APPDATA%/com.riftle.launcher/` → `%APPDATA%/riftle-launcher/`
- Line 56: DATA-07 row — `launcher.portable` → `riftle-launcher.portable`
- Line 90: DATA-07 row — `launcher.portable` → `riftle-launcher.portable`
- Lines 123, 129, 135: `launcher.portable` → `riftle-launcher.portable`
- Line 124: `%APPDATA%\com.riftle.launcher\` → `%APPDATA%\riftle-launcher\`
- Line 130: `%APPDATA%\com.riftle.launcher\launcher.db` → `%APPDATA%\riftle-launcher\launcher.db`
- Line 136: `%APPDATA%\com.riftle.launcher\` → `%APPDATA%\riftle-launcher\`

Read each file in full first, then apply all substitutions in a single Write operation per file. Do not use sed or bash string replacement — read and write with the file tools for accuracy.
  </action>
  <verify>
    <automated>grep -rn "com\.riftle\.launcher" D:/develop/projects/riftle/.claude/worktrees/pensive-buck/.planning/phases/02-data-layer/ | grep -v "identifier\|bundle\|com\.riftle\.launcher.*identifier\|from com\." | grep -c "." ; echo "Expected: 0 path occurrences"</automated>
  </verify>
  <done>
    - Zero occurrences of `com.riftle.launcher` remain as a filesystem path in Phase 2 files (bundle identifier context occurrences are fine)
    - Zero occurrences of bare `launcher.portable` remain (all updated to `riftle-launcher.portable`)
    - All updated paths match what src-tauri/src/paths.rs actually implements
  </done>
</task>

<task type="auto">
  <name>Task 2: Update Phase 1 planning files — APPDATA path references</name>
  <files>
    .planning/phases/01-project-scaffold-configuration/01-02-PLAN.md
    .planning/phases/01-project-scaffold-configuration/01-02-SUMMARY.md
    .planning/phases/01-project-scaffold-configuration/01-CONTEXT.md
    .planning/phases/01-project-scaffold-configuration/01-RESEARCH.md
    .planning/phases/01-project-scaffold-configuration/01-VERIFICATION.md
  </files>
  <action>
Phase 1 files only contain `com.riftle.launcher` in bundle identifier context (tauri.conf.json identifier, capability IDs, window labels). These do NOT need to change — they correctly reference the Tauri bundle identifier, not a filesystem path.

Verify this before writing: grep each file for `com.riftle.launcher` and confirm every occurrence is in bundle identifier context (identifier field, capability JSON, "Bundle identifier" sentences). If any occurrence appears as a filesystem path (e.g., `%APPDATA%\com.riftle.launcher\`), update it to `%APPDATA%\riftle-launcher\`.

Also check for `launcher.portable` (without `riftle-` prefix) in Phase 1 files and update to `riftle-launcher.portable` if found.

If no filesystem path occurrences are found, no writes are needed for Phase 1 files — document this in the task output.
  </action>
  <verify>
    <automated>grep -rn "launcher\.portable\b" D:/develop/projects/riftle/.claude/worktrees/pensive-buck/.planning/phases/01-project-scaffold-configuration/ 2>/dev/null | grep -v "riftle-launcher\.portable" ; echo "Expected: no output (no bare launcher.portable in Phase 1)"</automated>
  </verify>
  <done>
    - Phase 1 files confirmed: no filesystem APPDATA path uses `com.riftle.launcher` — all references are bundle identifier context
    - No bare `launcher.portable` occurrences in Phase 1 files
    - If any filesystem path occurrences were found and fixed, files are updated
  </done>
</task>

</tasks>

<verification>
After both tasks complete, run a final check across all planning phases:

```bash
# Should return zero results (no old APPDATA path in filesystem context):
grep -rn "APPDATA.*com\.riftle\.launcher\|com\.riftle\.launcher.*APPDATA" \
  D:/develop/projects/riftle/.claude/worktrees/pensive-buck/.planning/phases/ 2>/dev/null

# Should return zero results (no bare portable marker):
grep -rn "\blauncher\.portable\b" \
  D:/develop/projects/riftle/.claude/worktrees/pensive-buck/.planning/phases/ 2>/dev/null | \
  grep -v "riftle-launcher\.portable"
```

Both commands should produce no output.
</verification>

<success_criteria>
- All Phase 1 and Phase 2 GSD planning files use `riftle-launcher` as the APPDATA subdirectory name (matching src-tauri/src/paths.rs line 28)
- All Phase 1 and Phase 2 GSD planning files use `riftle-launcher.portable` as the portable marker filename (matching src-tauri/src/paths.rs line 20)
- Bundle identifier `com.riftle.launcher` in tauri.conf.json context is untouched throughout
- No changes made to Phase 3 files (already correct)
- No changes made to STATE.md decisions (historical record of decisions, bundle identifier context only)
</success_criteria>

<output>
No SUMMARY.md needed for this quick task — it is a documentation-only correction with no code changes.
</output>
