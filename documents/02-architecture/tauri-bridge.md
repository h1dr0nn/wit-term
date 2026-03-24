# Tauri Bridge (IPC Layer)

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

The Tauri Bridge is the communication layer between the Rust core and the React frontend. It uses two
main mechanisms: **Commands** (request-response) and **Events** (push
notifications).

---

## Commands (Frontend -> Rust)

Commands are synchronous from the frontend's perspective (they return a Promise) but
async internally in Rust.

### Naming Conventions

```
{domain}_{action}          # e.g.: session_create, config_get
{domain}_{entity}_{action} # e.g.: session_input_send
```

### Command Catalog

#### Session Commands

| Command | Input | Output | Description |
|---|---|---|---|
| `create_session` | `SessionConfig` | `string` (session_id) | Create a new session |
| `destroy_session` | `{ sessionId }` | `void` | Close a session |
| `list_sessions` | - | `SessionInfo[]` | List sessions |
| `send_input` | `{ sessionId, data }` | `void` | Send input to PTY |
| `submit_command` | `{ sessionId, command, commandId }` | `void` | Atomically capture state + write command to PTY |
| `resize_session` | `{ sessionId, cols, rows }` | `void` | Resize terminal |
| `get_session_grid` | `{ sessionId }` | `GridSnapshot` | Get current grid state |

#### Completion Commands

| Command | Input | Output | Description |
|---|---|---|---|
| `request_completions` | `CompletionRequest` | `Completion[]` | Request completions |
| `accept_completion` | `{ sessionId, completion }` | `void` | Accept a completion |

#### Context Commands

| Command | Input | Output | Description |
|---|---|---|---|
| `get_context` | `{ sessionId }` | `ContextData` | Get current context |
| `get_providers` | `{ sessionId }` | `ProviderInfo[]` | List active providers |

#### Config Commands

| Command | Input | Output | Description |
|---|---|---|---|
| `get_config` | - | `AppConfig` | Get full config |
| `set_config` | `Partial<AppConfig>` | `void` | Update config |
| `get_themes` | - | `ThemeInfo[]` | List themes |
| `get_theme` | `{ name }` | `Theme` | Get theme data |

### Command Implementation (Rust)

```rust
use tauri::command;

#[command]
async fn create_session(
    config: SessionConfig,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut manager = state.session_manager.lock().await;
    manager
        .create_session(config)
        .map(|id| id.to_string())
        .map_err(|e| e.to_string())
}

#[command]
async fn send_input(
    session_id: String,
    data: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.session_manager.lock().await;
    let session = manager
        .get_session(&session_id)
        .ok_or("Session not found")?;
    session
        .pty
        .write(data.as_bytes())
        .map(|_| ())
        .map_err(|e| e.to_string())
}

// Register commands in Tauri builder
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            create_session,
            destroy_session,
            list_sessions,
            send_input,
            submit_command,
            resize_session,
            get_session_grid,
            request_completions,
            accept_completion,
            get_context,
            get_providers,
            get_config,
            set_config,
            get_themes,
            get_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Events (Rust -> Frontend)

Events are fire-and-forget push notifications from Rust to the frontend.

### Event Catalog

| Event | Payload | Trigger |
|---|---|---|
| `terminal_output` | `{ sessionId, changes, cursor }` | PTY output parsed |
| `terminal_bell` | `{ sessionId }` | BEL character received |
| `terminal_title` | `{ sessionId, title }` | OSC title change |
| `session_exited` | `{ sessionId, exitCode }` | Shell process exited |
| `context_changed` | `{ sessionId, context }` | CWD or environment changed |
| `completion_hint` | `{ sessionId, hint }` | Inline hint available |
| `command_output` | `{ session_id, command_id, output, duration_ms }` | Command finished, full output + timing |
| `command_output_chunk` | `{ session_id, command_id, output }` | Incremental output chunk during execution |

### Event Emission (Rust)

```rust
use tauri::Emitter;

// Emit terminal output to specific session channel
fn emit_terminal_output(
    app: &tauri::AppHandle,
    session_id: &str,
    changes: Vec<CellChange>,
    cursor: CursorState,
) {
    app.emit(
        &format!("terminal_output_{}", session_id),
        TerminalOutputPayload { changes, cursor },
    ).ok();
}

// Emit context change
fn emit_context_changed(
    app: &tauri::AppHandle,
    session_id: &str,
    context: &ProjectContext,
) {
    app.emit(
        &format!("context_changed_{}", session_id),
        ContextPayload::from(context),
    ).ok();
}
```

### Event Subscription (Frontend)

```typescript
import { listen, UnlistenFn } from "@tauri-apps/api/event";

// In a React hook
useEffect(() => {
  const unlisteners: Promise<UnlistenFn>[] = [];

  unlisteners.push(
    listen<TerminalOutputPayload>(
      `terminal_output_${sessionId}`,
      (event) => {
        applyChanges(event.payload.changes);
        updateCursor(event.payload.cursor);
      }
    )
  );

  unlisteners.push(
    listen<ContextPayload>(
      `context_changed_${sessionId}`,
      (event) => {
        setContext(event.payload);
      }
    )
  );

  return () => {
    unlisteners.forEach((p) => p.then((fn) => fn()));
  };
}, [sessionId]);
```

---

## Data Types (Shared)

Shared TypeScript types must match Rust structs (via Serde):

```typescript
// types/session.ts
interface SessionConfig {
  shell?: string;        // Override default shell
  cwd?: string;          // Initial working directory
  env?: Record<string, string>;
  cols?: number;
  rows?: number;
}

interface SessionInfo {
  id: string;
  title: string;
  shell: string;
  cwd: string;
  createdAt: number;     // Unix timestamp
  isActive: boolean;
}

// types/terminal.ts
interface CellChange {
  row: number;
  col: number;
  content: string;
  fg: string;            // CSS color
  bg: string;
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strikethrough: boolean;
  dim: boolean;
}

interface CursorState {
  row: number;
  col: number;
  visible: boolean;
  shape: "block" | "underline" | "bar";
}

interface GridSnapshot {
  cells: CellChange[][];
  cursor: CursorState;
  scrollbackSize: number;
}

// types/completion.ts
interface CompletionRequest {
  input: string;
  cursorPos: number;
  sessionId: string;
}

interface CompletionItem {
  text: string;
  display: string;
  description: string;
  kind: "command" | "flag" | "argument" | "path" | "history";
  icon?: string;
}

// types/context.ts
interface ContextData {
  cwd: string;
  projectType: string | null;
  providers: ProviderInfo[];
  gitInfo: GitInfo | null;
}

interface ProviderInfo {
  name: string;
  active: boolean;
  details: Record<string, string>;
}

interface GitInfo {
  branch: string;
  status: string;        // "clean" | "dirty"
  ahead: number;
  behind: number;
  remote: string | null;
}
```

---

## Performance Considerations

### Batching Terminal Output

Terminal output can arrive very quickly (e.g., `cat large_file`). The frontend
needs to batch updates:

```rust
// Rust side: batch output events
fn pty_read_loop(/* ... */) {
    let mut pending_changes: Vec<CellChange> = Vec::new();
    let mut last_emit = Instant::now();
    let emit_interval = Duration::from_millis(8); // ~120 fps max

    loop {
        match pty.read(&mut buf) {
            Ok(n) => {
                let actions = parser.advance(&buf[..n]);
                for action in actions {
                    let changes = grid.apply(action);
                    pending_changes.extend(changes);
                }

                // Batch: only emit every 8ms
                if last_emit.elapsed() >= emit_interval {
                    emit_terminal_output(&app, session_id, pending_changes.drain(..).collect(), cursor);
                    last_emit = Instant::now();
                }
            }
            // ...
        }
    }
}
```

### Serialization

- Use `serde_json` for IPC (Tauri requirement)
- Minimize payload size: only send changed cells, not the full grid
- Consider binary serialization (MessagePack) if JSON becomes a bottleneck

### Event Throttling

- Terminal output: max 120 events/second (8ms interval)
- Context changes: debounce 500ms (file system events can be noisy)
- Completion requests: debounce 100ms (while user is typing)
