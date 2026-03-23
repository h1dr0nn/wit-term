# Terminal Emulation Research

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. History of Terminals

### 1.1 From Teletype to Hardware Terminal

**Teletype (TTY) - 1960s-1970s:**
- Electromechanical devices connected to mainframes via serial line
- Communicated using ASCII characters, no concept of "screen"
- Output was printed paper (paper tape), no cursor or positioning
- The term "TTY" is still used today in Unix (`/dev/tty`)

**DEC Hardware Terminals - 1970s-1990s:**

| Terminal | Year | Key Innovation |
|----------|------|----------------|
| **VT52** | 1975 | Escape sequences for cursor control, first screen-based editing |
| **VT100** | 1978 | ANSI escape codes (ANSI X3.64), 80/132 columns, became de facto standard |
| **VT220** | 1983 | 8-bit control codes, additional character sets (DEC Multinational) |
| **VT320** | 1987 | Multiple pages, enhanced attributes |
| **VT420** | 1990 | Rectangular area operations, multiple sessions |
| **VT520** | 1994 | Last of the line, Unicode support, bi-directional text |

> **Why VT100 matters:** Most terminal emulators today still claim "VT100 compatible" because VT100 was the first terminal to implement the ANSI standard. When an application wants to move the cursor, change color, or clear the screen - all use escape sequences originating from VT100.

### 1.2 Software Terminal Emulators

**First generation - 1980s-1990s:**
- **xterm** (1984): X Window System terminal emulator, still the reference implementation
- **rxvt** (1990s): Lightweight alternative to xterm
- **gnome-terminal**, **konsole**: Desktop environment terminals

**Modern generation - 2010s-2020s:**
- **Alacritty** (2017): GPU-accelerated, Rust, minimal
- **Kitty** (2017): GPU-accelerated, C + Python, feature-rich
- **WezTerm** (2018): Rust, GPU rendering, Lua config
- **Warp** (2022): Rust, blocks model, AI integration

---

## 2. Standards and Specifications

### 2.1 Core Standards

**ECMA-48 (5th Edition, 1991):**
- Full name: "Control Functions for Coded Character Sets"
- Defines: C0/C1 control codes, escape sequences, CSI sequences, SGR attributes
- Is a superset of ANSI X3.64
- Link: https://ecma-international.org/publications-and-standards/standards/ecma-48/

**ANSI X3.64 (1979, withdrawn 1997):**
- American National Standard for "Additional Controls for Use with American National Standard Code for Information Interchange"
- Withdrawn because it was merged into ECMA-48/ISO 6429
- The term "ANSI escape codes" is still commonly used even though the standard has been withdrawn

**ISO 6429 (1992):**
- International version, equivalent to ECMA-48
- Same content, different issuing organization

### 2.2 De Facto Standards

Beyond formal standards, much behavior is defined by implementation:
- **xterm extensions**: 256-color, true color, mouse tracking, focus events, bracketed paste
- **DEC private modes**: DECCKM, DECOM, DECAWM, DECTCEM - not part of ECMA-48
- **iTerm2 extensions**: Inline images, shell integration marks
- **Kitty extensions**: Keyboard protocol, graphics protocol

---

## 3. Modern Terminal Emulation

### 3.1 $TERM Variable and terminfo

**$TERM variable:**
- Tells the application what type of terminal is running
- Applications use this value to query the terminfo database
- Common values: `xterm-256color`, `screen-256color`, `tmux-256color`

**terminfo database:**
- Compiled database containing terminal capabilities
- Location: `/usr/share/terminfo/`, `~/.terminfo/`
- Compiled from source text files using the `tic` command
- Queried using `infocmp` or programmatically via `tgetent()`/`tigetstr()`

**What `xterm-256color` means:**
```
xterm       -> base terminal type (xterm compatible)
-256color   -> supports 256 color palette
```

Applications like vim, tmux will query terminfo for `xterm-256color` to learn:
- Number of supported colors (`colors#256`)
- Escape sequence to set foreground (`setaf`)
- Escape sequence to set background (`setab`)
- Cursor movement sequences (`cuf1`, `cub1`, `cuu1`, `cud1`)
- Screen clearing (`clear`, `ed`, `el`)
- etc.

### 3.2 Decisions for Wit

Does Wit need a custom terminfo entry or should it use the existing `xterm-256color`?

**Option A: Use `xterm-256color` (recommended initially)**
- Pros: Compatible immediately, no need to install custom terminfo
- Cons: Limited by capabilities that xterm declares

**Option B: Custom terminfo entry `wit-256color`**
- Pros: Declare exactly the capabilities Wit supports
- Cons: Need to distribute and install terminfo file, fallback if not installed

> **Recommendation:** Start with `xterm-256color`, move to custom entry when Wit has unique capabilities (e.g., custom keyboard protocol).

---

## 4. Key Resources and References

### 4.1 Essential Reading

| Resource | URL | What it covers |
|----------|-----|----------------|
| **vt100.net** | https://vt100.net/ | DEC terminal manuals, original VT100/VT220/VT320/VT520 documentation |
| **invisible-island.net/xterm** | https://invisible-island.net/xterm/ | xterm control sequences (ctlseqs.html), authoritative reference |
| **Paul Williams' VT Parser** | https://vt100.net/emu/dec_ansi_parser | State machine model for ANSI/DEC parsing, very clear diagram |
| **ECMA-48** | https://ecma-international.org/publications-and-standards/standards/ecma-48/ | Official standard document |
| **xterm.js** | https://github.com/xtermjs/xterm.js | TypeScript terminal emulator, good reference implementation |
| **Alacritty (vte crate)** | https://github.com/alacritty/vte | Rust VT parser, potential dependency for Wit |

### 4.2 Paul Williams' VT Parser Model

State machine model consisting of 16 states:

```
Ground -> Escape -> Escape Intermediate -> CSI Entry -> CSI Param -> CSI Intermediate -> CSI Ignore
                                                   -> DCS Entry -> DCS Param -> DCS Intermediate
                                                               -> DCS Passthrough
                -> OSC String
                -> SOS/PM/APC String
```

Key concepts:
- **State transitions** triggered by incoming bytes
- **Actions** executed on transitions: print, execute, hook, put, unhook, osc_start, osc_put, osc_end, csi_dispatch, esc_dispatch, collect, param, clear
- Parser is a **pure state machine**, knows nothing about terminal semantics
- Terminal handler interprets dispatched sequences

> **Important for Wit:** Parser architecture should be separate from terminal state. Wit's parser receives bytes, emits structured events (CSI, OSC, text), and the terminal grid handler applies events to state.

---

## 5. Terminal Capabilities

### 5.1 Must-Have Capabilities for Wit

**Text rendering:**
- Character attributes: bold, italic, underline, strikethrough, blink, inverse, invisible
- Foreground/background colors: 16 ANSI, 256 extended, 24-bit true color
- Cursor: block, underline, bar shapes; blinking/steady

**Screen management:**
- Alternate screen buffer (application mode - used by vim, less, htop)
- Scroll region (DECSTBM)
- Line wrapping (DECAWM)
- Origin mode (DECOM)

**Input handling:**
- Application cursor keys (DECCKM)
- Bracketed paste mode
- Mouse tracking (X10, normal, button, any-event, SGR encoding)
- Focus events

**OS integration:**
- Window/tab title (OSC 0/2)
- Working directory reporting (OSC 7)
- Clipboard access (OSC 52)
- Shell integration marks (OSC 133)

### 5.2 Nice-to-Have / Future

- Sixel graphics
- iTerm2 inline image protocol
- Kitty graphics protocol
- Kitty keyboard protocol
- Synchronized rendering (DEC mode 2026)

---

## 6. Unicode Support

### 6.1 Challenges

**Wide characters (CJK):**
- Characters occupy 2 cells instead of 1
- Need the `unicode-width` crate to determine width
- Edge cases: character at end of line with only 1 cell remaining - wrap or truncate?

**Combining characters:**
- Diacritical marks attach to the preceding character: `e` + `\u0301` = `e\u0301`
- A grapheme cluster can be multiple code points
- Cell model: 1 cell = 1 grapheme cluster, but can be multiple code points

**Emoji:**
- Emoji can be 1 or 2 cells wide
- Emoji sequences: a family emoji is 7 code points but renders as 1 grapheme (2 cells)
- ZWJ (Zero Width Joiner) sequences
- Skin tone modifiers
- Emoji presentation selectors (VS15, VS16)

**Ligatures:**
- Font ligatures (e.g., `fi`, `->`, `=>` in coding fonts)
- Terminal context: ligatures are complex because each character is 1 cell
- Some terminals (Kitty, WezTerm) support ligatures by rendering groups of cells together

### 6.2 Recommendations for Wit

1. **Use `unicode-segmentation`** for grapheme cluster boundaries
2. **Use `unicode-width`** for character width (1 vs 2 cells)
3. **Cell model:** Each cell stores 1 grapheme cluster + attributes
4. **Wide character handling:** Wide char writes to cell N, cell N+1 marked as continuation
5. **Start simple:** Support standard Unicode text, defer complex emoji rendering

---

## 7. Modern Extensions

### 7.1 True Color (24-bit)

```
ESC[38;2;R;G;Bm   - Set foreground color (RGB)
ESC[48;2;R;G;Bm   - Set background color (RGB)
```

- Supported by: xterm (v331+), iTerm2, Kitty, Alacritty, WezTerm, Windows Terminal
- Wit **MUST** support true color from the start - this is the minimum expectation for a modern terminal

### 7.2 Hyperlinks (OSC 8)

```
ESC]8;params;uri\ESC\\    - Start hyperlink
ESC]8;;\ESC\\              - End hyperlink
```

- Allows clickable links in terminal output
- Tools like `ls --hyperlink`, `gcc` output hyperlinks
- **Important for Wit:** a context-aware terminal should detect and render hyperlinks

### 7.3 Inline Images

**Sixel:**
- Original DEC protocol for raster graphics in terminal
- Binary format, complex
- Supported by: xterm, mlterm, foot, WezTerm

**iTerm2 Inline Images Protocol:**
```
ESC]1337;File=[args]:base64-data\a
```
- Simpler API, base64 encoded image
- Args: `name`, `size`, `width`, `height`, `inline`

**Kitty Graphics Protocol:**
- Most advanced, supports PNG/JPEG
- Transmit via escape sequences or shared memory
- Supports animation

> **Recommendation for Wit:** Defer inline images. Focus on core terminal first, add image support later (iTerm2 protocol is simplest to implement).

### 7.4 Notifications

**OSC 9 (iTerm2 growl):**
```
ESC]9;message\ESC\\
```

**OSC 777 (rxvt-unicode notification):**
```
ESC]777;notify;title;body\ESC\\
```

Wit can integrate with the OS notification system - useful for long-running commands.

### 7.5 Shell Integration (OSC 133)

```
ESC]133;A\ESC\\    - Prompt start
ESC]133;B\ESC\\    - Command start (after user presses Enter)
ESC]133;C\ESC\\    - Command output start
ESC]133;D;exit\ESC\\  - Command finished with exit code
```

> **Critical for Wit:** Shell integration marks are the foundation for context-aware features. Wit MUST support OSC 133 as early as possible.

---

## 8. Open Questions and Decisions

### 8.1 Parser Architecture

| Question | Options | Leaning |
|----------|---------|---------|
| Use `vte` crate or custom parser? | `vte` (proven, used by Alacritty) vs custom (full control) | Start with `vte`, consider custom later |
| Parser runs on which thread? | Dedicated parser thread vs async task | Dedicated thread with channel to UI |

### 8.2 Terminal Grid

| Question | Options | Leaning |
|----------|---------|---------|
| Cell representation? | `char` + attrs vs `String` (grapheme) + attrs | String for grapheme cluster support |
| Scrollback storage? | Ring buffer vs Vec with compaction | Ring buffer for memory efficiency |
| Scrollback limit? | Fixed vs configurable vs unlimited | Configurable, default 10,000 lines |

### 8.3 Rendering

| Question | Options | Leaning |
|----------|---------|---------|
| Render in React (Canvas/DOM) or Rust (wgpu/skia)? | React Canvas (simpler) vs Rust GPU (faster) | React Canvas initially, Rust GPU if performance is needed |
| Font rendering? | System fonts vs custom font stack | System fonts with configurable font family |
| Ligature support? | Day 1 vs later vs never | Later - complex, not essential |

### 8.4 Feature Priorities

**Phase 1 (MVP):**
- VT100/VT220 compatible (core ECMA-48)
- 256-color + true color
- UTF-8 with basic Unicode
- Alternate screen buffer
- Mouse tracking (SGR mode)
- Bracketed paste
- OSC 0/2 (title), OSC 7 (CWD)

**Phase 2:**
- OSC 133 (shell integration)
- OSC 8 (hyperlinks)
- OSC 52 (clipboard)
- Focus events
- Synchronized rendering

**Phase 3:**
- Inline images (iTerm2 protocol)
- Kitty keyboard protocol
- Sixel
- Custom terminfo entry

---

## References

1. ECMA-48 Standard: https://ecma-international.org/publications-and-standards/standards/ecma-48/
2. XTerm Control Sequences: https://invisible-island.net/xterm/ctlseqs/ctlseqs.html
3. VT100.net: https://vt100.net/
4. Paul Williams' Parser: https://vt100.net/emu/dec_ansi_parser
5. Terminal WG Specs: https://gitlab.freedesktop.org/terminal-wg/specifications
6. Unicode Standard Annex #29 (Grapheme Clusters): https://unicode.org/reports/tr29/
7. Unicode Standard Annex #11 (East Asian Width): https://unicode.org/reports/tr11/
