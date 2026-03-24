# Project Overview

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## What is Wit

Wit is a **context-aware terminal emulator** built from the ground up with Rust,
Tauri, and React. Wit detects the surrounding project environment - git, Docker, Node,
Cargo, Python - and provides intelligent command suggestions **without AI, without
cloud, without telemetry**.

### One-line description

> "A terminal that understands project context - knows you're in a git repo, using Docker,
> running Node, and suggests the right commands without AI, without cloud."

---

## Origin

The project started from the idea of building a terminal emulator inspired by
[Warp](https://www.warp.dev/), but with a different philosophy:

- **No AI integration** - completions are rule-based, deterministic, understandable
  and customizable
- **No login required** - a terminal is a personal tool, no account needed
- **No telemetry** - zero data collection, everything stays local
- **Open-source** - transparent, community-driven

### Why not use an existing terminal

| Terminal | Issue for Wit's use case |
|---|---|
| Warp | Requires login, AI-centric, closed-source core, telemetry |
| iTerm2 + zsh plugins | Good completions but requires configuring each plugin separately |
| Fig / Amazon Q CLI | Tried a similar idea, had to sell out to AWS |
| Kitty / Alacritty | Focused on performance, no context-awareness |
| Windows Terminal | Basic multiplexer, no smart completions |

### Core differentiator

Wit does not try to be "Warp but less" - it takes a completely different approach:

```
Warp:      AI + Cloud + Account -> Smart terminal
Wit:       Rules + Local + Open -> Context-aware terminal
```

Wit believes a terminal **does not need AI** to be smart. It only needs to:
1. Read the project structure (config files, directory layout)
2. Understand the current context (git branch, running processes, environment)
3. Apply the appropriate completion rules

Everything is deterministic, fast, and under user control.

### Agent-Aware Terminal — Killer Feature

Beyond context-aware completions, Wit introduces a new category: the **agent-aware terminal**. AI agent CLIs (Claude Code, Aider, Codex CLI, Copilot CLI) are exploding in usage, but they all run in "dumb" terminals. Developers must open additional tabs, editors, and tools to track what an agent is doing — token cost, which files are being modified, diffs, git status.

Wit solves this by auto-detecting agent processes and opening a **right sidebar dashboard** with:

- **Process detection** — automatically knows when an AI agent is running
- **Token/cost tracking** — realtime usage and cost counter
- **File change monitoring** — tracks which files the agent modifies with inline diffs
- **Activity timeline** — structured view of agent actions (thinking, editing, tool use)
- **Conversation view** — readable format of agent conversation (not raw terminal output)

The detection works through 4 progressive layers:

| Layer | Mechanism | Requires agent change |
|---|---|---|
| L1: Process Detection | Monitor PTY process tree | No |
| L2: Output Parsing | Parse agent stdout patterns | No |
| L3: Filesystem Watching | Track file changes via notify | No |
| L4: Wit Protocol | Two-way socket communication | Yes (optional) |

Layers 1-3 are fully passive — they work with any existing agent CLI without modification. Layer 4 is a future protocol for rich two-way integration.

**See:** [Agent Detection Spec](03-specifications/agent-detection.md) · [Agent Sidebar UI](04-ui-ux/sidebar-right.md) · [Wit Protocol](03-specifications/wit-protocol.md)

---

## Project Assessment

### Initial assessment (6 criteria, scale 0-10)

| Criterion | Score | Notes |
|---|---|---|
| Feasibility | 3/10 | Terminal emulator is extremely complex - PTY, ANSI, shell integration |
| Market Demand | 6/10 | Every developer uses a terminal, but many are already satisfied |
| Uniqueness | 3/10 | Warp, Kitty, Alacritty, iTerm2 are all strong competitors |
| Monetization | 4/10 | Terminal apps are very hard to monetize, users expect free |
| Tech Fit | 8/10 | Tauri + Rust + React is best-fit |
| Wow Factor | 4/10 | "Terminal with good autocomplete" is not enough to go viral |
| **Total** | **28/60** | Danger zone if measured by startup metrics |

### Re-assessment (excluding Feasibility & Monetization)

After pivoting toward a **passion / portfolio / learning** direction:

| Criterion | Score | Notes |
|---|---|---|
| Market Demand | 6/10 | Open-source will attract the developer community |
| Uniqueness | 5/10 | "Warp but free + OSS + no AI" is a reasonable value prop |
| Tech Fit | 8/10 | Excellent project for learning Rust |
| Wow Factor | 5/10 | Self-built terminal in Rust is very impressive on a portfolio |
| **Total** | **24/40** | Viable as a passion project |

---

## Project Goals

### Primary Goals

1. **Build a complete terminal emulator** - render, input, PTY, ANSI,
   scroll, selection, copy/paste
2. **Context-aware completion engine** - detect project type, load appropriate
   completions, smart Tab completion
3. **Multi-session management** - multiple terminal tabs/panes in one window
4. **Cross-platform** - macOS, Linux, Windows

### Secondary Goals

5. **Plugin system** - community-contributed completion rules
6. **Theming** - customizable colors, fonts, layouts
7. **Deep shell integration** - bash, zsh, fish, PowerShell

### Non-Goals

- AI / LLM integration
- Cloud sync / remote features
- Account system / authentication
- Telemetry / analytics
- IDE-level features (file editing, debugging)

---

## Tech Stack

| Layer | Technology | Role |
|---|---|---|
| Core / Backend | Rust | PTY, terminal emulation, ANSI parsing, context engine |
| App Framework | Tauri v2 | Native app wrapper, cross-platform, IPC |
| Frontend | React + TypeScript | UI rendering, sidebars, completion popup |
| Styling | Tailwind CSS | Utility-first CSS, theming |
| State Management | Zustand | Lightweight state for React |
| Build | Cargo + Vite | Rust build + frontend bundling |

### Why this stack

- **Rust**: High performance, memory safety, ideal for system-level work
- **Tauri v2**: Small binary (~10MB vs Electron ~150MB), native webview
- **React + TS**: Large ecosystem, type safety, easy to build complex UI
- **Zustand**: Simpler than Redux, powerful enough for terminal state

---

## Estimated Timeline

| Phase | Timeframe | Content |
|---|---|---|
| Phase 1: Foundation | Months 1-3 | PTY, basic rendering, input handling |
| Phase 2: Context | Months 4-6 | Context engine, completion system |
| Phase 3: Polish | Months 7-9 | UI/UX, themes, sessions, stability |
| Phase 4: Ecosystem | Months 10-12 | Plugins, packaging, community |

See [05-roadmap/](../05-roadmap/roadmap.md) for details on each phase.

---

## References

- [Warp](https://www.warp.dev/) - Terminal with AI, Rust-based
- [Alacritty](https://github.com/alacritty/alacritty) - GPU-accelerated terminal, Rust
- [Kitty](https://github.com/kovidgoyal/kitty) - GPU-based terminal, C/Python
- [WezTerm](https://github.com/wez/wezterm) - Rust terminal with Lua config
- [Zellij](https://github.com/zellij-org/zellij) - Terminal workspace, Rust
- [xterm.js](https://github.com/xtermjs/xterm.js) - Terminal frontend library (reference only, not used)
