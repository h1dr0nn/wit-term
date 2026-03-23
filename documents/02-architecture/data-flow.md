# Data Flow

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

This document describes the end-to-end data flows in Wit for the main use cases.
Each flow is presented as a sequence diagram.

---

## Flow 1: User Input -> Terminal Output

The most basic flow - the user presses a key, the shell processes it, and output appears on screen.

```
┌──────────┐    ┌───────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│ Keyboard │    │  Frontend │    │  Tauri   │    │   Rust   │    │  Shell   │
│  Event   │    │  (React)  │    │  Bridge  │    │   Core   │    │ Process  │
└────┬─────┘    └─────┬─────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘
     │                │               │               │               │
     │  keydown "a"   │               │               │               │
     ├───────────────►│               │               │               │
     │                │  encodeKey()  │               │               │
     │                ├──────────────►│               │               │
     │                │  invoke       │               │               │
     │                │  "send_input" │               │               │
     │                │  { data: "a" }│               │               │
     │                │               ├──────────────►│               │
     │                │               │  pty.write(b"a")              │
     │                │               │               ├──────────────►│
     │                │               │               │               │
     │                │               │               │  echo "a"    │
     │                │               │               │  + process   │
     │                │               │               │◄──────────────┤
     │                │               │               │               │
     │                │               │  pty.read()   │               │
     │                │               │  bytes: "a"   │               │
     │                │               │               │               │
     │                │               │  parse ANSI   │               │
     │                │               │  update grid  │               │
     │                │               │               │               │
     │                │  emit event   │               │               │
     │                │  "terminal_   │◄──────────────┤               │
     │                │   output"     │               │               │
     │                │◄──────────────┤               │               │
     │                │               │               │               │
     │  render cell   │               │               │               │
     │◄───────────────┤               │               │               │
     │                │               │               │               │
```

### Timing breakdown

| Step | Expected Time |
|---|---|
| Keydown -> encodeKey | < 1ms |
| invoke -> Rust | < 1ms |
| PTY write | < 1ms |
| Shell echo | < 1ms |
| PTY read | variable (buffered) |
| ANSI parse + grid update | < 1ms |
| Event emit -> Frontend | < 1ms |
| React render | < 5ms |
| **Total** | **< 16ms** (60fps target) |

---

## Flow 2: Tab Completion

The user presses Tab to trigger completion.

```
User presses Tab
     │
     ▼
Frontend: detect Tab key
     │
     ├── Get current input line from terminal buffer
     │   (or tracked input if shell integration is available)
     │
     ▼
Frontend: invoke("request_completions", {
  input: "git che",
  cursorPos: 7,
  sessionId: "abc123"
})
     │
     ▼
Rust: CompletionEngine.complete()
     │
     ├── 1. Parse input line
     │   command: "git"
     │   subcommand_prefix: "che"
     │
     ├── 2. Get active context
     │   providers: [git, node]
     │   cwd: "/home/user/project"
     │
     ├── 3. Query completion sources:
     │   ├── StaticSource: git subcommands matching "che*"
     │   │   -> ["checkout", "cherry", "cherry-pick"]
     │   ├── HistorySource: recent commands matching "git che*"
     │   │   -> ["checkout main", "checkout -b feature"]
     │   └── PathSource: N/A (not in path position)
     │
     ├── 4. Merge and rank results
     │   Score factors:
     │   - Exact prefix match bonus
     │   - Frecency from history
     │   - Context relevance
     │
     └── 5. Return ranked list
         [
           { text: "checkout", score: 0.95 },
           { text: "cherry-pick", score: 0.72 },
           { text: "cherry", score: 0.70 },
         ]
     │
     ▼
Frontend:
     ├── If single result -> auto-complete
     ├── If multiple results -> show CompletionPopup
     └── If no results -> do nothing (or beep)
```

---

## Flow 3: Context Detection

When the user changes the working directory or initializes a new session.

```
CWD changes to /home/user/my-rust-project
     │
     ▼
Rust: ContextEngine detects CWD change
     │ (via shell integration OSC, /proc, or polling)
     │
     ▼
ContextEngine: scan directory
     │
     ├── Walk up from CWD to root, check each directory:
     │
     │   /home/user/my-rust-project/
     │   ├── .git/               -> GitProvider.detect() = TRUE
     │   ├── Cargo.toml          -> RustProvider.detect() = TRUE
     │   ├── Dockerfile          -> DockerProvider.detect() = TRUE
     │   ├── package.json?       -> NodeProvider.detect() = FALSE
     │   └── pyproject.toml?     -> PythonProvider.detect() = FALSE
     │
     ├── For each active provider, gather details:
     │   ├── GitProvider.gather():
     │   │   branch: "main"
     │   │   status: "dirty" (3 modified files)
     │   │   remote: "origin"
     │   │
     │   ├── RustProvider.gather():
     │   │   name: "my-project"
     │   │   edition: "2021"
     │   │   targets: ["bin"]
     │   │
     │   └── DockerProvider.gather():
     │       base_image: "rust:1.78"
     │       compose: false
     │
     ├── Determine completion sets to load:
     │   ├── git completions (always for git repos)
     │   ├── cargo completions (Rust project)
     │   ├── docker completions (Dockerfile present)
     │   └── rustup completions (Rust project)
     │
     └── Update context state
     │
     ▼
Rust: emit("context_changed", {
  sessionId: "abc123",
  context: {
    cwd: "/home/user/my-rust-project",
    projectType: "rust",
    providers: [
      { name: "git", active: true, details: { branch: "main", ... } },
      { name: "rust", active: true, details: { edition: "2021", ... } },
      { name: "docker", active: true, details: { ... } },
    ],
    gitInfo: { branch: "main", status: "dirty", ... }
  }
})
     │
     ▼
Frontend:
     ├── Update ContextSidebar with new provider info
     ├── Update session title (optional)
     └── CompletionEngine now uses new completion sets
```

---

## Flow 4: Session Lifecycle

Creating, using, and closing a terminal session.

```
                     CREATE
                       │
                       ▼
Frontend: invoke("create_session", {
  shell: "/bin/zsh",
  cwd: "/home/user",
  cols: 80,
  rows: 24
})
     │
     ▼
Rust: SessionManager.create_session()
     │
     ├── 1. Create PTY pair
     │   master_fd, slave_fd = openpty()
     │
     ├── 2. Fork shell process
     │   child = fork()
     │   child: exec("/bin/zsh") with slave_fd
     │   parent: close slave_fd
     │
     ├── 3. Create terminal emulator
     │   grid = Grid::new(80, 24)
     │   parser = AnsiParser::new()
     │
     ├── 4. Start I/O thread
     │   spawn(pty_read_loop(master_fd, parser, grid))
     │
     ├── 5. Run context detection
     │   context = ContextEngine::scan("/home/user")
     │
     └── 6. Return session_id
     │
     ▼
Frontend:
     ├── Add to SessionSidebar
     ├── Create TerminalView
     ├── Subscribe to events
     └── Focus terminal
                       │
                       │  ... user works ...
                       │
                     DESTROY
                       │
                       ▼
Frontend: invoke("destroy_session", { sessionId })
  OR
Shell process exits naturally (exit, Ctrl+D)
     │
     ▼
Rust: SessionManager.destroy_session()
     │
     ├── 1. Signal read thread to stop
     │   shutdown_tx.send(())
     │
     ├── 2. Close PTY master fd
     │   close(master_fd)
     │
     ├── 3. Wait for child process
     │   waitpid(child_pid) -> exit_code
     │
     ├── 4. Clean up resources
     │   Drop grid, parser, context
     │
     └── 5. Emit exit event
         emit("session_exited", { sessionId, exitCode })
     │
     ▼
Frontend:
     ├── Remove from SessionSidebar
     ├── Destroy TerminalView
     ├── Unsubscribe events
     └── Focus next session (or show empty state)
```

---

## Flow 5: Configuration Change

The user changes settings (font size, theme, etc.).

```
User changes font size in Settings UI
     │
     ▼
Frontend: invoke("set_config", {
  fontSize: 16
})
     │
     ▼
Rust: ConfigManager.update()
     │
     ├── 1. Validate new value
     │   fontSize: 16 (within 10-24 range) ✓
     │
     ├── 2. Merge with existing config
     │   config.fontSize = 16
     │
     ├── 3. Persist to disk
     │   write ~/.config/wit/config.toml
     │
     └── 4. Emit config change event
         emit("config_changed", { key: "fontSize", value: 16 })
     │
     ▼
Frontend:
     ├── settingsStore.updateSetting("fontSize", 16)
     ├── TerminalView re-renders with new font size
     ├── Recalculate grid dimensions
     │   new cols/rows based on new cell size
     └── invoke("resize_session", { sessionId, cols, rows })
         for each active session
```

---

## State Ownership

| State | Owner | Access |
|---|---|---|
| Terminal grid | Rust (Arc<Mutex<Grid>>) | Rust writes, Frontend reads (via events) |
| Cursor position | Rust | Rust writes, Frontend reads (via events) |
| Scrollback buffer | Rust | Frontend reads (via command) on demand |
| Session list | Rust (SessionManager) | Frontend reads (via command/events) |
| Active session | Frontend (Zustand) | Frontend-only state |
| Context data | Rust (Arc<RwLock>) | Rust writes, Frontend reads (via events) |
| Completion results | Rust (computed) | Frontend reads (via command response) |
| UI layout state | Frontend (Zustand) | Frontend-only state |
| Config/Settings | Rust (on-disk) | Both read, Rust persists |
| Theme data | Rust (loaded from disk) | Frontend reads, applies to CSS |
