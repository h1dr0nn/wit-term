# Phase 1: Foundation (Months 1-3)

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Goals and Success Criteria

### Goals
1. Set up complete project infrastructure (build, test, lint, CI)
2. Implement PTY handling on all 3 platforms (Windows, macOS, Linux)
3. Build ANSI parser state machine that handles CSI/SGR sequences
4. Render terminal grid in React with cursor and scrolling
5. Handle all keyboard input (special keys, Ctrl, function keys)
6. Basic session lifecycle (create, destroy, error handling)

### Success Criteria
- [ ] Open Wit -> see shell prompt (bash/zsh/PowerShell depending on platform)
- [ ] Type `ls --color` -> see colored output
- [ ] Run `vim` -> can edit a file and exit normally
- [ ] Run `git log --oneline --graph` -> displays correctly
- [ ] Ctrl+C stops a running process
- [ ] Can resize window and terminal reflows

---

## Week-by-Week Breakdown

### Week 1-2: Project Setup

**Objective:** Build pipeline working, basic window displaying.

**Tasks:**
- [ ] Initialize Tauri v2 project with React/TypeScript frontend
- [ ] Setup Rust workspace structure (crates: `wit-core`, `wit-pty`, `wit-ansi`)
- [ ] Configure build pipeline: `cargo build` + `npm run build` + `tauri build`
- [ ] Setup linting: `clippy` (Rust), `eslint` + `prettier` (TypeScript)
- [ ] Setup testing: `cargo test`, `vitest` (React)
- [ ] CI pipeline (GitHub Actions): build on 3 platforms
- [ ] Basic Tauri window opens, displaying "Hello from Wit" in webview
- [ ] Dev workflow: `cargo tauri dev` hot-reload working

**Output:** Run `cargo tauri dev` -> window opens with webview. Build succeeds on CI.

**Risks:**
- Tauri v2 setup may encounter issues with platform-specific dependencies
- Hot-reload may be slow if Rust build takes long

### Week 3-4: PTY Creation and Basic I/O

**Objective:** Spawn shell process, send/receive data via PTY.

**Tasks:**
- [ ] Implement PTY abstraction trait in Rust
- [ ] **Linux/macOS:** Use `forkpty()` or crate `portable-pty` / custom implementation
- [ ] **Windows:** ConPTY API integration (`CreatePseudoConsole`)
- [ ] Spawn default shell ($SHELL on Unix, PowerShell on Windows)
- [ ] PTY read loop: read output from shell -> send via Tauri event
- [ ] PTY write: receive keystroke from frontend -> send to PTY
- [ ] Basic echo: type character in webview -> see character in terminal
- [ ] Handle PTY resize (SIGWINCH on Unix, `ResizePseudoConsole` on Windows)
- [ ] Error handling: shell exit, PTY errors, process cleanup

**Output:** Type `echo hello` -> see "hello". Type `ls` -> see file listing (no color yet).

**Risks:**
- **Windows ConPTY** is the biggest risk. Complex API, limited documentation, many edge cases.
- PTY size synchronization between frontend and backend may cause rendering issues.
- Need to handle encoding (UTF-8) correctly from the start.

**Mitigation:**
- Reference implementation from Alacritty, Wezterm, Windows Terminal source code
- Use `portable-pty` crate if custom implementation is too complex
- Write integration tests early for PTY I/O

### Week 5-6: ANSI Parser State Machine

**Objective:** Parse ANSI escape sequences, handle SGR colors.

**Tasks:**
- [ ] Implement state machine following the Paul Williams VT parser model
- [ ] States: Ground, Escape, EscapeIntermediate, CsiEntry, CsiParam, CsiIntermediate, CsiIgnore, OscString, DcsEntry, ...
- [ ] Parse CSI sequences: cursor movement (CUU, CUD, CUF, CUB), erase (ED, EL), scroll (SU, SD)
- [ ] Parse SGR sequences: 16 colors, 256 colors, RGB true color
- [ ] Parse SGR attributes: bold, italic, underline, blink, inverse, strikethrough
- [ ] Handle partial sequences (data arriving in the middle of an escape sequence)
- [ ] Unit tests with known ANSI sequences
- [ ] Test with vttest test suite (basic tests)

**Output:** Raw ANSI bytes -> structured events (Print, CSI, SGR, ...).

**Tech Decisions:**
- Parser output format: enum-based events or callback-based?
- Recommend: enum events, easy to test and serialize via IPC

**Risks:**
- The ANSI standard is very broad. Need to clearly define scope: VT100 + VT220 + xterm extensions.
- Some applications (vim, tmux, htop) use less common sequences.

### Week 7-8: Terminal Grid Rendering

**Objective:** Display terminal output in React, with cursor and scrolling.

**Tasks:**
- [ ] Design terminal grid data structure (rows x columns, each cell: char + style)
- [ ] Implement grid update logic: apply parsed ANSI events to grid
- [ ] React component `<TerminalGrid>` to render grid
- [ ] Rendering strategy: DOM-based (each row is 1 div) or Canvas 2D
- [ ] Cursor rendering: block, beam, underline cursor styles
- [ ] Cursor blink animation
- [ ] Basic scrolling: when output exceeds viewport, scroll down
- [ ] Scrollback buffer: retain old output (initial limit: 1000 lines)
- [ ] Font rendering: monospace font, measure character dimensions precisely
- [ ] Handle Unicode: wide characters (CJK), combining characters (diacritics)

**Output:** Run `ls --color` -> see colored file listing. Run `vim` -> see vim UI render.

**Tech Decisions to make:**
- **DOM vs Canvas:** DOM is easier to implement but may be slow with large output. Canvas is faster but more complex (text selection, accessibility).
- **Recommend:** Start with DOM, profile performance, switch to Canvas if needed.

**Preliminary performance targets:**
- Render 80x24 grid: < 5ms
- Scroll 1000 lines: smooth 60fps

### Week 9-10: Input Handling

**Objective:** Handle all keyboard input, basic clipboard.

**Tasks:**
- [ ] Map JavaScript KeyboardEvent -> terminal byte sequences
- [ ] Regular characters: UTF-8 encoding
- [ ] Ctrl sequences: Ctrl+C (0x03), Ctrl+D (0x04), Ctrl+Z (0x1A), ...
- [ ] Special keys: Enter, Backspace, Tab, Escape
- [ ] Arrow keys -> ANSI escape sequences (application mode vs normal mode)
- [ ] Function keys (F1-F12) -> escape sequences
- [ ] Home, End, PageUp, PageDown, Insert, Delete
- [ ] Alt/Meta key combinations
- [ ] Numpad keys
- [ ] Handle IME (Input Method Editor) for CJK input
- [ ] Basic clipboard: Ctrl+Shift+C (copy), Ctrl+Shift+V (paste)
- [ ] Paste: handle multi-line paste, bracketed paste mode
- [ ] Key repeat behavior: respect OS key repeat settings

**Output:** Can comfortably use vim, htop, nano, less. Ctrl+C stops a process.

**Risks:**
- Keyboard handling differs between platforms (especially macOS Cmd key)
- IME handling is complex, may defer to Phase 3

### Week 11-12: Integration Testing and Bug Fixes

**Objective:** Stabilize, fix bugs, basic session management.

**Tasks:**
- [ ] Integration test suite: spawn terminal, run commands, verify output
- [ ] Test matrix: bash, zsh, fish, PowerShell
- [ ] Test interactive apps: vim, nano, htop, top, less, man
- [ ] Test color output: `ls --color`, git log, compiler output
- [ ] Fix rendering bugs discovered during testing
- [ ] Fix input handling bugs
- [ ] Basic session management: create new session, destroy session
- [ ] Handle shell exit gracefully (show message, allow restart)
- [ ] Handle process crash/hang
- [ ] Memory leak check: run terminal for extended period
- [ ] Code cleanup and documentation for Phase 1 code

**Output:** Terminal stable for basic use. Ready for Phase 2.

---

## Phase 1 Deliverables

| # | Deliverable | Description |
| - | ----------- | ----------- |
| 1 | `wit-pty` crate | PTY abstraction working on Windows/macOS/Linux |
| 2 | `wit-ansi` crate | ANSI parser state machine, unit tested |
| 3 | `wit-core` crate | Terminal grid logic, session management |
| 4 | `<TerminalGrid>` component | React component rendering terminal output |
| 5 | Input handler | Full keyboard input -> PTY byte mapping |
| 6 | CI pipeline | Build + test on 3 platforms |
| 7 | Integration tests | Basic test suite for PTY I/O |

---

## Tech Decisions to Make During This Phase

| Decision | Options | Deadline | Impact |
| -------- | ------- | -------- | ------ |
| Rendering strategy | DOM / Canvas 2D / WebGL | Week 7 | Affects all rendering code |
| PTY crate | Custom / `portable-pty` / hybrid | Week 3 | Affects platform support |
| IPC pattern | Tauri events / commands / hybrid | Week 3 | Affects architecture |
| State management (React) | Zustand / Jotai / signals | Week 7 | Affects frontend architecture |
| Font measurement | DOM measurement / hardcoded / Canvas | Week 7 | Affects grid accuracy |

---

## Detailed Known Risks

### 1. PTY on Windows (Risk: High)

**Issue:** ConPTY API has many quirks:
- Output may include unexpected escape sequences
- Resize behavior differs from Unix PTY
- Some legacy console applications do not work well

**Mitigation:**
- Carefully study Windows Terminal source code (ConPTY reference implementation)
- Use `portable-pty` crate as fallback
- Allocate an extra 1 week buffer for Windows-specific issues

### 2. ANSI Edge Cases (Risk: Medium)

**Issue:** Many terminal applications use undocumented or non-standard sequences.

**Mitigation:**
- Implement core set first (CSI, SGR, basic OSC)
- Log unknown sequences instead of crashing
- Add sequences incrementally as issues arise

### 3. WebView Performance (Risk: Medium)

**Issue:** Rendering terminal grid in WebView may be slow, especially with large output.

**Mitigation:**
- Prototype rendering early (Week 1-2)
- Benchmark with large output (10000+ lines)
- Have backup plan: Canvas 2D or WebGL rendering

---

## Definition of "Phase 1 Complete"

Phase 1 is considered complete when **all** of the following conditions are met:

1. **Shell spawn:** Open Wit -> see shell prompt on all 3 platforms
2. **Basic commands:** `ls`, `cd`, `cat`, `echo`, `grep` work normally
3. **Colored output:** `ls --color`, `git diff`, `gcc` error output display correct colors
4. **Interactive apps:** `vim`, `nano`, `htop` render correctly and are usable
5. **Input:** Ctrl+C, Ctrl+D, Ctrl+Z, arrow keys, function keys work
6. **Clipboard:** Copy and paste text works
7. **Resize:** Resize window -> terminal reflows correctly
8. **Stability:** No crash during 1 hour of normal use
9. **CI:** Build succeeds on GitHub Actions for all 3 platforms

**If any condition is missing**, Phase 1 is not complete. Fix before starting Phase 2.
