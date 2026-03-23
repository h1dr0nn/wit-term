# Wit

**The terminal that gets it.**

A context-aware terminal emulator built with Rust. Wit detects your project environment - git, Docker, Node, Cargo, Python, and more - and provides intelligent completions without AI, without cloud, without telemetry. Just sharp, local instinct.

## What It Does

Wit reads your project structure and knows what you need:

- In a git repo? Git completions are ready.
- `package.json` present? npm/yarn/pnpm commands loaded.
- `Cargo.toml` found? Cargo completions activated.
- `Dockerfile` exists? Docker commands available.

All rule-based, deterministic, and fast. No black box, no cloud calls, no account required.

## Features

- **Context-aware completions** - detects project type and loads relevant command suggestions
- **Built from scratch** - custom terminal renderer, not a wrapper around xterm.js
- **Multi-session** - multiple terminal tabs in one window
- **Custom window chrome** - transparent title bar with blur effect
- **Theming** - built-in themes + custom theme support
- **Shell integration** - bash, zsh, fish, PowerShell
- **Cross-platform** - macOS, Linux, Windows
- **Privacy-first** - zero telemetry, zero data collection, everything local

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Core | Rust |
| App Framework | Tauri v2 |
| Frontend | React + TypeScript |
| Styling | Tailwind CSS |

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) v20+
- [pnpm](https://pnpm.io/)
- Platform-specific dependencies (see [setup guide](documents/06-development/setup.md))

### Build

```bash
# Clone
git clone https://github.com/h1dr0n/wit-term.git
cd wit-term

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

## Documentation

Full documentation is in the [documents/](documents/) folder:

- [Project Overview](documents/01-project/overview.md)
- [Architecture](documents/02-architecture/system-architecture.md)
- [Specifications](documents/03-specifications/)
- [UI/UX Design](documents/04-ui-ux/)
- [Roadmap](documents/05-roadmap/roadmap.md)
- [Development Guide](documents/06-development/setup.md)
- [Research](documents/07-research/)
- [Contributing](documents/08-community/contributing.md)

## Contributing

Contributions are welcome. The easiest way to start is by adding completion rules - no Rust knowledge needed, just TOML files. See the [completion contribution guide](documents/08-community/completion-contribution.md).

For code contributions, see [contributing guide](documents/08-community/contributing.md).

## License

[MPL-2.0](LICENSE.md)

## Author

**h1dr0n**
