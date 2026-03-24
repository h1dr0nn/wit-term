# Contributing to Wit

Wit is a context-aware terminal emulator built with Rust, Tauri v2, and React/TypeScript. Contributions of all kinds are welcome — from adding completion rules (no Rust needed) to fixing bugs in the core engine.

## Getting Started

### Prerequisites

- **Rust** 1.77+ (via [rustup](https://rustup.rs))
- **Node.js** 18+
- **pnpm** (package manager)
- **Tauri v2 prerequisites** — platform-specific system dependencies. See the [Tauri v2 prerequisites guide](https://v2.tauri.app/start/prerequisites/)

### Clone and Build

```bash
git clone https://github.com/<your-username>/wit-term.git
cd wit-term
pnpm install
pnpm tauri dev
```

## Ways to Contribute

Ordered roughly by difficulty:

1. **Add completion rules** — write TOML files describing CLI commands and flags. No Rust or TypeScript needed. This is the easiest and most impactful way to contribute.
2. **Report bugs / request features** — open a GitHub Issue with reproduction steps, expected vs actual behavior, and your environment (OS, shell, Wit version).
3. **Improve documentation** — fix typos, clarify explanations, add examples.
4. **Add context providers** — teach Wit to detect new project types (Rust knowledge required).
5. **Fix bugs or add features** — work on the Rust core (`src-tauri/`) or React frontend (`src/`).

## Adding Completion Rules

Completion rules are TOML files that describe a CLI command's subcommands, flags, and arguments. When a user presses Tab, Wit reads these files to provide suggestions.

### Where to put files

```
completions/
├── bundled/       # Shipped with the app
├── community/     # Community-contributed (optional install)
└── user/          # User-defined overrides
```

New contributions go in `completions/bundled/` (for common tools) or `completions/community/`.

### TOML format

```toml
# Completion rules for <command>
# Source: man <command>, <command> --help
# Author: <your-github-username>

wit_completion_version = "1.0"

[command]
name = "curl"
description = "Transfer data from or to a server"

[[command.flags]]
name = "--output"
short = "-o"
description = "Write output to file instead of stdout"
argument = "file"

[[command.flags]]
name = "--verbose"
short = "-v"
description = "Make the operation more talkative"

[[command.flags]]
name = "--request"
short = "-X"
description = "Specify HTTP method to use"
argument = "method"
argument_values = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]
```

### Style rules

- One file per command or command group (e.g., `git.toml` covers all git subcommands)
- Every flag and subcommand must have a `description`
- Sort flags alphabetically by long name
- Include short flags when they exist (e.g., `short = "-v"`)
- Cover common flags first — 30 well-described flags beats 200 without descriptions
- Comment header with source (man page version, docs URL) and author

### How to test

```bash
# Validate TOML syntax with any TOML linter, or:
pnpm tauri dev
# Then type the command and press Tab to see completions
```

You do not need to set up the full dev environment to contribute completion rules. Any text editor and a TOML linter are sufficient.

### Submit

```bash
git checkout -b completions/add-curl
git add completions/bundled/curl.toml
git commit -m "feat(completions): add curl completion rules"
git push origin completions/add-curl
```

Then open a Pull Request noting which command you added, the reference source, and how many flags/subcommands are covered.

## Code Contributions

### Branch naming

- `feature/xyz` — new features
- `fix/xyz` — bug fixes
- `refactor/xyz` — refactoring
- `completions/xyz` — completion rule additions
- `docs/xyz` — documentation changes

### Commit conventions

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add ssh context provider
fix: handle escaped quotes in ANSI parser
refactor: simplify completion ranking algorithm
docs: update architecture diagram
test: add PTY integration tests
chore: update dependencies
perf: optimize terminal grid rendering
```

### Code style

**Rust:**
- `rustfmt` defaults
- `clippy` with `-D warnings` (all warnings are errors)
- `thiserror` for library errors, `anyhow` for application errors

**TypeScript:**
- Strict mode enabled
- Functional components only
- No `any` types

## Pull Request Process

1. Create a branch from `main` following the naming convention above
2. Make your changes
3. Run tests and lints (see Development Commands below)
4. Push your branch and open a Pull Request
5. Provide a clear description, link related issues, include screenshots for UI changes
6. CI must pass — all tests and lints are checked automatically
7. Maintainers will review within 3-5 business days; expect 1-2 rounds of feedback

## Development Commands

```bash
pnpm install              # Install frontend dependencies
pnpm tauri dev            # Run in development mode (hot reload)
pnpm tauri build          # Build production binary
pnpm dev                  # Frontend only (no Rust backend)
pnpm lint                 # ESLint
pnpm format               # Prettier
pnpm test                 # Frontend tests
cargo test --manifest-path src-tauri/Cargo.toml    # Rust tests
cargo clippy --manifest-path src-tauri/Cargo.toml  # Rust lints
```

## License

Wit is licensed under **MPL-2.0**. By submitting a contribution, you agree that your contribution is licensed under the same terms.
