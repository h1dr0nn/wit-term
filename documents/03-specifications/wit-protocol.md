# Wit Protocol

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

The Wit Protocol is a two-way structured communication channel between the Wit terminal emulator and an AI coding agent running inside it. While Layers 1-3 of the [Agent Detection Engine](agent-detection.md) operate through passive observation, the Wit Protocol (Layer 4) enables the agent to **actively report its state** and **receive commands** from Wit.

The protocol is designed to be:

- **Simple** — newline-delimited JSON over a local socket; no HTTP, no TLS, no serialization frameworks.
- **Optional** — agents work fine without it; the protocol adds fidelity, not functionality gates.
- **Low-overhead** — messages are small, infrequent (relative to PTY throughput), and non-blocking.
- **Versioned** — forward-compatible message schema with explicit version negotiation.

---

## Transport

| Platform       | Transport                | Example Path                              |
|----------------|--------------------------|-------------------------------------------|
| macOS / Linux  | Unix domain socket       | `/tmp/wit-a1b2c3d4.sock`                  |
| Windows        | Named pipe               | `\\.\pipe\wit-a1b2c3d4`                   |

Wit creates the socket/pipe when a terminal session starts and removes it when the session ends. The path is deterministic and derived from the session ID.

---

## Environment Variable

Wit injects a single environment variable into the shell environment of every terminal session:

```
WIT_SOCKET=/tmp/wit-a1b2c3d4.sock
```

On Windows:

```
WIT_SOCKET=\\.\pipe\wit-a1b2c3d4
```

Agents that support the Wit Protocol check for this variable at startup. If it is set, the agent connects to the socket and initiates the handshake. If it is not set or the connection fails, the agent proceeds normally without protocol support.

The variable name `WIT_SOCKET` is intentionally generic. It does not encode transport type; the agent infers Unix socket vs named pipe from the path format.

---

## Message Format

All messages are **newline-delimited JSON (NDJSON)**. Each message is a single JSON object terminated by a `\n` character. Messages must not contain embedded newlines within the JSON body.

```
{"type":"handshake","protocol_version":1,"agent":"claude-code","agent_version":"1.2.3"}\n
{"type":"handshake_ack","protocol_version":1,"wit_version":"0.1.0","session_id":"a1b2c3d4"}\n
```

All messages share a common envelope:

```json
{
  "type": "<message_type>",
  ...fields specific to the message type
}
```

The `type` field is always a lowercase string with underscores. Unknown message types must be silently ignored by both sides (forward compatibility).

---

## Handshake

The handshake is the first exchange after the agent connects to the socket. It establishes protocol version agreement and session identity.

### Step 1: Agent sends handshake

```json
{
  "type": "handshake",
  "protocol_version": 1,
  "agent": "claude-code",
  "agent_version": "1.2.3",
  "capabilities": ["file_change", "thinking", "approval_request", "conversation"]
}
```

| Field              | Type     | Required | Description                                                   |
|--------------------|----------|----------|---------------------------------------------------------------|
| `type`             | string   | yes      | Always `"handshake"`.                                         |
| `protocol_version` | integer  | yes      | The highest protocol version the agent supports.              |
| `agent`            | string   | yes      | Canonical agent identifier (e.g., `"claude-code"`, `"aider"`). |
| `agent_version`    | string   | yes      | Semver version string of the agent.                           |
| `capabilities`     | string[] | no       | List of message types the agent intends to send.              |

### Step 2: Wit responds with acknowledgement

```json
{
  "type": "handshake_ack",
  "protocol_version": 1,
  "wit_version": "0.1.0",
  "session_id": "a1b2c3d4",
  "accepted_capabilities": ["file_change", "thinking", "approval_request", "conversation"]
}
```

| Field                    | Type     | Required | Description                                                        |
|--------------------------|----------|----------|--------------------------------------------------------------------|
| `type`                   | string   | yes      | Always `"handshake_ack"`.                                          |
| `protocol_version`       | integer  | yes      | The negotiated protocol version (min of both sides).               |
| `wit_version`            | string   | yes      | Semver version string of the Wit terminal.                         |
| `session_id`             | string   | yes      | The terminal session ID for correlation.                           |
| `accepted_capabilities`  | string[] | no       | Subset of capabilities Wit will process. Others are silently dropped. |

### Handshake Failure

If Wit cannot accept the handshake (e.g., protocol version 0 is unsupported), it responds with an error and closes the connection:

```json
{
  "type": "error",
  "code": "handshake_failed",
  "message": "Unsupported protocol version 0. Minimum supported: 1."
}
```

---

## Agent → Wit Messages

These messages are sent by the agent to report its state to Wit.

| Message Type       | Purpose                                             |
|--------------------|-----------------------------------------------------|
| `status`           | General status text update                          |
| `file_change`      | Report a file operation                             |
| `thinking`         | Signal thinking/reasoning phase start or end        |
| `approval_request` | Ask Wit to surface an approval prompt to the user   |
| `conversation`     | Mark a conversation turn boundary                   |
| `capabilities`     | Update capabilities mid-session                     |

### `status`

Report a general status update to display in the Wit sidebar.

```json
{
  "type": "status",
  "text": "Analyzing codebase structure...",
  "phase": "working",
  "progress": 0.35
}
```

| Field      | Type   | Required | Description                                                              |
|------------|--------|----------|--------------------------------------------------------------------------|
| `text`     | string | yes      | Human-readable status text.                                              |
| `phase`    | string | no       | One of `"idle"`, `"working"`, `"waiting"`, `"error"`. Default: `"working"`. |
| `progress` | float  | no       | Progress fraction 0.0-1.0. Omit if indeterminate.                        |

### `file_change`

Report that the agent has created, modified, or deleted a file.

```json
{
  "type": "file_change",
  "action": "edit",
  "path": "src/components/App.tsx",
  "description": "Added error boundary wrapper",
  "lines_added": 12,
  "lines_removed": 3
}
```

| Field          | Type    | Required | Description                                                      |
|----------------|---------|----------|------------------------------------------------------------------|
| `action`       | string  | yes      | One of `"create"`, `"edit"`, `"delete"`, `"read"`.               |
| `path`         | string  | yes      | File path relative to the project root.                          |
| `description`  | string  | no       | Human-readable summary of the change.                            |
| `lines_added`  | integer | no       | Number of lines added (for `create` and `edit`).                 |
| `lines_removed`| integer | no       | Number of lines removed (for `edit` and `delete`).               |

### `thinking`

Signal the start or end of a thinking/reasoning phase.

```json
{
  "type": "thinking",
  "state": "start",
  "model": "claude-sonnet-4-20250514",
  "token_budget": 10000
}
```

```json
{
  "type": "thinking",
  "state": "end",
  "tokens_used": 4200,
  "duration_ms": 3500
}
```

| Field          | Type    | Required | Description                                             |
|----------------|---------|----------|---------------------------------------------------------|
| `state`        | string  | yes      | `"start"` or `"end"`.                                   |
| `model`        | string  | no       | Model being used for thinking (on `start`).             |
| `token_budget` | integer | no       | Max thinking tokens allocated (on `start`).             |
| `tokens_used`  | integer | no       | Actual thinking tokens consumed (on `end`).             |
| `duration_ms`  | integer | no       | Thinking duration in milliseconds (on `end`).           |

### `approval_request`

Ask Wit to present an approval prompt to the user. This enables agents to delegate user-facing confirmations to the terminal UI rather than inline text prompts.

```json
{
  "type": "approval_request",
  "id": "req-001",
  "action": "edit",
  "path": "src/main.rs",
  "description": "Replace the main function with async version",
  "diff": "- fn main() {\n+ async fn main() -> Result<()> {",
  "timeout_ms": 60000
}
```

| Field         | Type    | Required | Description                                                   |
|---------------|---------|----------|---------------------------------------------------------------|
| `id`          | string  | yes      | Unique request ID for correlation with `approval_response`.   |
| `action`      | string  | yes      | One of `"edit"`, `"create"`, `"delete"`, `"execute"`.         |
| `path`        | string  | no       | File path, if the action relates to a file.                   |
| `description` | string  | yes      | Human-readable description of what is being requested.        |
| `diff`        | string  | no       | Unified diff preview of the proposed change.                  |
| `timeout_ms`  | integer | no       | How long to wait before the agent assumes rejection.          |

### `conversation`

Mark a conversation turn boundary. Allows Wit to segment the terminal output visually.

```json
{
  "type": "conversation",
  "role": "assistant",
  "turn_index": 3,
  "model": "claude-sonnet-4-20250514",
  "input_tokens": 15000,
  "output_tokens": 2300,
  "cost_usd": 0.042
}
```

| Field          | Type    | Required | Description                                                    |
|----------------|---------|----------|----------------------------------------------------------------|
| `role`         | string  | yes      | One of `"user"`, `"assistant"`, `"system"`, `"tool"`.          |
| `turn_index`   | integer | no       | Sequential turn number in the conversation.                    |
| `model`        | string  | no       | Model used for this turn.                                      |
| `input_tokens` | integer | no       | Input tokens consumed in this turn.                            |
| `output_tokens`| integer | no       | Output tokens produced in this turn.                           |
| `cost_usd`     | float   | no       | Cost of this turn in USD.                                      |

### `capabilities`

Update the set of capabilities mid-session. Sent if the agent enables or disables features dynamically.

```json
{
  "type": "capabilities",
  "add": ["approval_request"],
  "remove": ["thinking"]
}
```

| Field    | Type     | Required | Description                             |
|----------|----------|----------|-----------------------------------------|
| `add`    | string[] | no       | Capabilities to add.                    |
| `remove` | string[] | no       | Capabilities to remove.                 |

---

## Wit → Agent Messages

These messages are sent by Wit to the agent.

| Message Type        | Purpose                                              |
|---------------------|------------------------------------------------------|
| `approval_response` | User's answer to an `approval_request`               |
| `command`           | Control command (pause, resume)                      |
| `context`           | Push project context information to the agent        |

### `approval_response`

Deliver the user's decision on an `approval_request`.

```json
{
  "type": "approval_response",
  "id": "req-001",
  "approved": true
}
```

```json
{
  "type": "approval_response",
  "id": "req-001",
  "approved": false,
  "reason": "User declined the change."
}
```

| Field      | Type    | Required | Description                                            |
|------------|---------|----------|--------------------------------------------------------|
| `id`       | string  | yes      | The `id` from the corresponding `approval_request`.    |
| `approved` | boolean | yes      | `true` if the user approved, `false` otherwise.        |
| `reason`   | string  | no       | Human-readable reason for rejection.                   |

### `command`

Send a control command to the agent.

```json
{
  "type": "command",
  "action": "pause",
  "reason": "User requested pause."
}
```

```json
{
  "type": "command",
  "action": "resume"
}
```

| Field    | Type   | Required | Description                                                    |
|----------|--------|----------|----------------------------------------------------------------|
| `action` | string | yes      | One of `"pause"`, `"resume"`.                                  |
| `reason` | string | no       | Human-readable reason for the command.                         |

The agent should respect `pause` by halting work and `resume` by continuing. If the agent does not support pause/resume, it silently ignores the command.

### `context`

Push project context information to the agent. Sent after handshake and whenever context changes significantly (e.g., the user switches branches).

```json
{
  "type": "context",
  "project_root": "/Users/dev/my-project",
  "git_branch": "feature/agent-support",
  "git_dirty_files": 3,
  "environments": ["node", "rust"],
  "package_manager": "pnpm"
}
```

| Field              | Type     | Required | Description                                           |
|--------------------|----------|----------|-------------------------------------------------------|
| `project_root`     | string   | no       | Absolute path to the detected project root.           |
| `git_branch`       | string   | no       | Current git branch name.                              |
| `git_dirty_files`  | integer  | no       | Number of uncommitted files.                          |
| `environments`     | string[] | no       | Detected environment types from the context engine.   |
| `package_manager`  | string   | no       | Detected package manager (npm, pnpm, yarn, cargo, pip). |

---

## Protocol Versioning

The protocol uses integer versioning starting at `1`. The rules are:

1. **Negotiation.** During handshake, both sides declare the highest version they support. The effective version is `min(agent_version, wit_version)`.
2. **Additive changes.** New message types or new optional fields on existing messages do **not** increment the version. Both sides must silently ignore unknown `type` values and unknown fields.
3. **Breaking changes.** Removing a message type, changing the semantics of an existing field, or making an optional field required increments the version.
4. **Minimum version.** Wit always supports version 1. Older versions are not guaranteed.

---

## Error Handling

Either side can send an `error` message at any time:

```json
{
  "type": "error",
  "code": "invalid_message",
  "message": "Failed to parse JSON: unexpected token at position 42"
}
```

| Field     | Type   | Required | Description                              |
|-----------|--------|----------|------------------------------------------|
| `code`    | string | yes      | Machine-readable error code.             |
| `message` | string | yes      | Human-readable error description.        |

Standard error codes:

| Code                | Meaning                                      |
|---------------------|----------------------------------------------|
| `handshake_failed`  | Handshake could not be completed.            |
| `invalid_message`   | Message JSON could not be parsed.            |
| `unknown_type`      | Message type not recognized (informational). |
| `internal_error`    | Internal error on the sending side.          |
| `timeout`           | An expected response was not received in time.|

Errors are informational. Neither side should close the connection on a non-fatal error. Only `handshake_failed` results in connection termination.

---

## SDK Packages

To simplify adoption, the Wit Protocol will be published as thin client libraries in three ecosystems:

| Package                | Registry  | Language     | Purpose                                    |
|------------------------|-----------|--------------|--------------------------------------------|
| `wit-protocol`         | crates.io | Rust         | Rust agents and tooling                    |
| `@wit-term/protocol`   | npm       | TypeScript   | Node.js-based agents (Claude Code, Codex)  |
| `wit-protocol`         | PyPI      | Python       | Python-based agents (Aider)                |

Each library provides:

- Socket/pipe connection management
- Handshake negotiation
- Typed message construction and parsing
- Reconnection with exponential backoff

The libraries are intentionally minimal (no dependencies beyond the standard library and a JSON parser) to keep the integration burden low.

---

## Adoption Strategy

The Wit Protocol is designed for incremental adoption:

1. **Specification first.** The protocol is fully specified before any agent integration is proposed. This document is the source of truth.
2. **Libraries are thin.** Agent maintainers can integrate the protocol with minimal code changes. The SDK handles connection management; the agent only needs to call `send()` at appropriate points.
3. **Graceful absence.** If `WIT_SOCKET` is not set, the agent skips protocol initialization entirely. There is zero overhead when not running inside Wit.
4. **Layers 1-3 always work.** The protocol is an enhancement, not a requirement. All passive detection features work without agent cooperation.
5. **Integrate when traction.** Protocol integration PRs will be submitted to agent projects once Wit has a meaningful user base and the protocol has stabilized through internal testing.

---

## Related Documents

- [Agent Detection Engine](agent-detection.md) — the 4-layer detection architecture
- [Agent Adapters](agent-adapters.md) — per-agent passive output parsers (Layer 2)
- [Session Management](session-management.md) — terminal session lifecycle
