# ANSI Parser

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

The ANSI parser is responsible for converting the raw byte stream from PTY output into structured actions that the terminal emulator can execute. The parser is designed following the state machine model based on [Paul Williams' VT parser](https://vt100.net/emu/dec_ansi_parser) - the de facto standard for terminal emulator implementations.

The module is located at `src-tauri/src/parser/` and is designed to be stateful - maintaining state between calls to `advance()` to handle partial reads from the PTY.

---

## Parser State Machine

### States

The parser has 14 states. Each input byte causes a transition between states and/or triggers actions:

```
┌────────────────────────────────────────────────────────────┐
│                                                            │
│  ┌──────────┐    ESC     ┌──────────┐    [      ┌───────┐ │
│  │  GROUND  │───────────►│  ESCAPE  │──────────►│ CSI   │ │
│  │          │◄───────────│          │           │ ENTRY │ │
│  └──────────┘  execute   └──────────┘           └───────┘ │
│       │  ▲                    │                     │     │
│  print│  │                    │ ] P _ ^ ~           │     │
│       ▼  │                    ▼                     ▼     │
│    (output)             ┌──────────┐          ┌───────┐   │
│                         │  OSC     │          │ CSI   │   │
│                         │  STRING  │          │ PARAM │   │
│                         └──────────┘          └───────┘   │
│                                                     │     │
│                                                     ▼     │
│                                               ┌───────┐   │
│                                               │ CSI   │   │
│                                               │INTERMD│   │
│                                               └───────┘   │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

| State | Description |
|---|---|
| `Ground` | Default state. Printable chars - `Print` action. C0 controls - `Execute` action |
| `Escape` | After receiving ESC (0x1B). Waiting for the next byte to determine the sequence type |
| `EscapeIntermediate` | Received intermediate bytes (0x20-0x2F) after ESC |
| `CsiEntry` | Just received CSI (ESC [). Ready to read parameters |
| `CsiParam` | Reading CSI numeric parameters (0x30-0x39, 0x3B) |
| `CsiIntermediate` | Received intermediate bytes within a CSI sequence |
| `CsiIgnore` | Invalid CSI sequence - consume until final byte then discard |
| `OscString` | Reading OSC string content (after ESC ]) |
| `DcsEntry` | Just received DCS (ESC P). Ready to read parameters |
| `DcsParam` | Reading DCS parameters |
| `DcsIntermediate` | Received intermediate bytes within a DCS sequence |
| `DcsPassthrough` | Receiving DCS data payload |
| `DcsIgnore` | Invalid DCS sequence - consume until ST |
| `SosPmApc` | SOS/PM/APC strings - consume and ignore until ST |

### State Transitions

```rust
pub enum State {
    Ground,
    Escape,
    EscapeIntermediate,
    CsiEntry,
    CsiParam,
    CsiIntermediate,
    CsiIgnore,
    OscString,
    DcsEntry,
    DcsParam,
    DcsIntermediate,
    DcsPassthrough,
    DcsIgnore,
    SosPmApc,
}
```

### Byte Classifications

Each byte is classified to determine the transition:

| Range | Classification | Meaning |
|---|---|---|
| 0x00-0x17 | C0 control (excl. 0x18, 0x1A) | Control characters |
| 0x18 | CAN | Cancel current sequence, - Ground |
| 0x19 | (unused) | - |
| 0x1A | SUB | Cancel current sequence, - Ground |
| 0x1B | ESC | Start escape sequence |
| 0x1C-0x1F | C0 control | More control characters |
| 0x20-0x2F | Intermediate | Space through `/` |
| 0x30-0x39 | Parameter digit | `0` through `9` |
| 0x3A | Colon | Sub-parameter separator (SGR) |
| 0x3B | Semicolon | Parameter separator |
| 0x3C-0x3F | Parameter prefix | `< = > ?` (private marker) |
| 0x40-0x7E | Final byte | Determines the specific command |
| 0x7F | DEL | Ignored in most states |
| 0x80-0x8F | C1 control | 8-bit control codes (optional) |
| 0x90 | DCS | Device Control String |
| 0x9B | CSI | Control Sequence Introducer |
| 0x9C | ST | String Terminator |
| 0x9D | OSC | Operating System Command |

### Transition Table (Key Transitions)

**From Ground:**

| Input | Action | Next State |
|---|---|---|
| 0x00-0x1A (excl 0x18, 0x1B) | Execute | Ground |
| 0x18 | Execute (CAN) | Ground |
| 0x1B | - | Escape |
| 0x1C-0x1F | Execute | Ground |
| 0x20-0x7E | Print | Ground |
| 0x7F | Ignore | Ground |
| 0x80-0x8F | Execute (C1) | Ground |
| 0x90 | - | DcsEntry |
| 0x9B | - | CsiEntry |
| 0x9D | - | OscString |
| 0x9E, 0x9F | - | SosPmApc |

**From Escape:**

| Input | Action | Next State |
|---|---|---|
| 0x00-0x17, 0x19, 0x1C-0x1F | Execute | Escape |
| 0x18, 0x1A | Execute | Ground |
| 0x1B | - | Escape |
| 0x20-0x2F | Collect | EscapeIntermediate |
| 0x30-0x4F, 0x51-0x57, 0x59, 0x5A, 0x5C, 0x60-0x7E | EscDispatch | Ground |
| 0x50 (`P`) | - | DcsEntry |
| 0x58 (`X`) | - | SosPmApc |
| 0x5B (`[`) | - | CsiEntry |
| 0x5D (`]`) | - | OscString |
| 0x5E (`^`), 0x5F (`_`) | - | SosPmApc |
| 0x7F | Ignore | Escape |

**From CsiEntry:**

| Input | Action | Next State |
|---|---|---|
| 0x00-0x17, 0x19, 0x1C-0x1F | Execute | CsiEntry |
| 0x18, 0x1A | Execute | Ground |
| 0x20-0x2F | Collect | CsiIntermediate |
| 0x30-0x39, 0x3B | Param | CsiParam |
| 0x3A | - | CsiIgnore |
| 0x3C-0x3F | Collect (private) | CsiParam |
| 0x40-0x7E | CsiDispatch | Ground |
| 0x7F | Ignore | CsiEntry |

**From CsiParam:**

| Input | Action | Next State |
|---|---|---|
| 0x30-0x39, 0x3A, 0x3B | Param | CsiParam |
| 0x20-0x2F | Collect | CsiIntermediate |
| 0x40-0x7E | CsiDispatch | Ground |
| 0x3C-0x3F | - | CsiIgnore |

### Implementation

```rust
pub struct AnsiParser {
    state: State,
    params: ParamBuffer,
    intermediates: IntermediateBuffer,
    osc_buffer: Vec<u8>,
    utf8_decoder: Utf8Decoder,
}

/// Fixed-size parameter buffer (no allocation during parsing)
pub struct ParamBuffer {
    params: [u16; 32],   // Maximum 32 parameters
    len: usize,
    current: u16,        // Parameter being built
    has_current: bool,
    sub_params: bool,    // Colon-separated sub-parameters active
}

/// Fixed-size intermediate buffer
pub struct IntermediateBuffer {
    bytes: [u8; 4],     // Maximum 4 intermediate bytes
    len: usize,
}

impl AnsiParser {
    /// Feed bytes to parser, returns actions
    pub fn advance(&mut self, input: &[u8]) -> Vec<Action> {
        let mut actions = Vec::new();

        for &byte in input {
            // Anywhere transitions (CAN, SUB, ESC, C1 controls)
            if self.handle_anywhere(byte, &mut actions) {
                continue;
            }

            // State-specific transitions
            match self.state {
                State::Ground => self.ground(byte, &mut actions),
                State::Escape => self.escape(byte, &mut actions),
                State::EscapeIntermediate => self.escape_intermediate(byte, &mut actions),
                State::CsiEntry => self.csi_entry(byte, &mut actions),
                State::CsiParam => self.csi_param(byte, &mut actions),
                State::CsiIntermediate => self.csi_intermediate(byte, &mut actions),
                State::CsiIgnore => self.csi_ignore(byte, &mut actions),
                State::OscString => self.osc_string(byte, &mut actions),
                State::DcsEntry => self.dcs_entry(byte, &mut actions),
                State::DcsParam => self.dcs_param(byte, &mut actions),
                State::DcsIntermediate => self.dcs_intermediate(byte, &mut actions),
                State::DcsPassthrough => self.dcs_passthrough(byte, &mut actions),
                State::DcsIgnore => self.dcs_ignore(byte, &mut actions),
                State::SosPmApc => self.sos_pm_apc(byte, &mut actions),
            }
        }

        actions
    }

    /// Handle bytes that cause transitions from ANY state
    fn handle_anywhere(&mut self, byte: u8, actions: &mut Vec<Action>) -> bool {
        match byte {
            0x18 | 0x1A => {
                // CAN / SUB - cancel current sequence
                self.perform_action(Action::Execute(byte), actions);
                self.transition(State::Ground);
                true
            }
            0x1B => {
                // ESC - start new escape sequence
                self.transition(State::Escape);
                true
            }
            // 8-bit C1 controls (optional - most modern terminals use 7-bit)
            0x90 => { self.transition(State::DcsEntry); true }
            0x9B => { self.transition(State::CsiEntry); true }
            0x9C => { self.transition(State::Ground); true } // ST
            0x9D => { self.transition(State::OscString); true }
            0x98 | 0x9E | 0x9F => { self.transition(State::SosPmApc); true }
            _ => false, // Not an anywhere transition
        }
    }
}
```

---

## C0 Control Codes

C0 controls (0x00-0x1F) cause an `Execute` action when received in the `Ground` state (and most other states except string states).

| Hex | Name | Acronym | Behavior in Wit |
|---|---|---|---|
| 0x00 | Null | NUL | Ignored |
| 0x01 | Start of Heading | SOH | Ignored |
| 0x02 | Start of Text | STX | Ignored |
| 0x03 | End of Text | ETX | Ignored (Ctrl+C handled by PTY/shell) |
| 0x04 | End of Transmission | EOT | Ignored (Ctrl+D handled by PTY/shell) |
| 0x05 | Enquiry | ENQ | Return answerback string (configurable, default empty) |
| 0x06 | Acknowledge | ACK | Ignored |
| 0x07 | Bell | BEL | Trigger visual/audible bell notification |
| 0x08 | Backspace | BS | Move cursor left 1 column (stop at column 0) |
| 0x09 | Horizontal Tab | HT | Move cursor right to next tab stop |
| 0x0A | Line Feed | LF | Move cursor down 1 row. Scroll if at bottom of scroll region. If LNM mode: also do CR |
| 0x0B | Vertical Tab | VT | Same as LF |
| 0x0C | Form Feed | FF | Same as LF |
| 0x0D | Carriage Return | CR | Move cursor to column 0 |
| 0x0E | Shift Out | SO | Activate G1 character set |
| 0x0F | Shift In | SI | Activate G0 character set |
| 0x10 | Data Link Escape | DLE | Ignored |
| 0x11 | Device Control 1 | DC1/XON | Flow control - resume (ignored, handled by PTY driver) |
| 0x12 | Device Control 2 | DC2 | Ignored |
| 0x13 | Device Control 3 | DC3/XOFF | Flow control - pause (ignored, handled by PTY driver) |
| 0x14 | Device Control 4 | DC4 | Ignored |
| 0x15 | Negative Acknowledge | NAK | Ignored |
| 0x16 | Synchronous Idle | SYN | Ignored |
| 0x17 | End of Trans. Block | ETB | Ignored |
| 0x18 | Cancel | CAN | Cancel current escape sequence - Ground |
| 0x19 | End of Medium | EM | Ignored |
| 0x1A | Substitute | SUB | Cancel current escape sequence - Ground, print `?` |
| 0x1B | Escape | ESC | Start escape sequence - Escape state |
| 0x1C | File Separator | FS | Ignored |
| 0x1D | Group Separator | GS | Ignored |
| 0x1E | Record Separator | RS | Ignored |
| 0x1F | Unit Separator | US | Ignored |

### Implementation

```rust
fn execute_c0(&mut self, byte: u8) {
    match byte {
        0x07 => self.bell(),
        0x08 => self.backspace(),
        0x09 => self.horizontal_tab(),
        0x0A | 0x0B | 0x0C => self.linefeed(),
        0x0D => self.carriage_return(),
        0x0E => self.shift_out(),
        0x0F => self.shift_in(),
        _ => {} // Ignore all others
    }
}
```

---

## C1 Control Codes

C1 controls (0x80-0x9F) are 8-bit equivalents of ESC + byte. Wit supports both forms. In practice, most modern terminals send 7-bit (ESC + byte) because UTF-8 uses the 0x80+ range for multi-byte characters.

| 8-bit | 7-bit equiv | Name | Behavior |
|---|---|---|---|
| 0x84 | ESC D | IND | Index - move cursor down, scroll if needed |
| 0x85 | ESC E | NEL | Next Line - CR + LF |
| 0x88 | ESC H | HTS | Horizontal Tab Set at cursor column |
| 0x8D | ESC M | RI | Reverse Index - move cursor up, scroll down if needed |
| 0x8E | ESC N | SS2 | Single Shift 2 - G2 for next char only |
| 0x8F | ESC O | SS3 | Single Shift 3 - G3 for next char only |
| 0x90 | ESC P | DCS | Device Control String - start |
| 0x96 | ESC V | SPA | Start of Protected Area (ignored) |
| 0x97 | ESC W | EPA | End of Protected Area (ignored) |
| 0x9B | ESC [ | CSI | Control Sequence Introducer |
| 0x9C | ESC \ | ST | String Terminator |
| 0x9D | ESC ] | OSC | Operating System Command |
| 0x9E | ESC ^ | PM | Privacy Message (ignored) |
| 0x9F | ESC _ | APC | Application Program Command (ignored) |

---

## CSI Sequences

CSI (Control Sequence Introducer) sequences are the primary mechanism for terminal control. Format: `CSI [private] params [intermediate] final`

- CSI = `ESC [` (7-bit) or `0x9B` (8-bit)
- Private marker: `?`, `>`, `<`, `=` (optional, before params)
- Parameters: numeric, separated by `;` (semicolons) or `:` (colons for sub-params)
- Intermediate: bytes 0x20-0x2F (optional)
- Final byte: 0x40-0x7E (determines the command)

### Cursor Movement

| Sequence | Name | Acronym | Description |
|---|---|---|---|
| `CSI Pn A` | Cursor Up | CUU | Move cursor up `Pn` rows (default 1). Stop at top margin |
| `CSI Pn B` | Cursor Down | CUD | Move cursor down `Pn` rows (default 1). Stop at bottom margin |
| `CSI Pn C` | Cursor Forward | CUF | Move cursor right `Pn` columns (default 1). Stop at right margin |
| `CSI Pn D` | Cursor Backward | CUB | Move cursor left `Pn` columns (default 1). Stop at column 0 |
| `CSI Pn E` | Cursor Next Line | CNL | Move cursor down `Pn` rows + column 0 |
| `CSI Pn F` | Cursor Previous Line | CPL | Move cursor up `Pn` rows + column 0 |
| `CSI Pn G` | Cursor Horizontal Absolute | CHA | Move cursor to column `Pn` (1-indexed) |
| `CSI Pr ; Pc H` | Cursor Position | CUP | Move cursor to row `Pr`, column `Pc` (1-indexed, default 1;1) |
| `CSI Pr ; Pc f` | Horizontal and Vertical Position | HVP | Same as CUP |
| `CSI Pn d` | Vertical Position Absolute | VPA | Move cursor to row `Pn` (1-indexed) |
| `CSI s` | Save Cursor Position | SCP | Save cursor position (ANSI.SYS compat) |
| `CSI u` | Restore Cursor Position | RCP | Restore cursor position |

**DEC Save/Restore (preferred):**
- `ESC 7` (DECSC) - Save cursor position, attributes, character set, origin mode, wrap flag
- `ESC 8` (DECRC) - Restore all saved state

**Cursor Shape:**
- `CSI Ps SP q` (DECSCUSR) - Set cursor shape

| Ps | Shape |
|---|---|
| 0, 1 | Blinking block |
| 2 | Steady block |
| 3 | Blinking underline |
| 4 | Steady underline |
| 5 | Blinking bar |
| 6 | Steady bar |

### Erase

| Sequence | Name | Description |
|---|---|---|
| `CSI Ps J` | Erase in Display (ED) | 0=below, 1=above, 2=all, 3=scrollback |
| `CSI Ps K` | Erase in Line (EL) | 0=right, 1=left, 2=all |
| `CSI Pn X` | Erase Characters (ECH) | Erase `Pn` chars from cursor |
| `CSI Pn L` | Insert Lines (IL) | Insert `Pn` blank lines at cursor row |
| `CSI Pn M` | Delete Lines (DL) | Delete `Pn` lines at cursor row |
| `CSI Pn @` | Insert Characters (ICH) | Insert `Pn` blank chars at cursor |
| `CSI Pn P` | Delete Characters (DCH) | Delete `Pn` chars at cursor |

### Scroll

| Sequence | Name | Description |
|---|---|---|
| `CSI Pn S` | Scroll Up (SU) | Scroll content up `Pn` lines |
| `CSI Pn T` | Scroll Down (SD) | Scroll content down `Pn` lines (Pn <= 5 params only; 6+ params = mouse tracking) |
| `CSI Pt ; Pb r` | Set Scroll Region (DECSTBM) | Set top and bottom margins |
| `ESC D` | Index (IND) | Move down 1 line, scroll if needed |
| `ESC M` | Reverse Index (RI) | Move up 1 line, reverse scroll if needed |

### SGR (Select Graphic Rendition)

`CSI Pm m` - Set display attributes. Multiple attributes separated by `;`.

#### Basic Attributes

| Code | Set | Reset | Attribute |
|---|---|---|---|
| 0 | - | All | Reset all attributes to default |
| 1 | Bold | 22 | Bold / increased intensity |
| 2 | Dim | 22 | Dim / faint / decreased intensity |
| 3 | Italic | 23 | Italic |
| 4 | Underline | 24 | Underline |
| 5 | Slow blink | 25 | Blink (Wit renders but can be disabled) |
| 7 | Inverse | 27 | Swap foreground and background |
| 8 | Hidden | 28 | Invisible / concealed text |
| 9 | Strikethrough | 29 | Crossed-out / strikethrough |
| 21 | Double underline | 24 | Double underline (or bold off on some terminals) |
| 53 | Overline | 55 | Overline |

#### Foreground Colors (Text Color)

| Code | Color |
|---|---|
| 30 | Black |
| 31 | Red |
| 32 | Green |
| 33 | Yellow |
| 34 | Blue |
| 35 | Magenta |
| 36 | Cyan |
| 37 | White |
| 39 | Default foreground |
| 90 | Bright Black (Gray) |
| 91 | Bright Red |
| 92 | Bright Green |
| 93 | Bright Yellow |
| 94 | Bright Blue |
| 95 | Bright Magenta |
| 96 | Bright Cyan |
| 97 | Bright White |

#### Background Colors

| Code | Color |
|---|---|
| 40-47 | Standard backgrounds (same as 30-37) |
| 49 | Default background |
| 100-107 | Bright backgrounds (same as 90-97) |

#### 256-Color Mode

`CSI 38 ; 5 ; Pn m` - Set foreground to 256-color palette index `Pn`
`CSI 48 ; 5 ; Pn m` - Set background to 256-color palette index `Pn`

Palette layout:

| Range | Colors |
|---|---|
| 0-7 | Standard colors (same as SGR 30-37) |
| 8-15 | Bright colors (same as SGR 90-97) |
| 16-231 | 6x6x6 color cube: index = 16 + 36*r + 6*g + b (r,g,b in 0-5) |
| 232-255 | Grayscale ramp: 24 shades from dark to light |

#### True Color (24-bit)

`CSI 38 ; 2 ; Pr ; Pg ; Pb m` - Set foreground RGB
`CSI 48 ; 2 ; Pr ; Pg ; Pb m` - Set background RGB

**Colon-separated variant (spec-compliant):**
`CSI 38 : 2 : : Pr : Pg : Pb m` - Foreground RGB (with colorspace ID placeholder)
`CSI 48 : 2 : : Pr : Pg : Pb m` - Background RGB

Wit supports both semicolon (widely used) and colon (spec-correct) separators.

#### Underline Color (xterm extension)

`CSI 58 ; 2 ; Pr ; Pg ; Pb m` - Set underline color RGB
`CSI 58 ; 5 ; Pn m` - Set underline color 256-palette
`CSI 59 m` - Reset underline color to default

### Mode Set/Reset

**Standard modes:**
- `CSI Pm h` (SM) - Set Mode
- `CSI Pm l` (RM) - Reset Mode

**DEC private modes:**
- `CSI ? Pm h` (DECSET) - Set DEC Private Mode
- `CSI ? Pm l` (DECRST) - Reset DEC Private Mode

See the full list of modes at [terminal-emulator.md](./terminal-emulator.md#terminal-modes).

**Request mode (DECRQM):**
- `CSI Pm $ p` - Request ANSI mode status
- `CSI ? Pm $ p` - Request DEC private mode status

Response: `CSI Pm ; Ps $ y` where Ps = 1 (set), 2 (reset), 3 (permanently set), 4 (permanently reset), 0 (unknown)

### Device Status Reports

| Sequence | Name | Response |
|---|---|---|
| `CSI 5 n` | Device Status Report | `CSI 0 n` (OK) |
| `CSI 6 n` | Cursor Position Report | `CSI Pr ; Pc R` (1-indexed) |
| `CSI ? 6 n` | Extended CPR | `CSI ? Pr ; Pc R` |
| `CSI c` | Primary Device Attributes (DA1) | `CSI ? 62 ; 22 c` (VT220, ANSI color) |
| `CSI > c` | Secondary Device Attributes (DA2) | `CSI > 0 ; version ; 0 c` |
| `CSI = c` | Tertiary Device Attributes (DA3) | `CSI P ! \| hex ST` |

### Window Manipulation (XTWINOPS)

| Sequence | Description |
|---|---|
| `CSI 8 ; rows ; cols t` | Resize window to rows x cols |
| `CSI 14 t` | Report window size in pixels |
| `CSI 14 ; 2 t` | Report text area size in pixels |
| `CSI 18 t` | Report text area size in chars - `CSI 8 ; rows ; cols t` |
| `CSI 22 ; 0 t` | Save window title to stack |
| `CSI 23 ; 0 t` | Restore window title from stack |

---

## OSC Sequences

OSC (Operating System Command) format: `ESC ] Ps ; Pt BEL` or `ESC ] Ps ; Pt ST`

- `Ps`: numeric parameter
- `Pt`: string data (text content)
- Terminated by BEL (0x07) or ST (ESC \)

### Supported OSC Sequences

| Ps | Description | Example |
|---|---|---|
| 0 | Set icon name + window title | `ESC ] 0 ; My Title BEL` |
| 1 | Set icon name | `ESC ] 1 ; Icon BEL` |
| 2 | Set window title | `ESC ] 2 ; Window Title BEL` |
| 4 | Change/query color palette entry | `ESC ] 4 ; index ; spec BEL` |
| 7 | Current working directory (iTerm2/VTE) | `ESC ] 7 ; file://host/path BEL` |
| 8 | Hyperlinks | `ESC ] 8 ; params ; uri ST` ... `ESC ] 8 ; ; ST` |
| 10 | Query/set foreground color | `ESC ] 10 ; ? BEL` (query) |
| 11 | Query/set background color | `ESC ] 11 ; ? BEL` (query) |
| 12 | Query/set cursor color | `ESC ] 12 ; ? BEL` (query) |
| 52 | Clipboard access | `ESC ] 52 ; c ; base64data BEL` |
| 104 | Reset color palette entry | `ESC ] 104 BEL` (reset all) |
| 110 | Reset foreground color | `ESC ] 110 BEL` |
| 111 | Reset background color | `ESC ] 111 BEL` |
| 112 | Reset cursor color | `ESC ] 112 BEL` |
| 133 | Shell integration (FinalTerm) | `ESC ] 133 ; A BEL` (prompt start) |

### OSC 8 - Hyperlinks

Format: `OSC 8 ; params ; uri ST` (start link) ... `OSC 8 ; ; ST` (end link)

```
ESC ] 8 ; ; https://example.com ST Click here ESC ] 8 ; ; ST
```

Params (optional, semicolon-separated key=value):
- `id=value` - Link ID for grouping multiple link segments

### OSC 52 - Clipboard

```
ESC ] 52 ; Pc ; Pd BEL

Pc = clipboard selection:
  c = clipboard (primary)
  p = primary selection (X11)
  s = select (X11)

Pd = base64-encoded data, or ? to query
```

**Security:** Wit must be careful with clipboard write - by default only allow clipboard read queries; clipboard write needs user confirmation or config opt-in.

### OSC 133 - Shell Integration (FinalTerm/iTerm2)

| Sequence | Meaning |
|---|---|
| `OSC 133 ; A ST` | Prompt start |
| `OSC 133 ; B ST` | Command start (user pressed Enter) |
| `OSC 133 ; C ST` | Command output start |
| `OSC 133 ; D ; exitcode ST` | Command finished with exit code |

Shell integration allows Wit to know:
- Where the prompt is, where the command is, where the output is
- What exit code the last command returned
- The current CWD (combined with OSC 7)

---

## DCS Sequences

DCS (Device Control String) format: `ESC P Ps ; Ps ... final data ST`

### Supported DCS Sequences

| Sequence | Description | Priority |
|---|---|---|
| `DCS $ q Pt ST` | DECRQSS - Request setting | Level 3 |
| `DCS + q Pt ST` | XTGETTCAP - Query terminfo | Level 4 |
| `DCS q ...data... ST` | Sixel graphics | Level 4 (Future) |
| `DCS tmux; ...data... ST` | tmux passthrough | Level 3 |

### DECRQSS - Request Selection or Setting

`DCS $ q Pt ST` - Query current value of setting `Pt`

| Pt | Setting queried |
|---|---|
| `m` | SGR - current text attributes |
| `r` | DECSTBM - scroll region |
| `" p` | DECSCL - conformance level |
| `SP q` | DECSCUSR - cursor shape |

Response: `DCS 1 $ r Pt ST` (valid) or `DCS 0 $ r ST` (invalid)

### Sixel Graphics (Future - Level 4)

Sixel is a protocol for displaying raster images in the terminal. The DCS sequence contains encoded image data. Wit plans to support this in the future but it is not in the MVP scope.

---

## Handling Malformed/Incomplete Sequences

### Malformed Sequences

**Principle:** Never crash. Always recover gracefully.

| Situation | Handling |
|---|---|
| Unknown CSI final byte | Ignore sequence, log warning at debug level |
| Too many parameters (>32) | Ignore excess, process first 32 |
| Parameter overflow (>65535) | Clamp to 65535 |
| Unexpected bytes in escape | CsiIgnore state - consume until final byte |
| Invalid UTF-8 byte | Replace with U+FFFD |
| OSC string too long (>4096 bytes) | Truncate, process what we have |
| Nested ESC during sequence | Cancel current, start new (as per state machine) |

### Incomplete Sequences

PTY read may cut in the middle of an escape sequence:

```
Read 1: b"Hello \x1b[38;2;25"     <- CSI sequence cut mid-stream
Read 2: b"5;128;0mWorld"          <- Remaining part

Read 1: b"Unicode: \xc3"          <- UTF-8 2-byte sequence cut mid-stream
Read 2: b"\xa9 text"              <- Second byte (= "e")
```

**Handling:** The parser maintains state between calls to `advance()`. The state machine naturally handles this - the parser is in the `CsiParam` state when Read 1 ends, and continues in that state when Read 2 begins.

```rust
// Parser state is preserved between calls
let actions1 = parser.advance(b"Hello \x1b[38;2;25");
// actions1 = [Print('H'), Print('e'), ..., Print(' ')]
// Parser state: CsiParam (collecting "38;2;25")

let actions2 = parser.advance(b"5;128;0mWorld");
// actions2 = [CsiDispatch{params:[38,2,255,128,0], final:'m'}, Print('W'), ...]
// Parser state: Ground
```

### Timeout for Stale Sequences

If the parser is stuck in a non-Ground state for too long (e.g., ESC received but no next byte), a timeout could be implemented:

- After 100ms with no new bytes while the parser is in the Escape state - emit ESC as printable
- However, most implementations do not do this because PTY output arrives quickly

Wit recommendation: **Do not implement timeout.** Let the state machine handle it naturally. If stale state causes issues, reconsider later.

---

## Performance Considerations

### High-Throughput Scenarios

The terminal needs to handle fast output from commands such as:
- `cat large_file.txt` - megabytes of text
- `find / -name "*.log"` - thousands of lines
- `cargo build` - verbose build output
- `yes` - infinite output

### Parser Optimizations

**1. Zero-allocation hot path:**

```rust
// ParamBuffer and IntermediateBuffer are fixed-size arrays, no heap allocation
// Only allocate for Actions when dispatch is needed

// Optimization: batch Print actions
// Instead of Print('H'), Print('e'), Print('l'), Print('l'), Print('o')
// -> PrintString("Hello") for consecutive printable characters
pub enum Action {
    Print(char),
    PrintString(String), // Batch optimization
    // ...
}
```

**2. Lookup table for state transitions:**

```rust
// Pre-computed transition table: state x byte -> (action, next_state)
static TRANSITION_TABLE: [[Transition; 256]; 14] = /* computed at compile time */;

struct Transition {
    action: TransitionAction,
    next_state: State,
}

impl AnsiParser {
    fn advance_byte(&mut self, byte: u8) -> Option<Action> {
        let transition = &TRANSITION_TABLE[self.state as usize][byte as usize];
        let action = self.perform(transition.action, byte);
        self.state = transition.next_state;
        action
    }
}
```

**3. SIMD-accelerated ground state scanning:**

```rust
// Scan for escape bytes (0x1B) or C0 controls in bulk
// When in Ground state and processing printable text, use SIMD to find
// the next non-printable byte quickly

fn find_next_special_byte(input: &[u8]) -> usize {
    // Scalar fallback (SIMD version uses _mm256_cmpeq_epi8)
    input.iter().position(|&b| b < 0x20 || b == 0x7F).unwrap_or(input.len())
}
```

**4. Avoid Vec<Action> allocation:**

```rust
// Option A: Callback-based (no allocation)
pub trait ActionHandler {
    fn print(&mut self, c: char);
    fn execute(&mut self, byte: u8);
    fn csi_dispatch(&mut self, params: &[u16], intermediates: &[u8], final_byte: u8);
    fn osc_dispatch(&mut self, params: &[&[u8]]);
    fn esc_dispatch(&mut self, intermediates: &[u8], final_byte: u8);
    fn dcs_hook(&mut self, params: &[u16], intermediates: &[u8], final_byte: u8);
    fn dcs_put(&mut self, byte: u8);
    fn dcs_unhook(&mut self);
}

impl AnsiParser {
    pub fn advance_with_handler(&mut self, input: &[u8], handler: &mut impl ActionHandler) {
        // Directly call handler methods instead of building Vec<Action>
    }
}
```

### Benchmarking Targets

| Metric | Target | Measurement |
|---|---|---|
| Parse throughput | > 500 MB/s | `cat /dev/urandom \| head -c 100M` parsed per second |
| Latency per byte | < 5 ns | Time to process single byte through state machine |
| Memory per parser | < 512 bytes | Parser struct size (no heap allocations in steady state) |
| Action dispatch | < 50 ns | Time to process one CSI dispatch in terminal |

### Benchmarking

```rust
// Using the criterion crate
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn bench_parser_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");

    // Plain text (best case)
    let plain_text = "Hello, World! ".repeat(10000);
    group.throughput(Throughput::Bytes(plain_text.len() as u64));
    group.bench_function("plain_text", |b| {
        b.iter(|| {
            let mut parser = AnsiParser::new();
            parser.advance(plain_text.as_bytes());
        });
    });

    // Heavy escape sequences (worst case)
    let escape_heavy = "\x1b[38;2;255;128;0m█\x1b[0m".repeat(10000);
    group.throughput(Throughput::Bytes(escape_heavy.len() as u64));
    group.bench_function("escape_heavy", |b| {
        b.iter(|| {
            let mut parser = AnsiParser::new();
            parser.advance(escape_heavy.as_bytes());
        });
    });

    // Mixed content (realistic)
    let mixed = include_bytes!("../test_data/vttest_output.bin");
    group.throughput(Throughput::Bytes(mixed.len() as u64));
    group.bench_function("mixed_realistic", |b| {
        b.iter(|| {
            let mut parser = AnsiParser::new();
            parser.advance(mixed);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_parser_throughput);
criterion_main!(benches);
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_ascii() {
        let mut parser = AnsiParser::new();
        let actions = parser.advance(b"Hello");
        assert_eq!(actions, vec![
            Action::Print('H'),
            Action::Print('e'),
            Action::Print('l'),
            Action::Print('l'),
            Action::Print('o'),
        ]);
    }

    #[test]
    fn test_csi_cursor_up() {
        let mut parser = AnsiParser::new();
        let actions = parser.advance(b"\x1b[5A");
        assert_eq!(actions, vec![
            Action::CsiDispatch {
                params: vec![5],
                intermediates: vec![],
                final_byte: b'A',
            }
        ]);
    }

    #[test]
    fn test_sgr_true_color() {
        let mut parser = AnsiParser::new();
        let actions = parser.advance(b"\x1b[38;2;255;128;0m");
        assert_eq!(actions, vec![
            Action::CsiDispatch {
                params: vec![38, 2, 255, 128, 0],
                intermediates: vec![],
                final_byte: b'm',
            }
        ]);
    }

    #[test]
    fn test_incomplete_sequence() {
        let mut parser = AnsiParser::new();

        // First chunk - incomplete CSI
        let actions1 = parser.advance(b"\x1b[38;2;25");
        assert!(actions1.is_empty()); // No complete actions yet

        // Second chunk - completes the sequence
        let actions2 = parser.advance(b"5;128;0mX");
        assert_eq!(actions2.len(), 2); // CSI dispatch + Print('X')
    }

    #[test]
    fn test_osc_title() {
        let mut parser = AnsiParser::new();
        let actions = parser.advance(b"\x1b]0;My Title\x07");
        assert!(matches!(&actions[0], Action::OscDispatch(parts) if parts.len() == 2));
    }

    #[test]
    fn test_malformed_csi_ignored() {
        let mut parser = AnsiParser::new();
        // Invalid: parameter byte after intermediate
        let actions = parser.advance(b"\x1b[ 5A");
        // Should recover - 'A' dispatches from CsiIntermediate
    }
}
```

### Integration Tests

- vttest: run the [vttest](https://invisible-island.net/vttest/) test suite
- Terminal output capture: record output from real tools (vim, tmux, htop) and replay
- Fuzzing: fuzz the parser with arbitrary bytes, assert no panics

---

## References

- [Paul Williams' VT parser](https://vt100.net/emu/dec_ansi_parser) - State machine reference
- [xterm control sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) - Comprehensive CSI/OSC/DCS reference
- [ECMA-48](https://www.ecma-international.org/publications-and-standards/standards/ecma-48/) - Control functions standard
- [vte crate](https://docs.rs/vte) - Rust VT parser library (reference)
- [Alacritty parser](https://github.com/alacritty/vte) - Alacritty's VT parser fork
- [Terminal Guide](https://terminalguide.namepad.de/) - Interactive sequence reference
- [vttest](https://invisible-island.net/vttest/) - VT100/VT220 compatibility test suite
