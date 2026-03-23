# Competitor Analysis

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Warp

| Attribute | Detail |
|-----------|--------|
| **Website** | https://www.warp.dev/ |
| **Tech Stack** | Rust backend, Metal/Vulkan GPU rendering, custom UI framework |
| **Platform** | macOS (primary), Linux (beta), Windows (beta) |
| **License** | Closed source, free tier + paid plans |
| **First Release** | 2022 (public beta) |

### Key Features
- **Blocks model:** Each command + output is a separate "block" - can be selected, copied, shared
- **AI integration:** Built-in AI assistant (Warp AI), natural language -> command
- **Input editor:** Modern text editor for command input (syntax highlighting, multi-cursor, autocomplete)
- **Warp Drive:** Shared workflows, commands, notebooks
- **Themes and customization:** Extensive theme support

### Strengths
- Beautiful, modern UX - sets a new standard for terminal UX
- Blocks model makes navigating output easy
- GPU rendering is very fast
- Input editor experience far superior to traditional terminal
- Good onboarding for new users

### Weaknesses
- **Requires login/account** - dealbreaker for many developers
- **Closed source** - cannot audit, extend, or self-host
- **AI dependency** - many features tied to AI, requires internet
- **Telemetry concerns** - sends usage data
- **Heavy resource usage** - heavier than traditional terminals
- **Shell compatibility:** Custom input editor sometimes conflicts with shell features

### Lessons for Wit
- Blocks model is very useful - Wit should have a similar concept (via OSC 133 shell integration)
- Modern input editing experience - but implement differently (overlay/inline completion instead of replacing entire input)
- GPU rendering for performance
- Avoid requiring login - Wit must work fully offline
- Avoid closed source - Wit should be open source
- Don't make AI a hard dependency - AI is an enhancement, not core

---

## 2. Alacritty

| Attribute | Detail |
|-----------|--------|
| **Website** | https://alacritty.org/ |
| **Tech Stack** | Rust, OpenGL rendering |
| **Platform** | macOS, Linux, Windows, BSD |
| **License** | Apache 2.0 (open source) |
| **First Release** | 2017 |

### Key Features
- Minimal, fast terminal emulator
- GPU-accelerated rendering (OpenGL)
- YAML/TOML configuration (no GUI settings)
- Vi mode for terminal navigation
- Search in scrollback
- URL detection

### Strengths
- **Extremely fast** - consistently benchmarks as fastest terminal
- **Simple, focused** - does one thing well
- **Cross-platform** - same codebase, all major platforms
- **Proven Rust terminal** - demonstrated Rust is viable for terminal emulators
- **Well-tested VT parser** - `vte` crate extracted and widely used
- **Small binary** - efficient resource usage

### Weaknesses
- **No tabs, no splits** - intentional design choice, rely on tmux/zellij
- **No completions, no AI** - minimal by design
- **No GUI configuration** - text config only
- **No ligature support** - controversial omission
- **No image rendering** - Sixel, etc. not supported
- **Scrollback performance:** Large scrollback can be slow

### Lessons for Wit
- Rust for terminal emulator is viable and performant
- `vte` crate is a solid parser - Wit should use it (at least initially)
- Performance benchmarks - Alacritty sets the bar
- Cross-platform Rust code organization
- Wit needs tabs, splits - users expect them
- Wit needs richer features - but keep core fast

---

## 3. Kitty

| Attribute | Detail |
|-----------|--------|
| **Website** | https://sw.kovidgoyal.net/kitty/ |
| **Tech Stack** | C (core) + Python (extensions/config), OpenGL rendering |
| **Platform** | macOS, Linux (no Windows) |
| **License** | GPL v3 (open source) |
| **First Release** | 2017 |

### Key Features
- GPU-accelerated rendering (OpenGL)
- **Kitten plugins:** Python-based extension system (`kitty +kitten`)
- **Kitty graphics protocol:** Advanced image rendering in terminal
- **Kitty keyboard protocol:** Disambiguated key reporting
- Tabs and window splits (built-in multiplexer)
- Ligature support
- Unicode input
- Remote control via socket

### Strengths
- **Very fast** - GPU rendering, competitive with Alacritty
- **Feature-rich** - image rendering, splits, tabs, remote control
- **Extensible** - Python kittens for custom behavior
- **Protocol innovations:** Kitty keyboard protocol being adopted by other terminals
- **Image rendering protocol** - most advanced terminal graphics
- **Excellent documentation**

### Weaknesses
- **C + Python** - not Rust, more complex build, potential memory safety issues
- **Complex configuration** - many options, steep learning curve
- **No Windows support** - missing major platform
- **GPL license** - restrictive for commercial use
- **Opinionated author** - sometimes contentious community interactions

### Lessons for Wit
- Plugin system design - kittens model is interesting, Wit could use WASM instead of Python
- Graphics protocol - reference for image rendering
- Keyboard protocol - modern key encoding, Wit should support it
- Built-in multiplexer - users prefer not needing tmux
- Remote control API - useful for automation
- Avoid GPL if flexible licensing is desired
- Keep config simpler - provide good defaults

---

## 4. WezTerm

| Attribute | Detail |
|-----------|--------|
| **Website** | https://wezfurlong.org/wezterm/ |
| **Tech Stack** | Rust, OpenGL rendering, Lua configuration |
| **Platform** | macOS, Linux, Windows, FreeBSD |
| **License** | MIT (open source) |
| **First Release** | 2018 |

### Key Features
- GPU-accelerated rendering
- **Lua scripting** for configuration and customization
- Built-in multiplexer (tabs, splits, workspaces)
- SSH integration (connect to remote hosts, multiplex over SSH)
- Serial port connections
- Image rendering (iTerm2 protocol, Sixel, Kitty protocol)
- Ligature support
- Hyperlinks

### Strengths
- **Rust-based** - closest tech stack to Wit
- **True cross-platform** - excellent Windows support
- **Lua configuration** - powerful yet approachable
- **Multiplexer** - built-in, good alternative to tmux
- **Feature complete** - images, multiplexing, SSH, serial
- **MIT license** - permissive
- **Active development** by Wez Furlong (also creator of `portable-pty`)

### Weaknesses
- **Large binary** - ~30MB, many features = big
- **Complex codebase** - lots of features make code complex
- **Resource usage** - heavier than Alacritty
- **Documentation** - could be better organized
- **Rendering quirks** - occasional visual glitches reported

### Lessons for Wit
- **Closest reference project** - same language, similar goals
- `portable-pty` crate - WezTerm's author created it, Wit should use it
- Multiplexer architecture - how to implement tabs/splits in Rust
- Cross-platform approach - how WezTerm handles Windows ConPTY
- Lua for config - consider similar approach (though Wit may use TOML + plugin system)
- Avoid feature bloat - WezTerm tries to do everything, Wit should be more focused

---

## 5. Zellij

| Attribute | Detail |
|-----------|--------|
| **Website** | https://zellij.dev/ |
| **Tech Stack** | Rust |
| **Platform** | macOS, Linux (no native Windows) |
| **License** | MIT (open source) |
| **First Release** | 2021 |

### Key Features
- Terminal workspace/multiplexer (replacement for tmux/screen)
- **WASM plugin system** - plugins compiled to WebAssembly
- Session management
- Floating panes
- Built-in layouts
- Discoverable keybindings (status bar shows available actions)

### Strengths
- **WASM plugin system** - language-agnostic, sandboxed plugins
- **Great UX** - discoverable, beginner-friendly
- **Session management** - persist and restore sessions
- **Layout system** - declarative layout definitions
- **Modern Rust** - clean codebase

### Weaknesses
- **Not a terminal emulator** - runs inside another terminal (or via own renderer which is limited)
- **Performance** - slower than direct terminal emulators
- **No Windows** - lacks major platform
- **Plugin ecosystem** - still small
- **Resource usage** - overhead from being a layer on top

### Lessons for Wit
- **WASM plugins** - excellent idea, Wit should adopt WASM for plugin system
- **Discoverable UI** - status bar hints, progressive disclosure
- **Layout system** - declarative layouts useful for workspace concept
- **Session persistence** - users want this
- Wit IS a terminal emulator - can integrate multiplexer features directly

---

## 6. iTerm2

| Attribute | Detail |
|-----------|--------|
| **Website** | https://iterm2.com/ |
| **Tech Stack** | Objective-C, Cocoa framework |
| **Platform** | macOS only |
| **License** | GPL v2 (open source) |
| **First Release** | 2006 (iTerm2 rewrite of iTerm) |

### Key Features
- **Shell integration** - marks, navigation between prompts, command history
- Split panes, tabs, profiles
- **Triggers** - regex-based actions on terminal output
- **Python scripting API** - extensive automation
- Inline images (iTerm2 protocol, widely adopted)
- Semantic history (Cmd+click on filenames)
- tmux integration (control mode)
- Password manager
- Instant replay (rewind terminal state)
- Search, autocomplete, paste history

### Strengths
- **Most feature-complete terminal** - decades of development
- **Shell integration protocol** - OSC 133 originated here, now adopted widely
- **Trigger system** - powerful pattern matching on output
- **Mature and stable** - used by millions
- **Inline images** - iTerm2 protocol became de facto standard
- **Python API** - deep scripting capability

### Weaknesses
- **macOS only** - no cross-platform
- **Objective-C** - not Rust, aging codebase
- **Resource heavy** - significant memory usage
- **Slow rendering** - not GPU-accelerated (CPU rendered)
- **Complex settings** - overwhelming preference pane

### Lessons for Wit
- **Shell integration (OSC 133)** - Wit MUST implement this, it's the foundation for context-awareness
- **Trigger system** - regex actions on output, very powerful for automation
- **Semantic history** - click on filename -> open in editor, click on URL -> open browser
- **Inline images protocol** - implement iTerm2 protocol for compatibility
- **Profiles** - per-project/per-host terminal configurations
- **Instant replay** - interesting feature for debugging

---

## 7. Fig (now Amazon Q CLI)

| Attribute | Detail |
|-----------|--------|
| **Website** | https://aws.amazon.com/q/developer/ (formerly fig.io) |
| **Tech Stack** | TypeScript (completion engine), Rust (core), Swift (macOS UI) |
| **Platform** | macOS (primary), Linux |
| **License** | Closed source (was partially open) |
| **History** | Fig (startup) -> acquired by Amazon 2023 -> rebranded to Amazon Q Developer CLI |

### Key Features
- **Autocomplete overlay** - appears below cursor in any terminal
- **Completion specs** - community-contributed completion definitions (JSON/TypeScript)
- **AI-powered** - natural language to command
- **Dotfile management** - sync shell config
- **Scripts/workflows** - shareable command sequences

### Strengths
- **Completion UX** - inline dropdown completion, very intuitive
- **Community specs** - 600+ CLI tools with completion definitions
- **Completion spec format** - well-designed, machine-readable CLI descriptions
- **Cross-terminal** - works as overlay on any terminal

### Weaknesses
- **Acquired by Amazon** - pivoted away from developer tool focus
- **Closed source** - community lost access
- **Reliability issues** - overlay injection can be flaky
- **Privacy concerns** - Amazon telemetry
- **Discontinued (as Fig)** - uncertain future

### Lessons for Wit
- **Completion spec format** - study and adopt/adapt the format for Wit's completions
- **Community contribution model** - let community contribute completion specs
- **Inline completion UX** - dropdown below cursor, not in separate panel
- **CLI description format** - machine-readable descriptions of CLI tools
- Don't depend on overlay injection - Wit controls the terminal, integrate natively
- Avoid acquisition risk - stay open source, community-driven

---

## 8. Windows Terminal

| Attribute | Detail |
|-----------|--------|
| **Website** | https://github.com/microsoft/terminal |
| **Tech Stack** | C++, DirectWrite/Direct2D rendering |
| **Platform** | Windows only |
| **License** | MIT (open source) |
| **First Release** | 2019 |

### Key Features
- Modern Windows terminal replacing cmd.exe/conhost
- **ConPTY** - created alongside Windows Terminal
- GPU-accelerated text rendering (DirectWrite)
- Tabs, splits, profiles
- JSON configuration
- Customizable key bindings
- Quake mode (dropdown terminal)
- Emoji and Unicode support

### Strengths
- **ConPTY reference implementation** - best source for understanding Windows pseudo-console
- **Modern Windows terminal** - finally a good terminal on Windows
- **Open source** - can study ConPTY integration
- **Performance** - good rendering performance with DirectWrite
- **Profile system** - per-shell configurations

### Weaknesses
- **Windows only** - no cross-platform
- **C++** - complex codebase
- **Settings complexity** - JSON config not beginner-friendly
- **Limited extensibility** - no plugin system

### Lessons for Wit
- **ConPTY implementation reference** - study how WT uses ConPTY
- **Windows rendering** - DirectWrite approach for text
- **Profile system** - different profiles for cmd, PowerShell, WSL, Git Bash
- **JSON settings** - good format, but Wit might prefer TOML

---

## 9. Feature Comparison Matrix

| Feature | Warp | Alacritty | Kitty | WezTerm | Zellij | iTerm2 | Win Terminal | **Wit (planned)** |
|---------|------|-----------|-------|---------|--------|--------|-------------|------------------|
| **Language** | Rust | Rust | C/Python | Rust | Rust | Obj-C | C++ | **Rust** |
| **GPU Rendering** | Yes | Yes | Yes | Yes | No | No | Yes | **Planned** |
| **Cross-platform** | Partial | Yes | Partial | Yes | Partial | No | No | **Yes** |
| **Tabs** | Yes | No | Yes | Yes | Yes | Yes | Yes | **Yes** |
| **Splits** | Yes | No | Yes | Yes | Yes | Yes | Yes | **Yes** |
| **Completions** | Yes | No | No | No | No | Partial | No | **Yes** |
| **AI Integration** | Yes | No | No | No | No | No | No | **Planned** |
| **Shell Integration** | Yes | No | No | No | No | Yes | No | **Yes** |
| **Context Awareness** | Partial | No | No | No | No | Partial | No | **Yes (core)** |
| **Plugin System** | No | No | Yes (Python) | No | Yes (WASM) | Yes (Python) | No | **Yes (WASM)** |
| **Images** | No | No | Yes | Yes | No | Yes | No | **Planned** |
| **Ligatures** | Yes | No | Yes | Yes | N/A | Yes | Yes | **Planned** |
| **True Color** | Yes | Yes | Yes | Yes | Yes | Yes | Yes | **Yes** |
| **Hyperlinks** | Yes | No | Yes | Yes | No | Yes | Yes | **Yes** |
| **Open Source** | No | Yes | Yes | Yes | Yes | Yes | Yes | **Yes** |
| **Offline** | Partial | Yes | Yes | Yes | Yes | Yes | Yes | **Yes** |
| **Multiplexer** | No | No | Partial | Yes | Yes | No | No | **Planned** |
| **Search** | Yes | Yes | Yes | Yes | Yes | Yes | Yes | **Yes** |
| **Mouse** | Yes | Yes | Yes | Yes | Yes | Yes | Yes | **Yes** |

Legend: Yes = Supported | Partial = Partial/Limited | No = Not Supported

---

## 10. Competitive Positioning for Wit

### 10.1 Wit's Unique Value Proposition

Wit differentiates from competitors through the combination of:
1. **Context-awareness as core feature** - no current terminal puts context at the center
2. **Open source + offline-first** - unlike Warp (closed, requires login)
3. **Rust + Tauri** - modern, cross-platform, lighter than Electron
4. **WASM plugin system** - language-agnostic, sandboxed (like Zellij, but in a full terminal emulator)
5. **Completion engine** - built-in, community-driven specs (like Fig, but integrated)

### 10.2 Competitive Threats

| Threat | From | Mitigation |
|--------|------|------------|
| Feature parity | WezTerm, Kitty | Focus on UX, not feature count |
| AI integration | Warp, Amazon Q | Integrate AI as optional enhancement |
| Performance | Alacritty | GPU rendering, efficient Rust code |
| Ecosystem | iTerm2 (macOS) | Cross-platform, modern stack |
| Plugin ecosystem | Kitty, Zellij | WASM + community contribution tooling |

### 10.3 Target User Segments

1. **Power developers** - want context-aware features, completions, efficiency
2. **Warp refugees** - want modern UX without login/closed source
3. **Cross-platform devs** - need same experience on macOS + Linux + Windows
4. **Plugin creators** - want extensible terminal with modern plugin system

---

## References

1. Warp: https://www.warp.dev/
2. Alacritty: https://alacritty.org/ / https://github.com/alacritty/alacritty
3. Kitty: https://sw.kovidgoyal.net/kitty/ / https://github.com/kovidgoyal/kitty
4. WezTerm: https://wezfurlong.org/wezterm/ / https://github.com/wez/wezterm
5. Zellij: https://zellij.dev/ / https://github.com/zellij-org/zellij
6. iTerm2: https://iterm2.com/ / https://github.com/gnachman/iTerm2
7. Amazon Q Developer: https://aws.amazon.com/q/developer/
8. Windows Terminal: https://github.com/microsoft/terminal
