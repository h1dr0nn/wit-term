# System Architecture

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

Wit uses a **3-layer** architecture with clear communication between layers:

```
┌─────────────────────────────────────────────────────────────────────┐
│                        FRONTEND (React/TS)                         │
│                                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────────┐  │
│  │ Terminal  │  │ Session  │  │ Agent    │  │   Completion      │  │
│  │   View    │  │ Sidebar  │  │ Sidebar  │  │     Popup         │  │
│  └──────────┘  └──────────┘  └──────────┘  └───────────────────┘  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                    State Management (Zustand)                 │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│                        BRIDGE (Tauri IPC)                          │
│                                                                     │
│  Commands (invoke)          Events (emit/listen)                   │
│  Request ──────────►        ◄────────── Push notifications         │
│           ◄──────────       ──────────►                            │
│  Response                                                           │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│                        CORE (Rust)                                  │
│                                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────────┐  │
│  │   PTY    │  │ Terminal  │  │ Context  │  │   Completion      │  │
│  │ Manager  │  │ Emulator  │  │  Engine  │  │     Engine        │  │
│  └──────────┘  └──────────┘  └──────────┘  └───────────────────┘  │
│                                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────────┐  │
│  │  ANSI    │  │ Session  │  │  Config  │  │   Agent           │  │
│  │  Parser  │  │ Manager  │  │ Manager  │  │   Detection       │  │
│  └──────────┘  └──────────┘  └──────────┘  └───────────────────┘  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                    OS / Platform Layer                        │  │
│  │              (PTY, filesystem, processes)                     │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Layers

### Layer 1: Rust Core

Contains all heavy logic and performance-critical operations:

| Module | Responsibility |
|---|---|
| **PTY Manager** | Create and manage pseudo-terminal connections to shell processes |
| **Terminal Emulator** | State machine handling terminal emulation (VT100/xterm) |
| **ANSI Parser** | Parse escape sequences from PTY output into structured data |
| **Context Engine** | Detect project type, environment, git status |
| **Completion Engine** | Load, match, rank completions based on context and input |
| **Agent Detection** | Detect AI agent CLIs, parse output, watch filesystem, Wit Protocol |
| **Session Manager** | Manage lifecycle of multiple terminal sessions |
| **Config Manager** | Load/save user configuration, themes, keybindings |

**Principles:**
- All heavy I/O occurs in the Rust layer
- The frontend never directly accesses the file system or processes
- The state machine runs on a dedicated thread, does not block the main thread

### Layer 2: Tauri Bridge

Bridge between the Rust core and the React frontend:

| Mechanism | Direction | Used for |
|---|---|---|
| **Commands** | Frontend -> Rust | Request-response: create session, send input, get config |
| **Events** | Rust -> Frontend | Push: terminal output, context changes, completion updates |
| **State** | Shared | Tauri managed state, accessible from both sides |

**Principles:**
- Commands are synchronous from the caller's perspective (async internally)
- Events are fire-and-forget, the frontend subscribes to event channels
- Minimize data transfer - only send diffs instead of full state when possible

### Layer 3: React Frontend

Contains UI rendering and user interaction:

| Component | Responsibility |
|---|---|
| **Terminal View** | Render terminal grid, cursor, selection |
| **Session Sidebar** | Display and manage terminal sessions |
| **Context Sidebar** | Display environment info, git status |
| **Completion Popup** | Dropdown displaying completion suggestions |
| **Settings UI** | Configuration interface |

**Principles:**
- The frontend is a "thin client" - only renders and collects input
- Business logic resides in the Rust core
- State management is minimal, only for UI state

---

## Component Map

```
wit-term/
├── src-tauri/              # Rust core
│   ├── src/
│   │   ├── main.rs             # Tauri app entry point
│   │   ├── lib.rs              # Library root
│   │   ├── pty/                # PTY management
│   │   │   ├── mod.rs
│   │   │   ├── unix.rs             # Unix PTY (macOS, Linux)
│   │   │   └── windows.rs          # Windows ConPTY
│   │   ├── terminal/           # Terminal emulation
│   │   │   ├── mod.rs
│   │   │   ├── emulator.rs         # Core state machine
│   │   │   ├── grid.rs             # Grid/buffer management
│   │   │   ├── cell.rs             # Cell representation
│   │   │   ├── cursor.rs           # Cursor state
│   │   │   └── strip.rs            # ANSI stripping, grid-to-text, echo/prompt stripping, CWD extraction
│   │   ├── parser/             # ANSI parser
│   │   │   ├── mod.rs
│   │   │   ├── state_machine.rs    # Parser state machine
│   │   │   ├── params.rs           # Parameter parsing
│   │   │   └── handler.rs          # Action handlers
│   │   ├── context/            # Context detection
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs           # Context engine coordinator
│   │   │   ├── providers/          # Individual providers
│   │   │   │   ├── git.rs
│   │   │   │   ├── node.rs
│   │   │   │   ├── python.rs
│   │   │   │   ├── rust.rs
│   │   │   │   ├── docker.rs
│   │   │   │   └── generic.rs
│   │   │   └── watcher.rs          # File system watcher
│   │   ├── completion/         # Completion engine
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs           # Completion matching/ranking
│   │   │   ├── source.rs           # Completion data sources
│   │   │   ├── matcher.rs          # Fuzzy matching
│   │   │   └── data/              # Built-in completion data
│   │   ├── session/            # Session management
│   │   │   ├── mod.rs
│   │   │   └── manager.rs
│   │   ├── config/             # Configuration
│   │   │   ├── mod.rs
│   │   │   ├── settings.rs
│   │   │   ├── theme.rs
│   │   │   └── keybindings.rs
│   │   └── commands/           # Tauri commands (IPC handlers)
│   │       ├── mod.rs
│   │       ├── session.rs
│   │       ├── terminal.rs
│   │       ├── context.rs
│   │       └── config.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                    # React frontend
│   ├── main.tsx                # React entry point
│   ├── App.tsx                 # Root component
│   ├── components/
│   │   ├── terminal/           # Terminal rendering
│   │   │   ├── TerminalView.tsx
│   │   │   ├── BlocksView.tsx          # Warp-style command block rendering
│   │   │   ├── InputBar.tsx            # Command input bar with CWD display
│   │   │   ├── TerminalGrid.tsx
│   │   │   ├── TerminalCell.tsx
│   │   │   ├── Cursor.tsx
│   │   │   └── Selection.tsx
│   │   ├── sidebar/            # Sidebars
│   │   │   ├── SessionSidebar.tsx
│   │   │   ├── SessionItem.tsx
│   │   │   ├── ContextSidebar.tsx
│   │   │   └── ContextInfo.tsx
│   │   ├── completion/         # Completion UI
│   │   │   ├── CompletionPopup.tsx
│   │   │   ├── CompletionItem.tsx
│   │   │   └── InlineHint.tsx
│   │   └── common/             # Shared components
│   │       ├── Icon.tsx
│   │       ├── Tooltip.tsx
│   │       └── ScrollArea.tsx
│   ├── stores/                 # Zustand stores
│   │   ├── sessionStore.ts
│   │   ├── terminalStore.ts
│   │   ├── completionStore.ts
│   │   ├── contextStore.ts
│   │   └── settingsStore.ts
│   ├── hooks/                  # Custom React hooks
│   │   ├── useTerminal.ts
│   │   ├── useSession.ts
│   │   ├── useCompletion.ts
│   │   ├── useContext.ts
│   │   └── useKeybindings.ts
│   ├── utils/                  # Utilities
│   │   └── ansiParser.ts           # Frontend ANSI SGR parser for colored output
│   ├── lib/                    # Utilities
│   │   ├── tauri.ts                # Tauri IPC wrappers
│   │   ├── theme.ts                # Theme utilities
│   │   └── constants.ts            # App constants
│   └── styles/                 # Global styles
│       ├── globals.css
│       └── themes/
├── docs/                   # Documentation (this folder)
├── completions/            # Completion data files
│   ├── git.toml
│   ├── docker.toml
│   ├── node.toml
│   └── ...
├── themes/                 # Theme files
│   ├── wit-dark.toml
│   ├── wit-light.toml
│   └── ...
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.ts
└── README.md
```

---

## Communication Patterns

### Pattern 1: User types a key

```
User presses key
    │
    ▼
Frontend: keydown event
    │
    ├── Is it a completion trigger (Tab, Ctrl+Space)?
    │   YES ──► invoke("request_completions", { input, context })
    │           Rust: match + rank completions
    │           Return: completion list
    │           Frontend: show CompletionPopup
    │
    └── NO ──► invoke("send_input", { session_id, data: key })
              Rust: write to PTY master
              Shell: processes input
              Shell: writes output to PTY slave
              Rust: reads from PTY master
              Rust: parse ANSI sequences
              Rust: update terminal grid
              Rust: emit("terminal_output", { session_id, cells_changed })
              Frontend: update TerminalView
```

### Pattern 1b: Command Submission (Block Mode)

```
User presses Enter in InputBar
    │
    ▼
Frontend: submit command via invoke("submit_command", { sessionId, command, commandId })
    │
    ▼
Rust: atomically captures CaptureState (command_id, command, start_cursor_row, started_at)
      then writes command + "\r" to PTY
    │
    ▼
Shell: processes command, writes output to PTY
    │
    ▼
Rust: PTY read loop detects output, converts grid rows to text via grid_to_ansi_text()
      strips echo/prompt lines via strip.rs
      emits command_output_chunk { session_id, command_id, output } incrementally
    │
    ▼
Frontend: terminalStore.updateOutputChunk() appends to CapturedBlock
    │
    ▼
Rust: detects command completion (next prompt appears)
      emits command_output { session_id, command_id, output, duration_ms }
    │
    ▼
Frontend: terminalStore.finalizeOutput() marks block complete
          BlocksView renders CapturedOutputBlock with ANSI colors via AnsiOutput
```

### Pattern 2: User changes directory

```
Shell: cd /new/path
    │
    ▼
Rust: detect CWD change (via shell integration or /proc)
    │
    ▼
Context Engine: re-scan directory
    ├── Check marker files (.git, package.json, Cargo.toml, ...)
    ├── Update active context providers
    └── Reload relevant completion sets
    │
    ▼
Rust: emit("context_changed", { providers, cwd })
    │
    ▼
Frontend: update ContextSidebar, reload completion data
```

### Pattern 3: Session lifecycle

```
User clicks "New Tab"
    │
    ▼
Frontend: invoke("create_session", { shell, cwd })
    │
    ▼
Rust:
    ├── Spawn new PTY
    ├── Fork shell process attached to PTY
    ├── Create terminal emulator instance
    ├── Start I/O read loop on background thread
    ├── Run initial context detection
    └── Return session_id
    │
    ▼
Frontend:
    ├── Add session to SessionSidebar
    ├── Create TerminalView for session
    └── Subscribe to terminal_output events for session_id
```

---

## Threading Model

```
┌─────────────────────────────────────────────┐
│                Main Thread                    │
│                                               │
│  Tauri event loop                            │
│  Command handlers (brief)                    │
│  Event dispatching                           │
│                                               │
├─────────────────────────────────────────────┤
│            Per-Session Threads                │
│                                               │
│  Thread A: PTY I/O read loop                 │
│    - Reads PTY output continuously           │
│    - Feeds bytes to ANSI parser              │
│    - Updates terminal grid                   │
│    - Emits render events                     │
│                                               │
│  Thread B: PTY I/O write                     │
│    - Receives input from frontend            │
│    - Writes to PTY master                    │
│                                               │
├─────────────────────────────────────────────┤
│            Shared Threads                     │
│                                               │
│  Thread C: Context engine                    │
│    - File system watching                    │
│    - Periodic context re-evaluation          │
│                                               │
│  Thread D: Completion engine                 │
│    - Background matching/ranking             │
│    - Completion data loading                 │
│                                               │
└─────────────────────────────────────────────┘
```

**Synchronization:**
- Terminal grid: `Arc<Mutex<Grid>>` or lock-free ring buffer
- Context state: `Arc<RwLock<Context>>` - many readers, few writers
- Completion requests: channel-based (crossbeam or tokio mpsc)
- Event emission: through the Tauri event system (thread-safe)

---

## Error Handling Strategy

| Layer | Strategy |
|---|---|
| **PTY I/O** | Reconnect on failure, notify user. PTY death -> session cleanup |
| **ANSI Parser** | Silently skip malformed sequences. Log in debug mode. Never crash |
| **Context Engine** | Graceful degradation - if detection fails, use empty context |
| **Completion Engine** | Return empty list on error. Never block input |
| **Tauri Commands** | Return `Result<T, String>` - frontend displays errors in UI |
| **Frontend** | Error boundaries per component. Terminal view never crashes |

### Crash Recovery

- Session state (scrollback) can be persisted periodically
- PTY reconnection when the session is still alive
- Config file corruption -> fallback to defaults + notify user
