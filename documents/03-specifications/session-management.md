# Session Management

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

A session is the most fundamental unit in Wit. Each session represents a complete terminal working session - consisting of a PTY process, a shell, a terminal emulator instance, and a context detection scope.

This document describes how sessions are created, managed, switched, and destroyed.

---

## Session Concept

### Definition

A **Session** consists of:

```
┌──────────────────────────────────────────────────────────────┐
│                         Session                               │
│                                                               │
│  ┌─────────┐  ┌──────────────┐  ┌──────────┐  ┌──────────┐ │
│  │   PTY   │  │   Terminal   │  │ Context  │  │ Session  │  │
│  │ Process │  │   Emulator   │  │  State   │  │  Config  │  │
│  │         │  │   (VT state) │  │          │  │          │  │
│  │ shell   │  │ screen buffer│  │ cwd      │  │ title    │  │
│  │ stdin   │  │ cursor pos   │  │ git info │  │ shell    │  │
│  │ stdout  │  │ scroll hist  │  │ project  │  │ env vars │  │
│  │ stderr  │  │ attributes   │  │ type     │  │ cwd      │  │
│  └─────────┘  └──────────────┘  └──────────┘  └──────────┘  │
│                                                               │
│  ID: uuid-v4                                                  │
│  State: Active | Background | Exited                          │
│  Created: timestamp                                           │
└──────────────────────────────────────────────────────────────┘
```

### 1:1 Relationship

| Component | Count per Session |
|---|---|
| PTY process | 1 (exactly one shell process) |
| Terminal emulator | 1 (separate VT state machine) |
| Screen buffer | 1 (separate scrollback history) |
| Context state | 1 (separate CWD, git info, project type) |
| Tab (UI) | 1 (one tab corresponds to one session) |

Sessions do **not** share state with each other. Each session is completely isolated.

---

## Session Identification

### Session ID

```rust
pub struct SessionId(uuid::Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
```

- Each session has a unique UUID v4
- The ID is generated at session creation and does not change throughout the lifecycle
- Used to reference sessions in IPC commands and events

### Display Title

```rust
pub struct SessionTitle {
    /// User-set name (if any)
    custom: Option<String>,

    /// Auto-generated from shell + CWD
    auto: String,
}

impl SessionTitle {
    /// Display title: prefer custom, fallback to auto
    pub fn display(&self) -> &str {
        self.custom.as_deref().unwrap_or(&self.auto)
    }
}
```

**Auto-generated title rules:**

1. If shell integration is available (CWD tracking): show the current directory name
   - Example: `~/Projects/wit-term` - title = `wit-term`
2. If no shell integration: show the shell name
   - Example: `bash`, `zsh`, `powershell`
3. If a foreground process is running (not the shell itself): show the process name
   - Example: `vim`, `htop`, `npm run dev`

**Custom title:**

- Users can set a name via UI (double-click tab title)
- Or via OSC escape sequence: `\e]0;My Title\a`
- Custom title overrides auto title until the user clears the custom title

---

## Session Lifecycle

### State Machine

```
                    ┌──────────┐
       create() ──►│ Creating │
                    └────┬─────┘
                         │ PTY ready
                         v
                    ┌──────────┐
              ┌────│  Active  │◄────┐
              │    └────┬─────┘     │
              │         │           │
    switch()  │         │ switch()  │ switch()
    to other  │         │ to other  │ back
              │         v           │
              │    ┌──────────┐     │
              └───►│Background│─────┘
                   └────┬─────┘
                        │ shell exits
                        │ or destroy()
                        v
                   ┌──────────┐
                   │  Exited  │
                   └────┬─────┘
                        │ cleanup complete
                        v
                   ┌──────────┐
                   │ Destroyed│ (removed from memory)
                   └──────────┘
```

### States

| State | Description | PTY | UI |
|---|---|---|---|
| **Creating** | Spawning shell process, not yet ready | Initializing | Tab shows spinner |
| **Active** | Session is displayed, receiving input | Running, I/O active | Tab selected, terminal visible |
| **Background** | Session is running but not displayed | Running, output buffered | Tab unselected, badge if new output |
| **Exited** | Shell process has exited | Stopped | Tab shows exit indicator |
| **Destroyed** | Session has been cleaned up, removed | Cleaned up | Tab removed |

### Transitions

#### Create - Active

```rust
pub struct CreateSessionOptions {
    /// Shell to use (None = default shell)
    shell: Option<String>,

    /// Working directory (None = home directory)
    cwd: Option<PathBuf>,

    /// Environment variable overrides
    env: HashMap<String, String>,

    /// Custom title
    title: Option<String>,

    /// Terminal size (cols, rows)
    size: (u16, u16),
}

impl SessionManager {
    pub async fn create_session(
        &mut self,
        options: CreateSessionOptions,
    ) -> Result<SessionId> {
        // 1. Generate SessionId
        // 2. Spawn PTY with shell
        // 3. Create terminal emulator instance
        // 4. Initialize context detection
        // 5. Inject shell integration scripts (if enabled)
        // 6. Set state to Active
        // 7. Emit SessionCreated event
        // 8. Emit SessionActivated event
    }
}
```

**Target creation time:** < 200ms from user action to terminal ready.

#### Active - Background

When the user switches to another session:

1. Record cursor position and viewport scroll
2. PTY continues running; output is still read and processed by the terminal emulator
3. But the terminal emulator **does not send render events** to the frontend (saving CPU/IPC)
4. If there is new output - increment the **unread badge counter** on the tab

#### Background - Active

When the user switches back to the session:

1. Send **full screen state** to frontend (since the frontend was not tracking while in background)
2. Restore cursor position and viewport scroll
3. Reset unread badge counter
4. Resume sending render events

#### Active/Background - Exited

When the shell process exits (user types `exit`, or the process is killed):

1. Read remaining PTY output (flush buffer)
2. Capture exit code
3. Set state to Exited
4. Emit `SessionExited { id, exit_code }` event
5. **Do not auto-destroy** - the tab remains visible so the user can view the final output
6. Tab shows exit code badge: checkmark (code 0) or X with code (code != 0)
7. User must manually close the tab to destroy the session

#### Exited - Destroyed

When the user closes the tab (or calls destroy):

1. Cleanup PTY resources
2. Drop terminal emulator state
3. Remove tab from UI
4. Emit `SessionDestroyed { id }` event
5. Free memory

---

## Multi-Session Support

### Concurrent Sessions

Wit supports **multiple simultaneous sessions**. Only one session is **Active** at any given time (the session currently displayed). Other sessions are in **Background** state.

```rust
pub struct SessionManager {
    /// All sessions (Active + Background + Exited)
    sessions: HashMap<SessionId, Session>,

    /// Currently Active session (only 1)
    active_session: Option<SessionId>,

    /// Tab order (for UI)
    tab_order: Vec<SessionId>,

    /// Default shell configuration
    default_shell: ShellConfig,

    /// Maximum concurrent sessions
    max_sessions: usize,
}
```

### Resource Limits

| Resource | Limit | Configurable |
|---|---|---|
| Maximum concurrent sessions | 20 | Yes |
| Scrollback lines per session | 10,000 | Yes |
| Memory per session (estimated) | ~5-20MB | - |
| Total memory for all sessions | ~200MB soft limit | Yes |

When the user tries to create a session exceeding `max_sessions`:

- Show dialog: "Maximum sessions reached. Close a session to create a new one."
- Do not automatically close any session

### Background Session Optimization

Sessions in Background are optimized to conserve resources:

1. **Render throttling:** The terminal emulator still processes output, but does not send render events. Only the final state is kept.
2. **Scrollback trimming:** If total memory exceeds the soft limit, background sessions with the oldest scrollback are trimmed first.
3. **Context detection paused:** The context engine does not poll for background sessions (only resumes when activated).

---

## Tab Model

### Tab UI

```
┌──────────────────────────────────────────────────────────────────┐
│ [wit-term v] [api-server v] [+ New Tab]                         │
│  ^ active    ^ background                                        │
│              (has badge "3" if there are 3 new output lines)     │
└──────────────────────────────────────────────────────────────────┘
```

Each tab displays:

| Element | Description |
|---|---|
| Title | Session display title (see [Display Title](#display-title)) |
| Status indicator | Icon for state (running, exited) |
| Unread badge | Number of new output lines when in background |
| Close button | `x` to close tab (destroy session) |
| Context indicator | Small icon showing project type (git, node, cargo...) |

### Tab Ordering

```rust
pub struct TabOrder {
    /// Ordered list of session IDs (left to right)
    order: Vec<SessionId>,
}

impl TabOrder {
    /// Add new tab (always after the active tab, or at the end)
    pub fn insert_after_active(&mut self, active: &SessionId, new: SessionId);

    /// Move tab (drag-and-drop)
    pub fn move_tab(&mut self, from: usize, to: usize);

    /// Remove tab
    pub fn remove(&mut self, id: &SessionId);
}
```

**New tab position:** New tabs are created **right after the current active tab** (not at the end). Reason: users typically want related tabs near each other.

### Tab Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+T` | New tab (create new session) |
| `Ctrl+Shift+W` | Close current tab |
| `Ctrl+Tab` | Switch to next tab |
| `Ctrl+Shift+Tab` | Switch to previous tab |
| `Ctrl+1` ... `Ctrl+9` | Switch to tab N |
| `Ctrl+Shift+Left` | Move tab left |
| `Ctrl+Shift+Right` | Move tab right |

### Tab Group (Future Feature)

> **Status:** planned, not implemented in v1

Tab groups allow grouping related tabs:

```
┌───────────────────────┐  ┌──────────────────┐
│ v Project A           │  │ v Project B       │
│ [server] [client] [db]│  │ [api] [worker]    │
└───────────────────────┘  └──────────────────┘
```

- Groups have a label and color
- Collapse/expand groups
- Close group = close all tabs in the group
- Auto-group: automatically group tabs with the same project root CWD

---

## Split Panes (Future Feature)

> **Status:** planned, not implemented in v1

### Concept

Split panes allow dividing a tab into multiple terminal areas:

```
┌─────────────────────────────────────────────┐
│ Tab: Development                             │
│                                              │
│ ┌───────────────────┐ ┌───────────────────┐ │
│ │ Session A         │ │ Session B         │ │
│ │ (server)          │ │ (client)          │ │
│ │                   │ │                   │ │
│ │ $ npm run dev     │ │ $ npm run build   │ │
│ │ Server running... │ │ Building...       │ │
│ │                   │ │                   │ │
│ ├───────────────────┤ │                   │ │
│ │ Session C         │ │                   │ │
│ │ (tests)           │ │                   │ │
│ │ $ npm test        │ │                   │ │
│ │                   │ │                   │ │
│ └───────────────────┘ └───────────────────┘ │
└─────────────────────────────────────────────┘
```

### Split Types

- **Horizontal split:** Divide top/bottom
- **Vertical split:** Divide left/right
- Splits can be nested (creating complex layouts)

### Pane Navigation

| Shortcut | Action |
|---|---|
| `Alt+Left/Right/Up/Down` | Focus pane in direction |
| `Ctrl+Shift+D` | Split vertical |
| `Ctrl+Shift+E` | Split horizontal |
| `Ctrl+Shift+X` | Close pane (close session in pane) |
| `Alt+Shift+Left/Right/Up/Down` | Resize pane |

### Pane Model

```rust
pub enum PaneLayout {
    /// A single session
    Single(SessionId),
    /// Split in two
    Split {
        direction: SplitDirection,
        /// Ratio (0.0 - 1.0, divider position)
        ratio: f64,
        first: Box<PaneLayout>,
        second: Box<PaneLayout>,
    },
}

pub enum SplitDirection {
    Horizontal,  // top/bottom
    Vertical,    // left/right
}
```

---

## Session Configuration

### Per-Session Config

Each session can override the global config:

```rust
pub struct SessionConfig {
    /// Shell executable (override global default)
    shell: Option<String>,

    /// Shell arguments
    shell_args: Option<Vec<String>>,

    /// Working directory
    cwd: Option<PathBuf>,

    /// Environment variable overrides
    /// Key = var name, Value = Some(value) to set, None to unset
    env_overrides: HashMap<String, Option<String>>,

    /// Terminal size (None = inherit from window)
    size: Option<(u16, u16)>,

    /// Scrollback lines (override global default)
    scrollback_lines: Option<usize>,

    /// Shell integration level override
    shell_integration: Option<ShellIntegrationLevel>,
}
```

### Default Shell Detection

When `shell` is not specified, Wit detects the default shell:

| Platform | Detection Method | Typical Default |
|---|---|---|
| Linux | `$SHELL` environment variable | `/bin/bash` or `/bin/zsh` |
| macOS | `$SHELL` environment variable | `/bin/zsh` |
| Windows | Registry + `$COMSPEC` | `powershell.exe` or `cmd.exe` |

**Shell detection order (Windows):**

1. Check if PowerShell 7+ (`pwsh.exe`) is installed
2. Check Windows PowerShell (`powershell.exe`)
3. Fallback to `cmd.exe`

**Shell detection order (Unix):**

1. `$SHELL` environment variable
2. `/etc/passwd` entry for current user
3. Fallback to `/bin/sh`

### Default CWD

When `cwd` is not specified:

1. If there is another active session - use the CWD of the currently active session
2. If the app just started - use the home directory (`~`)
3. If the app was opened from a file manager / context menu - use that directory

---

## Session Events

Events are emitted via the Tauri event system for the frontend to react:

```rust
pub enum SessionEvent {
    /// New session created
    Created {
        id: SessionId,
        title: String,
        shell: String,
        cwd: PathBuf,
    },

    /// Session activated (switched to)
    Activated {
        id: SessionId,
        /// Previously active session (if any)
        previous: Option<SessionId>,
    },

    /// Session deactivated (switched away)
    Deactivated {
        id: SessionId,
    },

    /// Title changed (CWD change, user rename, process change)
    TitleChanged {
        id: SessionId,
        title: String,
    },

    /// CWD changed (from shell integration or heuristics)
    CwdChanged {
        id: SessionId,
        cwd: PathBuf,
    },

    /// Shell process exited
    Exited {
        id: SessionId,
        exit_code: i32,
    },

    /// Session destroyed (cleanup complete)
    Destroyed {
        id: SessionId,
    },

    /// New output while session is in background
    BackgroundOutput {
        id: SessionId,
        /// Number of new output lines since the user last viewed
        new_lines: usize,
    },

    /// Bell character received (from shell or program)
    Bell {
        id: SessionId,
    },
}
```

### Frontend Handling

| Event | Frontend Action |
|---|---|
| `Created` | Add new tab to tab bar |
| `Activated` | Show terminal view of session, focus input |
| `Deactivated` | Hide terminal view (but keep DOM for performance) |
| `TitleChanged` | Update tab title |
| `CwdChanged` | Update context sidebar (if visible) |
| `Exited` | Show exit indicator on tab |
| `Destroyed` | Remove tab from tab bar |
| `BackgroundOutput` | Update unread badge on tab |
| `Bell` | Flash tab title, optional system notification |

---

## Session Persistence

### Problem

Should sessions be saved and restored after app restart?

### Options Analysis

| Option | Advantages | Disadvantages |
|---|---|---|
| **No persistence** | Simple, clean start | User loses context on restart |
| **Save CWD only** | Restore tabs at the correct directory | Lose scrollback, running processes |
| **Full state save** | Near-seamless restart | Complex, large scrollback, shell state hard to restore |
| **tmux-style detach** | Perfect persistence | Needs background daemon, complex |

### Decision: Save CWD + Tab Layout (v1)

**Wit v1 will save:**

- Number of tabs and their order
- CWD of each tab at the time of close
- Custom title (if any)
- Shell override (if any)
- Active tab index

**Wit v1 will NOT save:**

- Scrollback history
- Shell state (environment variables, aliases)
- Running processes
- Terminal screen buffer

**Persistence file:** `~/.local/share/wit/sessions.json`

```json
{
    "version": 1,
    "saved_at": "2026-03-23T10:30:00Z",
    "active_tab_index": 0,
    "tabs": [
        {
            "cwd": "/home/user/Projects/wit-term",
            "title": null,
            "shell": null
        },
        {
            "cwd": "/home/user/Projects/api-server",
            "title": "API Dev",
            "shell": "/bin/zsh"
        }
    ]
}
```

### Restore Behavior

When the app starts:

1. Read `sessions.json`
2. If the file exists and is valid:
   - Create sessions according to saved tabs
   - Each session opens a new shell at the saved CWD
   - Activate tab according to `active_tab_index`
3. If the file does not exist or is invalid:
   - Create 1 default session (default shell, home directory)

### Configuration

```toml
[session]
# Restore sessions on startup
restore_on_startup = true

# When to save sessions
save_on_exit = true

# Behavior when closing the last tab
close_last_tab_action = "new_tab"  # "new_tab" | "close_app" | "ask"

# Default shell (override OS default)
# default_shell = "/bin/zsh"

# Default CWD for new sessions
# default_cwd = "~"

# Maximum concurrent sessions
max_sessions = 20

# Scrollback lines per session
scrollback_lines = 10000
```

---

## Default Behavior

### App Startup

```
App starts
    |
    +-- sessions.json exists?
    |   +-- Yes -> Restore saved tabs
    |   +-- No  -> Create 1 default tab
    |
    +-- Default tab:
    |   +-- Shell: OS default (or config override)
    |   +-- CWD: Home directory (or directory app was opened from)
    |   +-- Title: auto-generated
    |
    +-- Ready
```

### New Tab (Ctrl+Shift+T)

```
User presses Ctrl+Shift+T
    |
    +-- Create new session:
    |   +-- Shell: same as active session (or default)
    |   +-- CWD: same as active session's CWD
    |   +-- Title: auto-generated
    |
    +-- Tab position: after the active tab
    |
    +-- Focus new tab
```

### Close Tab (Ctrl+Shift+W)

```
User presses Ctrl+Shift+W
    |
    +-- Shell process running?
    |   +-- Yes, only the shell is running (no foreground process)
    |   |   +-- Close immediately (destroy session)
    |   |
    |   +-- Yes, a foreground process is running (e.g., vim, npm)
    |       +-- Show confirmation: "Process 'npm' is running. Close anyway?"
    |           +-- Yes -> Send SIGHUP, wait 2s, SIGKILL if needed, destroy
    |           +-- No  -> Cancel
    |
    +-- Session already Exited?
    |   +-- Close immediately (destroy session)
    |
    +-- Is this the last tab?
    |   +-- close_last_tab_action = "new_tab"
    |   |   +-- Create new tab before closing the current tab
    |   +-- close_last_tab_action = "close_app"
    |   |   +-- Close app
    |   +-- close_last_tab_action = "ask"
    |       +-- Show dialog: "Close window?"
    |
    +-- Activate the next tab (or previous if at the last tab)
```

### App Close

```
User closes app (window close button, Ctrl+Q, etc.)
    |
    +-- Any session with a foreground process?
    |   +-- Yes -> Show confirmation: "2 sessions have running processes. Quit?"
    |   |   +-- Yes -> Save sessions, cleanup, quit
    |   |   +-- No  -> Cancel
    |   +-- No  -> Proceed
    |
    +-- save_on_exit = true?
    |   +-- Yes -> Save tab layout to sessions.json
    |
    +-- Cleanup all sessions:
    |   +-- Send SIGHUP to all shell processes
    |   +-- Wait up to 2 seconds
    |   +-- SIGKILL remaining
    |   +-- Drop all resources
    |
    +-- App exit
```

---

## Rust Types Summary

```rust
pub struct Session {
    pub id: SessionId,
    pub state: SessionState,
    pub config: SessionConfig,
    pub title: SessionTitle,
    pub created_at: SystemTime,

    // Internal components (not exposed via IPC)
    pty: PtyHandle,
    terminal: TerminalEmulator,
    context: SessionContext,
}

pub enum SessionState {
    Creating,
    Active,
    Background { unread_lines: usize },
    Exited { exit_code: i32 },
}

pub struct SessionManager {
    sessions: HashMap<SessionId, Session>,
    active_session: Option<SessionId>,
    tab_order: Vec<SessionId>,
    config: SessionManagerConfig,
}

/// Tauri IPC commands
#[tauri::command]
async fn create_session(options: CreateSessionOptions) -> Result<SessionId>;

#[tauri::command]
async fn destroy_session(id: SessionId) -> Result<()>;

#[tauri::command]
async fn activate_session(id: SessionId) -> Result<()>;

#[tauri::command]
async fn list_sessions() -> Result<Vec<SessionInfo>>;

#[tauri::command]
async fn rename_session(id: SessionId, title: String) -> Result<()>;

#[tauri::command]
async fn get_session_info(id: SessionId) -> Result<SessionInfo>;

/// Session info exposed to frontend (subset of full Session)
pub struct SessionInfo {
    pub id: SessionId,
    pub state: SessionState,
    pub title: String,
    pub shell: String,
    pub cwd: PathBuf,
    pub created_at: SystemTime,
    pub exit_code: Option<i32>,
    pub unread_lines: usize,
}
```

---

## Testing Strategy

### Unit Tests

- SessionManager: create, activate, destroy, lifecycle transitions
- TabOrder: insert, move, remove, edge cases (empty, single tab)
- SessionTitle: auto-generation rules, custom override
- Default shell detection: mock OS environment

### Integration Tests

- Full lifecycle: create - activate - background - activate - exit - destroy
- Multi-session: create 5 sessions, switch back and forth, verify state
- Persistence: save - reload - verify layout restored
- Resource cleanup: verify no leaks after destroy

### Edge Cases

- Close app while session is Creating (PTY not yet ready)
- Shell crash (SIGSEGV) - session should transition to Exited
- Create session when max_sessions has been reached
- Restore sessions when saved CWD no longer exists
- Rapid tab switching (debounce activation events)

---

## Known Limitations and Future Work

1. **v1.0:** Tabs only, no splits, no groups
2. **v1.1:** Tab groups
3. **v2.0:** Split panes
4. **Future:** Session sharing (pair programming)
5. **Future:** Session recording/playback (asciinema-style)
6. **Future:** tmux-like detach/reattach (background daemon)
