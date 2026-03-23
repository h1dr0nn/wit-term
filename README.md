# Wit

**The terminal that gets it.**

A context-aware terminal emulator built with Rust. Wit detects your project environment - git, Docker, Node, Cargo, Python, and more - and provides intelligent completions without AI, without cloud, without telemetry. Just sharp, local instinct.

## What It Does

Wit reads your project structure and knows what you need:

- In a git repo? Git completions are ready, with actual branch names.
- `package.json` present? npm/yarn/pnpm commands loaded, with real script names.
- `Cargo.toml` found? Cargo completions activated.
- `Dockerfile` exists? Docker commands available, with running container names.

All rule-based, deterministic, and fast. No black box, no cloud calls, no account required.

## Features

- **Context-aware completions** - detects project type and loads relevant command suggestions with dynamic data (git branches, npm scripts, docker containers)
- **Built from scratch** - custom terminal emulator with VT100/xterm ANSI parser, not a wrapper around xterm.js
- **Multi-session** - multiple terminal tabs with sidebar navigation (Ctrl+T/W/Tab)
- **14 command groups** - git, npm, yarn, pnpm, cargo, docker, kubectl, pip, ssh, make, systemctl, brew, apt, and more
- **Theming** - 8 built-in themes (Catppuccin, Dracula, Tokyo Night, Nord, etc.) with hot-reload and CSS custom properties
- **Settings UI** - font, theme, cursor, and scrollback configuration (Ctrl+,)
- **Command palette** - quick access to all commands (Ctrl+Shift+P)
- **Context sidebar** - project info at a glance (git status, Node scripts, etc.)
- **Search** - find text in terminal output (Ctrl+Shift+F)
- **Shell integration** - bash, zsh, fish, PowerShell with CWD tracking
- **Virtual scrolling** - smooth performance with large scrollback
- **Plugin system** - extend with custom completions via TOML plugins
- **Cross-platform** - macOS, Linux, Windows
- **Privacy-first** - zero telemetry, zero data collection, everything local

## Install

### From Releases

Download the latest release for your platform from [GitHub Releases](https://github.com/nickcdryan/wit-term/releases):

| Platform | Format |
|----------|--------|
| macOS | `.dmg` |
| Linux | `.deb`, `.AppImage` |
| Windows | `.msi` |

### Building from Source

#### Prerequisites

- [Rust](https://rustup.rs/) 1.77+ (with rustfmt and clippy)
- [Node.js](https://nodejs.org/) v18+
- [pnpm](https://pnpm.io/)
- Platform-specific [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

#### Build

```bash
git clone https://github.com/nickcdryan/wit-term.git
cd wit-term
pnpm install
pnpm tauri dev        # development mode
pnpm tauri build      # production build
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Tab | Trigger completions |
| Ctrl+T | New session |
| Ctrl+W | Close session |
| Ctrl+Tab | Next session |
| Ctrl+1-9 | Switch to session by index |
| Ctrl+B | Toggle session sidebar |
| Ctrl+Shift+B | Toggle context sidebar |
| Ctrl+Shift+P | Command palette |
| Ctrl+Shift+F | Search in terminal |
| Ctrl+, | Settings |
| Ctrl+Shift+C | Copy |
| Ctrl+Shift+V | Paste |

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Core | Rust |
| App Framework | Tauri v2 |
| Frontend | React + TypeScript |
| Styling | Tailwind CSS |
| State | Zustand |

## Documentation

Full documentation is in the [documents/](documents/) folder:

- [Architecture](documents/02-architecture/system-architecture.md)
- [Specifications](documents/03-specifications/)
- [UI/UX Design](documents/04-ui-ux/)
- [Roadmap](documents/ROADMAP.md)

## Contributing

Contributions are welcome! The easiest way to start is by adding completion rules - no Rust knowledge needed, just TOML files.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full guide.

## License

[MPL-2.0](LICENSE.md)
