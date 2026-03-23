# Vision and Goals

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Vision

Wit aims to become the **default terminal emulator for developers who value
privacy and smart completions** - where the terminal understands the project
environment without needing an internet connection, without AI, without data collection.

### Long-term Vision

Over 2-3 years, Wit aims to become:

1. **The most beloved open-source terminal emulator** among developers
   who care about privacy and local-first tools
2. **An open platform for community completions** - the largest repository
   of completion rules contributed by the community, covering every popular CLI tool
3. **A technical reference** for building modern terminal emulators in
   Rust - clean codebase, documented, easy to learn from

---

## Design Principles

### 1. Local-first, always

All of Wit's features must work completely offline. No network call
is mandatory. User data never leaves the machine.

```
Yes: Read local file system to detect context
Yes: Completion rules bundled in the app
Yes: Config stored locally as files

No:  Send telemetry
No:  Require login
No:  Cloud sync
No:  Remote completion lookup
```

### 2. Deterministic, not magic

Users must always understand **why** Wit suggests a specific completion. No
black box, no "AI guessing".

```
Yes: "Wit detects Cargo.toml -> load Rust completions" - user understands the logic
Yes: Rule-based, user can read and edit rules
Yes: Debug mode shows which rules are active

No:  "AI thinks you want to run this command" - unexplainable
No:  Probabilistic suggestions
No:  Hidden heuristics
```

### 3. Fast by default

A terminal is a tool used thousands of times per day. Every interaction must feel
**instant** (< 16ms for rendering, < 50ms for completions).

```
Yes: Rust core for performance-critical paths
Yes: Lazy loading - only load completions when needed
Yes: Zero-allocation parsing paths where possible
Yes: Completion matching on background thread

No:  Loading spinner when opening terminal
No:  Perceptible delay when pressing keys
No:  Heavy startup time
```

### 4. Progressive disclosure

Wit must be simple when first used and powerful when needed. Do not overwhelm
new users with too many features.

```
Yes: Opens as a normal terminal - type commands, run commands
Yes: Tab completion feels natural, no learning required
Yes: Advanced features (custom rules, plugins) hidden behind config
Yes: Right sidebar only appears when user wants it

No:  Force user to configure before using
No:  UI full of buttons and panels by default
No:  Mandatory onboarding flow
```

### 5. Composable and extensible

Wit is designed for extensibility from day one. The community can contribute
completion rules, themes, and plugins without modifying core.

```
Yes: Completion rules are data files (TOML/JSON), not code
Yes: Themes are config files, hot-reloadable
Yes: Plugin API is clear, documented
Yes: Core engine exposes enough hooks for extensions

No:  Hard-coded completion logic
No:  Monolithic architecture
No:  Frequent breaking changes
```

---

## Specific Goals

### Must Have (P0) - Cannot ship without these

| ID | Goal | Acceptance Criteria |
|---|---|---|
| G-01 | Working terminal emulation | Can run bash/zsh/fish/PowerShell, renders ANSI correctly |
| G-02 | Basic input handling | Keyboard input, special keys, clipboard, IME |
| G-03 | Context detection | Detect >= 5 project types: git, Node, Python, Rust, Docker |
| G-04 | Tab completion | Suggest commands + flags + arguments when pressing Tab |
| G-05 | Multi-session | Open multiple terminal sessions, switch between sessions |
| G-06 | Cross-platform | Runs on macOS, Linux, Windows |

### Should Have (P1) - Highly desired

| ID | Goal | Acceptance Criteria |
|---|---|---|
| G-07 | Inline completion hints | Display faded suggestions while typing (like fish) |
| G-08 | Completion popup UI | Dropdown showing list of completions with descriptions |
| G-09 | Session sidebar | Left sidebar for managing sessions |
| G-10 | Theming | At least 3 built-in themes, user can create custom themes |
| G-11 | Scrollback buffer | Scroll up to view old output, search within scrollback |
| G-12 | Selection & copy | Mouse selection, keyboard selection, clipboard |

### Nice to Have (P2) - Bonus

| ID | Goal | Acceptance Criteria |
|---|---|---|
| G-13 | Right sidebar | Display environment info, git status, running processes |
| G-14 | Plugin system | API for third-party completion rules |
| G-15 | Split panes | Split terminal into multiple panes |
| G-16 | Shell integration | Inject shell functions for deep integration |
| G-17 | Command history | Smart history search, frecency ranking |
| G-18 | Image rendering | Display inline images (iTerm2 protocol) |

---

## Non-Goals

Things Wit **intentionally does not do**:

| Non-Goal | Reason |
|---|---|
| AI / LLM integration | Core identity - Wit uses rules, not AI |
| Cloud sync | Privacy-first - no data ever leaves the machine |
| Account system | A terminal does not need login |
| Built-in text editor | A terminal is not an IDE |
| SSH client features | Users use the regular `ssh` command |
| Remote development | Focus on local development |
| Telemetry / analytics | Zero data collection |
| Mobile support | A terminal is a desktop tool |

---

## Target Users

### Primary: Privacy-conscious developers

- Care about data privacy, do not want telemetry
- Prefer open-source tools
- Use the terminal many times every day
- Already use zsh + plugins or fish but want a better experience

### Secondary: Beginner developers

- New to the terminal
- Need suggestions to know which commands to use
- Smart completions help them learn faster

### Tertiary: Power users

- Want to customize everything
- Care about performance
- Willing to contribute completion rules

---

## Success Metrics

Since this is a passion/portfolio project, success is not measured by revenue but by:

| Metric | Target (12 months) |
|---|---|
| GitHub stars | >= 500 |
| Contributors | >= 10 |
| Completion rule sets | >= 20 ecosystems |
| Platforms supported | macOS + Linux + Windows |
| Terminal compatibility | Can run 95% of common CLI tools |
| Startup time | < 500ms cold start |
| Input latency | < 16ms key-to-screen |
