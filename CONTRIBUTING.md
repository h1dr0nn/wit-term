# Contributing to Wit Terminal

Thank you for your interest in contributing to Wit! This guide will help you get started.

## Getting Started

### Prerequisites

- **Rust** 1.77+ (with `rustfmt` and `clippy`)
- **Node.js** 18+ and **pnpm**
- **Tauri v2** prerequisites (see [Tauri docs](https://v2.tauri.app/start/prerequisites/))

### Setup

```bash
git clone https://github.com/nickcdryan/wit-term.git
cd wit-term
pnpm install
pnpm tauri dev
```

### Running Tests

```bash
# Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Rust lints
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Frontend lints
pnpm lint

# Frontend formatting
pnpm format
```

## Contributing Completions

The easiest way to contribute is by adding new command completion files. Completions are TOML files in the `completions/` directory.

### Step 1: Create a TOML file

Create `completions/<command>.toml`:

```toml
[command]
name = "mycommand"
description = "Description of the command"

[[command.flags]]
name = "--help"
short = "-h"
description = "Show help information"

[[command.subcommands]]
name = "subcommand"
description = "What this subcommand does"
aliases = ["sc"]

[[command.subcommands.flags]]
name = "--verbose"
short = "-v"
description = "Enable verbose output"
```

### Step 2: Validate your file

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Ensure the TOML parses correctly and existing tests still pass.

### Step 3: Submit a PR

1. Fork the repo
2. Create a branch: `git checkout -b feature/completion-mycommand`
3. Add your file
4. Run tests
5. Submit a Pull Request

### Completion Data Format

See `documents/03-specifications/completion-data-format.md` for the full specification.

Key rules:
- One TOML file per command
- Required fields: `command.name`, `command.description`
- Flags need `name` and `description`; `short` is optional
- Subcommands need `name` and `description`; `aliases` is optional
- Subcommands can have their own `flags`

## Contributing Themes

Themes are TOML files in the `themes/` directory.

### Theme Format

```toml
[theme]
name = "My Theme"
author = "Your Name"

[theme.colors]
foreground = "#ffffff"
background = "#000000"
cursor = "#ffffff"
selection_bg = "#444444"
selection_fg = "#ffffff"

# Standard 16 ANSI colors
black = "#000000"
red = "#cc0000"
green = "#00cc00"
yellow = "#cccc00"
blue = "#0000cc"
magenta = "#cc00cc"
cyan = "#00cccc"
white = "#cccccc"

bright_black = "#555555"
bright_red = "#ff5555"
bright_green = "#55ff55"
bright_yellow = "#ffff55"
bright_blue = "#5555ff"
bright_magenta = "#ff55ff"
bright_cyan = "#55ffff"
bright_white = "#ffffff"
```

## Code Contributions

### Architecture

- **Rust** (`src-tauri/src/`): PTY, terminal emulation, ANSI parsing, context engine, completion engine, plugin system
- **React/TypeScript** (`src/`): UI rendering, stores, hooks
- **Bridge**: Tauri IPC commands and events

### Style Guide

- **Rust**: `rustfmt` defaults, `clippy` with `-D warnings`
- **TypeScript**: strict mode, functional components, no `any`
- **Commits**: [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `refactor:`, etc.)

### PR Guidelines

1. Keep PRs focused — one feature or fix per PR
2. Include tests for new Rust code
3. Run `cargo clippy` and `pnpm lint` before submitting
4. Update documentation if behavior changes
5. Write clear commit messages

## Reporting Issues

- Use the [GitHub Issues](https://github.com/nickcdryan/wit-term/issues) page
- Include your OS, shell, and Wit version
- Provide steps to reproduce

## License

By contributing, you agree that your contributions will be licensed under MPL-2.0.
