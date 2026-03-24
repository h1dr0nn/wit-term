# Wit - Agent-Aware Terminal

**The terminal that gets it.**

A context-aware, agent-aware terminal emulator built with Rust. Wit detects your project environment - git, Docker, Node, Cargo, Python - and provides intelligent completions without AI, without cloud, without telemetry. When you run an AI coding agent, Wit detects it automatically and opens a live dashboard so you can see exactly what it's doing.

## What It Does

### Context-Aware Completions

Wit reads your project structure and knows what you need:

- In a git repo? Git completions are ready, with actual branch names.
- `package.json` present? npm/yarn/pnpm commands loaded, with real script names.
- `Cargo.toml` found? Cargo completions activated.
- `Dockerfile` exists? Docker commands available, with running container names.

All rule-based, deterministic, and fast. No black box, no cloud calls, no account required.

### Agent-Aware Dashboard

AI coding agents (Claude Code, Aider, Codex CLI, Copilot CLI) run in terminals - but terminals don't understand them. You end up switching between tabs, editors, and git just to track what the agent is doing.

Wit changes this. When an agent starts, Wit auto-detects it and opens a **sidebar dashboard** showing:

- **Token & cost tracking** - real-time usage counter so you know what you're spending
- **File change monitoring** - which files the agent creates, modifies, or deletes, with inline diffs
- **Activity timeline** - structured view of agent actions (thinking, tool use, file edits, errors)
- **Conversation view** - readable agent-user exchanges, not raw terminal noise

**No agent modification required.** Wit works with existing agent CLIs through passive observation:

| Layer | How it works | Needs agent change? |
|-------|-------------|---------------------|
| Process Detection | Monitors PTY child process tree | No |
| Output Parsing | Per-agent adapters parse terminal output | No |
| Filesystem Watching | Tracks file changes with git baseline diff | No |
| Wit Protocol | Two-way structured IPC (future) | Yes (opt-in) |

Layers 1–3 work out of the box with any agent. Layer 4 is an optional protocol for deeper integration.

## Features

**Terminal Core**
- **Built from scratch** - custom terminal emulator with VT100/xterm ANSI parser, not a wrapper around xterm.js
- **Command blocks** - Warp-style block rendering with captured output per command
- **ANSI color output** - full color rendering in command blocks (16, 256, and RGB colors)
- **Wide character support** - proper Unicode width handling for CJK characters
- **Multi-session** - multiple terminal tabs with sidebar navigation (Ctrl+T/W/Tab)
- **Virtual scrolling** - smooth performance with large scrollback
- **Cross-platform** - macOS, Linux, Windows

**Context & Completions**
- **Context-aware completions** - detects project type and loads relevant command suggestions with dynamic data (git branches, npm scripts, docker containers)
- **14 command groups** - git, npm, yarn, pnpm, cargo, docker, kubectl, pip, ssh, make, systemctl, brew, apt, and more
- **Ghost text completions** - inline suggestion hints with Tab to accept
- **Runtime detection** - shows Node.js, Python, Rust versions in the input bar
- **Plugin system** - extend with custom completions via TOML plugins

**Agent Dashboard**
- **Auto-detection** - recognizes Claude Code, Aider, Codex CLI, Copilot CLI when they start
- **Token & cost counter** - real-time tracking with color-coded thresholds
- **Activity timeline** - thinking, tool use, file edits, errors as structured entries
- **File change tracker** - files modified by the agent with expandable inline diffs and undo
- **Conversation view** - clean display of agent-user exchanges

**UI & Customization**
- **Theming** - 8 built-in themes (Catppuccin, Dracula, Tokyo Night, Nord, etc.) with hot-reload
- **Settings UI** - font, theme, cursor, and scrollback configuration (Ctrl+,)
- **Command palette** - quick access to all commands (Ctrl+Shift+P)
- **Context sidebar** - project info at a glance (git status, Node scripts, etc.)
- **Search** - find text in terminal output (Ctrl+Shift+F)
- **Shell integration** - bash, zsh, fish, PowerShell with CWD tracking
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
| Tab | Accept ghost text / trigger completions |
| Ctrl+T | New session |
| Ctrl+W | Close session |
| Ctrl+Tab | Next session |
| Ctrl+1-9 | Switch to session by index |
| Ctrl+B | Toggle session sidebar |
| Ctrl+Shift+B | Toggle context sidebar |
| Ctrl+Shift+A | Toggle agent sidebar |
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
- [Agent Detection](documents/03-specifications/agent-detection.md)
- [Agent Adapters](documents/03-specifications/agent-adapters.md)
- [Wit Protocol](documents/03-specifications/wit-protocol.md)
- [UI/UX Design](documents/04-ui-ux/)
- [Roadmap](documents/05-roadmap/roadmap.md)

## Contributing

Contributions are welcome! The easiest way to start is by adding completion rules - no Rust knowledge needed, just TOML files.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full guide.

## License

[MPL-2.0](LICENSE.md)
