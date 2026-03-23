# Development Environment Setup

> **Status:** Active
> **Last Updated:** 2026-03-23
> **Owner:** Wit Team

Guide for setting up the development environment for the Wit terminal emulator.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Platform-Specific Setup](#platform-specific-setup)
- [Clone and Initial Setup](#clone-and-initial-setup)
- [Project Structure Overview](#project-structure-overview)
- [Development Commands](#development-commands)
- [Environment Variables](#environment-variables)
- [IDE Setup](#ide-setup)
- [Troubleshooting](#troubleshooting)
- [First Contribution Walkthrough](#first-contribution-walkthrough)

---

## Prerequisites

Required tools before getting started:

| Tool | Version | Notes |
|------|---------|-------|
| **Rust** (via rustup) | stable latest | `rustup update stable` |
| **Node.js** | v20+ | LTS recommended |
| **pnpm** (preferred) | v8+ | `npm install -g pnpm` - npm can be used but pnpm is faster |
| **Git** | 2.30+ | |

Verify versions:

```bash
rustc --version
node --version
pnpm --version
git --version
```

---

## Platform-Specific Setup

### macOS

```bash
# Xcode Command Line Tools (required)
xcode-select --install

# Homebrew packages
brew install cmake pkg-config
```

Tauri v2 on macOS requires Xcode CLT to build native code. Full Xcode installation is not needed.

### Linux (Ubuntu/Debian)

```bash
# Build essentials and Tauri dependencies
sudo apt update
sudo apt install -y \
  build-essential \
  curl \
  wget \
  file \
  libwebkit2gtk-4.1-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libgtk-3-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev
```

### Linux (Fedora)

```bash
sudo dnf install -y \
  gcc-c++ \
  webkit2gtk4.1-devel \
  openssl-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  gtk3-devel \
  libsoup3-devel \
  javascriptcoregtk4.1-devel
```

### Windows

1. **Visual Studio Build Tools** - Install from [visualstudio.microsoft.com](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - Select workload: "Desktop development with C++"
   - Includes: MSVC compiler, Windows SDK, C++ CMake tools
2. **WebView2** - Usually already present on Windows 10/11. If not, download from [developer.microsoft.com](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
3. **Windows SDK** - Already included in Visual Studio Build Tools

> **Note:** On Windows, ensure rustup uses the `stable-x86_64-pc-windows-msvc` toolchain.

---

## Clone and Initial Setup

```bash
# Clone repository
git clone https://github.com/<your-username>/wit-term.git
cd wit-term

# Install Rust dependencies (automatic during build)
# Install frontend dependencies
pnpm install

# Run to verify setup
pnpm tauri dev
```

The first run will take a few minutes to compile Rust code. Subsequent runs will be faster thanks to incremental compilation.

---

## Project Structure Overview

```
wit-term/
├── src-tauri/           # Rust backend (Tauri app)
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── pty/         # PTY management
│   │   ├── parser/      # ANSI/VT parser
│   │   └── ...
│   ├── Cargo.toml       # Rust dependencies
│   └── tauri.conf.json  # Tauri configuration
├── src/                 # React frontend
│   ├── components/      # React components
│   ├── hooks/           # Custom React hooks
│   ├── stores/          # State management
│   ├── App.tsx          # Root component
│   └── main.tsx         # Entry point
├── public/              # Static assets
├── docs/                # Documentation
├── package.json         # Node.js dependencies
├── vite.config.ts       # Vite configuration
├── tailwind.config.js   # Tailwind CSS configuration
├── tsconfig.json        # TypeScript configuration
└── README.md
```

---

## Development Commands

### Full Application (Tauri + Frontend)

```bash
# Run app in development mode (hot reload for frontend, rebuild for Rust)
pnpm tauri dev

# Build production
pnpm tauri build
```

### Frontend Only

```bash
# Run frontend dev server (without Tauri backend)
pnpm dev

# Build frontend
pnpm build

# Preview production build
pnpm preview
```

### Rust Only

```bash
# Run tests
cd src-tauri
cargo test

# Run clippy lints
cargo clippy -- -W clippy::all

# Format code
cargo fmt

# Check without building
cargo check
```

### Other Commands

```bash
# Lint frontend
pnpm lint

# Format frontend
pnpm format

# Run all tests (frontend + backend)
pnpm test
```

---

## Environment Variables

Configurable environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Rust log level (`debug`, `trace`, `info`, `warn`, `error`) |
| `WIT_DEV` | - | Enable debug features when set to `1` |

Create a `.env` file at the root directory (already in `.gitignore`):

```env
RUST_LOG=debug
WIT_DEV=1
```

---

## IDE Setup

### VS Code (Recommended)

#### Recommended Extensions

Create or update file `.vscode/extensions.json`:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "bradlc.vscode-tailwindcss",
    "tauri-apps.tauri-vscode",
    "dbaeumer.vscode-eslint",
    "esbenp.prettier-vscode",
    "fill-labs.dependi",
    "vadimcn.vscode-lldb"
  ]
}
```

#### Settings

Create or update file `.vscode/settings.json`:

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  },
  "[typescriptreact]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "tailwindCSS.experimental.classRegex": [
    ["cn\\(([^)]*)\\)", "[\"'`]([^\"'`]*)[\"'`]"]
  ],
  "typescript.preferences.importModuleSpecifier": "non-relative"
}
```

### Other IDEs

- **IntelliJ/CLion:** Install the Rust plugin and Tauri plugin
- **Neovim:** Use `rust-analyzer` LSP, `typescript-language-server`, `tailwindcss-language-server`

---

## Troubleshooting

### Rust build fails on Linux

```
error: linker `cc` not found
```

**Fix:** Install build-essential: `sudo apt install build-essential`

### WebKit2GTK not found (Linux)

```
Package webkit2gtk-4.1 was not found
```

**Fix:** Install all Tauri dependencies following the [Platform-Specific Setup](#linux-ubuntudebian) guide.

### `pnpm tauri dev` hangs at build step

**Fix:** The first Rust build takes a long time (5-10 minutes). Check CPU usage - if it's still compiling, just wait.

### Port 1420 is already in use

```
Error: Port 1420 is already in use
```

**Fix:** Kill the process using port 1420 or change the port in `vite.config.ts`.

### Windows: LNK1104 - cannot open file

```
LINK : fatal error LNK1104: cannot open file '*.lib'
```

**Fix:** Ensure Visual Studio Build Tools is installed with the "Desktop development with C++" workload.

### Cargo.lock conflict when merging

**Fix:** Accept one side then run `cargo update` to regenerate.

---

## First Contribution Walkthrough

Step-by-step guide for newcomers:

1. **Fork repository** on GitHub
2. **Clone fork** to your machine:
   ```bash
   git clone https://github.com/<your-username>/wit-term.git
   cd wit-term
   ```
3. **Setup environment** following the guide above
4. **Create a new branch:**
   ```bash
   git checkout -b feature/my-first-change
   ```
5. **Make your changes** - start with issues labeled `good first issue`
6. **Run tests:**
   ```bash
   cargo test
   pnpm test
   ```
7. **Commit following Conventional Commits:**
   ```bash
   git commit -m "feat: add support for xyz"
   ```
8. **Push and create Pull Request:**
   ```bash
   git push origin feature/my-first-change
   ```
9. **Open a PR on GitHub** - fill in the full description, link to related issue
10. **Wait for review** - respond to feedback, update code if needed

> See also: [Git Workflow](git-workflow.md) and [Coding Standards](coding-standards.md)
