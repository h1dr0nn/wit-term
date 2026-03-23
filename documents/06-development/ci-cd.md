# CI/CD Pipeline

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

CI/CD configuration for Wit using GitHub Actions. The pipeline ensures code quality, successful builds on all platforms, and automated release creation with separate development and production channels.

---

## Table of Contents

- [Overview](#overview)
- [Tag Strategy](#tag-strategy)
- [CI Pipeline](#ci-pipeline)
- [Matrix Builds](#matrix-builds)
- [Caching](#caching)
- [Development Pipeline](#development-pipeline)
- [Release Pipeline](#release-pipeline)
- [In-App Update System](#in-app-update-system)
- [Workflow Files](#workflow-files)
- [Badge Setup](#badge-setup)

---

## Overview

| Pipeline | Trigger | Purpose | Signing |
|----------|---------|---------|---------|
| **CI** | Push to `main`, pull requests | Lint, build, test | None |
| **Development** | Push tag `develop-v*` | Build dev binaries, unsigned | No signing, no `.sig` files |
| **Release** | Push tag `release-v*` | Build release binaries, sign, publish | Signed with `.sig` files |

### CI Platform

- **GitHub Actions** - integrates directly with the repo, free for open-source projects.
- Runners: `ubuntu-latest`, `macos-latest`, `windows-latest`.

---

## Tag Strategy

Wit uses a dual-tag system to separate development builds from production releases:

### `develop-v*` - Development Builds

```bash
# Examples
git tag develop-v0.1.0-alpha.1
git tag develop-v0.2.0-beta.3
git tag develop-v0.3.0-rc.1
```

- Triggers the **Development Pipeline**
- Builds binaries for all platforms
- Does **NOT** sign binaries (no `.sig` files generated)
- Does **NOT** create a GitHub Release (or creates as draft/prerelease)
- Used for internal testing, QA, and development builds
- Update channel: `develop` - only users on the develop channel receive these

### `release-v*` - Production Releases

```bash
# Examples
git tag release-v0.1.0
git tag release-v0.2.0
git tag release-v1.0.0
```

- Triggers the **Release Pipeline**
- Builds optimized binaries for all platforms
- **Signs all binaries** with Tauri's updater private key
- Generates `.sig` signature files for each binary
- Creates a **public GitHub Release** with all artifacts
- Update channel: `stable` - all users receive these updates
- Users can check for updates and install directly from within the app

### Tag Flow

```
feature branch → PR → main → develop-v0.2.0-beta.1  (test build)
                              develop-v0.2.0-beta.2  (fix + test)
                              develop-v0.2.0-rc.1    (release candidate)
                              release-v0.2.0         (production release)
```

### Version Naming Convention

| Tag Pattern | Example | Channel | Signing |
|-------------|---------|---------|---------|
| `develop-v{semver}-alpha.N` | `develop-v0.2.0-alpha.1` | develop | No |
| `develop-v{semver}-beta.N` | `develop-v0.2.0-beta.3` | develop | No |
| `develop-v{semver}-rc.N` | `develop-v0.2.0-rc.1` | develop | No |
| `release-v{semver}` | `release-v0.2.0` | stable | Yes |

---

## CI Pipeline

### Triggers

```yaml
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
```

### Pipeline Stages

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌──────────┐
│  Lint   │ →  │  Build  │ →  │  Test   │ →  │ Security │
└─────────┘    └─────────┘    └─────────┘    └──────────┘
```

#### Stage 1: Lint

Check code style and quality:

- **rustfmt** - Verify Rust code is formatted
- **clippy** - Rust lint warnings/errors
- **eslint** - TypeScript/React lint
- **prettier** - Frontend code formatting

#### Stage 2: Build

Compile the entire project:

- **cargo build** - Build Rust backend (debug mode for CI speed)
- **pnpm build** - Build frontend
- Build on **3 platforms** via matrix strategy

#### Stage 3: Test

Run all tests:

- **cargo test** - Rust unit + integration tests
- **pnpm test** - Frontend unit + component tests

#### Stage 4: Security

Check for vulnerabilities:

- **cargo audit** - Check Rust dependencies for CVEs
- **npm audit** - Check Node.js dependencies for CVEs

---

## Matrix Builds

Build and test on 3 platforms simultaneously:

```yaml
strategy:
  fail-fast: false
  matrix:
    platform:
      - os: ubuntu-latest
        rust-target: x86_64-unknown-linux-gnu
      - os: macos-latest
        rust-target: aarch64-apple-darwin
      - os: windows-latest
        rust-target: x86_64-pc-windows-msvc
```

- `fail-fast: false` - Continue building on other platforms if one fails, to know the full status.

---

## Caching

Caching reduces build time from ~15 minutes to ~5 minutes:

### Cargo Cache

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/bin/
      ~/.cargo/registry/index/
      ~/.cargo/registry/cache/
      ~/.cargo/git/db/
      src-tauri/target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-
```

### Node Modules Cache

```yaml
- uses: actions/cache@v4
  with:
    path: node_modules
    key: ${{ runner.os }}-node-${{ hashFiles('pnpm-lock.yaml') }}
    restore-keys: |
      ${{ runner.os }}-node-
```

### Cache Effectiveness

| Cache | Time Saved | Notes |
|-------|------------|-------|
| Cargo registry | ~2-3 min | Downloading dependencies |
| Cargo target | ~5-8 min | Incremental compilation |
| node_modules | ~1-2 min | pnpm install |

---

## Development Pipeline

### Trigger

```yaml
on:
  push:
    tags:
      - 'develop-v*'
```

### Behavior

1. **Build** release-optimized binaries on all 3 platforms
2. **Skip signing** - no `TAURI_SIGNING_PRIVATE_KEY` used, no `.sig` files
3. **Create GitHub Release** as **prerelease** (not shown as latest)
4. **Upload artifacts** - unsigned binaries only
5. **Update endpoint** - writes update manifest to `develop` channel

### Why No Signing for Dev Builds

- Faster builds (signing adds time)
- No need to manage key access for dev builds
- Dev builds are for internal testing, not end users
- `.sig` files are only meaningful for the auto-updater's integrity check

---

## Release Pipeline

### Trigger

```yaml
on:
  push:
    tags:
      - 'release-v*'
```

### Behavior

1. **Build** release-optimized binaries on all 3 platforms
2. **Sign all binaries** using `TAURI_SIGNING_PRIVATE_KEY`
3. **Generate `.sig` files** for each platform binary
4. **Create GitHub Release** as **latest release**
5. **Upload artifacts** - signed binaries + `.sig` files
6. **Update endpoint** - writes update manifest to `stable` channel
7. **Generate update JSON** - Tauri update manifest for in-app updater

### Signing Setup

Tauri uses Ed25519 keys for update signing:

```bash
# Generate key pair (one-time setup)
pnpm tauri signer generate -w ~/.tauri/wit-term.key

# This creates:
# ~/.tauri/wit-term.key       - private key (keep secret!)
# ~/.tauri/wit-term.key.pub   - public key (embed in app)
```

Store in GitHub Secrets:

| Secret | Value |
|--------|-------|
| `TAURI_SIGNING_PRIVATE_KEY` | Contents of `wit-term.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for the key (if set) |

Public key goes into `tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6...",
      "endpoints": [
        "https://github.com/<owner>/wit-term/releases/latest/download/latest.json"
      ]
    }
  }
}
```

---

## In-App Update System

Wit supports checking for updates and installing them directly from within the app using Tauri's built-in updater plugin.

### How It Works

```
┌──────────┐    ┌─────────────────┐    ┌──────────────────┐    ┌─────────────┐
│ User     │    │ Wit App         │    │ GitHub Releases  │    │ Download +  │
│ clicks   │ →  │ calls updater   │ →  │ fetch latest.json│ →  │ Install     │
│ "Check   │    │ check_update()  │    │ compare versions │    │ restart app │
│ Update"  │    │                 │    │                  │    │             │
└──────────┘    └─────────────────┘    └──────────────────┘    └─────────────┘
```

### Update Manifest (latest.json)

The release pipeline generates this JSON file and uploads it as a release artifact:

```json
{
  "version": "0.2.0",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2026-06-15T12:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6...",
      "url": "https://github.com/<owner>/wit-term/releases/download/release-v0.2.0/Wit_0.2.0_aarch64.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6...",
      "url": "https://github.com/<owner>/wit-term/releases/download/release-v0.2.0/Wit_0.2.0_x64.app.tar.gz"
    },
    "linux-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6...",
      "url": "https://github.com/<owner>/wit-term/releases/download/release-v0.2.0/Wit_0.2.0_amd64.AppImage.tar.gz"
    },
    "windows-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6...",
      "url": "https://github.com/<owner>/wit-term/releases/download/release-v0.2.0/Wit_0.2.0_x64-setup.nsis.zip"
    }
  }
}
```

### Tauri Configuration

`src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "active": true,
      "dialog": false,
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6...",
      "endpoints": [
        "https://github.com/<owner>/wit-term/releases/latest/download/latest.json"
      ],
      "windows": {
        "installMode": "passive"
      }
    }
  }
}
```

- `dialog: false` - we use a custom UI instead of the default system dialog
- `installMode: "passive"` - on Windows, install silently in the background

### Frontend Implementation

```typescript
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

// Check for updates (called from Help menu > "Check for Updates")
async function checkForUpdates() {
  const update = await check();

  if (update === null) {
    // No update available
    showNotification("You're on the latest version.");
    return;
  }

  // Show update available UI
  const userConfirmed = await showUpdateDialog({
    currentVersion: update.currentVersion,
    newVersion: update.version,
    releaseNotes: update.body,
    date: update.date,
  });

  if (!userConfirmed) return;

  // Download and install
  let downloaded = 0;
  let contentLength = 0;

  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case "Started":
        contentLength = event.data.contentLength ?? 0;
        showProgress(0);
        break;
      case "Progress":
        downloaded += event.data.chunkLength;
        const percent = contentLength > 0
          ? Math.round((downloaded / contentLength) * 100)
          : 0;
        showProgress(percent);
        break;
      case "Finished":
        showProgress(100);
        break;
    }
  });

  // Prompt user to restart
  const shouldRestart = await showRestartDialog();
  if (shouldRestart) {
    await relaunch();
  }
}
```

### Rust Side (Plugin Setup)

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        // ... other plugins
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Cargo Dependencies

```toml
[dependencies]
tauri-plugin-updater = "2"
tauri-plugin-process = "2"
```

### Frontend Dependencies

```json
{
  "dependencies": {
    "@tauri-apps/plugin-updater": "^2.0.0",
    "@tauri-apps/plugin-process": "^2.0.0"
  }
}
```

### Update UI Flow

```
Help > "Check for Updates"
         │
         ▼
    ┌─────────────┐
    │  Checking... │  (spinner)
    └──────┬──────┘
           │
     ┌─────┴─────┐
     │            │
     ▼            ▼
  No update    Update available
  available    ┌──────────────────────┐
  (toast)      │ Wit v0.2.0 available │
               │                      │
               │ What's new:          │
               │ - Bug fixes          │
               │ - Performance        │
               │                      │
               │ [Skip]  [Install]    │
               └──────────┬───────────┘
                          │
                          ▼
               ┌──────────────────────┐
               │ Downloading...       │
               │ ████████░░░░ 67%     │
               └──────────┬───────────┘
                          │
                          ▼
               ┌──────────────────────┐
               │ Update installed!    │
               │ Restart to apply.    │
               │                      │
               │ [Later]  [Restart]   │
               └──────────────────────┘
```

### Update Channels (Future)

For users who want to opt into development builds:

```toml
# ~/.config/wit/config.toml
[updates]
channel = "stable"   # "stable" or "develop"
auto_check = true    # check on startup
check_interval = 86400  # seconds (24 hours)
```

When `channel = "develop"`, the updater fetches from:
```
https://github.com/<owner>/wit-term/releases/download/develop-latest/latest.json
```

When `channel = "stable"` (default), it fetches from:
```
https://github.com/<owner>/wit-term/releases/latest/download/latest.json
```

---

## Workflow Files

### CI Workflow

File: `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Install system dependencies (Linux)
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev libssl-dev \
            libayatana-appindicator3-dev librsvg2-dev \
            libgtk-3-dev libsoup-3.0-dev \
            libjavascriptcoregtk-4.1-dev

      - name: Check Rust formatting
        run: cargo fmt --manifest-path src-tauri/Cargo.toml --check

      - name: Run Clippy
        run: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

      - uses: pnpm/action-setup@v4
        with:
          version: 9

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm

      - run: pnpm install --frozen-lockfile
      - run: pnpm lint
      - run: pnpm format --check

  build-and-test:
    name: Build & Test (${{ matrix.platform.os }})
    needs: lint
    runs-on: ${{ matrix.platform.os }}
    strategy:
      fail-fast: false
      matrix:
        platform:
          - os: ubuntu-latest
          - os: macos-latest
          - os: windows-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install system dependencies (Linux)
        if: matrix.platform.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev libssl-dev \
            libayatana-appindicator3-dev librsvg2-dev \
            libgtk-3-dev libsoup-3.0-dev \
            libjavascriptcoregtk-4.1-dev

      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            src-tauri/target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: ${{ runner.os }}-cargo-

      - uses: pnpm/action-setup@v4
        with:
          version: 9

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm

      - run: pnpm install --frozen-lockfile

      - name: Build frontend
        run: pnpm build

      - name: Run Rust tests
        run: cargo test --manifest-path src-tauri/Cargo.toml

      - name: Run frontend tests
        run: pnpm test

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-audit
        run: cargo install cargo-audit --locked

      - name: Cargo audit
        run: cargo audit --file src-tauri/Cargo.lock

      - uses: pnpm/action-setup@v4
        with:
          version: 9

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm

      - run: pnpm install --frozen-lockfile
      - run: pnpm audit --audit-level=high
```

### Development Build Workflow

File: `.github/workflows/develop.yml`

```yaml
name: Development Build

on:
  push:
    tags:
      - 'develop-v*'

permissions:
  contents: write

jobs:
  develop:
    name: Dev Build (${{ matrix.platform.os }})
    runs-on: ${{ matrix.platform.os }}
    strategy:
      fail-fast: false
      matrix:
        platform:
          - os: ubuntu-latest
          - os: macos-latest
          - os: windows-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install system dependencies (Linux)
        if: matrix.platform.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev libssl-dev \
            libayatana-appindicator3-dev librsvg2-dev \
            libgtk-3-dev libsoup-3.0-dev \
            libjavascriptcoregtk-4.1-dev

      - uses: pnpm/action-setup@v4
        with:
          version: 9

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm

      - run: pnpm install --frozen-lockfile

      # Build WITHOUT signing - no TAURI_SIGNING_PRIVATE_KEY
      - name: Build Tauri app (unsigned)
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "Wit ${{ github.ref_name }} (Dev)"
          releaseBody: |
            **Development build - not for production use.**

            This is an unsigned development build. For stable releases,
            see [latest release](https://github.com/${{ github.repository }}/releases/latest).
          releaseDraft: false
          prerelease: true
```

### Release Workflow

File: `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    tags:
      - 'release-v*'

permissions:
  contents: write

jobs:
  release:
    name: Release (${{ matrix.platform.os }})
    runs-on: ${{ matrix.platform.os }}
    strategy:
      fail-fast: false
      matrix:
        platform:
          - os: ubuntu-latest
          - os: macos-latest
          - os: windows-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install system dependencies (Linux)
        if: matrix.platform.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev libssl-dev \
            libayatana-appindicator3-dev librsvg2-dev \
            libgtk-3-dev libsoup-3.0-dev \
            libjavascriptcoregtk-4.1-dev

      - uses: pnpm/action-setup@v4
        with:
          version: 9

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm

      - run: pnpm install --frozen-lockfile

      # Extract version from tag (release-v0.2.0 -> 0.2.0)
      - name: Extract version
        id: version
        shell: bash
        run: echo "version=${GITHUB_REF_NAME#release-v}" >> "$GITHUB_OUTPUT"

      # Build WITH signing - generates .sig files
      - name: Build Tauri app (signed)
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "Wit v${{ steps.version.outputs.version }}"
          releaseBody: |
            ## Wit v${{ steps.version.outputs.version }}

            See the [CHANGELOG](https://github.com/${{ github.repository }}/blob/main/CHANGELOG.md) for details.

            ### Update from within Wit
            Go to **Help > Check for Updates** to update directly from the app.
          releaseDraft: true
          prerelease: false
          updaterJsonPreferNsis: true
```

### Update Manifest Workflow

File: `.github/workflows/update-manifest.yml`

This workflow runs after the release workflow completes and generates the `latest.json` manifest file that the in-app updater reads.

```yaml
name: Update Manifest

on:
  release:
    types: [published]

permissions:
  contents: write

jobs:
  update-manifest:
    # Only run for release tags, not develop tags
    if: startsWith(github.event.release.tag_name, 'release-v')
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Extract version
        id: version
        run: echo "version=${TAG#release-v}" >> "$GITHUB_OUTPUT"
        env:
          TAG: ${{ github.event.release.tag_name }}

      - name: Download release assets
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          mkdir -p artifacts
          gh release download "${{ github.event.release.tag_name }}" \
            --dir artifacts \
            --pattern "*.sig" \
            --pattern "*.json"

      - name: Generate latest.json
        run: |
          python3 scripts/generate-update-manifest.py \
            --version "${{ steps.version.outputs.version }}" \
            --tag "${{ github.event.release.tag_name }}" \
            --artifacts-dir artifacts \
            --output latest.json \
            --repo "${{ github.repository }}"

      - name: Upload latest.json to release
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          gh release upload "${{ github.event.release.tag_name }}" \
            latest.json --clobber
```

---

## Badge Setup

Add badges to `README.md` to display CI status:

```markdown
[![CI](https://github.com/<owner>/wit-term/actions/workflows/ci.yml/badge.svg)](https://github.com/<owner>/wit-term/actions/workflows/ci.yml)
[![Release](https://github.com/<owner>/wit-term/actions/workflows/release.yml/badge.svg)](https://github.com/<owner>/wit-term/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
```

---

## Quick Reference

### Creating a Dev Build

```bash
# Tag and push
git tag develop-v0.2.0-beta.1
git push origin develop-v0.2.0-beta.1

# Result: unsigned binaries uploaded as GitHub prerelease
```

### Creating a Production Release

```bash
# Tag and push
git tag release-v0.2.0
git push origin release-v0.2.0

# Result:
# 1. Signed binaries + .sig files uploaded
# 2. GitHub Release created (draft - review and publish manually)
# 3. latest.json generated for in-app updater
# 4. Users see "Update available" in Help > Check for Updates
```

### Required GitHub Secrets

| Secret | Required For | How to Generate |
|--------|-------------|-----------------|
| `TAURI_SIGNING_PRIVATE_KEY` | Release pipeline | `pnpm tauri signer generate` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Release pipeline | Set during key generation |

---

> See also: [Testing Strategy](testing-strategy.md) for testing details, [Build and Release](build-and-release.md) for the full release process.
