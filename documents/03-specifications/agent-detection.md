# Agent Detection Engine

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

The Agent Detection Engine is the subsystem responsible for recognizing when an AI coding agent is running inside a Wit terminal session, extracting structured information from its output, and tracking the filesystem changes it produces. Detection is fully automatic and requires zero configuration from the user.

The engine uses a **4-layer progressive detection architecture**. Each layer builds on the previous one and unlocks additional features:

| Layer | Name                | Mechanism                          | Requires Agent Cooperation |
|-------|---------------------|------------------------------------|----------------------------|
| 1     | Process Detection   | PTY child process tree inspection  | No                         |
| 2     | Output Parsing      | Pattern matching on raw PTY bytes  | No                         |
| 3     | Filesystem Watching | `notify` + `git2` diffing          | No                         |
| 4     | Wit Protocol        | Structured IPC via socket/pipe     | Yes                        |

Layers 1-3 are **passive observation** and work with any agent today without modification. Layer 4 requires the agent to opt in by connecting to the Wit Protocol socket, which provides the richest data at highest fidelity.

---

## Principles

1. **Zero-config.** The user should never need to tell Wit that an agent is running. If `claude` or `aider` appears in the process tree, detection kicks in automatically.
2. **Observe, don't interfere.** Detection must never alter the agent's behavior, block its I/O, or inject bytes into the PTY stream. All inspection is read-only.
3. **Adapter-per-agent.** Each agent has a unique output format. Rather than a universal parser, Wit loads a purpose-built adapter module once the agent identity is known. See [Agent Adapters](agent-adapters.md).
4. **Progressive depth.** Start cheap (process list scan), go deeper only when warranted. If Layer 2 parsing fails, the engine degrades gracefully instead of crashing.

---

## Layer 1 — Process Detection

### Mechanism

When a terminal session is active, the engine periodically inspects the PTY child process tree to identify known agent processes. The scan walks the process tree rooted at the shell process (the direct child of the PTY) and matches executable names against the agent database.

### Agent Database

The agent database is a static table compiled into the binary. Each entry maps one or more process name patterns to an agent identity:

| Agent             | Primary Pattern          | Indirect Patterns                                        |
|-------------------|--------------------------|----------------------------------------------------------|
| Claude Code       | `claude`                 | `npx claude`, `npx @anthropic-ai/claude-code`           |
| Aider             | `aider`                  | `python -m aider`, `python3 -m aider`, `pipx run aider` |
| Codex CLI         | `codex`                  | `npx codex`                                              |
| Copilot CLI       | `github-copilot-cli`     | `ghcs`                                                   |

### Matching Rules

- **Direct match:** The executable name (basename, no extension) exactly matches the primary pattern. Example: a process with argv[0] = `/usr/local/bin/claude` matches `claude`.
- **Indirect match:** The process command line contains a known indirect pattern. This handles wrapper invocations like `npx claude` where the actual executable is `node` but the arguments reveal the agent. The engine inspects the full command line (`/proc/{pid}/cmdline` on Linux, `sysinfo` process info on macOS/Windows).
- **Precedence:** Direct match wins over indirect match. If multiple agents match (unlikely but possible), the agent with the deepest process tree position (closest to the leaf) wins.

### Platform Implementation

The engine uses the `sysinfo` crate for cross-platform process inspection. This crate provides a unified API across Linux, macOS, and Windows for:

- Enumerating processes
- Reading process name, command line, parent PID
- Reading process start time and working directory

### Polling Strategy

| Terminal State          | Poll Interval |
|-------------------------|---------------|
| Idle (no recent input)  | 500ms         |
| After command execution | 100ms         |
| Agent already detected  | 2000ms        |

The poll interval increases once an agent is detected because the engine only needs to confirm the agent is still running. If the agent process disappears, the engine transitions to the `Ended` lifecycle state (see below).

### Output

When an agent is detected, Layer 1 produces an `AgentIdentity` struct:

```rust
/// Result of Layer 1 process detection.
pub struct AgentIdentity {
    /// Canonical agent name (e.g., "claude-code", "aider").
    pub name: String,

    /// Process ID of the agent.
    pub pid: u32,

    /// Timestamp when the agent process started.
    pub started_at: SystemTime,

    /// Working directory of the agent process, if available.
    pub working_directory: Option<PathBuf>,

    /// Detected agent version, if parseable from process args.
    pub version: Option<String>,

    /// Whether this was a direct or indirect match.
    pub match_type: MatchType,
}

pub enum MatchType {
    Direct,
    Indirect,
}
```

On successful detection, the engine immediately loads the corresponding `AgentAdapter` (Layer 2) and starts the filesystem watcher (Layer 3).

---

## Layer 2 — Output Parsing

### Mechanism

Once an agent is identified, the engine activates the corresponding adapter module. All raw PTY bytes are piped through the adapter's parser **before** they reach the terminal emulator for rendering. The adapter extracts structured events from the byte stream without modifying it.

The key constraint is that parsing must be non-blocking and must never delay byte delivery to the renderer. Adapters operate on a copy of the byte stream in a background task.

### AgentAdapter Trait

Every adapter implements the following trait:

```rust
pub trait AgentAdapter: Send + Sync {
    /// Canonical name of the agent this adapter handles.
    fn name(&self) -> &str;

    /// Process name patterns that trigger this adapter (used by Layer 1).
    fn process_patterns(&self) -> &[&str];

    /// Parse a chunk of raw PTY output bytes and return extracted events.
    /// Called incrementally as bytes arrive. The adapter maintains internal
    /// state to handle partial matches across chunk boundaries.
    fn parse_output(&mut self, raw: &[u8]) -> Vec<AgentEvent>;

    /// Reset internal parser state. Called when the agent session ends
    /// or when the parser enters an unrecoverable state.
    fn reset(&mut self);
}
```

### AgentEvent Enum

Adapters emit a stream of `AgentEvent` values. Not every adapter will emit every variant; the set depends on what the agent exposes in its output.

```rust
pub enum AgentEvent {
    /// Token usage report (input tokens, output tokens, cache hits).
    TokenUsage {
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: Option<u64>,
        cache_write_tokens: Option<u64>,
    },

    /// Cost update (cumulative session cost in USD).
    Cost {
        total_usd: f64,
        delta_usd: Option<f64>,
    },

    /// The agent switched models.
    ModelChange {
        model: String,
    },

    /// The agent entered a thinking/reasoning phase.
    ThinkingStart,

    /// The agent exited the thinking/reasoning phase.
    ThinkingEnd,

    /// The agent started an action (file edit, command execution, etc.).
    ActionStart {
        action: FileAction,
        path: Option<PathBuf>,
        description: Option<String>,
    },

    /// The agent completed an action.
    ActionEnd {
        action: FileAction,
        path: Option<PathBuf>,
        success: bool,
    },

    /// A file was referenced in the agent's output.
    FileReference {
        path: PathBuf,
        action: FileAction,
    },

    /// A new conversation turn began.
    ConversationTurn {
        role: Role,
        index: u32,
    },

    /// The agent reported an error.
    Error {
        message: String,
        code: Option<String>,
    },

    /// A status text line that does not fit other categories.
    /// Used as the fallback when specific parsing fails.
    StatusText {
        text: String,
    },
}
```

### Supporting Enums

```rust
/// Types of file actions agents perform.
pub enum FileAction {
    Read,
    Create,
    Edit,
    Delete,
}

/// Conversation roles.
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}
```

### Parsing Pipeline

1. Raw PTY bytes arrive on the session's reader thread.
2. Bytes are forwarded to the terminal emulator for rendering (unmodified).
3. A copy of the bytes is sent to the adapter's `parse_output` method on a background task via a bounded channel.
4. Emitted `AgentEvent` values are forwarded to the frontend via Tauri events and stored in the `AgentSession` log.

If the adapter's channel is full (backpressure), bytes are dropped from the parsing copy only. The terminal emulator always receives the complete stream.

---

## Layer 3 — Filesystem Watching

### Mechanism

When an agent is detected, the engine snapshots the current git state (if the working directory is a git repository) and starts a filesystem watcher. This allows Wit to show a real-time summary of files the agent has created, modified, or deleted during its session.

### Baseline Snapshot

On agent detection, the engine records:

- The current `HEAD` commit hash
- The set of tracked files and their hashes (from the git index)
- The set of untracked files

This baseline is used to compute a diff at any point during or after the agent session.

### Watch Rules

| Rule                      | Detail                                               |
|---------------------------|------------------------------------------------------|
| Respect `.gitignore`      | Files matching `.gitignore` patterns are excluded     |
| Ignore `node_modules/`    | Always excluded regardless of `.gitignore`            |
| Ignore `target/`          | Always excluded (Rust build artifacts)                |
| Ignore `.git/`            | Internal git data excluded                            |
| Max depth                 | 5 directory levels from the project root              |
| Debounce                  | 100ms — coalesce rapid file system events             |

### Data Structures

```rust
/// Tracks the full lifecycle of an agent session including file changes.
pub struct AgentSession {
    /// Agent identity from Layer 1.
    pub identity: AgentIdentity,

    /// When the session started (agent first detected).
    pub started_at: SystemTime,

    /// When the session ended (agent process exited), if applicable.
    pub ended_at: Option<SystemTime>,

    /// Git commit hash at session start, if in a git repo.
    pub git_baseline: Option<String>,

    /// Accumulated file changes during the session.
    pub file_changes: Vec<FileChange>,

    /// Parsed events from Layer 2 output parsing.
    pub events: Vec<TimestampedEvent>,
}

/// A single file change observed during an agent session.
pub struct FileChange {
    /// Path relative to the project root.
    pub path: PathBuf,

    /// Type of change.
    pub change_type: ChangeType,

    /// When the filesystem event was received.
    pub detected_at: SystemTime,

    /// When the file was last modified (from filesystem metadata).
    pub modified_at: Option<SystemTime>,

    /// File size in bytes after the change, if applicable.
    pub size_bytes: Option<u64>,
}

/// Types of filesystem changes.
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf },
}

/// An AgentEvent with a timestamp.
pub struct TimestampedEvent {
    pub event: AgentEvent,
    pub timestamp: SystemTime,
}
```

### Technology

| Crate      | Purpose                                                       |
|------------|---------------------------------------------------------------|
| `notify`   | Cross-platform filesystem event watching (inotify/FSEvents/ReadDirectoryChanges) |
| `git2`     | Read git index, compute baseline, generate diffs              |
| `similar`  | Line-level diff computation for change summaries              |

### Change Aggregation

Multiple filesystem events for the same path within the debounce window are collapsed into a single `FileChange`. If a file is created and then immediately modified, only a `Created` event is recorded. If a file is created and then deleted within the same debounce window, no event is emitted.

---

## Layer 4 — Wit Protocol

Layer 4 is an opt-in structured communication channel between the agent and Wit. Unlike Layers 1-3, it requires the agent to actively connect and send messages.

When Wit detects an agent (Layer 1), it sets the `WIT_SOCKET` environment variable pointing to a Unix domain socket (macOS/Linux) or named pipe (Windows). If the agent recognizes this variable, it can connect and exchange structured JSON messages.

Layer 4 provides the highest fidelity data because the agent reports its own state directly rather than Wit inferring it from process inspection and output parsing.

For the full protocol specification, see [Wit Protocol](wit-protocol.md).

---

## Detection Lifecycle

The detection engine operates as a state machine with the following states:

```
┌──────┐   process found   ┌──────────┐   adapter loaded   ┌────────┐
│ Idle │ ───────────────► │ Detected │ ──────────────────► │ Active │
└──────┘                   └──────────┘                     └────┬───┘
   ▲                                                             │
   │                                                             ▼
   │         process exited   ┌───────┐   watcher active   ┌────────────┐
   └───────────────────────── │ Ended │ ◄────────────────── │ Monitoring │
                              └───────┘                     └────────────┘
```

| State      | Description                                                                                      |
|------------|--------------------------------------------------------------------------------------------------|
| Idle       | No agent detected. Layer 1 polling at idle interval (500ms).                                     |
| Detected   | Agent process found. `AgentIdentity` created. Loading adapter and watcher.                       |
| Active     | Adapter loaded, Layer 2 parsing active. Waiting for first filesystem change or protocol connect.  |
| Monitoring | Layer 3 watcher running. Full event stream flowing to frontend.                                  |
| Ended      | Agent process exited. Watcher runs for 2 more seconds to capture final writes, then stops.       |

Transitions:

- **Idle → Detected:** Layer 1 finds a matching process in the PTY child tree.
- **Detected → Active:** The adapter module is loaded and `parse_output` is ready to receive bytes.
- **Active → Monitoring:** The filesystem watcher is initialized and the git baseline is captured.
- **Monitoring → Ended:** The agent process PID is no longer in the process list.
- **Ended → Idle:** The post-exit watcher grace period expires. Session data is finalized and persisted.

---

## Feature Matrix

This table shows which features each detection layer enables:

| Feature                        | L1 Process | L2 Output | L3 Filesystem | L4 Protocol |
|--------------------------------|:----------:|:---------:|:--------------:|:-----------:|
| Agent name in status bar       | x          |           |                |             |
| Agent-specific sidebar theme   | x          |           |                |             |
| Token usage display            |            | x         |                | x           |
| Cost tracking                  |            | x         |                | x           |
| Model indicator                |            | x         |                | x           |
| Thinking/reasoning spinner     |            | x         |                | x           |
| File change summary            |            | x         | x              | x           |
| Real-time diff viewer          |            |           | x              | x           |
| Conversation turn markers      |            | x         |                | x           |
| Approval request UI            |            |           |                | x           |
| Pause/resume agent             |            |           |                | x           |
| Undo last file change          |            |           | x              | x           |
| Session replay                 |            | x         | x              | x           |

---

## Related Documents

- [Agent Adapters](agent-adapters.md) — per-agent parser modules and the `AgentAdapter` trait
- [Wit Protocol](wit-protocol.md) — Layer 4 structured IPC specification
- [Context Engine](context-engine.md) — project environment detection (git, Node, Rust, etc.)
- [Session Management](session-management.md) — terminal session lifecycle
- [PTY Handling](pty-handling.md) — PTY management and byte stream routing
