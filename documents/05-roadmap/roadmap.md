# Wit Terminal - 12-Month Overview Roadmap

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview of 4 Phases

| Phase | Name         | Timeline      | Main Objective                                       |
| ----- | ------------ | ------------- | ---------------------------------------------------- |
| 1     | Foundation   | Months 1-3    | PTY, renderer, basic I/O, ANSI parser                |
| 2     | Context      | Months 4-6    | Context engine, completion system, shell integration |
| 3     | Polish       | Months 7-9    | UI/UX, themes, sessions, stability, performance      |
| 4     | Ecosystem    | Months 10-12  | Plugin system, packaging, community, cross-platform  |

---

## Phase Details

### Phase 1: Foundation (Months 1-3)

Build the core technical foundation. By the end of this phase, Wit must function as a basic terminal - open a shell, type commands, see colored output.

**Focus areas:**
- Project setup (Tauri v2 + React + Rust build pipeline)
- PTY creation and management on all 3 platforms
- ANSI parser state machine (CSI, SGR, basic escape sequences)
- Terminal grid rendering in React
- Input handling (special keys, Ctrl sequences, function keys)
- Basic session lifecycle (create / destroy)

**See details:** [phase-1-foundation.md](./phase-1-foundation.md)

### Phase 2: Context (Months 4-6)

Transform Wit from a "regular terminal" into a "context-aware terminal". This is the project's main differentiator.

**Focus areas:**
- Context engine with provider trait architecture
- Built-in providers (git, node, python, rust, docker)
- Completion data format and completion files
- Fuzzy matching and ranking algorithm
- Shell integration via OSC sequences
- Tab completion UI (inline hints, popup)

**See details:** [phase-2-context.md](./phase-2-context.md)

### Phase 3: Polish (Months 7-9)

Take Wit from "working prototype" to "daily driver". Focus on UX, performance, and stability.

**Focus areas:**
- Multi-session management with sidebar UI
- Theming system and default themes
- Text selection, clipboard, search, URL detection
- Performance optimization (scrollback, memory, startup time)
- Settings UI and keybinding customization
- Context sidebar and command palette

**See details:** [phase-3-polish.md](./phase-3-polish.md)

### Phase 4: Ecosystem (Months 10-12)

Prepare for public release. Plugin system, packaging, documentation, community.

**Focus areas:**
- Plugin system architecture and API
- Community completion contribution workflow
- Cross-platform testing and platform-specific fixes
- Packaging (.dmg, .deb, .AppImage, .msi, auto-update)
- Documentation site, README with demo GIFs
- Beta testing and v0.1.0 release

**See details:** [phase-4-ecosystem.md](./phase-4-ecosystem.md)

---

## Visual Timeline

```
Month:  1     2     3     4     5     6     7     8     9     10    11    12
        |-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|

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

Milestones:
        *           *     *                 *                 *                 *
        M1          M2    M4                M7                M8                M9
    First Light  Color TV  Foundation     Phase 2          Daily Driver      v0.1.0
                           Complete       Complete
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
```

**Critical path:** PTY -> ANSI Parser -> Grid Rendering -> Input Handling -> Context Engine -> Completion System

If Phase 1 is delayed, all subsequent phases are affected. PTY handling is the most critical dependency.

---

## Risk Factors and Mitigation

| Risk | Probability | Impact | Mitigation |
| ---- | ----------- | ------ | ---------- |
| PTY on Windows more complex than expected (ConPTY edge cases) | High | High | Research beforehand, reference Alacritty/Wezterm source. Allocate extra buffer time for Windows PTY. |
| ANSI parser does not cover enough edge cases | Medium | Medium | Use test suite from vttest. Implement incrementally, fix as issues arise. |
| Tauri v2 has breaking changes or bugs | Medium | High | Pin version early. Monitor Tauri changelog. Have a fallback plan if needed. |
| Performance rendering terminal grid in WebView | Medium | High | Prototype early (weeks 1-2). If Canvas is too slow, switch to WebGL or native rendering. |
| Context engine too slow when scanning large projects | Low | Medium | Async scanning, caching, debounce. Limit scan depth. |
| Scope creep - adding features outside the plan | High | Medium | Strict phase boundaries. Feature requests go to backlog, not into current phase. |
| Burnout (passion project, 1 developer) | Medium | High | Realistic timeline. Allow flexibility in schedule. Celebrate milestones. |

---

## Decision Points

Important decisions need to be made at each phase boundary before continuing.

### End of Phase 1 -> Start Phase 2

- **Rendering strategy**: Is Canvas 2D fast enough or do we need WebGL/OffscreenCanvas?
- **ANSI parser scope**: Implement full VT100/VT220 or only the necessary subset?
- **State management**: Zustand/Jotai/Redux - which one for React state?
- **IPC pattern**: Which Tauri command pattern works best for streaming PTY data?

### End of Phase 2 -> Start Phase 3

- **Completion data format**: YAML/TOML/JSON - which format is best for community contribution?
- **Context detection accuracy**: Accurate enough or do we need more heuristics?
- **Shell integration depth**: CWD tracking only or deeper integration (prompt parsing, command history)?

### End of Phase 3 -> Start Phase 4

- **Plugin sandboxing**: WASM-based or trust-based plugins?
- **Distribution channel**: GitHub Releases, Homebrew, AUR, winget - which to prioritize?
- **Licensing model**: MIT, Apache 2.0, or GPL?
- **Community platform**: GitHub Discussions, Discord, or both?

---

## See Also

- [Detailed Milestones](./milestones.md)
- [Phase 1: Foundation](./phase-1-foundation.md)
- [Phase 2: Context](./phase-2-context.md)
- [Phase 3: Polish](./phase-3-polish.md)
- [Phase 4: Ecosystem](./phase-4-ecosystem.md)
