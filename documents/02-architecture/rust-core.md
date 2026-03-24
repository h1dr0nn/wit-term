# Rust Core Architecture

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

The Rust core is the heart of Wit. All critical logic - PTY management, terminal
emulation, ANSI parsing, context detection, completion matching - runs in the
Rust layer to ensure performance and memory safety.

---

## Crate Structure

```
src-tauri/
├── Cargo.toml              # Workspace root
└── src/
    ├── lib.rs              # Library exports
    ├── main.rs             # Tauri entry point
    ├── pty/                # PTY abstraction layer
    ├── terminal/           # Terminal emulation engine
    ├── parser/             # ANSI escape sequence parser
    ├── context/            # Context detection engine
    ├── completion/         # Completion matching engine
    ├── session/            # Session lifecycle management
    ├── config/             # Configuration management
    └── commands/           # Tauri IPC command handlers
```

In the long term, this can be split into a workspace with multiple crates:

```toml
# Cargo.toml (workspace)
[workspace]
members = [
    "crates/wit-pty",        # PTY abstraction
    "crates/wit-terminal",   # Terminal emulation
    "crates/wit-parser",     # ANSI parser
    "crates/wit-context",    # Context engine
    "crates/wit-completion", # Completion engine
    "crates/wit-app",        # Tauri app (binary)
]
```

**Note:** Initially keep a flat structure within a single crate. Only split when
the codebase is large enough and boundaries are clearly defined.

---

## Module: PTY (`pty/`)

### Responsibility
- Create pseudo-terminal pair (master/slave)
- Spawn shell process attached to the PTY slave
- Read output from PTY master (non-blocking)
- Write input to PTY master
- Resize PTY when the terminal window changes size
- Cleanup when the session ends

### Abstraction Layer

```rust
/// Platform-agnostic PTY interface
pub trait Pty: Send {
    /// Read bytes from the PTY master (shell output)
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;

    /// Write bytes to the PTY master (user input)
    fn write(&mut self, data: &[u8]) -> io::Result<usize>;

    /// Resize the PTY to new dimensions
    fn resize(&self, cols: u16, rows: u16) -> io::Result<()>;

    /// Get the PID of the child process
    fn child_pid(&self) -> u32;

    /// Wait for the child process to exit
    fn wait(&mut self) -> io::Result<ExitStatus>;
}

/// Platform-specific implementations
#[cfg(unix)]
pub struct UnixPty { ... }

#[cfg(windows)]
pub struct ConPty { ... }

/// Factory function
pub fn create_pty(config: &PtyConfig) -> io::Result<Box<dyn Pty>> {
    #[cfg(unix)]
    { Ok(Box::new(UnixPty::new(config)?)) }

    #[cfg(windows)]
    { Ok(Box::new(ConPty::new(config)?)) }
}
```

### PtyConfig

```rust
pub struct PtyConfig {
    /// Shell to launch (e.g., "/bin/zsh", "powershell.exe")
    pub shell: PathBuf,

    /// Arguments to pass to the shell
    pub args: Vec<String>,

    /// Initial working directory
    pub cwd: PathBuf,

    /// Environment variables to set
    pub env: HashMap<String, String>,

    /// Initial terminal size
    pub cols: u16,
    pub rows: u16,
}
```

### I/O Loop

```rust
/// Runs on a dedicated thread per session
fn pty_read_loop(
    pty: &mut dyn Pty,
    parser: &mut AnsiParser,
    grid: &Arc<Mutex<Grid>>,
    event_tx: &Sender<TerminalEvent>,
) {
    let mut buf = [0u8; 4096];
    loop {
        match pty.read(&mut buf) {
            Ok(0) => break, // EOF - shell exited
            Ok(n) => {
                let bytes = &buf[..n];
                // Parse ANSI sequences and update grid
                let actions = parser.advance(bytes);
                let mut grid = grid.lock().unwrap();
                for action in actions {
                    grid.apply(action);
                }
                // Notify frontend of changes
                event_tx.send(TerminalEvent::Render).ok();
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    event_tx.send(TerminalEvent::Exited).ok();
}
```

---

## Module: Terminal Emulation (`terminal/`)

### Responsibility
- Manage the terminal grid (2D array of cells)
- Handle cursor movement, scrolling, line wrapping
- Implement VT100/xterm terminal modes
- Alternate screen buffer management
- Scrollback buffer

### Grid Architecture

```rust
/// A single cell in the terminal grid
#[derive(Clone, Default)]
pub struct Cell {
    /// The character displayed (grapheme cluster)
    pub content: CompactString,

    /// Visual attributes
    pub attrs: CellAttrs,

    /// Whether this cell has been modified since last render
    pub dirty: bool,
}

#[derive(Clone, Default)]
pub struct CellAttrs {
    pub fg: Color,
    pub bg: Color,
    pub flags: AttrFlags, // bold, italic, underline, etc.
}

bitflags! {
    pub struct AttrFlags: u16 {
        const BOLD          = 0b0000_0001;
        const ITALIC        = 0b0000_0010;
        const UNDERLINE     = 0b0000_0100;
        const STRIKETHROUGH = 0b0000_1000;
        const INVERSE       = 0b0001_0000;
        const HIDDEN        = 0b0010_0000;
        const DIM           = 0b0100_0000;
        const BLINK         = 0b1000_0000;
    }
}

/// Terminal color representation
pub enum Color {
    Named(NamedColor),    // 16 standard ANSI colors
    Indexed(u8),          // 256-color palette
    Rgb(u8, u8, u8),      // True color (24-bit)
    Default,              // Terminal default fg/bg
}
```

### Buffer Management

```rust
pub struct Grid {
    /// Visible area: cols x rows
    cells: Vec<Vec<Cell>>,

    /// Scrollback buffer (lines that scrolled off the top)
    scrollback: VecDeque<Vec<Cell>>,

    /// Maximum scrollback lines
    max_scrollback: usize,

    /// Grid dimensions
    cols: usize,
    rows: usize,

    /// Cursor state
    cursor: Cursor,

    /// Terminal modes
    modes: TerminalModes,

    /// Scroll region (top, bottom)
    scroll_region: (usize, usize),
}

pub struct Cursor {
    pub col: usize,
    pub row: usize,
    pub visible: bool,
    pub shape: CursorShape,
    pub attrs: CellAttrs,  // Attributes applied to new characters
}

pub enum CursorShape {
    Block,
    Underline,
    Bar,
}
```

### Terminal Modes

```rust
pub struct TerminalModes {
    /// DEC Private Mode flags
    pub cursor_keys: bool,          // DECCKM
    pub origin_mode: bool,          // DECOM
    pub auto_wrap: bool,            // DECAWM (default: true)
    pub cursor_visible: bool,       // DECTCEM (default: true)
    pub alt_screen: bool,           // Alternate screen buffer
    pub bracketed_paste: bool,      // Bracketed paste mode
    pub focus_reporting: bool,      // Focus in/out events
    pub mouse_tracking: MouseMode,  // Mouse event reporting

    /// Standard modes
    pub insert_mode: bool,          // IRM
    pub linefeed_mode: bool,        // LNM
}

pub enum MouseMode {
    None,
    Click,       // Report button press only
    Drag,        // Report button press + drag
    Motion,      // Report all motion
}
```

### Module: Strip Utilities (`terminal/strip.rs`)

The `strip.rs` module provides text extraction and cleanup utilities for the command capture pipeline:

```rust
/// Convert grid rows to text with ANSI SGR escape codes preserved
pub fn grid_to_ansi_text(grid: &Grid, start_row: usize, end_row: usize) -> String;

/// Strip ANSI escape sequences from text, returning plain text
pub fn strip_ansi(text: &str) -> String;

/// Remove the echoed command line and shell prompt from captured output
pub fn strip_echo_and_prompt(output: &str, command: &str) -> String;

/// Extract the current working directory from a shell prompt line
/// by reading the grid row at the cursor position (supports PS1/CMD prompt formats)
pub fn extract_cwd_from_prompt(grid: &Grid, cursor_row: usize) -> Option<String>;
```

**Usage:** Called by the PTY read loop when a command capture is active. `grid_to_ansi_text` converts the terminal grid cells (with their color attributes) into a string containing ANSI SGR codes, which the frontend can parse for colored rendering. `strip_echo_and_prompt` removes the echoed command and any trailing prompt so only the actual command output remains.

---

## Module: ANSI Parser (`parser/`)

### Responsibility
- Parse raw bytes from PTY output into structured actions
- Implement state machine following ANSI/VT100/xterm standards
- Handle incomplete sequences (partial reads)
- Provide structured output for the terminal emulator

### State Machine

The parser uses a state machine based on
[Paul Williams' VT parser](https://vt100.net/emu/dec_ansi_parser):

```rust
pub enum ParserState {
    Ground,         // Normal text processing
    Escape,         // After ESC
    EscapeIntermed, // ESC + intermediate bytes
    CsiEntry,       // After CSI (ESC [)
    CsiParam,       // Reading CSI parameters
    CsiIntermed,    // CSI intermediate bytes
    CsiIgnore,      // Invalid CSI, consume until final
    OscString,      // OSC string content
    DcsEntry,       // After DCS
    DcsParam,       // DCS parameters
    DcsIntermed,    // DCS intermediate
    DcsPassthrough, // DCS passthrough content
    DcsIgnore,      // Invalid DCS
    SosPmApc,       // SOS/PM/APC strings (ignored)
}
```

### Actions

```rust
/// Actions produced by the parser
pub enum Action {
    /// Print a character to the screen
    Print(char),

    /// Execute a C0/C1 control code
    Execute(u8),

    /// CSI dispatch - most common escape sequences
    CsiDispatch {
        params: Vec<u16>,
        intermediates: Vec<u8>,
        final_byte: u8,
    },

    /// ESC dispatch
    EscDispatch {
        intermediates: Vec<u8>,
        final_byte: u8,
    },

    /// OSC dispatch (title, hyperlinks, etc.)
    OscDispatch(Vec<Vec<u8>>),

    /// DCS hook/put/unhook
    DcsHook { params: Vec<u16>, intermediates: Vec<u8>, final_byte: u8 },
    DcsPut(u8),
    DcsUnhook,
}
```

### Handler

The handler converts Actions into Grid operations:

```rust
impl Grid {
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Print(c) => self.put_char(c),
            Action::Execute(byte) => match byte {
                0x08 => self.backspace(),     // BS
                0x09 => self.tab(),           // HT
                0x0A => self.linefeed(),      // LF
                0x0D => self.carriage_return(),// CR
                0x07 => self.bell(),          // BEL
                _ => {}
            },
            Action::CsiDispatch { params, final_byte, .. } => {
                match final_byte {
                    b'A' => self.cursor_up(params),
                    b'B' => self.cursor_down(params),
                    b'C' => self.cursor_forward(params),
                    b'D' => self.cursor_backward(params),
                    b'H' | b'f' => self.cursor_position(params),
                    b'J' => self.erase_display(params),
                    b'K' => self.erase_line(params),
                    b'm' => self.set_graphics(params),  // SGR
                    b'r' => self.set_scroll_region(params),
                    // ... many more handlers
                    _ => {}
                }
            },
            // ... other action types
        }
    }
}
```

---

## Module: Context Engine (`context/`)

Details at [03-specifications/context-engine.md](../03-specifications/context-engine.md).

### Architecture Overview

```rust
pub struct ContextEngine {
    providers: Vec<Box<dyn ContextProvider>>,
    watcher: FileSystemWatcher,
    current_context: Arc<RwLock<ProjectContext>>,
}

pub trait ContextProvider: Send + Sync {
    /// Provider name (e.g., "git", "node", "docker")
    fn name(&self) -> &str;

    /// Marker files to detect this provider
    fn markers(&self) -> &[&str];

    /// Detect whether this provider is active at this directory
    fn detect(&self, dir: &Path) -> bool;

    /// Gather detailed context info
    fn gather(&self, dir: &Path) -> ContextInfo;

    /// List of completion sets to load
    fn completion_sets(&self) -> Vec<String>;
}
```

---

## Module: Completion Engine (`completion/`)

Details at [03-specifications/completion-engine.md](../03-specifications/completion-engine.md).

### Architecture Overview

```rust
pub struct CompletionEngine {
    sources: Vec<Box<dyn CompletionSource>>,
    matcher: FuzzyMatcher,
    history: CommandHistory,
}

pub trait CompletionSource: Send + Sync {
    /// Source name
    fn name(&self) -> &str;

    /// Return completions for the current input
    fn complete(&self, request: &CompletionRequest) -> Vec<Completion>;
}

pub struct CompletionRequest {
    pub input: String,           // Full input line
    pub cursor_pos: usize,       // Cursor position in input
    pub cwd: PathBuf,            // Current working directory
    pub context: ProjectContext, // Active context
}

pub struct Completion {
    pub text: String,            // Text to insert
    pub display: String,         // Text shown in popup
    pub description: String,     // Description
    pub kind: CompletionKind,    // Command, flag, argument, path
    pub score: f64,              // Ranking score
    pub source: String,          // Which source provided this
}
```

---

## Module: Session Manager (`session/`)

```rust
pub struct SessionManager {
    sessions: HashMap<SessionId, Session>,
    active_session: Option<SessionId>,
    next_id: AtomicU32,
}

pub struct Session {
    pub id: SessionId,
    pub title: String,
    pub pty: Box<dyn Pty>,
    pub grid: Arc<Mutex<Grid>>,
    pub context: Arc<RwLock<ProjectContext>>,
    pub shell: ShellInfo,
    pub created_at: Instant,
    pub cwd: PathBuf,

    /// Command capture state, shared between IPC handler and PTY reader threads
    pub capture_state: Arc<Mutex<CaptureState>>,

    // Thread handles
    read_thread: Option<JoinHandle<()>>,
    shutdown_tx: Option<Sender<()>>,
}

/// Tracks the currently executing command for output capture.
/// Set by submit_command, read by the PTY read loop, cleared on command completion.
pub struct CaptureState {
    /// ID of the active command (None if no capture in progress)
    pub active_command_id: Option<String>,
    /// The command text being captured
    pub active_command: Option<String>,
    /// Grid row where capture started (to know which rows contain output)
    pub start_cursor_row: Option<usize>,
    /// When the command was submitted
    pub started_at: Option<Instant>,
}

pub struct ShellInfo {
    pub path: PathBuf,      // e.g., /bin/zsh
    pub name: String,       // e.g., "zsh"
    pub version: String,    // e.g., "5.9"
}

impl SessionManager {
    pub fn create_session(&mut self, config: SessionConfig) -> Result<SessionId>;
    pub fn destroy_session(&mut self, id: SessionId) -> Result<()>;
    pub fn get_session(&self, id: SessionId) -> Option<&Session>;
    pub fn get_session_mut(&mut self, id: SessionId) -> Option<&mut Session>;
    pub fn list_sessions(&self) -> Vec<SessionInfo>;
    pub fn set_active(&mut self, id: SessionId) -> Result<()>;

    /// Atomically set capture state and write command to PTY.
    /// Called by the submit_command IPC handler.
    pub fn submit_command(&mut self, id: SessionId, command: String, command_id: String) -> Result<()>;
}
```

---

## Dependencies (Rust Crates)

### Core Dependencies

```toml
[dependencies]
tauri = { version = "2", features = ["shell-open"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
crossbeam-channel = "0.5"
parking_lot = "0.12"         # Faster Mutex/RwLock
bitflags = "2"
compact_str = "0.8"          # Small string optimization
unicode-width = "0.2"        # Character width detection (wcwidth equivalent)
toml = "0.8"                 # Config/completion data parsing
notify = "7"                 # File system watching
dirs = "6"                   # Standard directory paths
log = "0.4"
env_logger = "0.11"

# Platform-specific
[target.'cfg(unix)'.dependencies]
nix = { version = "0.29", features = ["term", "process"] }
libc = "0.2"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_System_Console",
    "Win32_System_Threading",
    "Win32_Security",
] }
winpty-rs = "0.3"            # or conpty API directly
```

### Development Dependencies

```toml
[dev-dependencies]
criterion = "0.5"            # Benchmarking
proptest = "1"               # Property-based testing
tempfile = "3"               # Temporary files for tests
insta = "1"                  # Snapshot testing
```
