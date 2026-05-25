# GitHub Release Strategy

## Why This Document Exists

As Riftle approaches a distributable state, we need a reliable, repeatable, and automated way to provide binaries to users. Manual builds are prone to environment-specific issues and are difficult to manage across different platforms.

This document outlines the strategy for automating builds and releases using GitHub Actions and the official Tauri release workflow.

## Context

Riftle is a Tauri-based application. Tauri provides a first-party GitHub Action (`tauri-apps/tauri-action`) that handles:
- Cross-platform builds (Windows, macOS, Linux).
- Automatic creation of GitHub Releases.
- Uploading of installer artifacts (MSI, EXE, AppImage, etc.).
- Replacement of version placeholders.

## Implementation Plan

### Step 1: Automated Workflow Configuration
A GitHub Actions workflow is defined in `.github/workflows/release.yml`. This workflow is configured to:
- Trigger only on version tags (e.g., `v1.0.0`).
- Build on `windows-latest` (expandable to `macos-latest` and `ubuntu-latest`).
- Use `pnpm` for dependency management.
- Utilize `tauri-action` to build and draft the release.
- Create and upload a portable Windows zip containing `riftle.exe`, `riftle-launcher.portable`, and `README_portable.txt`.

### Step 2: Version Synchronization
Before any release, the version number must be synchronized across three core files:
1. `package.json`
2. `src-tauri/tauri.conf.json`
3. `src-tauri/Cargo.toml`

The `tauri-action` uses the version defined in `tauri.conf.json` to name the release and tag.

### Step 3: Release Triggering
Releases are triggered by pushing a git tag. This decouples the build process from regular commits to the `main` branch.
```bash
git tag v0.1.0
git push origin v0.1.0
```

### Step 4: Verification and Publishing
The workflow creates a **Draft Release**. This allows for a final manual check:
1. Verify the artifacts (installer EXE, MSI if enabled, and portable zip) were uploaded correctly.
2. Write release notes in the GitHub UI.
3. Click "Publish release" to make it visible to the public.

## Decisions

| Feature | Decision |
|---|---|
| CI Provider | **GitHub Actions** (native integration with repository) |
| Release Tool | **tauri-action** (official, handles artifact uploading) |
| Trigger | **Git Tags** (`v*`) |
| Release State | **Draft by default** (prevents accidental incomplete releases) |
| Portable Artifact | **Automated zip** (`riftle.exe` + `riftle-launcher.portable` + `README_portable.txt`) |

## GitHub Token

The workflow uses `${{ secrets.GITHUB_TOKEN }}`. This token is created automatically by GitHub Actions for each workflow run. It does not need to be added manually in repository secrets.

The workflow-level permission below is what allows the token to create and update releases:

```yaml
permissions:
  contents: write
```

Only add a separate personal access token if the release process later needs access outside this repository or permissions that `GITHUB_TOKEN` cannot provide.

## File Changes Summary

| File | Action | Purpose |
|---|---|---|
| `.github/workflows/release.yml` | **Create** | Define the CI/CD pipeline for releases |
| `.github/release/README_portable.txt` | **Create** | User-facing instructions bundled into portable release zips |
| `.planning/docs/release-strategy.md` | **Create** | Document the release process |

## Verification

1. **Tag Push:** Push a dummy tag (e.g., `v0.0.0-test`) and verify the action starts.
2. **Build Success:** Monitor the GitHub Actions tab for successful completion.
3. **Artifact Check:** Ensure the draft release contains the expected Windows installers and `Riftle-<version>-portable-windows-x64.zip`.
4. **Cleanup:** Delete test tags and draft releases after verification.

## Release Procedures

Run local commands from the repository root:

```powershell
cd D:\develop\projects\riftle
```

The release workflow is tag-driven. The pushed tag must match the app version in `src-tauri/tauri.conf.json`, because `tauri-action` replaces `v__VERSION__` from that file. Keep versions synchronized across:

1. `package.json`
2. `src-tauri/tauri.conf.json`
3. `src-tauri/Cargo.toml`

Tags containing a hyphen, such as `v1.0.0-beta.1`, are marked as GitHub prereleases by `.github/workflows/release.yml`.

Every release also gets a portable zip named:

```text
Riftle-<version>-portable-windows-x64.zip
```

The portable zip contains:

1. `riftle.exe`
2. `riftle-launcher.portable`
3. `README_portable.txt`

The marker file enables portable mode. When `riftle.exe` starts and sees `riftle-launcher.portable` next to it, Riftle stores settings, the database, and extracted icons in a sibling `data` folder.

### Test Release: `v0.0.0-test.1`

Use this to verify the real release path without publishing anything.

1. Create a temporary branch:

   ```powershell
   git status --short
   git switch -c codex/test-release-v0.0.0-test.1
   ```

2. Change the version to exactly `0.0.0-test.1` in:

   - `package.json`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`

3. Commit and push the temporary branch:

   ```powershell
   git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
   git commit -m "Prepare test release v0.0.0-test.1"
   git push -u origin codex/test-release-v0.0.0-test.1
   ```

4. Create and push the test tag:

   ```powershell
   git tag v0.0.0-test.1
   git push origin v0.0.0-test.1
   ```

5. In GitHub, open **Actions** and wait for the `publish` workflow run for `v0.0.0-test.1` to complete.

6. In GitHub, open **Releases** and inspect the draft release `Riftle v0.0.0-test.1`.

7. Verify that the expected Windows installer assets and `Riftle-0.0.0-test.1-portable-windows-x64.zip` were uploaded. Do not publish the test release.

8. Download `Riftle-0.0.0-test.1-portable-windows-x64.zip`, extract it to a user-writable folder, run `riftle.exe`, and confirm that a sibling `data` folder is created.

9. Clean up the GitHub draft release:

   - Go to **Releases**.
   - Open the draft `Riftle v0.0.0-test.1`.
   - Delete the draft release.

10. Clean up the test tag and branch locally and remotely:

   ```powershell
   git push origin --delete v0.0.0-test.1
   git tag -d v0.0.0-test.1
   git switch main
   git branch -D codex/test-release-v0.0.0-test.1
   git push origin --delete codex/test-release-v0.0.0-test.1
   ```

### First Beta Release: `v1.0.0-beta.1`

Use exactly:

```text
version: 1.0.0-beta.1
tag:     v1.0.0-beta.1
```

1. Start from an up-to-date `main` branch and create a release prep branch:

   ```powershell
   git switch main
   git pull
   git switch -c codex/release-v1.0.0-beta.1
   ```

2. Change the version to exactly `1.0.0-beta.1` in:

   - `package.json`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`

3. Update `CHANGELOG.md` so the released notes are under a versioned section and a new empty unreleased section remains above it:

   ```markdown
   ## Unreleased — In development

   No changes yet.

   ## 1.0.0-beta.1 — 2026-05-25
   ```

   Move the existing release notes from the previous `Unreleased — In development` section under `1.0.0-beta.1`.

4. Verify locally:

   ```powershell
   pnpm install --frozen-lockfile
   pnpm build
   cd src-tauri
   cargo test
   cd ..
   ```

5. Commit and push the release prep branch:

   ```powershell
   git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml CHANGELOG.md
   git commit -m "Prepare release v1.0.0-beta.1"
   git push -u origin codex/release-v1.0.0-beta.1
   ```

6. Open a pull request, review it, and merge it into `main`.

7. Tag the merge commit on `main`:

   ```powershell
   git switch main
   git pull
   git tag v1.0.0-beta.1
   git push origin v1.0.0-beta.1
   ```

8. In GitHub, open **Actions** and wait for the `publish` workflow run for `v1.0.0-beta.1` to complete.

9. In GitHub, open **Releases** and inspect the draft release `Riftle v1.0.0-beta.1`. It should be marked as a prerelease.

10. Replace the placeholder release body with the full `1.0.0-beta.1` section copied manually from `CHANGELOG.md`.

11. Verify the uploaded installer assets and `Riftle-1.0.0-beta.1-portable-windows-x64.zip`.

12. Download the portable zip, extract it to a user-writable folder, run `riftle.exe`, and confirm that a sibling `data` folder is created.

13. Click **Publish release**.

### First Stable Release: `v1.0.0`

Use this when the beta has been validated and the same feature set is ready to publish as the first stable release.

Use exactly:

```text
version: 1.0.0
tag:     v1.0.0
```

1. Start from an up-to-date `main` branch and create a release prep branch:

   ```powershell
   git switch main
   git pull
   git switch -c codex/release-v1.0.0
   ```

2. Change the version to exactly `1.0.0` in:

   - `package.json`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`

3. Update `CHANGELOG.md`.

   If the stable release contains the same content as `1.0.0-beta.1`, create a new `1.0.0` section above the beta section and summarize that this is the stable release of the beta-tested feature set. Keep the existing `Unreleased — In development` section at the top:

   ```markdown
   ## Unreleased — In development

   No changes yet.

   ## 1.0.0 — 2026-05-25

   Stable release of the Riftle 1.0 feature set previously published as `1.0.0-beta.1`.

   ## 1.0.0-beta.1 — 2026-05-25
   ```

   If additional fixes landed after the beta, list those fixes under `1.0.0` as well.

4. Verify locally:

   ```powershell
   pnpm install --frozen-lockfile
   pnpm build
   cd src-tauri
   cargo test
   cd ..
   ```

5. Commit and push the release prep branch:

   ```powershell
   git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml CHANGELOG.md
   git commit -m "Prepare release v1.0.0"
   git push -u origin codex/release-v1.0.0
   ```

6. Open a pull request, review it, and merge it into `main`.

7. Tag the merge commit on `main`:

   ```powershell
   git switch main
   git pull
   git tag v1.0.0
   git push origin v1.0.0
   ```

8. In GitHub, open **Actions** and wait for the `publish` workflow run for `v1.0.0` to complete.

9. In GitHub, open **Releases** and inspect the draft release `Riftle v1.0.0`. It should not be marked as a prerelease.

10. Replace the placeholder release body with the full `1.0.0` section copied manually from `CHANGELOG.md`.

11. Verify the uploaded installer assets and `Riftle-1.0.0-portable-windows-x64.zip`.

12. Download the portable zip, extract it to a user-writable folder, run `riftle.exe`, and confirm that a sibling `data` folder is created.

13. Click **Publish release**.
