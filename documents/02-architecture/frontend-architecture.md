# Frontend Architecture

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

Wit's frontend is a React + TypeScript application running in a Tauri webview.
The frontend serves as a **thin rendering layer** - it receives data from the Rust core and renders
the UI, collects user input and sends it back to Rust.

---

## Tech Stack

| Technology | Version | Role |
|---|---|---|
| React | 19+ | UI framework |
| TypeScript | 5+ | Type safety |
| Vite | 6+ | Build tool, dev server |
| Zustand | 5+ | State management |
| Tailwind CSS | 4+ | Styling |
| @tauri-apps/api | 2+ | Tauri IPC bindings |

---

## Component Tree

```
<App>
├── <Layout>
│   ├── <SessionSidebar>            # Left sidebar
│   │   ├── <SidebarHeader>
│   │   ├── <SessionList>
│   │   │   └── <SessionItem> ×N
│   │   └── <NewSessionButton>
│   │
│   ├── <MainContent>               # Center
│   │   ├── <TabBar>
│   │   │   └── <Tab> ×N
│   │   └── <TerminalContainer>
│   │       ├── <TerminalView>      # Terminal rendering
│   │       │   ├── <BlocksView>       # Warp-style command blocks
│   │       │   │   └── <CapturedOutputBlock> ×N
│   │       │   │       ├── Block header (command, CWD, branch, duration)
│   │       │   │       └── <AnsiOutput>   # Renders ANSI-colored text
│   │       │   ├── <InputBar>         # Command input with CWD display
│   │       │   ├── <TerminalGrid>
│   │       │   │   └── <TerminalRow> ×rows
│   │       │   ├── <Cursor>
│   │       │   └── <Selection>
│   │       ├── <CompletionPopup>   # Floating overlay
│   │       │   └── <CompletionItem> ×N
│   │       └── <InlineHint>        # Ghost text after cursor
│   │
│   └── <ContextSidebar>            # Right sidebar (optional)
│       ├── <SidebarHeader>
│       ├── <ProjectInfo>
│       ├── <GitInfo>
│       ├── <EnvironmentInfo>
│       └── <ActiveProviders>
│
└── <Overlays>
    ├── <CommandPalette>
    ├── <SettingsModal>
    └── <Notifications>
```

---

## State Management

### Store Architecture

Uses Zustand with **multiple small stores** instead of one large global store:

```typescript
// stores/sessionStore.ts
interface SessionState {
  sessions: Map<string, SessionInfo>;
  activeSessionId: string | null;

  // Actions
  createSession: () => Promise<string>;
  destroySession: (id: string) => Promise<void>;
  setActiveSession: (id: string) => void;
  updateSession: (id: string, updates: Partial<SessionInfo>) => void;
}

export const useSessionStore = create<SessionState>((set, get) => ({
  sessions: new Map(),
  activeSessionId: null,

  createSession: async () => {
    const id = await invoke<string>("create_session", { /* config */ });
    set((state) => {
      const sessions = new Map(state.sessions);
      sessions.set(id, { id, title: `Session ${sessions.size + 1}` });
      return { sessions, activeSessionId: id };
    });
    return id;
  },
  // ...
}));
```

```typescript
// stores/terminalStore.ts
interface TerminalState {
  // Per-session terminal state
  grids: Map<string, GridData>;
  cursors: Map<string, CursorData>;
  scrollOffsets: Map<string, number>;

  // Command capture blocks (Warp-style block mode)
  capturedBlocks: Map<string, CapturedBlock[]>;  // sessionId -> blocks

  // Actions
  updateGrid: (sessionId: string, changes: CellChange[]) => void;
  updateCursor: (sessionId: string, cursor: CursorData) => void;
  setScrollOffset: (sessionId: string, offset: number) => void;

  // Command capture actions
  addCapturedBlock: (sessionId: string, block: CapturedBlock) => void;
  updateOutputChunk: (commandId: string, output: string) => void;
  finalizeOutput: (commandId: string, output: string, durationMs: number) => void;
}

// CapturedBlock replaces the older FrontendBlock concept
interface CapturedBlock {
  id: string;           // commandId (UUID)
  command: string;      // the command text
  cwd: string;          // working directory at submission
  gitBranch?: string;   // git branch at submission
  submittedAt: number;  // timestamp
  outputText: string;   // accumulated output (plain text with ANSI codes)
  lastChunkAt?: number; // timestamp of last chunk received
  durationMs?: number;  // total execution time (set on finalize)
}
```

```typescript
// stores/completionStore.ts
interface CompletionState {
  visible: boolean;
  items: CompletionItem[];
  selectedIndex: number;
  position: { x: number; y: number };
  inlineHint: string | null;

  // Actions
  show: (items: CompletionItem[], position: Position) => void;
  hide: () => void;
  selectNext: () => void;
  selectPrevious: () => void;
  accept: () => void;
}
```

```typescript
// stores/contextStore.ts
interface ContextState {
  cwd: string;
  providers: ProviderInfo[];
  projectType: string | null;
  gitInfo: GitInfo | null;
  environmentInfo: EnvironmentInfo;

  // Actions (updated via events from Rust)
  setContext: (context: ContextData) => void;
}
```

```typescript
// stores/settingsStore.ts
interface SettingsState {
  theme: string;
  fontSize: number;
  fontFamily: string;
  cursorStyle: "block" | "underline" | "bar";
  cursorBlink: boolean;
  scrollbackSize: number;
  sidebarVisible: boolean;
  contextSidebarVisible: boolean;

  // Actions
  updateSetting: <K extends keyof SettingsState>(key: K, value: SettingsState[K]) => void;
  loadSettings: () => Promise<void>;
  saveSettings: () => Promise<void>;
}
```

---

## Terminal Rendering

### Rendering Strategy

Terminal rendering is the most performance-critical part of the frontend. There are three
approaches:

| Approach | Pros | Cons |
|---|---|---|
| **DOM per cell** | Simple, accessible | Slow for large grids (80x24 = 1,920 elements) |
| **Canvas 2D** | Fast, pixel control | No text selection, accessibility issues |
| **DOM per row** | Good balance | Moderate complexity |

**Decision: DOM per row** - each row is a `<div>`, text rendering uses
`<span>` with inline styles for attributes. Reasons:
- Text selection works naturally
- Better accessibility than Canvas
- Performance is fast enough with virtualization

### Virtualized Rendering

Only render rows visible in the viewport + buffer:

```typescript
interface VirtualizedGridProps {
  totalRows: number;        // scrollback + visible
  visibleRows: number;      // viewport height
  rowHeight: number;        // px per row (font-size × line-height)
  scrollOffset: number;     // current scroll position
  renderRow: (index: number) => React.ReactNode;
}

function VirtualizedGrid({ totalRows, visibleRows, rowHeight, scrollOffset, renderRow }: VirtualizedGridProps) {
  const bufferRows = 5; // Extra rows above/below viewport
  const startRow = Math.max(0, scrollOffset - bufferRows);
  const endRow = Math.min(totalRows, scrollOffset + visibleRows + bufferRows);

  return (
    <div style={{ height: totalRows * rowHeight, position: "relative" }}>
      {Array.from({ length: endRow - startRow }, (_, i) => {
        const rowIndex = startRow + i;
        return (
          <div
            key={rowIndex}
            style={{
              position: "absolute",
              top: rowIndex * rowHeight,
              height: rowHeight,
              width: "100%",
            }}
          >
            {renderRow(rowIndex)}
          </div>
        );
      })}
    </div>
  );
}
```

### Cell Rendering

```typescript
interface TerminalCell {
  content: string;        // Character(s)
  fg: string;             // Foreground color (CSS)
  bg: string;             // Background color (CSS)
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strikethrough: boolean;
  dim: boolean;
}

function renderRow(cells: TerminalCell[]): React.ReactNode {
  // Group consecutive cells with same attributes into spans
  const spans: { text: string; style: React.CSSProperties }[] = [];
  let currentSpan = { text: "", style: {} as React.CSSProperties };

  for (const cell of cells) {
    const style = cellToStyle(cell);
    if (stylesEqual(style, currentSpan.style)) {
      currentSpan.text += cell.content || " ";
    } else {
      if (currentSpan.text) spans.push({ ...currentSpan });
      currentSpan = { text: cell.content || " ", style };
    }
  }
  if (currentSpan.text) spans.push(currentSpan);

  return (
    <div className="terminal-row">
      {spans.map((span, i) => (
        <span key={i} style={span.style}>{span.text}</span>
      ))}
    </div>
  );
}
```

### ANSI Output Rendering (Block Mode)

The `AnsiOutput` component (using `src/utils/ansiParser.ts`) renders plain text containing ANSI SGR escape codes as styled HTML. This is used by `CapturedOutputBlock` in the `BlocksView` to display command output with colors.

```typescript
// utils/ansiParser.ts
// Parses ANSI SGR sequences (e.g., \x1b[31m for red, \x1b[1m for bold)
// and produces an array of segments with text and style information.
// The AnsiOutput React component maps these segments to <span> elements.
```

This replaces the older grid-row-slicing approach (FlatBlock) with a simpler text-based rendering pipeline: Rust captures the output as text with ANSI codes via `grid_to_ansi_text()`, and the frontend parses those codes for display.

---

## Input Handling

### Keyboard Input Flow

```typescript
function useTerminalInput(sessionId: string) {
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    // 1. Check for Wit-specific shortcuts first
    if (isWitShortcut(e)) {
      handleWitShortcut(e);
      e.preventDefault();
      return;
    }

    // 2. Check for completion interactions
    if (completionStore.visible) {
      if (handleCompletionKey(e)) {
        e.preventDefault();
        return;
      }
    }

    // 3. Encode key as terminal sequence and send to PTY
    const sequence = encodeKey(e);
    if (sequence) {
      invoke("send_input", { sessionId, data: sequence });
      e.preventDefault();
    }
  }, [sessionId]);

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}
```

### Key Encoding

```typescript
function encodeKey(e: KeyboardEvent): string | null {
  const { key, ctrlKey, altKey, shiftKey } = e;

  // Special keys
  if (key === "Enter")     return "\r";
  if (key === "Backspace") return "\x7f";
  if (key === "Tab")       return "\t";
  if (key === "Escape")    return "\x1b";
  if (key === "ArrowUp")   return "\x1b[A";
  if (key === "ArrowDown") return "\x1b[B";
  if (key === "ArrowRight")return "\x1b[C";
  if (key === "ArrowLeft") return "\x1b[D";
  if (key === "Home")      return "\x1b[H";
  if (key === "End")       return "\x1b[F";
  if (key === "PageUp")    return "\x1b[5~";
  if (key === "PageDown")  return "\x1b[6~";
  if (key === "Delete")    return "\x1b[3~";
  if (key === "Insert")    return "\x1b[2~";

  // Function keys
  if (key.startsWith("F") && key.length <= 3) {
    return encodeFunctionKey(parseInt(key.slice(1)));
  }

  // Ctrl + letter
  if (ctrlKey && key.length === 1 && /[a-z]/i.test(key)) {
    return String.fromCharCode(key.toUpperCase().charCodeAt(0) - 64);
  }

  // Alt + key (send as ESC prefix)
  if (altKey && key.length === 1) {
    return "\x1b" + key;
  }

  // Regular character
  if (key.length === 1 && !ctrlKey && !altKey) {
    return key;
  }

  return null;
}
```

---

## Event System (Tauri Integration)

### Listening to Rust Events

```typescript
// hooks/useTerminalEvents.ts
function useTerminalEvents(sessionId: string) {
  const updateGrid = useTerminalStore((s) => s.updateGrid);
  const updateCursor = useTerminalStore((s) => s.updateCursor);

  useEffect(() => {
    // Subscribe to terminal output events
    const unlisten = listen<TerminalOutputEvent>(
      `terminal_output_${sessionId}`,
      (event) => {
        const { cells_changed, cursor } = event.payload;
        updateGrid(sessionId, cells_changed);
        updateCursor(sessionId, cursor);
      }
    );

    return () => { unlisten.then((fn) => fn()); };
  }, [sessionId]);
}

// hooks/useCommandCapture.ts
function useCommandCaptureEvents(sessionId: string) {
  const updateOutputChunk = useTerminalStore((s) => s.updateOutputChunk);
  const finalizeOutput = useTerminalStore((s) => s.finalizeOutput);

  useEffect(() => {
    const unlisteners: Promise<UnlistenFn>[] = [];

    // Incremental output chunks during command execution
    unlisteners.push(
      listen<CommandOutputChunk>(
        `command_output_chunk`,
        (event) => {
          const { command_id, output } = event.payload;
          updateOutputChunk(command_id, output);
        }
      )
    );

    // Final output when command completes
    unlisteners.push(
      listen<CommandOutput>(
        `command_output`,
        (event) => {
          const { command_id, output, duration_ms } = event.payload;
          finalizeOutput(command_id, output, duration_ms);
        }
      )
    );

    return () => {
      unlisteners.forEach((p) => p.then((fn) => fn()));
    };
  }, [sessionId]);
}
```

### Invoking Rust Commands

```typescript
// lib/tauri.ts - Type-safe wrappers
import { invoke } from "@tauri-apps/api/core";

export const api = {
  session: {
    create: (config: SessionConfig) =>
      invoke<string>("create_session", { config }),
    destroy: (sessionId: string) =>
      invoke<void>("destroy_session", { sessionId }),
    list: () =>
      invoke<SessionInfo[]>("list_sessions"),
    sendInput: (sessionId: string, data: string) =>
      invoke<void>("send_input", { sessionId, data }),
    resize: (sessionId: string, cols: number, rows: number) =>
      invoke<void>("resize_session", { sessionId, cols, rows }),
  },
  completion: {
    request: (input: string, cursorPos: number) =>
      invoke<CompletionItem[]>("request_completions", { input, cursorPos }),
  },
  context: {
    get: (sessionId: string) =>
      invoke<ContextData>("get_context", { sessionId }),
  },
  config: {
    get: () =>
      invoke<AppConfig>("get_config"),
    set: (config: Partial<AppConfig>) =>
      invoke<void>("set_config", { config }),
  },
};
```

---

## Performance Guidelines

### Rendering Performance

1. **Batch updates** - Combine multiple cell changes into a single React render cycle
2. **Avoid re-renders** - Use `React.memo` for TerminalRow, `useMemo` for styles
3. **Virtualize scrollback** - Only render visible rows
4. **RAF throttling** - Limit render rate with `requestAnimationFrame`
5. **Web Workers** - Process heavy work (diff calculation) on a worker thread if needed

### Memory Management

1. **Scrollback limit** - Default 10,000 lines, configurable
2. **Lazy loading** - Completion data loaded on-demand, not at startup
3. **Cleanup** - Destroy event listeners when session is closed

### Metrics Targets

| Metric | Target |
|---|---|
| Key-to-screen latency | < 16ms (60fps) |
| Completion popup appearance | < 50ms |
| Session switch | < 100ms |
| Cold start (frontend) | < 200ms |
| Memory per session | < 20MB |
