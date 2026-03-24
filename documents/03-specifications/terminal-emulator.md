# Terminal Emulation Engine

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

The terminal emulation engine is the module responsible for simulating the behavior of a hardware terminal (VT100/xterm) in software. This module receives actions from the ANSI parser and updates the terminal grid - including cursor movement, text rendering, scrolling, mode switching, and screen buffer management.

The module is located at `src-tauri/src/terminal/` and serves as the central connection point between the ANSI parser (input) and the frontend renderer (output).

---

## Compatibility Targets

### Primary Standards

| Standard | Scope | Support Level |
|---|---|---|
| **VT100** | Basic cursor, scroll, character sets | Full |
| **VT102** | Auto-repeat, printer (printer ignored) | Full (except printer) |
| **VT220** | 8-bit controls, DECRQM, DECSCL | Partial |
| **VT320** | Rectangular area ops | Selective |
| **VT420** | Rectangular editing | Selective |
| **xterm** | 256-color, true color, mouse, bracketed paste, OSC | Full |
| **ECMA-48** | Standard escape sequences | Full |

### Conformance Approach

Wit does not need to implement 100% of every legacy DEC feature. The goal is **xterm-256color compatible** - sufficient to run all modern terminal applications:

- `vim`, `neovim`, `emacs` - full TUI support
- `tmux`, `screen` - multiplexer compatibility
- `htop`, `btop`, `top` - system monitors
- `git log`, `git diff` - pager/color output
- `fzf`, `bat`, `ripgrep` - modern CLI tools
- `npm`, `cargo`, `docker` - build tool output
- `ssh` - remote sessions

### TERM Variable

Wit sets `TERM=xterm-256color` for shell sessions. Compatibility with the corresponding terminfo entry must be ensured.

---

## Terminal Modes

### DEC Private Modes (DECSET/DECRST)

Enabled via `CSI ? Pm h` (DECSET), disabled via `CSI ? Pm l` (DECRST).

| Mode | Name | Default | Description |
|---|---|---|---|
| 1 | DECCKM | OFF | Cursor keys send application sequences (ESC O A instead of ESC [ A) |
| 2 | DECANM | ON | ANSI mode (vs VT52 mode). Wit always stays in ANSI mode |
| 3 | DECCOLM | OFF | 132 column mode. Wit handles resize separately; this mode only triggers a resize event |
| 4 | DECSCLM | OFF | Smooth scroll. Wit ignores this (always uses jump scroll) |
| 5 | DECSCNM | OFF | Reverse video - swap default fg/bg for the entire screen |
| 6 | DECOM | OFF | Origin mode - cursor addressing relative to scroll region |
| 7 | DECAWM | ON | Auto-wrap - wrap to next line when cursor exceeds right margin |
| 8 | DECARM | ON | Auto-repeat keys. Handled by OS; Wit ignores this |
| 9 | X10 Mouse | OFF | X10 mouse reporting - button press only |
| 12 | Cursor blink | OFF | Start/stop cursor blinking |
| 25 | DECTCEM | ON | Cursor visibility - show/hide cursor |
| 45 | Reverse wraparound | OFF | Allow backspace to wrap to end of previous line |
| 47 | Alternate screen | OFF | Switch to alternate screen buffer (legacy) |
| 66 | DECNKM | OFF | Application keypad mode |
| 69 | DECLRMM | OFF | Left/right margin mode (vertical split scroll) |
| 1000 | Mouse click | OFF | Report mouse button press/release events |
| 1002 | Mouse drag | OFF | Report button press + motion while button held |
| 1003 | Mouse any | OFF | Report all mouse motion events |
| 1004 | Focus events | OFF | Report focus in/out events (CSI I / CSI O) |
| 1005 | UTF-8 mouse | OFF | UTF-8 mouse coordinate encoding |
| 1006 | SGR mouse | OFF | SGR-style mouse encoding (preferred, no 223 limit) |
| 1007 | Alternate scroll | OFF | Scroll wheel sends cursor up/down in alternate screen |
| 1034 | Meta sends ESC | ON | 8th bit interpretation |
| 1047 | Alternate screen | OFF | Switch to/from alternate screen buffer |
| 1048 | Save cursor | - | Save/restore cursor position (paired with 1047) |
| 1049 | Alt screen + cursor | OFF | Combine 1047 + 1048: save cursor, switch alt screen, clear |
| 2004 | Bracketed paste | OFF | Wrap pasted text in ESC [200~ ... ESC [201~ |
| 2026 | Synchronized output | OFF | Begin/end synchronized update (reduce flicker) |

### Standard Modes (SM/RM)

Enabled via `CSI Pm h`, disabled via `CSI Pm l`.

| Mode | Name | Default | Description |
|---|---|---|---|
| 2 | KAM | OFF | Keyboard action mode - lock keyboard |
| 4 | IRM | OFF | Insert mode - characters push existing chars right |
| 12 | SRM | OFF | Send/receive mode - local echo |
| 20 | LNM | OFF | Linefeed/newline mode - LF implies CR |

---

## Character Processing

### UTF-8 Decoding

Wit processes input as a byte stream and decodes UTF-8 according to RFC 3629:

```
Bytes   | Bits | Range
1 byte  | 7    | U+0000 - U+007F
2 bytes | 11   | U+0080 - U+07FF
3 bytes | 16   | U+0800 - U+FFFF
4 bytes | 21   | U+10000 - U+10FFFF
```

**UTF-8 error handling:**
- Invalid byte - replace with U+FFFD (REPLACEMENT CHARACTER)
- Overlong encoding - reject, replace with U+FFFD
- Surrogate halves (U+D800-U+DFFF) - reject
- Codepoints > U+10FFFF - reject
- Incomplete sequence at the end of buffer - retain bytes, wait for more data

**Implementation:**

The ANSI parser state machine (`state_machine.rs`) includes a built-in multi-byte UTF-8 decoder. Rather than a separate `Utf8Decoder` struct, the parser itself maintains UTF-8 decoding state:

```rust
// In the parser state machine (state_machine.rs)
pub struct StateMachine {
    // ... parser state fields ...

    /// Buffer for in-progress multi-byte UTF-8 sequence
    utf8_buf: [u8; 4],
    /// Number of bytes accumulated so far
    utf8_len: u8,
    /// Total bytes needed for the current sequence
    utf8_needed: u8,
}
```

When the parser encounters a leading byte with high bits set (indicating a multi-byte sequence), it buffers bytes in `utf8_buf` until `utf8_len == utf8_needed`, then decodes the complete codepoint and emits a `Print(char)` action. Incomplete sequences at buffer boundaries are preserved across `advance()` calls.
```

### Grapheme Clusters

A "character" displayed on the terminal may consist of multiple Unicode codepoints:

- **Base character** + combining marks: `e` = `e` + `\u{0301}` (combining acute)
- **Emoji sequences**: `👨‍💻` = `👨` + ZWJ + `💻`
- **Flag sequences**: `🇻🇳` = `🇻` + `🇳` (Regional Indicators)

**Rules:**
1. Each `Cell` stores one grapheme cluster (using `CompactString`)
2. Combining characters are appended to the `Cell.content` of the current cell
3. Zero-width joiners (ZWJ) are treated as combining characters
4. The `unicode-segmentation` crate is used to determine grapheme boundaries

### Wide Characters (CJK)

East Asian Wide characters occupy 2 columns on the terminal:

- CJK Unified Ideographs (U+4E00-U+9FFF): Chinese characters
- CJK Compatibility Ideographs
- Katakana, Hiragana (fullwidth forms)
- Hangul syllables
- Fullwidth Latin (A, B, ...)
- Some emoji

**Handling:**
1. Use the `unicode-width` crate (added as a dependency) to determine character width (wcwidth equivalent)
2. The `handle_print` method in the emulator checks `UnicodeWidthChar::width(c)` for every printed character
3. Wide character occupies the current cell + a spacer cell in the next column
4. The spacer cell is marked as `WIDE_CONTINUATION` (placeholder)
5. When overwriting a wide character or continuation cell - clear the entire pair
6. Wide character at the last column - wrap to the next line (do not split)

```rust
impl Cell {
    pub fn is_wide(&self) -> bool;
    pub fn is_wide_continuation(&self) -> bool;
}

impl Grid {
    fn put_char(&mut self, c: char) {
        let width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);

        match width {
            0 => {
                // Combining character - append to previous cell
                self.append_to_current_cell(c);
            }
            1 => {
                // Normal character - occupy 1 cell
                self.set_current_cell(c);
                self.advance_cursor(1);
            }
            2 => {
                // Wide character - occupy 2 cells
                if self.cursor.col >= self.cols - 1 {
                    // Not enough space on current line - wrap
                    self.wrap_cursor();
                }
                self.set_current_cell_wide(c);
                self.advance_cursor(2);
            }
            _ => {} // Ignore
        }
    }
}
```

### Combining Characters

Combining characters (U+0300-U+036F and many other ranges) do not occupy their own cell but attach to the base character before them:

```
Input:  'e' U+0301    -> Cell content: "e" (1 cell)
Input:  'a' U+0308 U+0301 -> Cell content: "a&#x308;&#x301;" (1 cell, 2 combining marks)
```

**Limit:** Maximum 4 combining characters per cell to prevent abuse.

---

## Line Processing

### Line Wrapping (DECAWM)

When auto-wrap mode is ON (default):

1. Cursor is at the last column and receives another printable character:
   - Set flag `pending_wrap = true` on cursor
   - Character is not yet written
2. Next character:
   - If `pending_wrap`: wrap cursor to new line (or scroll), clear flag
   - Write character at the new position

**Note:** Cursor movement commands (CUP, CUF, etc.) clear the `pending_wrap` flag without triggering a wrap.

When auto-wrap mode is OFF:
- Characters overwrite the last cell continuously; the cursor does not move

### Scroll Regions (DECSTBM)

`CSI Pt ; Pb r` - set scroll region from line `Pt` to line `Pb` (1-indexed).

```rust
pub struct ScrollRegion {
    pub top: usize,    // Inclusive, 0-indexed internally
    pub bottom: usize, // Inclusive, 0-indexed internally
}
```

**Rules:**
1. Default: top = 0, bottom = rows - 1 (entire screen)
2. `Pt` must be less than `Pb`; both must be within screen bounds
3. When DECOM is ON: cursor addressing is relative to scroll region
4. When DECOM is OFF: cursor addressing is absolute, but scrolling only occurs within the region
5. Linefeed at the last line of the scroll region - scroll the region up by 1 line
6. Reverse index at the first line of the scroll region - scroll the region down by 1 line
7. Setting scroll region - move cursor to (0,0) or (top, 0) depending on DECOM

### Insert/Delete Lines

**Insert Lines (IL):** `CSI Pn L`
- Insert `Pn` blank lines at the cursor row
- Lines below are pushed down; lines exceeding the scroll region bottom are lost
- Cursor moves to column 0

**Delete Lines (DL):** `CSI Pn M`
- Delete `Pn` lines starting from the cursor row
- Lines below are pulled up; blank lines are added at the scroll region bottom
- Cursor moves to column 0

**Insert Characters (ICH):** `CSI Pn @`
- Insert `Pn` blank characters at the cursor position
- Characters after are pushed right; characters exceeding the right margin are lost

**Delete Characters (DCH):** `CSI Pn P`
- Delete `Pn` characters starting from the cursor position
- Characters after are pulled left; blank characters are added at the right margin

---

## Screen Operations

### Erase Operations

**Erase in Display (ED):** `CSI Ps J`

| Ps | Behavior |
|---|---|
| 0 (default) | Erase from cursor to end of screen |
| 1 | Erase from beginning of screen to cursor (inclusive) |
| 2 | Erase entire screen |
| 3 | Erase scrollback buffer (xterm extension) |

**Erase in Line (EL):** `CSI Ps K`

| Ps | Behavior |
|---|---|
| 0 (default) | Erase from cursor to end of line |
| 1 | Erase from beginning of line to cursor (inclusive) |
| 2 | Erase entire line |

**Erase Characters (ECH):** `CSI Pn X`
- Erase `Pn` characters starting from cursor; cursor does not move

**General rules for erase:**
- "Erase" means fill with space character using the current background color
- Erase does not move the cursor (unless the spec says otherwise)
- Erase respects current SGR background color attribute

### Scroll Operations

**Scroll Up (SU):** `CSI Pn S`
- Scroll content within the scroll region up by `Pn` lines
- Blank lines are added at the bottom of the scroll region
- Lines scrolled off the top are added to the scrollback buffer

**Scroll Down (SD):** `CSI Pn T`
- Scroll content within the scroll region down by `Pn` lines
- Blank lines are added at the top of the scroll region
- Lines scrolled off the bottom are lost (not added to scrollback)

**Index (IND):** `ESC D`
- Move cursor down 1 line; scroll if at bottom of scroll region

**Reverse Index (RI):** `ESC M`
- Move cursor up 1 line; scroll down if at top of scroll region

**Next Line (NEL):** `ESC E`
- Move cursor down 1 line + carriage return

### Alternate Screen Buffer

The alternate screen buffer allows applications (vim, less, htop) to use a separate screen; when exiting, the original screen is restored.

**Activation sequences:**
- `CSI ? 1049 h` - Save cursor + switch to alt screen + clear (preferred)
- `CSI ? 47 h` - Switch to alt screen (legacy, does not save cursor)
- `CSI ? 1047 h` - Switch to alt screen

**Deactivation:**
- `CSI ? 1049 l` - Switch to main screen + restore cursor
- `CSI ? 47 l` - Switch to main screen
- `CSI ? 1047 l` - Switch to main screen + clear alt

**Implementation:**

```rust
pub struct TerminalBuffers {
    /// Primary screen buffer
    main: Grid,

    /// Alternate screen buffer (same size, no scrollback)
    alt: Grid,

    /// Currently active buffer
    active: BufferType,

    /// Saved cursor position (for alt screen switch)
    saved_cursor: Option<Cursor>,
}

pub enum BufferType {
    Main,
    Alternate,
}

impl TerminalBuffers {
    pub fn switch_to_alt(&mut self) {
        self.saved_cursor = Some(self.main.cursor.clone());
        self.active = BufferType::Alternate;
        self.alt.clear();
    }

    pub fn switch_to_main(&mut self) {
        self.active = BufferType::Main;
        if let Some(cursor) = self.saved_cursor.take() {
            self.main.cursor = cursor;
        }
    }

    pub fn active_grid(&self) -> &Grid { ... }
    pub fn active_grid_mut(&mut self) -> &mut Grid { ... }
}
```

**Notes:**
- The alternate screen buffer **has no scrollback** - lines scrolled off the top are lost
- Mouse scroll in the alt screen may send arrow keys (mode 1007)
- Resize affects both buffers

---

## Tab Stops

### Default Tab Stops

When the terminal initializes or after a hard reset, tab stops are set every 8 columns: columns 8, 16, 24, 32, ...

### Tab Stop Operations

**Horizontal Tab (HT):** `0x09`
- Move cursor right to the next tab stop
- If no more tab stops - move to the last column
- Tab does not print spaces (cells in between are not modified)

**Backward Tab (CBT):** `CSI Pn Z`
- Move cursor left to the previous tab stop `Pn` times

**Set Tab Stop (HTS):** `ESC H`
- Set tab stop at the current cursor column

**Clear Tab Stop (TBC):** `CSI Ps g`

| Ps | Behavior |
|---|---|
| 0 | Clear tab stop at cursor column |
| 3 | Clear all tab stops |

**Implementation:**

```rust
pub struct TabStops {
    /// Bitset - bit i = 1 if column i is a tab stop
    stops: Vec<bool>,
}

impl TabStops {
    pub fn new(cols: usize) -> Self {
        let mut stops = vec![false; cols];
        for i in (8..cols).step_by(8) {
            stops[i] = true;
        }
        Self { stops }
    }

    pub fn next_stop(&self, col: usize) -> usize { ... }
    pub fn prev_stop(&self, col: usize) -> usize { ... }
    pub fn set(&mut self, col: usize) { ... }
    pub fn clear(&mut self, col: usize) { ... }
    pub fn clear_all(&mut self) { ... }
    pub fn resize(&mut self, new_cols: usize) { ... }
}
```

---

## Character Sets

### G0/G1 Designations

VT100 supports two character set slots: G0 and G1.

**Designate character set:**
- `ESC ( B` - G0 = US ASCII (default)
- `ESC ( 0` - G0 = DEC Special Graphics
- `ESC ( A` - G0 = UK (replaces `#` with `£`)
- `ESC ) B` - G1 = US ASCII
- `ESC ) 0` - G1 = DEC Special Graphics

**Invoke character set:**
- `SI` (Shift In, `0x0F`) - activate G0
- `SO` (Shift Out, `0x0E`) - activate G1
- `ESC N` - Single Shift 2 (G2 for next character only)
- `ESC O` - Single Shift 3 (G3 for next character only)

### DEC Special Graphics Character Set

When Special Graphics is active, some printable ASCII codes display line-drawing characters:

| ASCII | Hex | Special Graphics Character |
|---|---|---|
| `j` | 0x6A | ┘ (bottom-right corner) |
| `k` | 0x6B | ┐ (top-right corner) |
| `l` | 0x6C | ┌ (top-left corner) |
| `m` | 0x6D | └ (bottom-left corner) |
| `n` | 0x6E | ┼ (crossing lines) |
| `q` | 0x71 | ─ (horizontal line) |
| `t` | 0x74 | ├ (left tee) |
| `u` | 0x75 | ┤ (right tee) |
| `v` | 0x76 | ┴ (bottom tee) |
| `w` | 0x77 | ┬ (top tee) |
| `x` | 0x78 | │ (vertical line) |
| `a` | 0x61 | ▒ (checker board) |
| `f` | 0x66 | ° (degree symbol) |
| `g` | 0x67 | ± (plus/minus) |
| `o` | 0x6F | ⎺ (scan line 1) |
| `p` | 0x70 | ⎻ (scan line 3) |
| `r` | 0x72 | ⎼ (scan line 7) |
| `s` | 0x73 | ⎽ (scan line 9) |
| `~` | 0x7E | · (bullet/middle dot) |
| `y` | 0x79 | ≤ (less than or equal) |
| `z` | 0x7A | ≥ (greater than or equal) |
| `{` | 0x7B | π (pi) |
| `\|` | 0x7C | ≠ (not equal) |
| `}` | 0x7D | £ (pound sterling) |

**Implementation:**

```rust
pub enum CharSet {
    Ascii,
    DecSpecialGraphics,
    UkAscii,
}

impl CharSet {
    pub fn map(&self, c: char) -> char {
        match self {
            CharSet::Ascii => c,
            CharSet::DecSpecialGraphics => match c {
                'j' => '┘',
                'k' => '┐',
                'l' => '┌',
                'm' => '└',
                'n' => '┼',
                'q' => '─',
                't' => '├',
                'u' => '┤',
                'v' => '┴',
                'w' => '┬',
                'x' => '│',
                // ... etc
                _ => c,
            },
            CharSet::UkAscii => match c {
                '#' => '£',
                _ => c,
            },
        }
    }
}

pub struct CharSetState {
    pub g0: CharSet,
    pub g1: CharSet,
    pub active: CharSetSlot, // G0 or G1
    pub single_shift: Option<CharSet>,
}
```

---

## Reset Operations

### Soft Reset (DECSTR) - `CSI ! p`

Soft reset returns the terminal to a sane state without clearing the screen:

| Attribute | Reset value |
|---|---|
| Cursor visibility (DECTCEM) | Visible |
| Origin mode (DECOM) | OFF |
| Auto-wrap (DECAWM) | ON |
| Insert mode (IRM) | OFF - replace mode |
| Cursor keys (DECCKM) | Normal (arrow keys send CSI sequences) |
| Keypad mode (DECNKM) | Numeric |
| Reverse video (DECSCNM) | OFF |
| Scroll region | Full screen |
| SGR attributes | All off, default colors |
| Character sets | G0=ASCII, G1=ASCII, GL=G0 |
| Saved cursor | Cleared |
| Tab stops | NOT reset (preserved) |
| Screen content | NOT cleared (preserved) |

### Hard Reset (RIS) - `ESC c`

Full reset - equivalent to power cycle:

| Attribute | Reset value |
|---|---|
| All soft reset items | Reset |
| Tab stops | Default (every 8 columns) |
| Screen content | Cleared |
| Scrollback buffer | Cleared |
| Alternate screen | Cleared, switch to main |
| Mouse mode | OFF |
| Bracketed paste | OFF |
| Focus reporting | OFF |
| Window title | Reset to default |

---

## Conformance Levels

### Level 1: Basic Terminal (MVP)

Sufficient to run an interactive shell and basic command-line tools:

- [x] Print ASCII characters
- [x] C0 controls: BEL, BS, HT, LF, CR
- [x] Basic cursor: CUU, CUD, CUF, CUB, CUP
- [x] Erase: ED (0,1,2), EL (0,1,2)
- [x] SGR: bold, underline, inverse, basic 8 colors
- [x] Auto-wrap (DECAWM)
- [x] Cursor visibility (DECTCEM)
- [x] Scroll region (DECSTBM)
- [x] LNM (linefeed/newline mode)

### Level 2: Full Terminal

Sufficient to run TUI applications (vim, tmux, htop):

- [x] All of Level 1
- [x] SGR: 256 colors, true color, italic, strikethrough, dim
- [x] Alternate screen buffer (mode 1049)
- [x] Insert/delete lines and characters (IL, DL, ICH, DCH)
- [x] Scroll up/down (SU, SD)
- [x] DEC Special Graphics character set
- [x] Save/restore cursor (DECSC/DECRC)
- [x] Tab stops (HTS, TBC)
- [x] Origin mode (DECOM)
- [x] Insert mode (IRM)
- [x] Soft reset (DECSTR)
- [x] Hard reset (RIS)
- [x] UTF-8 full support
- [x] Wide characters

### Level 3: Extended Terminal

Modern terminal features and ecosystem compatibility:

- [x] All of Level 2
- [x] Mouse tracking (modes 1000, 1002, 1003, 1006)
- [x] Bracketed paste (mode 2004)
- [x] Focus reporting (mode 1004)
- [x] OSC sequences: window title (0, 2), hyperlinks (8)
- [x] Synchronized output (mode 2026)
- [x] Cursor shape (DECSCUSR)
- [x] OSC color queries/set (4, 10, 11, 12)
- [x] Device attributes (DA1, DA2)
- [x] DECRQM (request mode)
- [x] Combining characters, grapheme clusters

### Level 4: Advanced (Future)

Not needed for MVP; to be considered when the ecosystem demands it:

- [ ] Sixel graphics (DCS)
- [ ] Kitty keyboard protocol
- [ ] Kitty graphics protocol
- [ ] Left/right margins (DECLRMM)
- [ ] Rectangular area operations (DECSERA, DECCRA)
- [ ] Bi-directional text support
- [ ] XTGETTCAP/XTSETTCAP

---

## Scrollback Buffer

### Design

```rust
pub struct ScrollbackBuffer {
    /// Ring buffer of lines that scrolled off top of main screen
    lines: VecDeque<Row>,

    /// Maximum number of lines to keep
    capacity: usize,
}

pub struct Row {
    /// Cells in this line
    cells: Vec<Cell>,

    /// Whether this line is a continuation (wrapped) from the previous line
    is_continuation: bool,

    /// Timestamp when this line scrolled off screen (optional)
    timestamp: Option<Instant>,
}
```

### Capacity

- Default: 10,000 lines
- Configurable: 0 (disabled) - 100,000 lines
- Each line is estimated at ~200-500 bytes (depending on content) - 10K lines ~ 2-5 MB

### Behavior

- When content scrolls off the top of the main screen (or scroll region top if the scroll region == full screen) - add to scrollback
- When capacity is exceeded - drop oldest lines (FIFO)
- Alternate screen buffer does **not** add to scrollback
- User scrolls up - read from scrollback buffer
- New output while user is scrolled up - do not auto-scroll (configurable)

---

## Resize Handling

When the terminal resizes (user drags window):

1. Receive new dimensions (cols, rows)
2. Resize PTY (`TIOCSWINSZ` / `ResizePseudoConsole`)
3. Resize both main and alternate grid:
   - **Columns increase:** extend each row with empty cells
   - **Columns decrease:** truncate rows (content is lost), rewrap if possible
   - **Rows increase:** add empty rows at bottom, pull lines from scrollback if available
   - **Rows decrease:** push excess bottom rows into scrollback
4. Cursor position: clamp to new bounds
5. Scroll region: reset to full screen
6. Tab stops: extend or truncate

**Reflow/Rewrap (optional enhancement):**
- Lines with the `is_continuation` flag can be unwrapped and rewrapped to the new column width
- Improves UX when resizing but is significantly more complex
- Implement at Level 3+ if time permits

---

## Performance Considerations

### Grid Operations

- Use a flat `Vec<Cell>` instead of `Vec<Vec<Cell>>` for cache locality
- Index: `grid[row * cols + col]`
- Dirty tracking per-row (bitmap) to minimize render updates
- Batch scroll operations (scroll n lines = 1 memmove, not n times)

### Memory

- `Cell` size target: <= 24 bytes (CompactString + attrs)
- Grid 200 cols x 50 rows = 10,000 cells = ~240 KB
- Scrollback 10K lines x 200 cols = 2M cells = ~48 MB (worst case; use sparse storage for lines that are mostly spaces)

### Rendering

- Dirty flag per-cell or per-row - only send changed cells to frontend
- Debounce render events (max 60fps)
- Synchronized output mode (2026) - batch updates between BSU/ESU markers

---

## References

- [VT100 User Guide](https://vt100.net/docs/vt100-ug/) - DEC VT100 documentation
- [VT510 Programmer Manual](https://vt100.net/docs/vt510-rm/) - Comprehensive DEC terminal reference
- [xterm control sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) - xterm docs
- [ECMA-48](https://www.ecma-international.org/publications-and-standards/standards/ecma-48/) - Control functions standard
- [Alacritty](https://github.com/alacritty/alacritty) - Rust terminal reference implementation
- [WezTerm](https://github.com/wez/wezterm) - Rust terminal reference
