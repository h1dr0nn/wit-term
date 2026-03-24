# Agent Adapters

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

Agent Adapters are per-agent parser modules that extract structured events from the raw PTY output of AI coding agents. Each agent (Claude Code, Aider, Codex CLI, Copilot CLI) has a unique output format with different conventions for displaying costs, file edits, thinking indicators, and conversation turns. Rather than building a universal parser, Wit uses an **adapter-per-agent** pattern where each adapter is purpose-built for a specific agent's output format.

Adapters are the implementation behind Layer 2 of the [Agent Detection Engine](agent-detection.md). They are loaded dynamically based on the agent identity established by Layer 1 (Process Detection).

---

## AgentAdapter Trait

Every adapter implements the `AgentAdapter` trait. This is the core interface between the detection engine and the agent-specific parsing logic.

```rust
pub trait AgentAdapter: Send + Sync {
    /// Returns the canonical name of the agent this adapter handles.
    /// Must match the `name` field in the agent database used by Layer 1.
    /// Examples: "claude-code", "aider", "codex-cli", "copilot-cli".
    fn name(&self) -> &str;

    /// Returns the set of process name patterns that trigger this adapter.
    /// Used by Layer 1 to associate a detected process with the correct adapter.
    /// Patterns are matched against the process executable basename and command line.
    /// Examples: &["claude", "npx @anthropic-ai/claude-code"]
    fn process_patterns(&self) -> &[&str];

    /// Parses a chunk of raw PTY output bytes and returns any extracted events.
    ///
    /// This method is called incrementally as bytes arrive from the PTY. The adapter
    /// must maintain internal state to handle patterns that span chunk boundaries.
    /// For example, a cost display line may arrive across two chunks:
    ///   chunk 1: b"Total cost: $0."
    ///   chunk 2: b"42\r\n"
    ///
    /// The adapter must buffer partial matches and emit the event only when the
    /// full pattern is recognized.
    ///
    /// Returns an empty Vec if no events were recognized in this chunk.
    /// Must never block or perform I/O.
    fn parse_output(&mut self, raw: &[u8]) -> Vec<AgentEvent>;

    /// Resets all internal parser state to the initial condition.
    /// Called when:
    /// - The agent session ends (process exits)
    /// - The parser enters an unrecoverable state (too many parse errors)
    /// - The user manually resets the session
    fn reset(&mut self);
}
```

### Implementation Requirements

- **Thread safety.** The trait requires `Send + Sync`. Adapters are created on the session thread and the `parse_output` method is called from a background parsing task. Internal state must be protected accordingly (typically via interior mutability or by ensuring the adapter is only accessed from a single task).
- **Performance.** `parse_output` is called for every PTY output chunk, potentially thousands of times per second during heavy agent output. Parsing must be fast. Avoid allocations in the hot path where possible; prefer reusable buffers.
- **No side effects.** Adapters must not write to the filesystem, make network calls, or modify the PTY byte stream. They are pure observers.
- **Graceful failure.** If a pattern match fails or the output format is unrecognized, the adapter should emit an `AgentEvent::StatusText` with the raw line rather than returning an error. Parsing errors must never propagate upward as panics.

---

## Adapter Loading

When Layer 1 identifies an agent process, the detection engine loads the corresponding adapter:

1. Layer 1 produces an `AgentIdentity` with a `name` field (e.g., `"claude-code"`).
2. The engine looks up the adapter registry, a static map from agent names to adapter constructors.
3. The constructor creates a new adapter instance, optionally parameterized by the detected agent version.
4. The adapter is handed to the Layer 2 parsing task, which begins feeding it PTY output chunks.

```rust
/// Registry of all built-in adapters.
fn create_adapter(identity: &AgentIdentity) -> Option<Box<dyn AgentAdapter>> {
    match identity.name.as_str() {
        "claude-code" => Some(Box::new(ClaudeCodeAdapter::new(identity.version.as_deref()))),
        "aider" => Some(Box::new(AiderAdapter::new(identity.version.as_deref()))),
        "codex-cli" => Some(Box::new(CodexCliAdapter::new(identity.version.as_deref()))),
        "copilot-cli" => Some(Box::new(CopilotCliAdapter::new(identity.version.as_deref()))),
        _ => None,
    }
}
```

If no adapter exists for the detected agent, Layers 2 events are unavailable but Layer 1 and Layer 3 still function. The sidebar will show the agent name and file changes, but not token usage, cost, or conversation structure.

---

## Version-Aware Parsing

Agent output formats evolve across versions. An adapter must handle multiple format variants gracefully.

### Version Detection

The agent version is determined at Layer 1 from the process command line or, if available, from the `--version` flag output captured during process inspection. The version string is passed to the adapter constructor.

### Parser Variant Selection

Adapters use the version to select the appropriate parsing strategy:

```rust
impl ClaudeCodeAdapter {
    pub fn new(version: Option<&str>) -> Self {
        let parser_variant = match version {
            Some(v) if semver_gte(v, "2.0.0") => ParserVariant::V2,
            Some(v) if semver_gte(v, "1.5.0") => ParserVariant::V1_5,
            _ => ParserVariant::V1,
        };
        Self {
            parser_variant,
            buffer: Vec::new(),
            // ...
        }
    }
}
```

### Graceful Degradation

If parsing with the selected variant fails repeatedly (more than 10 consecutive parse errors), the adapter switches to a **fallback mode** that only emits `AgentEvent::StatusText` events with raw output lines. This ensures the sidebar always shows something useful even if the output format has changed in a way the adapter does not yet understand.

The fallback transition is logged so that developers can identify when an adapter needs updating.

---

## Built-in Adapters

### Claude Code Adapter

The Claude Code adapter handles output from Anthropic's Claude Code CLI agent.

#### Process Patterns

| Pattern                            | Match Type |
|------------------------------------|------------|
| `claude`                           | Direct     |
| `npx claude`                       | Indirect   |
| `npx @anthropic-ai/claude-code`   | Indirect   |

#### Output Patterns

| Pattern                  | Regex / Heuristic                                      | Emitted Event                     |
|--------------------------|--------------------------------------------------------|-----------------------------------|
| Cost display             | `\$\d+\.\d{2,4}` following "cost" or "total"          | `AgentEvent::Cost`                |
| Model indicator          | `claude-[a-z]+-\d+-\d{8}` or `claude-\d+(\.\d+)?-[a-z]+` | `AgentEvent::ModelChange`     |
| File edit block start    | Line starting with edit marker (e.g., `--- a/path`)   | `AgentEvent::ActionStart { action: Edit }` |
| File edit block end      | Empty line or next section marker after diff           | `AgentEvent::ActionEnd { action: Edit }`   |
| File create              | "Created" or "Wrote" followed by path                 | `AgentEvent::ActionStart { action: Create }` |
| Thinking start           | Thinking indicator prefix or spinner start             | `AgentEvent::ThinkingStart`       |
| Thinking end             | Thinking indicator disappears or output resumes        | `AgentEvent::ThinkingEnd`         |
| Tool use                 | Tool name in brackets or structured tool output        | `AgentEvent::ActionStart`         |
| User turn                | Input prompt marker (e.g., `>` or `Human:`)           | `AgentEvent::ConversationTurn { role: User }` |
| Assistant turn           | Response start marker                                  | `AgentEvent::ConversationTurn { role: Assistant }` |
| Token usage              | "tokens" with numeric values for input/output          | `AgentEvent::TokenUsage`          |
| Error                    | Error-prefixed output or ANSI red text                 | `AgentEvent::Error`               |

#### Sidebar Customization

When the Claude Code adapter is active, the Wit sidebar can show:

- Current model name and cost per turn
- Cumulative session cost
- Thinking/reasoning indicator with elapsed time
- File changes with inline diff preview
- Conversation turn timeline

---

### Aider Adapter

The Aider adapter handles output from the Aider AI pair programming tool.

#### Process Patterns

| Pattern              | Match Type |
|----------------------|------------|
| `aider`              | Direct     |
| `python -m aider`    | Indirect   |
| `python3 -m aider`   | Indirect   |
| `pipx run aider`     | Indirect   |

#### Output Patterns

| Pattern              | Regex / Heuristic                                      | Emitted Event                     |
|----------------------|--------------------------------------------------------|-----------------------------------|
| Model display        | `Model:` prefix line or `/model` command output        | `AgentEvent::ModelChange`         |
| Token/cost report    | `Tokens:` line with send/receive counts                | `AgentEvent::TokenUsage`          |
| Cost report          | `Cost:` line with dollar amounts                       | `AgentEvent::Cost`                |
| Edit indicator       | `<<<<<<< SEARCH` / `=======` / `>>>>>>> REPLACE` markers | `AgentEvent::ActionStart { action: Edit }` |
| Applied edit         | "Applied edit to" followed by file path                | `AgentEvent::ActionEnd { action: Edit }` |
| Commit message       | "Commit" followed by hash and message                  | `AgentEvent::StatusText`          |
| Linter output        | Linter/formatter error or warning lines                | `AgentEvent::Error`               |
| Added files          | "Added" followed by file path "to the chat"            | `AgentEvent::FileReference { action: Read }` |
| User prompt          | Input prompt `> ` at start of line                     | `AgentEvent::ConversationTurn { role: User }` |
| Response start       | First output line after user prompt processing         | `AgentEvent::ConversationTurn { role: Assistant }` |

#### Notes

Aider uses a distinctive SEARCH/REPLACE block format for file edits. The adapter tracks the state machine across these blocks to accurately report which file is being edited and when the edit is complete. Aider also supports multiple edit formats (whole, diff, udiff); the adapter must handle all of them.

---

### Codex CLI Adapter

The Codex CLI adapter handles output from OpenAI's Codex CLI tool.

#### Process Patterns

| Pattern  | Match Type |
|----------|------------|
| `codex`  | Direct     |
| `npx codex` | Indirect |

#### Output Patterns

| Pattern              | Regex / Heuristic                                      | Emitted Event                     |
|----------------------|--------------------------------------------------------|-----------------------------------|
| Model indicator      | Model name in startup output                           | `AgentEvent::ModelChange`         |
| File operations      | File path references in action output                  | `AgentEvent::FileReference`       |
| Command execution    | Shell command execution indicators                     | `AgentEvent::ActionStart`         |
| Error output         | Error-prefixed lines                                   | `AgentEvent::Error`               |
| Status lines         | All other structured output                            | `AgentEvent::StatusText`          |

The Codex CLI adapter is minimal compared to Claude Code and Aider, reflecting the agent's simpler output format. As the Codex CLI evolves, the adapter will be expanded.

---

### Copilot CLI Adapter

The Copilot CLI adapter handles output from GitHub Copilot CLI.

#### Process Patterns

| Pattern               | Match Type |
|-----------------------|------------|
| `github-copilot-cli`  | Direct     |
| `ghcs`                | Direct     |

#### Output Patterns

| Pattern              | Regex / Heuristic                                      | Emitted Event                     |
|----------------------|--------------------------------------------------------|-----------------------------------|
| Suggestion output    | Command suggestion block                               | `AgentEvent::StatusText`          |
| Explanation          | Explanation text following suggestion                  | `AgentEvent::StatusText`          |
| Error output         | Error messages                                         | `AgentEvent::Error`               |

The Copilot CLI adapter is intentionally simple. Copilot CLI is primarily a command suggestion tool rather than an autonomous agent, so the adapter focuses on capturing its suggestions and explanations as status text.

---

## Adding a New Adapter

To add support for a new AI coding agent, follow these steps:

### Step 1: Create the Adapter Module

Create a new file at `src-tauri/src/agent/adapters/{agent_name}.rs`:

```rust
use crate::agent::{AgentAdapter, AgentEvent};

pub struct MyAgentAdapter {
    buffer: Vec<u8>,
    // Internal parser state...
}

impl MyAgentAdapter {
    pub fn new(version: Option<&str>) -> Self {
        Self {
            buffer: Vec::new(),
        }
    }
}

impl AgentAdapter for MyAgentAdapter {
    fn name(&self) -> &str {
        "my-agent"
    }

    fn process_patterns(&self) -> &[&str] {
        &["my-agent", "npx my-agent"]
    }

    fn parse_output(&mut self, raw: &[u8]) -> Vec<AgentEvent> {
        let mut events = Vec::new();
        // Append raw bytes to internal buffer
        self.buffer.extend_from_slice(raw);
        // Scan buffer for complete patterns
        // Emit events for recognized patterns
        // Remove consumed bytes from buffer
        events
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }
}
```

### Step 2: Register in the Adapter Registry

Add the new adapter to the `create_adapter` function in `src-tauri/src/agent/mod.rs`:

```rust
"my-agent" => Some(Box::new(MyAgentAdapter::new(identity.version.as_deref()))),
```

### Step 3: Add to the Agent Database

Add process patterns to the Layer 1 agent database so that the new agent is detected:

```rust
AgentEntry {
    name: "my-agent",
    primary_pattern: "my-agent",
    indirect_patterns: &["npx my-agent"],
},
```

### Step 4: Collect Sample Output

Gather representative output samples from the agent across different operations:

- Startup and initialization
- File reads and edits
- Thinking/reasoning phases (if applicable)
- Error conditions
- Cost and token reporting (if applicable)
- Multiple agent versions (if formats differ)

Save these samples as test fixtures in `src-tauri/tests/fixtures/agents/{agent_name}/`.

### Step 5: Write Tests

Create snapshot tests that feed sample output through the adapter and verify the emitted events. See the Testing Strategy section below.

### Step 6: Document

Add a subsection to this document describing the new adapter's process patterns, output patterns, and any special parsing considerations.

---

## Testing Strategy

Adapter testing relies heavily on **snapshot tests** with captured agent output. This approach ensures adapters correctly parse real-world output and that parser regressions are caught early.

### Test Structure

```
src-tauri/tests/
├── fixtures/
│   └── agents/
│       ├── claude-code/
│       │   ├── v1.0/
│       │   │   ├── startup.raw        # Raw PTY bytes from agent startup
│       │   │   ├── file-edit.raw      # File edit session output
│       │   │   ├── thinking.raw       # Thinking phase output
│       │   │   └── cost-report.raw    # Cost display output
│       │   └── v2.0/
│       │       └── ...
│       ├── aider/
│       │   ├── v0.50/
│       │   │   ├── search-replace.raw
│       │   │   ├── whole-file.raw
│       │   │   └── commit.raw
│       │   └── ...
│       ├── codex-cli/
│       │   └── ...
│       └── copilot-cli/
│           └── ...
└── agent_adapter_tests.rs
```

### Snapshot Test Pattern

Each test feeds a raw output fixture through the adapter and compares the emitted events against a stored snapshot:

```rust
#[test]
fn claude_code_v1_cost_display() {
    let mut adapter = ClaudeCodeAdapter::new(Some("1.0.0"));
    let raw = include_bytes!("fixtures/agents/claude-code/v1.0/cost-report.raw");
    let events = adapter.parse_output(raw);
    insta::assert_debug_snapshot!(events);
}
```

### Test Categories

| Category            | What It Validates                                              |
|---------------------|----------------------------------------------------------------|
| Event extraction    | Correct `AgentEvent` variants emitted for known patterns       |
| Chunk splitting     | Events are correctly emitted when patterns span chunk boundaries |
| Version variants    | Different output formats per agent version are handled         |
| Fallback behavior   | Unrecognized output produces `StatusText` instead of errors    |
| Reset behavior      | `reset()` clears all internal state cleanly                    |
| Performance         | Parsing 1MB of output completes in under 10ms                 |

### Capturing Test Fixtures

To capture raw PTY output for test fixtures, use the `script` command or Wit's built-in session recording:

```bash
# On macOS/Linux
script -q output.raw -c "claude"

# Use Wit's debug mode (future)
wit --record-pty-output=output.raw
```

Fixtures must be committed to the repository as binary files. Each fixture should be small (under 100KB) and focused on a single output pattern.

---

## Related Documents

- [Agent Detection Engine](agent-detection.md) — the 4-layer detection architecture
- [Wit Protocol](wit-protocol.md) — Layer 4 structured IPC specification
- [PTY Handling](pty-handling.md) — PTY byte stream management
- [Terminal Emulator](terminal-emulator.md) — ANSI parsing and rendering pipeline
