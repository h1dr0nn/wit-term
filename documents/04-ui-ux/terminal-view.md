# Terminal View

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Overview

The terminal view is the central component of Wit. This is where the user interacts with the shell, views output, and enters commands. The design must ensure the terminal operates smoothly, accurately, and with fast responsiveness.

---

## 2. Layout

### 2.1. Position Within Layout

```
+---+--------------------------+---+
|   |                          |   |
| L |     Terminal View        | R |
| e |     (main area)          | i |
| f |                          | g |
| t |                          | h |
|   |                          | t |
| S |                          | S |
| i |                          | i |
| d |                          | d |
| e |                          | e |
|   |                          |   |
+---+--------------------------+---+
```

- **Position**: occupies all remaining space after subtracting sidebars
- **Min width**: 400px (remaining space after sidebars)
- **Min height**: 300px
- **No separate padding**: the terminal grid occupies the entire view area
- **Border**: no border with sidebars (uses a divider colored `--color-border-muted`)

### 2.2. Multiple Terminals

- Each session has its own terminal view
- Only the active session's terminal is displayed
- Inactive terminals continue running in the background but are not rendered
- Switching sessions swaps the terminal view (no transition animation)

---

## 3. Character Grid Rendering

### 3.1. Cell Dimensions

- **Cell width**: calculated from font metrics of JetBrains Mono at the current size
- **Cell height**: `font_size * line_height_ratio`
- **Line height ratio**: 1.3 (default), adjustable in settings
- **Example**: font 14px -> cell width ~8.4px, cell height ~18.2px

### 3.2. Grid Calculation

```
columns = floor(viewport_width / cell_width)
rows = floor(viewport_height / cell_height)
```

- Grid always uses integer column and row counts
- Remaining space (fractional pixels) is evenly distributed as padding on left/right and top/bottom
- When the window resizes, the grid is recalculated and the PTY is notified (SIGWINCH or equivalent)

### 3.3. Rendering Pipeline

1. Receive buffer update from Rust core (via Tauri IPC)
2. Parse attribute changes (colors, bold, italic, underline)
3. Render to canvas or DOM grid
4. Only re-render changed cells (dirty region tracking)

### 3.4. Rendering Method

- **Preferred**: HTML canvas (WebGL if possible) for best performance
- **Fallback**: DOM-based rendering (each cell is a span)
- **Ligatures**: support JetBrains Mono ligatures (!=, =>, ->, etc.)
- **Unicode**: full Unicode support, including wide characters (CJK), emoji

---

## 4. Cursor

### 4.1. Cursor Shapes

| Shape      | Description                    | When to use          |
|------------|--------------------------------|----------------------|
| Block      | Filled rectangle, inverts text color | Default (normal mode) |
| Underline  | Horizontal line under the character | Changed via settings or escape sequence |
| Bar (beam) | Thin vertical line to the left of the character | Insert mode (e.g., in vim) |

### 4.2. Cursor Appearance

- **Color**: `#D4A857` (accent color), changeable in theme
- **Block cursor text**: inverse color (text uses the background color)
- **Opacity when unfocused**: 60%, shows only outline (no fill)

### 4.3. Blink Animation

- **Blink interval**: 530ms on, 530ms off (total cycle 1060ms)
- **Blink stops while typing**: cursor does not blink while user is typing, only blinks after 500ms idle
- **Disable blink**: option in settings, or by `prefers-reduced-motion`
- **CSS implementation**:

```css
@keyframes cursor-blink {
  0%, 49% { opacity: 1; }
  50%, 100% { opacity: 0; }
}

.cursor.blinking {
  animation: cursor-blink 1060ms step-end infinite;
}
```

---

## 5. Text Selection

### 5.1. Selection Methods

| Action                  | Result                           |
|-------------------------|----------------------------------|
| Click + drag            | Select text from start point to end point |
| Double-click            | Select an entire word             |
| Triple-click            | Select an entire line             |
| Shift + Click           | Extend selection from previous position to click position |
| Ctrl + Shift + A        | Select all text in scrollback     |

### 5.2. Word Boundaries

- Word delimiters: spaces, tabs, and the characters `()[]{}|;:'"<>,.!@#$%^&*`
- Double-click on a path (e.g., `/usr/bin/bash`): selects the entire path (recognizes / as part of the word)
- Double-click on a URL: selects the entire URL

### 5.3. Selection Appearance

- **Background**: `--color-primary` with opacity 30% (`#58E6D94D`)
- **Text**: keeps original color (no inversion)
- **Multi-line**: selection covers from end of one line to start of the next line
- **Selection handle**: none (desktop app, not mobile)

### 5.4. Clipboard Integration

- Selection auto-copies to primary selection (Linux) - optional, disabled by default
- Ctrl+Shift+C: copy selection to system clipboard
- Ctrl+Shift+V: paste from system clipboard
- Right-click menu: Copy / Paste options

---

## 6. Scrollback

### 6.1. Scrollback Buffer

- **Default size**: 10,000 lines
- **Max configurable**: 100,000 lines
- **Unlimited option**: yes, but warns about memory usage
- Buffer is stored in Rust core, frontend only renders the visible portion

### 6.2. Scroll Navigation

| Action               | Behavior                             |
|----------------------|--------------------------------------|
| Scroll wheel         | Scroll 3 lines per notch             |
| Shift + scroll       | Scroll 1 page (faster)               |
| Page Up              | Scroll up 1 page                     |
| Page Down            | Scroll down 1 page                   |
| Shift + Home         | Scroll to top of scrollback          |
| Shift + End          | Scroll to bottom (latest output)     |

### 6.3. Scrollbar

- **Default**: completely hidden
- **Show**: when user begins scrolling (fade in)
- **Auto-hide**: after 1.5s of no scrolling (fade out)
- **Position**: right side of terminal view
- **Size**: 6px width, border-radius full
- **Color**: thumb `--color-border`, hover `--color-text-muted`
- **Draggable**: scrollbar thumb can be dragged
- **Option**: "Always show scrollbar" in settings

### 6.4. Scroll Position Indicator

- When viewing scrollback (not at bottom), display a badge in the bottom-right corner: "Scroll to bottom" with an arrow icon
- Click the badge to scroll to bottom immediately
- New output while scrolling: does not auto-scroll down, displays indicator "New output below"

---

## 7. Search in Terminal

### 7.1. Activation

- **Shortcut**: Ctrl+Shift+F
- **UI**: search bar appears in the top-right corner of the terminal view
- **Position**: overlays the terminal, does not push the terminal down

### 7.2. Search Bar UI

```
+--------------------------------------------------+
|  [icon] Search: [_input_________] [N/M] [^] [v] [x] |
+--------------------------------------------------+
```

- Input: auto-focus when opened, placeholder "Search..."
- Count: "3/15" (current match / total matches)
- Navigation: Up/Down buttons or Enter/Shift+Enter
- Close: X button or Escape
- Options (expandable): case sensitive toggle, regex toggle, whole word toggle

### 7.3. Match Highlighting

- **Match background**: `--color-accent` with opacity 30%
- **Current match**: `--color-accent` with opacity 60%, border 1px solid `--color-accent`
- **Match indicator on scrollbar**: small dots colored `--color-accent` on the scrollbar track

### 7.4. Search Behavior

- Searches the entire scrollback buffer
- Incremental search: highlights update as user types
- Wrap around: when reaching the last match, wraps to the first match
- Persist highlights when navigating (until search is closed)

---

## 8. URL Detection & Links

### 8.1. URL Patterns

Recognized patterns:
- `http://` and `https://` URLs
- `file:///` paths
- Email addresses (future)
- File paths (future, e.g., `/path/to/file:42`)

### 8.2. URL Appearance

- **Underline**: dotted underline on hover
- **Color**: keeps original terminal color, only adds underline
- **Cursor**: changes to pointer on hover (while holding Ctrl)

### 8.3. URL Interaction

- **Ctrl + Click**: opens URL in default browser
- **Right-click**: context menu with "Open Link", "Copy Link"
- **Hover tooltip**: shows full URL after 500ms delay (if URL is truncated)
- **Security**: warns when URL points to an IP address or unusual port (optional)

---

## 9. Terminal Bell

### 9.1. Visual Bell (Default)

- **Effect**: brief flash of the entire terminal view
- **Implementation**: overlay flash with `--color-text` opacity 10%, duration 100ms
- **Alternative**: only flash the bell icon in the tab/session title

### 9.2. Audio Bell

- **Default**: disabled
- **Option**: enable in settings
- **Sound**: system bell or custom sound file

### 9.3. Bell Options

- Visual bell (default)
- Audio bell
- Both
- None (completely disabled)
- Urgent hint: set window urgent flag when bell fires and terminal is not focused

---

## 10. Resize Behavior

### 10.1. Resize Flow

1. User resizes window (or sidebar toggle/resize)
2. Terminal view recalculates available space
3. Recalculates `columns` and `rows` from new dimensions
4. Sends SIGWINCH (or equivalent) to PTY with new size
5. Re-renders grid with new dimensions

### 10.2. Content Reflow

- **Reflow on**: when column count changes, lines longer than the new column count will wrap
- **Reflow off** (option): keeps original lines, scrolls horizontally if needed
- Default: reflow on (matches behavior of most terminal emulators)

### 10.3. Minimum Size

- Terminal minimum: 80 columns x 24 rows (standard)
- If window is too small to fit 80x24, reduces but shows a warning
- Absolute minimum: 20 columns x 5 rows

### 10.4. Performance

- Debounce resize events: 16ms (1 frame at 60fps)
- During resize, render at current frame rate (do not wait for resize to finish)
- When resize ends, do a final render and send final size to PTY

---

## 11. Fullscreen Mode

### 11.1. Activation

- **Shortcut**: F11
- **Menu**: View > Fullscreen
- **Button**: icon button on title bar (ArrowsOutSimple)

### 11.2. Behavior

- Fullscreen hides the title bar and taskbar
- Terminal view expands to fill the entire screen
- Sidebars still work normally (toggle via shortcut)
- Press F11 again to exit fullscreen

### 11.3. UI Adaptation

- Hide title bar when fullscreen (or use an overlay title bar that shows on hover over top edge)
- Tab bar (if any) moves inside the terminal area or is hidden
- Overlays (search, completion popup) still work normally

---

## 12. Command Blocks View (Warp-style)

### 12.1. Overview

Wit supports a "block mode" rendering approach inspired by Warp, where each command and its output are displayed as discrete visual blocks rather than a continuous scrolling terminal. This is implemented via the `BlocksView` and `InputBar` components.

### 12.2. Layout

```
┌──────────────────────────────────────────────────────────────┐
│  BlocksView (scrollable list of command blocks)              │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ $ cargo build                     ~/wit-term  main  2s │  │
│  │ ──────────────────────────────────────────────────────  │  │
│  │    Compiling wit-term v0.1.0                            │  │
│  │     Finished dev [unoptimized] target(s) in 2.34s       │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ $ git status                      ~/wit-term  main  <1s│  │
│  │ ──────────────────────────────────────────────────────  │  │
│  │ On branch main                                          │  │
│  │ Changes not staged for commit:                          │  │
│  │   modified:   src/lib.rs                                │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│  InputBar                                                    │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ ~/wit-term (main)                                       │  │
│  │ $ [cursor]                                              │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### 12.3. CapturedOutputBlock

Each command block (`CapturedOutputBlock` component) displays:

| Element | Description |
|---|---|
| **Command** | The submitted command text |
| **CWD** | Working directory at submission time |
| **Git branch** | Branch name (if in a git repo) |
| **Duration** | Execution time badge (shown after completion) |
| **Output** | Command output rendered with ANSI colors via `AnsiOutput` |

### 12.4. AnsiOutput Component

The `AnsiOutput` component uses `src/utils/ansiParser.ts` to parse plain text containing ANSI SGR escape codes and render it as styled `<span>` elements. This replaces the older approach of slicing grid rows (`FlatBlock`) with a simpler text-based pipeline.

Supported SGR codes:
- Foreground/background colors (standard, 256-color, true color)
- Bold, italic, underline, strikethrough, dim
- Reset (`\x1b[0m`)

### 12.5. InputBar

The `InputBar` component provides a Warp-style command input area:

- Displays the current working directory and git branch inside the input area
- Taller than a traditional single-line prompt
- On Enter: calls `submit_command` IPC and creates a new `CapturedBlock`
- Supports standard terminal shortcuts (Ctrl+C, Ctrl+D, etc.) by forwarding to `send_input`

### 12.6. Data Flow

1. User types in `InputBar` and presses Enter
2. Frontend generates a `commandId` (UUID) and calls `addCapturedBlock`
3. Frontend invokes `submit_command(sessionId, command, commandId)`
4. Rust sets `CaptureState` and writes command to PTY
5. PTY read loop emits `command_output_chunk` events with incremental output
6. Frontend appends chunks to `CapturedBlock.outputText`
7. On completion, Rust emits `command_output` with full output and duration
8. Frontend calls `finalizeOutput` to mark the block complete

---

## 13. Performance Requirements

| Metric                  | Target                         |
|-------------------------|--------------------------------|
| Input latency           | < 10ms (keystroke to screen)   |
| Scroll FPS              | 60fps                          |
| Large output rendering  | > 100,000 lines/sec throughput |
| Resize latency          | < 16ms (1 frame)               |
| Memory per terminal     | < 50MB (10K scrollback)        |

**Benchmarks**: compare with Alacritty, WezTerm, Kitty to ensure competitive performance.
