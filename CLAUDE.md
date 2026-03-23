# CLAUDE.md

## Project

Wit is a context-aware terminal emulator built with Rust, Tauri v2, and React/TypeScript. It detects project environments (git, Docker, Node, Cargo, Python) and provides intelligent completions without AI, cloud, or telemetry.

## Repository Structure

```
wit-term/
├── src-tauri/          # Rust core (PTY, terminal emulation, ANSI parser, context engine, completion engine)
├── src/                # React frontend (terminal view, sidebars, completion popup)
├── documents/          # Full project documentation (architecture, specs, roadmap, research)
├── completions/        # Completion data files (TOML)
├── themes/             # Theme files (TOML)
├── README.md
├── CLAUDE.md           # This file
└── LICENSE.md          # MPL-2.0
```

## Tech Stack

- **Rust** - core engine (PTY, ANSI parser, context detection, completion matching)
- **Tauri v2** - app framework, IPC bridge between Rust and frontend
- **React + TypeScript** - UI rendering
- **Tailwind CSS** - styling
- **Zustand** - state management
- **Vite** - frontend build tool

## Key Commands

```bash
pnpm install              # Install dependencies
pnpm tauri dev            # Run in development mode (hot reload)
pnpm tauri build          # Build production binary
pnpm dev                  # Frontend only (no Rust)
pnpm lint                 # ESLint
pnpm format               # Prettier
cargo test --manifest-path src-tauri/Cargo.toml   # Rust tests
cargo clippy --manifest-path src-tauri/Cargo.toml # Rust lints
pnpm test                 # Frontend tests
```

## Architecture

Three-layer architecture:

1. **Rust Core** - all heavy logic: PTY management, terminal emulation (VT100/xterm), ANSI parsing, context detection, completion matching. Runs on dedicated threads per session.
2. **Tauri Bridge** - IPC via commands (request-response) and events (push). Frontend calls `invoke()`, Rust emits events.
3. **React Frontend** - thin rendering layer. Terminal grid (DOM per row, virtualized), sidebars, completion popup.

## Conventions

- **Rust**: rustfmt defaults, clippy with `-D warnings`, `thiserror` for library errors, `anyhow` for app errors
- **TypeScript**: strict mode, functional components only, no `any`
- **Commits**: Conventional Commits (feat:, fix:, refactor:, docs:, test:, chore:, perf:)
- **Branches**: feature/xyz, fix/xyz, refactor/xyz
- **Completion data**: TOML format, one file per command group

## Documentation

All documentation is in `documents/` with 8 sections. See `documents/README.md` for the full index. Key docs:

- Architecture: `documents/02-architecture/system-architecture.md`
- ANSI parser spec: `documents/03-specifications/ansi-parser.md`
- Context engine spec: `documents/03-specifications/context-engine.md`
- Completion data format: `documents/03-specifications/completion-data-format.md`
- CI/CD (develop-v*/release-v* tags): `documents/06-development/ci-cd.md`

## CI/CD Tag Strategy

- `develop-v*` tags: build without signing, GitHub prerelease
- `release-v*` tags: build with signing (.sig files), GitHub release, in-app update manifest

## License

MPL-2.0
