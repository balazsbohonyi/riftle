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
1. Verify the artifacts (MSI, EXE) were uploaded correctly.
2. Write release notes in the GitHub UI.
3. Click "Publish release" to make it visible to the public.

## Decisions

| Feature | Decision |
|---|---|
| CI Provider | **GitHub Actions** (native integration with repository) |
| Release Tool | **tauri-action** (official, handles artifact uploading) |
| Trigger | **Git Tags** (`v*`) |
| Release State | **Draft by default** (prevents accidental incomplete releases) |

## File Changes Summary

| File | Action | Purpose |
|---|---|---|
| `.github/workflows/release.yml` | **Create** | Define the CI/CD pipeline for releases |
| `.planning/docs/release-strategy.md` | **Create** | Document the release process |

## Verification

1. **Tag Push:** Push a dummy tag (e.g., `v0.0.0-test`) and verify the action starts.
2. **Build Success:** Monitor the GitHub Actions tab for successful completion.
3. **Artifact Check:** Ensure the draft release contains the expected Windows installers.
4. **Cleanup:** Delete test tags and draft releases after verification.
