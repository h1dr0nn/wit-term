# Wit Terminal — Development Roadmap

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

| Phase | Name         | Timeline      | Status | Main Objective                                       |
| ----- | ------------ | ------------- | ------ | ---------------------------------------------------- |
| 1     | Foundation   | Months 1-3    | Done   | PTY, renderer, basic I/O, ANSI parser                |
| 2     | Context      | Months 4-6    | Done   | Context engine, completion system, shell integration |
| 3     | Polish       | Months 7-9    | ~85%   | UI/UX, themes, sessions, stability, performance      |
| 4     | Ecosystem    | Months 10-12  | ~45%   | Plugin system, packaging, community, cross-platform  |
| 5     | Agent        | Months 13-16  | ~50%   | Agent detection, adapters, sidebar dashboard, Wit Protocol |

---

## Phase 1: Foundation (Months 1-3) — DONE

Build the core technical foundation. By the end of this phase, Wit must function as a basic terminal — open a shell, type commands, see colored output.

**Progress:**
- [x] Project setup (Tauri v2 + React + Rust build pipeline)
- [x] PTY creation and management on all 3 platforms (Unix + Windows ConPTY)
- [x] ANSI parser state machine (14-state VT parser based on Paul Williams model)
- [x] Terminal grid rendering in React (DOM-based with scrollback)
- [x] Input handling (special keys, Ctrl sequences, arrow keys, clipboard)
- [x] Basic session lifecycle (create / destroy)
- [x] Wide character support (Unicode width for CJK characters)
- [x] Command blocks (Warp-style captured output per command)
- [x] ANSI color rendering in blocks (16, 256, RGB)

**See details:** [phase-1-foundation.md](./phase-1-foundation.md)

## Phase 2: Context (Months 4-6) — DONE

Transform Wit from a "regular terminal" into a "context-aware terminal". This is the project's first differentiator.

**Progress:**
- [x] Context engine with provider trait architecture (async scanning, caching, TTL)
- [x] Built-in providers: git, node, python, rust, docker, go, java (7 providers)
- [x] Completion data format: 14 bundled TOML files (git, npm, yarn, pnpm, cargo, docker, kubectl, pip, ssh, make, systemctl, brew, apt, general)
- [x] Fuzzy matching and ranking algorithm
- [x] Shell integration via OSC 7 (CWD tracking)
- [x] Tab completion UI (inline ghost text + popup)
- [x] Runtime version detection (Node.js, Python, Rust versions in InputBar)
- [x] Path completion source
- [x] Dynamic context-aware completions (git branches, npm scripts, docker containers)

**See details:** [phase-2-context.md](./phase-2-context.md)

## Phase 3: Polish (Months 7-9) — In Progress (~85%)

Take Wit from "working prototype" to "daily driver". Focus on UX, performance, and stability.

**Progress:**
- [x] Multi-session management with sidebar UI (create, delete, switch, search)
- [x] Theming system: 12 built-in themes (Catppuccin, Dracula, Tokyo Night, Nord, Solarized, One Dark/Light, GitHub Light, Wit Dark/Light) with hot-reload
- [x] Clipboard support (copy/paste)
- [x] Search overlay (Ctrl+Shift+F) with regex, case-sensitive, match navigation
- [x] Settings UI (font, theme, cursor, scrollback)
- [x] Context sidebar (detected providers, expandable details, refresh)
- [x] Command palette (Ctrl+Shift+P) with categorized commands
- [ ] Text selection with visual feedback in terminal grid
- [x] URL detection and Ctrl+click to open
- [ ] Split panes (horizontal/vertical)
- [ ] Keybinding customization UI
- [x] Session persistence between app restarts
- [ ] Performance benchmarks (startup time, scroll FPS, memory usage)

**See details:** [phase-3-polish.md](./phase-3-polish.md)

## Phase 4: Ecosystem (Months 10-12) — In Progress (~45%)

Prepare for public release. Plugin system, packaging, documentation, community.

**Progress:**
- [x] Plugin system architecture (trait, manifest, loader)
- [x] Packaging config (DMG, AppImage, Deb, MSI targets in tauri.conf.json)
- [x] CI/CD pipeline (ci.yml: lint + build on 3 platforms, develop.yml, release.yml)
- [x] Cross-platform build pipeline (macOS, Linux, Windows)
- [x] Bundle resources (completions, themes, shell integration)
- [ ] Example plugin with documentation
- [x] Community completion contribution workflow / guide (CONTRIBUTING.md)
- [ ] Auto-update (Tauri updater integration)
- [ ] Distribution channels (Homebrew, AUR, winget)
- [ ] Documentation site
- [ ] Demo GIFs in README
- [ ] Beta testing program
- [ ] v0.1.0 GitHub Release

**See details:** [phase-4-ecosystem.md](./phase-4-ecosystem.md)

---

## Phase 5: Agent-Aware Terminal (Months 13-16) — In Progress (Sub-Phase A+B done)

Wit's killer feature: auto-detect AI agent CLIs and provide a rich sidebar dashboard. This creates a new category — **agent-aware terminal** — and is Wit's strongest differentiator from every other terminal emulator.

### Why This Matters

AI coding agents (Claude Code, Aider, Codex CLI, Copilot CLI) are becoming mainstream development tools, but they all run inside "dumb" terminals. Developers must juggle multiple windows just to understand what an agent is doing. Wit solves this by making the terminal itself agent-aware.

### Architecture: 4-Layer Progressive Detection

```
Layer 1: Process Detection ──► Know WHICH agent is running
Layer 2: Output Parsing    ──► Know WHAT the agent is doing (tokens, cost, actions)
Layer 3: Filesystem Watch  ──► Know WHICH FILES the agent changed
Layer 4: Wit Protocol      ──► TWO-WAY communication (approval, context injection)
```

Layers 1-3 are **passive** — they observe without modifying the agent. Layer 4 is an **opt-in protocol** for agents that want deeper integration.

### Sub-Phase A: Foundation (Weeks 1-3)

**Goal:** Detect agent processes and show the sidebar skeleton.

- [x] A1: Process detection engine using `sysinfo` crate → `AgentDetected` / `AgentExited` events
- [x] A2: Agent pattern database (process names, CLI args) → Match Claude Code, Aider, Codex CLI, Copilot CLI
- [x] A3: Polling (2s interval) → CPU-efficient detection
- [x] A4: `AgentSidebar` React component with auto-open/close → Sidebar shell with Activity/Files tabs
- [x] A5: Keybinding Ctrl+Shift+A for manual toggle → Per-tab sidebar isolation

**Exit criteria:** When a user runs `claude` in Wit, the sidebar opens showing "Claude Code detected (PID xxxx)".

### Sub-Phase B: Claude Code Adapter (Weeks 4-7)

**Goal:** Full working dashboard for one agent — Claude Code.

- [x] B1: `AgentAdapter` trait definition → `fn parse_output(&mut self, data: &[u8]) -> Vec<AgentEvent>`
- [x] B2: `ClaudeCodeAdapter` implementation → Parse cost, tokens, model, thinking, tool use, file edits
- [x] B3: `AgentEvent` enum → ThinkingStart/End, ToolUse, FileEdit, TokenUpdate, CostUpdate, ModelInfo, StatusText, Error
- [x] B4: Filesystem watcher using `notify` crate → Debounced file change events with ignore patterns
- [x] B5: Activity Timeline component → Live timeline with colored dots, relative timestamps, auto-scroll
- [x] B6: Files Tab component → File list with created/modified/deleted status and summary bar
- [x] B7: Token/Cost counter in sidebar header → Abbreviated format, color-coded thresholds ($1 warning, $5 danger)

**Exit criteria:** Run Claude Code in Wit → sidebar shows live activity, token cost, and all file changes with diffs.

### Sub-Phase C: Expand Adapters (Weeks 8-10)

**Goal:** Support multiple agents using the validated adapter interface.

- [ ] C1: `AiderAdapter` → Parse Aider output (commits, edits, cost)
- [ ] C2: `CodexAdapter` → Parse Codex CLI output
- [ ] C3: `CopilotAdapter` → Parse GitHub Copilot CLI output
- [ ] C4: Refine `AgentAdapter` trait → Document for community adapter contributions
- [ ] C5: Auto-detection routing → Match detected agent → correct adapter

**Exit criteria:** All 4 agents detected and parsed correctly. Adapter interface documented and stable.

### Sub-Phase D: Wit Protocol (Weeks 11-16)

**Goal:** Two-way structured communication between Wit and agents.

- [ ] D1: Socket server (Unix domain socket / Windows named pipe) → One socket per session via `$WIT_SOCKET`
- [ ] D2: NDJSON message protocol → Agent→Wit: status, file_change, usage, approval_request
- [ ] D3: Wit→Agent messages → approval_response, command (pause/resume), context
- [ ] D4: Approval Request UI → Modal card with approve/reject/timeout
- [ ] D5: Context injection → Respond to agent context requests with project info
- [ ] D6: `wit-protocol` Rust crate → Published on crates.io
- [ ] D7: `@nicwit/wit-protocol` npm package → Published on npm
- [ ] D8: `wit-protocol` Python package → Published on PyPI

**Exit criteria:** An agent using the Wit Protocol SDK can send structured events and receive approval responses through the socket.

### Phase 5 Summary

```
Week:   1     2     3     4     5     6     7     8     9    10    11    12    13    14    15    16
        |-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|

Sub-Phase A: Foundation
        |===========|
        Detection  Sidebar  Polish

Sub-Phase B: Claude Code Adapter
                    |===================|
                    Adapter  Watcher  Activity  Files  Cost

Sub-Phase C: Expand Adapters
                                        |===========|
                                        Aider  Codex  Copilot  Refine

Sub-Phase D: Wit Protocol
                                                      |=========================|
                                                      Socket  Messages  Approval  SDKs

Milestones:
        *           *                   *             *                           *
        M10         M11                 M12           M13                         M14
    Agent Detect  Sidebar Live     Claude Code     Multi-Agent               Protocol v1
                                   Dashboard       Support
```

**See details:** [agent-detection-milestones.md](./agent-detection-milestones.md)

---

## Full Visual Timeline

```
Month:  1     2     3     4     5     6     7     8     9     10    11    12    13    14    15    16
        |-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|

Phase 1: Foundation
        |=================|
        Setup  PTY  ANSI  Grid  Input  Test

Phase 2: Context
                          |=================|
                          Engine Providers Completions Shell UI

Phase 3: Polish
                                            |=================|
                                            Sessions Themes Select Perf Settings

Phase 4: Ecosystem
                                                              |=================|
                                                              Plugins Pkg Docs Release

Phase 5: Agent-Aware Terminal
                                                                                |=================|
                                                                                Detect Adapt Proto SDKs

Milestones:
        *           *     *                 *                 *                 *     *           *
        M1          M2    M4                M7                M8                M9    M12         M14
    First Light  Color TV  Foundation     Phase 2          Daily Driver      v0.1.0  Claude    Protocol
                           Complete       Complete                           Release Dashboard    v1
```

---

## Dependencies Between Phases

```
Phase 1: Foundation
    ├── PTY handling ──────────────────► Phase 2: Shell integration
    ├── ANSI parser ──────────────────► Phase 2: OSC sequence support
    ├── Terminal grid ────────────────► Phase 3: Scrollback optimization
    ├── Input handling ───────────────► Phase 3: Keybinding customization
    └── Session lifecycle ────────────► Phase 3: Multi-session management

Phase 2: Context
    ├── Context engine ───────────────► Phase 3: Context sidebar UI
    ├── Completion system ────────────► Phase 4: Plugin API for completions
    └── Provider trait ───────────────► Phase 4: Community providers

Phase 3: Polish
    ├── Theming system ───────────────► Phase 4: Theme marketplace
    ├── Settings infrastructure ──────► Phase 4: Plugin settings
    └── Performance baseline ─────────► Phase 4: Cross-platform benchmarks

Phase 4: Ecosystem
    ├── Plugin architecture ──────────► Phase 5: Community adapters
    └── Packaging pipeline ───────────► Phase 5: SDK publishing

Phase 5: Agent-Aware Terminal
    ├── A1: Process Detection ────────► A4: Sidebar UI (needs events)
    ├── A4: Sidebar UI ───────────────► B5-B7: Dashboard tabs (needs container)
    ├── B1: Adapter trait ────────────► C1-C3: More adapters (needs interface)
    ├── B4: Filesystem watcher ───────► B6: Files tab (needs file events)
    └── C1-C3: All adapters ──────────► D1-D8: Protocol design (informs spec)
```

**Critical path:** PTY → ANSI Parser → Grid → Context Engine → Completions → Plugin System → Agent Detection → Adapter → Protocol

---

## Risk Factors and Mitigation

| Risk | Probability | Impact | Mitigation |
| ---- | ----------- | ------ | ---------- |
| PTY on Windows more complex than expected (ConPTY edge cases) | High | High | Research beforehand, reference Alacritty/Wezterm source. Allocate extra buffer time. |
| ANSI parser does not cover enough edge cases | Medium | Medium | Use test suite from vttest. Implement incrementally, fix as issues arise. |
| Tauri v2 has breaking changes or bugs | Medium | High | Pin version early. Monitor changelog. Have a fallback plan. |
| Performance rendering terminal grid in WebView | Medium | High | Prototype early. If Canvas is too slow, switch to WebGL or native rendering. |
| Context engine too slow on large projects | Low | Medium | Async scanning, caching, debounce. Limit scan depth. |
| Agent output format changes between versions | High | Medium | Version-aware adapters with graceful degradation. Snapshot tests per version. |
| Filesystem watching perf on large repos | Medium | Medium | Debounce (500ms), ignore `node_modules/`/`target/`. Limit watch depth. |
| Protocol adoption by agent developers is slow | Medium | Low | Layers 1-3 must be fully sufficient standalone. Protocol is additive. |
| Adapter maintenance burden across agent updates | High | Medium | CI tests against latest agent versions. Community contributions for updates. |
| Scope creep | High | Medium | Strict phase boundaries. Feature requests go to backlog, not current phase. |

---

## Decision Points

### End of Phase 1 → Start Phase 2

- **Rendering strategy**: Is Canvas 2D fast enough or do we need WebGL/OffscreenCanvas?
- **ANSI parser scope**: Implement full VT100/VT220 or only the necessary subset?
- **State management**: Zustand/Jotai/Redux — which one for React state?
- **IPC pattern**: Which Tauri command pattern works best for streaming PTY data?

### End of Phase 2 → Start Phase 3

- **Completion data format**: YAML/TOML/JSON — which format is best for community contribution?
- **Context detection accuracy**: Accurate enough or do we need more heuristics?
- **Shell integration depth**: CWD tracking only or deeper integration (prompt parsing, command history)?

### End of Phase 3 → Start Phase 4

- **Plugin sandboxing**: WASM-based or trust-based plugins?
- **Distribution channel**: GitHub Releases, Homebrew, AUR, winget — which to prioritize?
- **Licensing model**: MIT, Apache 2.0, or GPL?
- **Community platform**: GitHub Discussions, Discord, or both?

### End of Phase 4 → Start Phase 5

- **Agent priority**: Which agent has the highest user demand? Start with that adapter.
- **Detection reliability**: Is process detection robust enough cross-platform?
- **Sidebar UX**: Should the sidebar auto-open or require user confirmation?

### After Phase 5B → Expand or Protocol?

- Is the `AgentAdapter` trait flexible enough for agents beyond Claude Code?
- Should adapters be in-process (compiled) or out-of-process (plugin)?
- Do Layers 1-3 provide sufficient value without the protocol?
- Is there demand from agent developers for structured communication?
- If proceeding with Phase D, which SDK (Rust, npm, PyPI) is highest priority?

### After Phase 5D → Ecosystem Strategy

- Open-source the Wit Protocol specification for community adoption?
- Partner with specific agent projects for native integration?
- Build a registry for community-contributed adapters?

---

## See Also

- [Detailed Milestones](./milestones.md)
- [Phase 1: Foundation](./phase-1-foundation.md)
- [Phase 2: Context](./phase-2-context.md)
- [Phase 3: Polish](./phase-3-polish.md)
- [Phase 4: Ecosystem](./phase-4-ecosystem.md)
- [Agent Detection Milestones](./agent-detection-milestones.md)
