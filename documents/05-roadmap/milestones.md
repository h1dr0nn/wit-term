# Wit Terminal - Milestones

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Milestones Overview

```
Month:  1     2     3     4     5     6     7     8     9     10    11    12
        |-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|

   M1 --*                                    First Light (Week 4)
   M2 --------*                               Color TV (Week 8)
   M3 ----------*                             Keyboard Warrior (Week 10)
   M4 ------------*                           Foundation Complete (Month 3)
   M5 ------------------*                     Context Aware (Week 16)
   M6 ------------------------*               Smart Tab (Week 24)
   M7 ------------------------*               Phase 2 Complete (Month 6)
   M8 ----------------------------------*     Daily Driver (Month 9)
   M9 --------------------------------------------------*  v0.1.0 (Month 12)
```

---

## M1: "First Light"

| Field              | Value                                     |
| ------------------ | ----------------------------------------- |
| **Target**         | End of Week 4 (end of month 1)            |
| **Phase**          | Phase 1: Foundation                       |
| **Dependencies**   | None                                      |
| **Significance**   | First time seeing a shell prompt in Wit    |

### Acceptance Criteria

- [ ] Run `cargo tauri dev` -> Wit window opens successfully
- [ ] Shell prompt displays in window (bash, zsh, or PowerShell depending on OS)
- [ ] Type `echo hello` + Enter -> see "hello" appear
- [ ] Type `ls` + Enter -> see file listing (color not yet required)
- [ ] Type `pwd` + Enter -> see current directory
- [ ] PTY process terminates cleanly when closing window
- [ ] Works on at least 1 platform (Linux or macOS)

### Testable Scenarios

```
Scenario 1: Basic echo
  1. Open Wit
  2. Type: echo "Hello from Wit"
  3. Expected: see "Hello from Wit" in output
  4. Pass: YES / NO

Scenario 2: File listing
  1. Open Wit
  2. Type: ls -la
  3. Expected: see file listing with permissions
  4. Pass: YES / NO

Scenario 3: Clean exit
  1. Open Wit
  2. Close window (click X or Ctrl+Q)
  3. Expected: no zombie process, app exits cleanly
  4. Pass: YES / NO
```

---

## M2: "Color TV"

| Field              | Value                                          |
| ------------------ | ---------------------------------------------- |
| **Target**         | End of Week 8 (end of month 2)                |
| **Phase**          | Phase 1: Foundation                            |
| **Dependencies**   | M1 (First Light)                               |
| **Significance**   | Terminal renders ANSI colors correctly, vim works |

### Acceptance Criteria

- [ ] `ls --color=auto` shows files with correct colors (directories, executables, symlinks)
- [ ] `git log --oneline --graph --all` renders graph characters and colors correctly
- [ ] `git diff` shows additions (green) and deletions (red) with correct colors
- [ ] 16 ANSI colors render accurately
- [ ] 256 color mode works (test: `for i in {0..255}; do printf "\033[38;5;${i}m%3d " $i; done`)
- [ ] True color (24-bit) works (test: ANSI true color gradient scripts)
- [ ] Bold, italic, underline attributes render correctly
- [ ] `vim` opens a file -> syntax highlighting displays -> can navigate and edit
- [ ] `vim` exits normally with `:q`
- [ ] `htop` or `top` renders correctly (bars, colors, layout)
- [ ] Cursor displays at the correct position in all above scenarios

### Testable Scenarios

```
Scenario 1: Colored ls
  1. cd into a directory with many file types
  2. Type: ls --color=auto
  3. Expected: directories = blue, executables = green, symlinks = cyan
  4. Pass: YES / NO

Scenario 2: Vim workflow
  1. Type: vim test.py
  2. Type: i (enter insert mode)
  3. Type: print("hello")
  4. Press: Esc
  5. Type: :wq
  6. Expected: syntax highlighted, file saved
  7. Pass: YES / NO

Scenario 3: True color
  1. Run true color test script
  2. Expected: smooth color gradient, no banding
  3. Pass: YES / NO
```

---

## M3: "Keyboard Warrior"

| Field              | Value                                            |
| ------------------ | ------------------------------------------------ |
| **Target**         | End of Week 10 (mid month 3)                    |
| **Phase**          | Phase 1: Foundation                              |
| **Dependencies**   | M2 (Color TV)                                    |
| **Significance**   | All keyboard input works correctly               |

### Acceptance Criteria

- [ ] Ctrl+C stops a running process (test: `sleep 100`, Ctrl+C, prompt returns)
- [ ] Ctrl+D sends EOF (test: `cat`, type text, Ctrl+D, cat exits)
- [ ] Ctrl+Z suspends process (test: `vim`, Ctrl+Z, `fg` to resume)
- [ ] Ctrl+L clears screen
- [ ] Arrow keys: navigate command history (up/down), move cursor (left/right)
- [ ] Home / End: beginning / end of line
- [ ] Page Up / Page Down: scroll in less/man
- [ ] Function keys (F1-F12): work in apps that use them (htop, mc)
- [ ] Tab: shell completion (built-in shell tab, not yet Wit completion)
- [ ] Alt+B / Alt+F: word navigation in readline
- [ ] Ctrl+Shift+C: copy selected text
- [ ] Ctrl+Shift+V: paste clipboard
- [ ] Bracketed paste: paste multi-line text correctly

### Testable Scenarios

```
Scenario 1: Process control
  1. Type: sleep 100
  2. Press: Ctrl+C
  3. Expected: sleep stops, prompt returns immediately
  4. Pass: YES / NO

Scenario 2: Clipboard round-trip
  1. Type: echo "test clipboard"
  2. Select "test clipboard" text
  3. Ctrl+Shift+C
  4. Type: echo "pasted: " (no Enter)
  5. Ctrl+Shift+V
  6. Enter
  7. Expected: see "pasted: test clipboard"
  8. Pass: YES / NO

Scenario 3: vim keys
  1. Type: vim
  2. Test: arrow keys, Page Up/Down, Home/End
  3. Test: Esc, i, :, /
  4. Test: Ctrl+W (window commands)
  5. Expected: all keys produce correct vim behavior
  6. Pass: YES / NO

Scenario 4: readline navigation
  1. Type: echo "one two three four five"
  2. Press: Home -> cursor goes to start
  3. Press: End -> cursor goes to end
  4. Press: Ctrl+A -> cursor goes to start
  5. Press: Alt+F -> cursor moves forward one word
  6. Expected: all navigation correct
  7. Pass: YES / NO
```

---

## M4: "Foundation Complete"

| Field              | Value                                          |
| ------------------ | ---------------------------------------------- |
| **Target**         | End of Month 3 (end of Week 12)               |
| **Phase**          | Phase 1: Foundation (end)                      |
| **Dependencies**   | M1, M2, M3                                     |
| **Significance**   | Basic terminal complete, ready for Phase 2     |

### Acceptance Criteria

- [ ] All criteria from M1, M2, M3 still pass
- [ ] Window resize -> terminal reflows correctly (text wraps correctly)
- [ ] Session create: can open a new terminal session
- [ ] Session destroy: close session cleanly, no leaked resources
- [ ] Shell exit handling: shell exits -> friendly message, no crash
- [ ] Scrollback: scroll up to see old output (at least 1000 lines)
- [ ] CI pipeline: build succeeds on macOS, Linux, Windows
- [ ] Integration tests pass on at least 2 platforms
- [ ] No memory leak: run for 1 hour, memory usage is stable
- [ ] Error handling: app does not crash with malformed input or edge cases

### Testable Scenarios

```
Scenario 1: Extended stability
  1. Open Wit
  2. Run: for i in $(seq 1 1000); do echo "line $i"; done
  3. Scroll up -> see line 1
  4. Scroll down -> see line 1000
  5. Note memory usage
  6. Repeat 5 times
  7. Expected: memory does not increase significantly, no crash
  8. Pass: YES / NO

Scenario 2: Resize reflow
  1. Type: echo "this is a long line that should wrap when the window is narrow"
  2. Resize window narrower
  3. Expected: text wraps at window edge
  4. Resize window wider
  5. Expected: text unwraps (or stays wrapped, acceptable)
  6. Pass: YES / NO

Scenario 3: Cross-platform
  1. Build on macOS -> run tests -> pass
  2. Build on Linux -> run tests -> pass
  3. Build on Windows -> run tests -> pass
  4. Pass: YES / NO (per platform)
```

---

## M5: "Context Aware"

| Field              | Value                                            |
| ------------------ | ------------------------------------------------ |
| **Target**         | End of Week 16 (mid month 4)                    |
| **Phase**          | Phase 2: Context                                 |
| **Dependencies**   | M4 (Foundation Complete)                         |
| **Significance**   | Wit detects project type and displays context info |

### Acceptance Criteria

- [ ] `cd` into a git repo -> Wit detects "Git repository"
- [ ] Git context shows: branch name, clean/dirty status
- [ ] `cd` into a Node project -> Wit detects "Node.js project"
- [ ] Node context shows: package name, Node version (if available)
- [ ] `cd` into a Rust project -> Wit detects "Rust project"
- [ ] Rust context shows: crate name
- [ ] `cd` into a directory with Dockerfile -> Wit detects "Docker project"
- [ ] `cd` into a Python project -> Wit detects "Python project"
- [ ] Multiple contexts: git + Node -> both detected
- [ ] Context updates when `cd`-ing to a different directory (< 200ms)
- [ ] Context detection does not affect terminal performance

### Testable Scenarios

```
Scenario 1: Git detection
  1. cd into a git repo (e.g., ~/projects/wit-term)
  2. Expected: UI shows git branch name
  3. Type: git checkout -b test-branch
  4. Expected: branch name updates to "test-branch"
  5. Pass: YES / NO

Scenario 2: Multi-context
  1. cd into a directory with both .git and package.json
  2. Expected: detects both Git and Node.js
  3. Pass: YES / NO

Scenario 3: Context switch
  1. cd into a git repo -> see git context
  2. cd into a Rust project -> see Rust context, git context (if also has .git)
  3. cd ~ -> context clears (or minimal)
  4. Expected: each cd, context updates < 200ms
  5. Pass: YES / NO
```

---

## M6: "Smart Tab"

| Field              | Value                                              |
| ------------------ | -------------------------------------------------- |
| **Target**         | End of Week 24 (end of month 6)                   |
| **Phase**          | Phase 2: Context                                   |
| **Dependencies**   | M5 (Context Aware)                                 |
| **Significance**   | Context-aware tab completion works                  |

### Acceptance Criteria

- [ ] Type `git ` + Tab -> popup shows git subcommands (commit, push, pull, ...)
- [ ] Type `git co` + Tab -> auto-completes to `git commit` (or shows popup if ambiguous)
- [ ] Type `git checkout ` + Tab -> shows list of actual branches
- [ ] Type `npm run ` + Tab in a Node project -> shows scripts from package.json
- [ ] Type `cargo ` + Tab in a Rust project -> shows cargo subcommands
- [ ] Fuzzy matching: `git chk` + Tab -> suggests `checkout`
- [ ] Inline hint: ghost text shows for top suggestion
- [ ] Completion popup: keyboard navigable (Up/Down, Enter, Esc)
- [ ] Popup shows description for each completion item
- [ ] Completion response time < 50ms

### Testable Scenarios

```
Scenario 1: Git completions
  1. cd into a git repo
  2. Type: git (space)
  3. Press: Tab
  4. Expected: popup shows subcommands (add, commit, push, pull, ...)
  5. Type: co
  6. Expected: popup filters (commit, config, ...)
  7. Press: Enter on "commit"
  8. Expected: command line = "git commit "
  9. Pass: YES / NO

Scenario 2: Dynamic branch completion
  1. cd into a git repo with branches: main, develop, feature/auth
  2. Type: git checkout (space)
  3. Press: Tab
  4. Expected: popup shows main, develop, feature/auth
  5. Type: fea
  6. Expected: popup filters to feature/auth
  7. Pass: YES / NO

Scenario 3: npm scripts
  1. cd into a Node project with scripts: dev, build, test, lint
  2. Type: npm run (space)
  3. Press: Tab
  4. Expected: popup shows dev, build, test, lint
  5. Pass: YES / NO

Scenario 4: Fuzzy matching
  1. Type: git chk
  2. Press: Tab
  3. Expected: suggests "checkout" (fuzzy match)
  4. Pass: YES / NO
```

---

## M7: "Phase 2 Complete"

| Field              | Value                                         |
| ------------------ | --------------------------------------------- |
| **Target**         | End of Month 6 (end of Week 24)              |
| **Phase**          | Phase 2: Context (end)                        |
| **Dependencies**   | M5, M6                                        |
| **Significance**   | Completion system operational, context-aware   |

### Acceptance Criteria

- [ ] All criteria from M5 and M6 still pass
- [ ] At least 10 command groups have completions
- [ ] Shell integration scripts for bash, zsh, fish, PowerShell
- [ ] CWD tracking works via OSC 7
- [ ] Command history integration: frequent commands rank higher
- [ ] Completion data format documented
- [ ] Context engine stable - no crashes, no false detection
- [ ] Performance: completion < 50ms, context detection < 100ms

### Testable Scenarios

```
Scenario 1: Full workflow
  1. Open Wit
  2. cd into a project with git + Node
  3. Type git commands with Tab -> completions work
  4. Type npm commands with Tab -> completions work
  5. cd to a Rust project
  6. Type cargo commands with Tab -> completions work
  7. Everything fast, smooth, no lag
  8. Pass: YES / NO

Scenario 2: 10 command groups
  Verify completions for:
  - [ ] git    - [ ] npm     - [ ] cargo
  - [ ] docker - [ ] kubectl - [ ] ssh
  - [ ] cd/ls  - [ ] make    - [ ] pip
  - [ ] systemctl
```

---

## M8: "Daily Driver"

| Field              | Value                                       |
| ------------------ | ------------------------------------------- |
| **Target**         | End of Month 9 (end of Week 36)            |
| **Phase**          | Phase 3: Polish (end)                       |
| **Dependencies**   | M7 (Phase 2 Complete)                       |
| **Significance**   | Polished enough to replace the default terminal |

### Acceptance Criteria

- [ ] All criteria from previous milestones still pass
- [ ] Multi-session: open 5+ sessions, switch smoothly
- [ ] Split panes: horizontal and vertical splits work
- [ ] At least 8 themes available, hot-reload works
- [ ] Text selection + copy/paste works on 3 platforms
- [ ] Search (Ctrl+Shift+F): find text in scrollback
- [ ] URL detection + Ctrl+click -> opens browser
- [ ] Settings UI: font, theme, keybindings customizable
- [ ] Command palette (Ctrl+Shift+P) works
- [ ] Startup < 500ms, input latency < 16ms, memory < 100MB for 5 sessions
- [ ] 8 hours of continuous use without crash
- [ ] Developer (author) has used Wit for daily work for at least 1 week

### Testable Scenarios

```
Scenario 1: Daily work simulation
  1. Open Wit
  2. Open 3 sessions: dev server, editor, git operations
  3. Use continuously for 4 hours
  4. Switch between sessions, copy/paste, search
  5. Expected: no crash, no lag, no frustration
  6. Pass: YES / NO

Scenario 2: Performance check
  - Startup time: ___ms (target: < 500ms)
  - Input latency: ___ms (target: < 16ms)
  - Memory (5 sessions): ___MB (target: < 100MB)
  - Pass: YES / NO
```

---

## M9: "v0.1.0"

| Field              | Value                                     |
| ------------------ | ----------------------------------------- |
| **Target**         | End of Month 12 (end of Week 48)         |
| **Phase**          | Phase 4: Ecosystem (end)                  |
| **Dependencies**   | M8 (Daily Driver)                         |
| **Significance**   | First public release                       |

### Acceptance Criteria

- [ ] All criteria from previous milestones still pass
- [ ] Plugin API documented, 1+ example plugin works
- [ ] CONTRIBUTING.md exists with clear contribution workflow
- [ ] Validation tooling: `wit validate` works
- [ ] **Packages available:**
  - [ ] macOS: .dmg (signed + notarized)
  - [ ] Linux: .deb + .AppImage
  - [ ] Windows: .msi
- [ ] Auto-update works
- [ ] README.md has:
  - [ ] Demo GIF
  - [ ] Installation instructions for 3 platforms
  - [ ] CI badge, version badge, license badge
- [ ] Documentation site live
- [ ] GitHub Release v0.1.0 published
- [ ] At least 3 beta testers have tried it and given feedback
- [ ] At least 1 community post published (Reddit/HN)
- [ ] No known critical bugs

### Testable Scenarios

```
Scenario 1: Fresh install (per platform)
  1. Download package from GitHub Releases
  2. Install following instructions
  3. Open Wit for the first time
  4. Shell prompt appears
  5. Type commands, Tab completions work
  6. Switch theme
  7. Open multiple sessions
  8. Expected: everything works out of the box
  9. Pass: YES / NO (macOS / Linux / Windows)

Scenario 2: New user journey
  1. Find Wit on GitHub
  2. Read README -> understand what Wit does
  3. Download -> install -> open
  4. Use for 15 minutes -> "aha moment" with completions
  5. Expected: positive first impression
  6. Pass: YES / NO

Scenario 3: Contributor journey
  1. Read CONTRIBUTING.md
  2. Fork repo, add completion for 1 command
  3. Run `wit validate` -> pass
  4. Submit PR
  5. Expected: clear process, no confusion
  6. Pass: YES / NO
```

---

## Milestone Dependency Graph

```
M1 (First Light)
 └──> M2 (Color TV)
       └──> M3 (Keyboard Warrior)
             └──> M4 (Foundation Complete)
                   └──> M5 (Context Aware)
                         └──> M6 (Smart Tab)
                               └──> M7 (Phase 2 Complete)
                                     └──> M8 (Daily Driver)
                                           └──> M9 (v0.1.0)
```

Each milestone builds on the previous ones. None can be skipped. If a milestone is delayed, all subsequent milestones are affected.

---

## Tracking

Update status for each milestone when achieved:

| Milestone | Target | Actual | Status | Notes |
| --------- | ------ | ------ | ------ | ----- |
| M1: First Light | Week 4 | - | Pending | |
| M2: Color TV | Week 8 | - | Pending | |
| M3: Keyboard Warrior | Week 10 | - | Pending | |
| M4: Foundation Complete | Week 12 | - | Pending | |
| M5: Context Aware | Week 16 | - | Pending | |
| M6: Smart Tab | Week 24 | - | Pending | |
| M7: Phase 2 Complete | Week 24 | - | Pending | |
| M8: Daily Driver | Week 36 | - | Pending | |
| M9: v0.1.0 | Week 48 | - | Pending | |
