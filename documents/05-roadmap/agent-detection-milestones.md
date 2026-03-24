# Agent Detection — Implementation Milestones

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Overview

This document defines the phased implementation plan for the Agent-Aware Terminal feature. The plan progresses from simple process detection (Phase A) through full protocol support (Phase D), with each phase delivering usable functionality independently.

| Phase | Name                   | Estimated Duration | Main Objective                                          |
|-------|------------------------|--------------------|---------------------------------------------------------|
| A     | Foundation             | 2-3 weeks          | Process detection engine + sidebar UI skeleton           |
| B     | Claude Code Adapter    | 3-4 weeks          | Full dashboard for Claude Code via output parsing        |
| C     | Expand Adapters        | 2-3 weeks          | Adapters for Aider, Codex CLI, Copilot CLI               |
| D     | Wit Protocol           | 4-6 weeks          | First-party protocol for structured agent communication  |

---

## 2. Phase A: Foundation (2-3 weeks)

Build the detection infrastructure and the sidebar UI shell. By the end of this phase, Wit must detect when an agent starts in a terminal session and open a sidebar showing the agent name and PID.

### 2.1. A1: Process Detection Engine

**Objective**: detect agent processes running inside Wit terminal sessions.

**Tasks:**

1. Integrate the `sysinfo` crate into the Rust core
2. Implement process tree monitoring for each PTY session
   - Walk the process tree from the shell PID
   - Identify child processes matching known agent patterns
3. Build the agent database with pattern matching rules
   - Process name patterns (e.g., `claude`, `aider`, `codex`)
   - Command-line argument patterns (e.g., `--agent`, `--interactive`)
   - Store as a static registry in Rust, extensible via config
4. Implement polling with adaptive interval
   - Default interval: 2 seconds
   - Adaptive: increase to 5 seconds when idle, decrease to 1 second after recent activity
   - Efficient: only scan the process subtree of the session's shell PID
5. Emit Tauri events on detection state changes

**Deliverables:**

- `AgentDetected { session_id, agent_name, pid, detection_method }` event
- `AgentExited { session_id, agent_name, pid, exit_code }` event
- Unit tests for pattern matching against known agent process signatures

### 2.2. A2: Sidebar UI Skeleton

**Objective**: build the right sidebar component with the basic lifecycle.

**Tasks:**

1. Create the `AgentSidebar` React component
   - Header, tab bar (placeholder tabs), content area, actions bar
   - Resize handle on the left border (280px–50% range)
   - Persist width to user preferences
2. Implement auto-open/close lifecycle
   - Listen for `AgentDetected` events from the Rust backend
   - Auto-open the sidebar with slide animation
   - Transition to "Session Ended" state on `AgentExited`
3. Wire up keybinding (Ctrl+Shift+A) for manual toggle
4. Per-tab isolation: each terminal tab owns its own sidebar state

**Deliverables:**

- Sidebar opens automatically showing "Claude Code detected (PID 1234)"
- Sidebar header displays agent name and icon
- Sidebar closes/opens via keyboard shortcut
- Tabs are visible but show placeholder "Coming in Phase B" content

---

## 3. Phase B: Claude Code Adapter (3-4 weeks)

Build the first complete adapter targeting Claude Code. This phase delivers a fully functional agent dashboard for one agent. The learnings inform the adapter interface used by Phase C.

### 3.1. B1: Output Parsing Adapter

**Objective**: parse Claude Code's terminal output to extract structured events.

**Tasks:**

1. Analyze Claude Code's output patterns
   - Cost/token reporting format
   - Model name display
   - Thinking block markers
   - Tool use output patterns
   - File edit summaries
2. Implement `ClaudeCodeAdapter` struct implementing the `AgentAdapter` trait
   - `fn parse_output(&mut self, line: &str) -> Vec<AgentEvent>`
   - Maintain internal state machine for multi-line parsing
3. Define the `AgentEvent` enum
   - `Thinking { content }`
   - `ToolUse { tool_name, args, result }`
   - `FileEdit { path, action }`
   - `TokenUpdate { input_tokens, output_tokens }`
   - `CostUpdate { total_cost }`
   - `ModelInfo { model_name }`
   - `Error { message }`
4. Wire adapter output into the Tauri event system

**Deliverables:**

- `ClaudeCodeAdapter` parsing live terminal output
- Stream of `AgentEvent` values emitted as Tauri events
- Integration tests against captured Claude Code output samples

### 3.2. B2: Filesystem Watcher

**Objective**: track file changes made by the agent for the Files tab.

**Tasks:**

1. Integrate the `notify` crate for filesystem watching
2. On `AgentDetected`, take a git baseline snapshot
   - Record the git HEAD commit hash
   - Snapshot the working tree state (`git status --porcelain`)
3. Watch the project directory for file changes
   - Debounce events: 500ms window to coalesce rapid writes
   - Ignore patterns: `.git/`, `node_modules/`, `target/`, `__pycache__/`, common build artifacts
4. Compute diffs against baseline on demand (when user expands a file entry)
5. Emit `FileChanged { path, action, diff_summary }` events

**Deliverables:**

- Real-time file change detection during agent sessions
- Git baseline snapshot for accurate diff comparison
- Debounced, filtered filesystem events

### 3.3. B3: Activity + Files Tabs

**Objective**: implement the Activity timeline and Files change list in the sidebar.

**Tasks:**

1. Activity Timeline component
   - Render `AgentEvent` values as timeline entries
   - Entry type icons (Thinking, Tool Use, File Edit, Error)
   - Current entry highlight
   - Auto-scroll with manual pause/resume
   - Relative timestamps switching to absolute after 1 hour
2. Files Tab component
   - Summary bar with file count and +/- totals
   - File entry list with status icons
   - Click-to-expand inline diff view
   - Syntax highlighted diff rendering
   - "Undo" button per file (invokes git restore)
3. Tab badge counts (e.g., "3" on Files tab for 3 changed files)

**Deliverables:**

- Activity tab showing live agent actions as a timeline
- Files tab showing all changed files with expandable diffs
- Undo functionality restoring individual files to baseline

### 3.4. B4: Cost/Token Counter

**Objective**: display real-time cost and token usage in the sidebar header.

**Tasks:**

1. Token/Cost counter component in the header
   - Abbreviated format (12.4k tokens, $0.03)
   - Color thresholds (normal, warning > $1, danger > $5)
2. Wire `TokenUpdate` and `CostUpdate` events to the counter
3. Model badge display from `ModelInfo` events

**Deliverables:**

- Live token and cost counters in the sidebar header
- Full Claude Code dashboard working end-to-end: detection, activity, files, cost tracking

---

## 4. Phase C: Expand Adapters (2-3 weeks)

Add adapters for additional agent CLIs, using the adapter interface validated in Phase B.

### 4.1. Aider Adapter

- Parse Aider's output format (commit messages, file edits, cost reporting)
- Implement `AiderAdapter` struct
- Test against captured Aider session output

### 4.2. Codex CLI Adapter

- Parse Codex CLI's output patterns
- Implement `CodexAdapter` struct
- Test against captured Codex session output

### 4.3. Copilot CLI Adapter

- Parse GitHub Copilot CLI output patterns
- Implement `CopilotAdapter` struct
- Test against captured Copilot CLI session output

### 4.4. Refine AgentAdapter Interface

- Review the `AgentAdapter` trait based on Phase B and early Phase C experience
- Identify common patterns and agent-specific extensions
- Document the trait interface for future community adapters
- Consider a plugin mechanism for third-party adapters

**Deliverables:**

- Three additional adapters with test coverage
- Refined and documented `AgentAdapter` trait
- Adapter selection logic that auto-detects which adapter to use based on the detected agent

---

## 5. Phase D: Wit Protocol (4-6 weeks, future)

Implement the first-party Wit Protocol for structured, bidirectional communication between agents and the terminal. This phase is future work and depends on Phase A-C learnings.

### 5.1. Socket Server

- Implement a Unix domain socket (Linux/macOS) and named pipe (Windows) server
- One socket per terminal session, path exposed via `$WIT_SOCKET` environment variable
- Handle concurrent connections (agent + potential extensions)
- JSON message framing over the socket

### 5.2. Protocol Message Handling

- Implement message types: `activity`, `file_change`, `usage`, `conversation`, `approval_request`, `approval_response`, `context_request`, `context_response`
- Validate incoming messages against the protocol schema
- Route messages to the appropriate sidebar components

### 5.3. Approval Request UI

- Implement the Approval Request Card component (see agent-dashboard.md Section 6)
- Wire approve/reject buttons to send `approval_response` messages back through the socket
- Handle timeout and auto-reject logic

### 5.4. Context Injection

- Respond to `context_request` messages with project context from the context engine
- Provide: git status, project structure, environment info, recent commands
- Rate-limit context responses to prevent abuse

### 5.5. SDK Packages

- **Rust crate**: `wit-protocol` on crates.io
- **npm package**: `@nicwit/wit-protocol` on npm
- **Python package**: `wit-protocol` on PyPI
- Each SDK provides: connect, send message, receive message, type definitions

**Deliverables:**

- Working Wit Protocol server integrated into the terminal
- Approval request flow end-to-end
- SDK packages published for Rust, JavaScript, and Python

---

## 6. Dependencies

```
Phase A: Foundation
    ├── A1: Process Detection ─────────► A2: Sidebar UI (needs events)
    └── A2: Sidebar UI ────────────────► Phase B (needs container)

Phase B: Claude Code Adapter
    ├── B1: Output Parsing ────────────► B3: Activity Tab (needs events)
    ├── B1: Output Parsing ────────────► B4: Cost Counter (needs events)
    ├── B2: Filesystem Watcher ────────► B3: Files Tab (needs file events)
    └── B3 + B4 ───────────────────────► Phase C (validates interface)

Phase C: Expand Adapters
    └── All adapters ──────────────────► Phase D (informs protocol design)

Phase D: Wit Protocol
    └── Independent of A-C for core implementation,
        but UI components from B3/B4 are reused
```

**Critical path:** A1 -> A2 -> B1 -> B3 -> Phase C evaluation -> Phase D decision

Phase D can begin in parallel with Phase C if resources allow, since the protocol server is independent of the adapter work.

---

## 7. Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Agent output format changes between versions | High | Medium | Version-aware adapters with graceful degradation. Fall back to generic display if parsing fails. Maintain test fixtures per agent version. |
| Performance impact of filesystem watching on large repos | Medium | Medium | Debounce all events (500ms). Use ignore patterns for `node_modules/`, `target/`, etc. Limit watch depth. Benchmark on repos with 10k+ files. |
| Protocol adoption by agent developers is slow | Medium | Low | Layers 1-3 must be fully sufficient as standalone features. The protocol (Layer 4) is additive, not required. |
| Process detection false positives | Medium | Low | Require multiple signals (process name + CLI args). Allow user to dismiss false detections. Maintain a suppression list. |
| Adapter maintenance burden across agent updates | High | Medium | Automated CI tests against latest agent versions. Community contributions for adapter updates. Adapter error isolation (one broken adapter does not affect others). |
| Socket/pipe security on multi-user systems | Low | High | Restrict socket permissions to the current user (0600). Validate client PID matches the expected agent process. |

---

## 8. Decision Points

### After Phase B: Adapter Interface Review

- Is the `AgentAdapter` trait flexible enough to support agents beyond Claude Code?
- Should adapters be in-process (compiled into Wit) or out-of-process (plugin)?
- Is output parsing reliable enough, or do we need a hybrid approach (parsing + filesystem heuristics)?
- What is the minimum viable adapter for agents with limited output structure?

### After Phase C: Protocol Investment Decision

- Do Layers 1-3 provide sufficient value without the protocol?
- Is there demand from agent developers for structured communication?
- Should we invest in the full protocol (Phase D) or redirect effort to more adapters?
- If proceeding with Phase D, which SDK (Rust, npm, or PyPI) has the highest priority based on the agent ecosystem?

### After Phase D: Ecosystem Strategy

- Open-source the protocol specification for community adoption?
- Partner with specific agent projects for native integration?
- Build a marketplace or registry for community-contributed adapters?

---

## See Also

- [Agent Sidebar (Right)](../04-ui-ux/sidebar-right.md)
- [Agent Dashboard Components](../04-ui-ux/agent-dashboard.md)
- [Roadmap Overview](./roadmap.md)
