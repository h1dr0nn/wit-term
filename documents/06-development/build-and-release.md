# Build and Release

> **Status:** Active
> **Last Updated:** 2026-03-23
> **Owner:** Wit Team

Build and release process for the Wit terminal emulator. Includes development builds, production builds, platform-specific packaging, and the release checklist.

---

## Table of Contents

- [Development Build](#development-build)
- [Production Build](#production-build)
- [Build Output](#build-output)
- [Platform-Specific Builds](#platform-specific-builds)
- [Tauri Build Configuration](#tauri-build-configuration)
- [Cross-Compilation](#cross-compilation)
- [Version Management](#version-management)
- [Release Process Checklist](#release-process-checklist)
- [Auto-Update Mechanism](#auto-update-mechanism)

---

## Development Build

```bash
pnpm tauri dev
```

**Characteristics:**

- **Hot reload** for frontend - React/TypeScript/CSS changes are applied immediately
- **Automatic rebuild** for Rust - Rust code changes trigger recompilation (takes a few seconds)
- **Debug symbols** included - can debug with LLDB/GDB/VS Code
- **Rust debug assertions** enabled - panics on logic errors
- **Faster than** production build - no optimization, incremental compilation

### Frontend-only development

```bash
pnpm dev
```

Runs the Vite dev server without the Tauri backend. Useful when working only on UI/styling.

---

## Production Build

```bash
pnpm tauri build
```

**Characteristics:**

- **Optimized** - Rust built with `--release` (LTO, strip symbols)
- **Minified** - Frontend assets minified by Vite
- **Bundled** - Creates platform-specific installers and packages
- **Build time**: 5-15 minutes depending on platform and machine configuration

---

## Build Output

After building, output is located in:

```
src-tauri/target/release/
├── wit-term              # Binary (Linux/macOS)
├── wit-term.exe          # Binary (Windows)
└── bundle/
    ├── appimage/         # Linux AppImage
    │   └── wit-term_0.1.0_amd64.AppImage
    ├── deb/              # Linux .deb package
    │   └── wit-term_0.1.0_amd64.deb
    ├── dmg/              # macOS disk image
    │   └── Wit.dmg
    ├── macos/            # macOS app bundle
    │   └── Wit.app
    ├── msi/              # Windows installer
    │   └── Wit_0.1.0_x64_en-US.msi
    └── nsis/             # Windows NSIS installer
        └── Wit_0.1.0_x64-setup.exe
```

**Each bundle includes:**

- Compiled Rust binary
- Bundled frontend assets (HTML, JS, CSS)
- Platform-specific runtime (WebView2 on Windows, WebKit on Linux/macOS)
- App icon and metadata

---

## Platform-Specific Builds

### macOS

| Format | File | Description |
|--------|------|-------------|
| `.app` | `Wit.app` | Application bundle - drag to Applications |
| `.dmg` | `Wit.dmg` | Disk image installer - contains `.app` with Applications symlink |

**Code Signing (Optional):**

```bash
# Requires Apple Developer certificate
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
pnpm tauri build
```

**Notarization (Optional):**

```bash
export APPLE_ID="your@email.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="TEAM_ID"
pnpm tauri build
```

> Code signing and notarization are required so users do not see the "unidentified developer" warning. Can be skipped during development.

### Linux

| Format | File | Description |
|--------|------|-------------|
| `.AppImage` | `wit-term_0.1.0_amd64.AppImage` | Portable - runs directly, no installation needed |
| `.deb` | `wit-term_0.1.0_amd64.deb` | Debian/Ubuntu package - `sudo dpkg -i` |
| Binary tarball | - | Create manually if needed |

**Linux notes:**

- AppImage is the most common format, runs on most distros.
- `.deb` is only for Debian-based distros (Ubuntu, Pop!_OS, etc.).
- (Future) RPM package for Fedora/RHEL.

### Windows

| Format | File | Description |
|--------|------|-------------|
| `.msi` | `Wit_0.1.0_x64_en-US.msi` | Windows Installer - standard install/uninstall |
| `.exe` (NSIS) | `Wit_0.1.0_x64-setup.exe` | NSIS installer - more customizable |

**Code Signing (Optional):**

```bash
# Requires code signing certificate (.pfx)
export TAURI_SIGNING_PRIVATE_KEY="path/to/cert.pfx"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="password"
pnpm tauri build
```

> Code signing helps avoid the Windows SmartScreen warning. A self-signed cert can be used for testing.

---

## Tauri Build Configuration

File `src-tauri/tauri.conf.json` contains the main build configuration:

```jsonc
{
  "productName": "Wit",
  "identifier": "com.wit-term.app",
  "version": "0.1.0",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build"
  },
  "app": {
    "windows": [
      {
        "title": "Wit",
        "width": 900,
        "height": 600,
        "minWidth": 400,
        "minHeight": 300,
        "decorations": true,
        "resizable": true
      }
    ]
  },
  "bundle": {
    "active": true,
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "targets": "all",
    "macOS": {
      "minimumSystemVersion": "10.15"
    },
    "windows": {
      "webviewInstallMode": {
        "type": "downloadBootstrapper"
      }
    },
    "linux": {
      "desktopTemplate": null
    }
  }
}
```

**Important fields:**

- `productName` - Display name of the app
- `identifier` - Unique app identifier (reverse domain)
- `version` - App version (sync with Cargo.toml and package.json)
- `bundle.targets` - `"all"` to build all formats, or specify explicitly: `["dmg", "appimage", "msi"]`

---

## Cross-Compilation

### From CI (Recommended)

- Cross-compilation is handled automatically via **GitHub Actions matrix builds**.
- Each platform builds on its corresponding runner (macOS runner for .dmg, Ubuntu for .AppImage, Windows for .msi).
- **No need** to set up cross-compilation on the local machine.

### From local machine (Not required)

- Cross-compiling from macOS/Linux to Windows (or vice versa) is **difficult and not recommended**.
- If testing on another platform is needed, use a VM or CI.
- Tauri has basic cross-compilation support but requires installing the toolchain and dependencies of the target platform.

---

## Version Management

Version is stored in **3 places** - they must be kept in sync:

| File | Field | Example |
|------|-------|---------|
| `src-tauri/Cargo.toml` | `version` | `version = "0.1.0"` |
| `package.json` | `version` | `"version": "0.1.0"` |
| `src-tauri/tauri.conf.json` | `version` | `"version": "0.1.0"` |

### Version sync script (optional)

Create a script to update all at once:

```bash
#!/bin/bash
# scripts/bump-version.sh
VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/bump-version.sh 0.2.0"
  exit 1
fi

# Update Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml

# Update package.json
pnpm version $VERSION --no-git-tag-version

# Update tauri.conf.json
jq ".version = \"$VERSION\"" src-tauri/tauri.conf.json > tmp.json && mv tmp.json src-tauri/tauri.conf.json

echo "Version updated to $VERSION"
```

---

## Release Process Checklist

End-to-end release process:

### 1. Prepare release

- [ ] All features/fixes for the release are merged into `main`
- [ ] CI passes on `main`
- [ ] Manual testing on at least 1 platform

### 2. Update version

```bash
# Update version numbers
./scripts/bump-version.sh 0.2.0

# Update Cargo.lock
cd src-tauri && cargo check && cd ..
```

### 3. Update CHANGELOG

```markdown
## [0.2.0] - 2026-03-23

### Added
- Tab completion with context awareness (#42)
- Split pane support (#38)

### Fixed
- Cursor position incorrect after resize (#45)
- Unicode rendering issue with CJK characters (#41)

### Changed
- Improved parser performance by 40% (#43)
```

### 4. Commit and tag

```bash
git add -A
git commit -m "chore: release v0.2.0"
git tag -a v0.2.0 -m "Release v0.2.0"
```

### 5. Push (trigger CI release)

```bash
git push origin main
git push origin v0.2.0
```

### 6. Verify release

- [ ] GitHub Actions release workflow runs successfully
- [ ] Release draft is created on GitHub Releases
- [ ] Binaries for all 3 platforms are uploaded
- [ ] Download and test binary on at least 1 platform

### 7. Publish release

- [ ] Edit release draft - add detailed release notes
- [ ] Uncheck "Draft" - publish release
- [ ] Announce release (if there is a community)

---

## Auto-Update Mechanism

Tauri has a built-in updater for automatic app updates.

### How it works

1. App checks the update URL periodically (or on startup)
2. If a new version is available, downloads the update package
3. Verifies the signature of the update package
4. Applies the update and restarts the app

### Configuration

In `tauri.conf.json`:

```jsonc
{
  "plugins": {
    "updater": {
      "active": true,
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY_HERE",
      "endpoints": [
        "https://github.com/<owner>/wit-term/releases/latest/download/latest.json"
      ]
    }
  }
}
```

### Signing

Update packages must be signed for security:

```bash
# Generate key pair (one time)
pnpm tauri signer generate -w ~/.tauri/wit-term.key

# Set environment variables when building release
export TAURI_SIGNING_PRIVATE_KEY=$(cat ~/.tauri/wit-term.key)
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="your-password"
```

- **Private key**: Keep secret, only use in CI (store in GitHub Secrets)
- **Public key**: Place in `tauri.conf.json` (pubkey field)

### Update Flow

```
App starts
    |
    v
Check update endpoint
    |
    v
New version available? --No--> Continue normally
    |
    Yes
    v
Show dialog "Update available"
    |
    v
User selects "Update" --No--> Remind later
    |
    Yes
    v
Download + verify + install
    |
    v
Restart app with new version
```

> **Note:** Auto-update will be implemented after the app has its first stable release. During the alpha/beta stage, users will manually download new versions from GitHub Releases.

---

> See also: [CI/CD Pipeline](ci-cd.md) for CI configuration for release builds.
