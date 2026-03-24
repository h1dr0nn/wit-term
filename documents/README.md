# Wit - Documentation

> **"The terminal that gets it."**

Technical and design documentation for the Wit project - a context-aware terminal emulator
built with Rust, Tauri, and React.

---

## Table of Contents

### 01 - Project

Project overview, vision, branding, and glossary.

| Document | Description |
|---|---|
| [overview.md](01-project/overview.md) | Project overview, origin, direction |
| [vision-and-goals.md](01-project/vision-and-goals.md) | Vision, goals, non-goals, design principles |
| [branding.md](01-project/branding.md) | Name, tagline, color palette, icon, typography |
| [glossary.md](01-project/glossary.md) | Technical terms used in the project |

### 02 - Architecture

System architecture and overall technical design.

| Document | Description |
|---|---|
| [system-architecture.md](02-architecture/system-architecture.md) | Overall architecture, layer diagram, component map |
| [rust-core.md](02-architecture/rust-core.md) | Rust core engine - PTY, parser, context, completion |
| [frontend-architecture.md](02-architecture/frontend-architecture.md) | React/TypeScript frontend - components, state, rendering |
| [tauri-bridge.md](02-architecture/tauri-bridge.md) | Tauri IPC layer - commands, events, protocol |
| [data-flow.md](02-architecture/data-flow.md) | End-to-end data flow in the system |

### 03 - Specifications

Detailed technical specifications for each subsystem.

| Document | Description |
|---|---|
| [terminal-emulator.md](03-specifications/terminal-emulator.md) | Terminal emulation - VT100/xterm compatibility |
| [pty-handling.md](03-specifications/pty-handling.md) | Pseudo-terminal management per OS |
| [ansi-parser.md](03-specifications/ansi-parser.md) | ANSI/VT escape sequence parser |
| [context-engine.md](03-specifications/context-engine.md) | Context detection engine - project type, environment |
| [completion-engine.md](03-specifications/completion-engine.md) | Completion system - matching, ranking, display |
| [completion-data-format.md](03-specifications/completion-data-format.md) | Schema and format for completion data |
| [session-management.md](03-specifications/session-management.md) | Multi-session management |
| [shell-integration.md](03-specifications/shell-integration.md) | Shell integration - bash, zsh, fish, PowerShell |
| [agent-detection.md](03-specifications/agent-detection.md) | Agent detection engine - 4-layer progressive detection |
| [agent-adapters.md](03-specifications/agent-adapters.md) | Agent adapter system - per-agent output parsing |
| [wit-protocol.md](03-specifications/wit-protocol.md) | Wit Protocol - two-way agent communication |

### 04 - UI/UX

User interface and user experience design.

| Document | Description |
|---|---|
| [design-system.md](04-ui-ux/design-system.md) | Design tokens, spacing, typography, components |
| [terminal-view.md](04-ui-ux/terminal-view.md) | Terminal rendering view - grid, cursor, selection |
| [sidebar-left.md](04-ui-ux/sidebar-left.md) | Session management sidebar |
| [sidebar-right.md](04-ui-ux/sidebar-right.md) | Agent sidebar - AI agent monitoring dashboard |
| [agent-dashboard.md](04-ui-ux/agent-dashboard.md) | Agent dashboard components - timeline, files, conversation |
| [completion-popup.md](04-ui-ux/completion-popup.md) | Autocomplete dropdown UI |
| [header.md](04-ui-ux/header.md) | Header / title bar layout and behavior |
| [window-decoration.md](04-ui-ux/window-decoration.md) | Custom window chrome, menus, transparency, controls |
| [themes.md](04-ui-ux/themes.md) | Theming system and default themes |
| [keyboard-shortcuts.md](04-ui-ux/keyboard-shortcuts.md) | Keybindings and customization |

### 05 - Roadmap

Development roadmap, phases, and milestones.

| Document | Description |
|---|---|
| [roadmap.md](05-roadmap/roadmap.md) | Overall 12-month roadmap |
| [phase-1-foundation.md](05-roadmap/phase-1-foundation.md) | Phase 1: Foundation - PTY, renderer, basic I/O |
| [phase-2-context.md](05-roadmap/phase-2-context.md) | Phase 2: Context - detection engine, completions |
| [phase-3-polish.md](05-roadmap/phase-3-polish.md) | Phase 3: Polish - UI, themes, sessions, UX |
| [phase-4-ecosystem.md](05-roadmap/phase-4-ecosystem.md) | Phase 4: Ecosystem - plugins, community, packaging |
| [milestones.md](05-roadmap/milestones.md) | Milestone definitions and acceptance criteria |
| [agent-detection-milestones.md](05-roadmap/agent-detection-milestones.md) | Agent detection feature milestones |

### 06 - Development

Development process, code standards, and tooling.

| Document | Description |
|---|---|
| [setup.md](06-development/setup.md) | Development environment setup |
| [coding-standards.md](06-development/coding-standards.md) | Code style, conventions, naming |
| [git-workflow.md](06-development/git-workflow.md) | Git branching model, commit conventions |
| [testing-strategy.md](06-development/testing-strategy.md) | Testing approach - unit, integration, e2e |
| [ci-cd.md](06-development/ci-cd.md) | CI/CD pipeline design |
| [build-and-release.md](06-development/build-and-release.md) | Build process and release workflow |

### 07 - Research

Technical research and references.

| Document | Description |
|---|---|
| [terminal-emulation.md](07-research/terminal-emulation.md) | Research on terminal emulation standards |
| [ansi-escape-codes.md](07-research/ansi-escape-codes.md) | Comprehensive ANSI escape codes reference |
| [pty-internals.md](07-research/pty-internals.md) | PTY internals on Unix and Windows |
| [competitor-analysis.md](07-research/competitor-analysis.md) | Analysis of Warp, Kitty, Alacritty, WezTerm |
| [rust-crates.md](07-research/rust-crates.md) | Useful Rust crates for the project |

### 08 - Community

Contribution guidelines and extensibility.

| Document | Description |
|---|---|
| [contributing.md](08-community/contributing.md) | Project contribution guide |
| [plugin-system.md](08-community/plugin-system.md) | Plugin system architecture |
| [completion-contribution.md](08-community/completion-contribution.md) | How to contribute completion rules |

---

## Documentation Conventions

- Documentation is written in **Markdown** in English
- Technical terms are kept in English
- Each document has a header with metadata: status, last updated, owner
- Diagrams use ASCII art or Mermaid syntax
- Code examples use syntax highlighting

## Document Status

| Status | Meaning |
|---|---|
| `draft` | Being written, not yet complete |
| `review` | Writing complete, needs review |
| `approved` | Reviewed and approved |
| `living` | Living document, continuously updated |
