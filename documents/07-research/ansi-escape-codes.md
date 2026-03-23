# ANSI Escape Codes - Reference Document

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. C0 Control Codes (0x00-0x1F, 0x7F)

Single-byte control characters, no escape prefix needed.

| Hex | Dec | Abbr | Name | Behavior | Priority |
|-----|-----|------|------|----------|----------|
| `00` | 0 | NUL | Null | Ignored | Must |
| `01` | 1 | SOH | Start of Heading | Ignored | Skip |
| `02` | 2 | STX | Start of Text | Ignored | Skip |
| `03` | 3 | ETX | End of Text | Ignored (Ctrl+C handled at PTY level) | Skip |
| `04` | 4 | EOT | End of Transmission | Ignored (Ctrl+D handled at PTY level) | Skip |
| `05` | 5 | ENQ | Enquiry | Return answerback string | Should |
| `06` | 6 | ACK | Acknowledge | Ignored | Skip |
| `07` | 7 | BEL | Bell | Trigger visual/audible bell | Must |
| `08` | 8 | BS | Backspace | Move cursor left one column (does not delete) | Must |
| `09` | 9 | HT | Horizontal Tab | Move cursor to next tab stop | Must |
| `0A` | 10 | LF | Line Feed | Move cursor down, scroll if at bottom | Must |
| `0B` | 11 | VT | Vertical Tab | Same as LF | Must |
| `0C` | 12 | FF | Form Feed | Same as LF | Must |
| `0D` | 13 | CR | Carriage Return | Move cursor to column 0 | Must |
| `0E` | 14 | SO | Shift Out | Switch to G1 character set | Should |
| `0F` | 15 | SI | Shift In | Switch to G0 character set | Should |
| `10`-`17` | 16-23 | DLE-ETB | - | Ignored | Skip |
| `18` | 24 | CAN | Cancel | Abort current escape sequence | Must |
| `19` | 25 | EM | End of Medium | Ignored | Skip |
| `1A` | 26 | SUB | Substitute | Abort current escape sequence, display error char | Must |
| `1B` | 27 | ESC | Escape | Start escape sequence | Must |
| `1C`-`1F` | 28-31 | FS-US | Separators | Ignored | Skip |
| `7F` | 127 | DEL | Delete | Ignored | Must |

---

## 2. ESC Sequences (ESC + single byte)

Format: `ESC` (0x1B) + character

| Sequence | Name | Behavior | Priority |
|----------|------|----------|----------|
| `ESC 7` | DECSC | Save cursor position, attributes, character set | Must |
| `ESC 8` | DECRC | Restore cursor position, attributes, character set | Must |
| `ESC =` | DECKPAM | Application keypad mode | Must |
| `ESC >` | DECKPNM | Normal keypad mode | Must |
| `ESC D` | IND | Index - move cursor down, scroll if at bottom | Must |
| `ESC E` | NEL | Next Line - move to start of next line, scroll if at bottom | Must |
| `ESC H` | HTS | Horizontal Tab Set - set tab stop at current column | Must |
| `ESC M` | RI | Reverse Index - move cursor up, scroll down if at top | Must |
| `ESC N` | SS2 | Single Shift 2 - next char from G2 set | Should |
| `ESC O` | SS3 | Single Shift 3 - next char from G3 set | Should |
| `ESC P` | DCS | Device Control String - start DCS sequence | Should |
| `ESC Z` | DECID | Identify Terminal - return device attributes (same as `CSI c`) | Should |
| `ESC [` | CSI | Control Sequence Introducer - start CSI sequence | Must |
| `ESC \` | ST | String Terminator - end OSC/DCS/APC/PM/SOS string | Must |
| `ESC ]` | OSC | Operating System Command - start OSC sequence | Must |
| `ESC ^` | PM | Privacy Message - ignored string | Should |
| `ESC _` | APC | Application Program Command - ignored string | Should |
| `ESC c` | RIS | Full Reset - reset terminal to initial state | Must |
| `ESC ( C` | SCS G0 | Designate G0 Character Set (C = charset code) | Should |
| `ESC ) C` | SCS G1 | Designate G1 Character Set | Should |
| `ESC * C` | SCS G2 | Designate G2 Character Set | Future |
| `ESC + C` | SCS G3 | Designate G3 Character Set | Future |

Character set codes: `B` = US ASCII, `0` = DEC Special Graphics (line drawing), `A` = UK

---

## 3. CSI Sequences

Format: `ESC [` + params + intermediate + final byte
Params: semicolon-separated decimal numbers (e.g., `1;2;3`)
Default param value is usually 0 or 1 (specified per sequence)

### 3.1 Cursor Positioning

| Sequence | Final | Name | Params | Default | Behavior | Priority |
|----------|-------|------|--------|---------|----------|----------|
| `CSI n A` | A | CUU | n = count | 1 | Cursor Up n rows | Must |
| `CSI n B` | B | CUD | n = count | 1 | Cursor Down n rows | Must |
| `CSI n C` | C | CUF | n = count | 1 | Cursor Forward n columns | Must |
| `CSI n D` | D | CUB | n = count | 1 | Cursor Backward n columns | Must |
| `CSI n E` | E | CNL | n = count | 1 | Cursor Next Line (move down n, column 0) | Must |
| `CSI n F` | F | CPL | n = count | 1 | Cursor Previous Line (move up n, column 0) | Must |
| `CSI n G` | G | CHA | n = column | 1 | Cursor Horizontal Absolute (move to column n) | Must |
| `CSI r;c H` | H | CUP | r = row, c = col | 1;1 | Cursor Position (move to row r, column c) | Must |
| `CSI n I` | I | CHT | n = count | 1 | Cursor Horizontal Tab (move forward n tab stops) | Should |
| `CSI n Z` | Z | CBT | n = count | 1 | Cursor Backward Tab (move backward n tab stops) | Should |
| `CSI r;c f` | f | HVP | r = row, c = col | 1;1 | Horizontal Vertical Position (same as CUP) | Must |
| `CSI n d` | d | VPA | n = row | 1 | Vertical Position Absolute (move to row n) | Must |
| `CSI s` | s | SCP | - | - | Save Cursor Position (ANSI.SYS variant) | Should |
| `CSI u` | u | RCP | - | - | Restore Cursor Position (ANSI.SYS variant) | Should |

> **Note:** Row/column values are 1-based. CUP(1,1) = top-left corner.

### 3.2 Erase

| Sequence | Final | Name | Params | Default | Behavior | Priority |
|----------|-------|------|--------|---------|----------|----------|
| `CSI n J` | J | ED | n = mode | 0 | Erase in Display | Must |
| | | | 0 | | Erase from cursor to end of screen | |
| | | | 1 | | Erase from start of screen to cursor | |
| | | | 2 | | Erase entire screen | |
| | | | 3 | | Erase scrollback buffer (xterm extension) | |
| `CSI n K` | K | EL | n = mode | 0 | Erase in Line | Must |
| | | | 0 | | Erase from cursor to end of line | |
| | | | 1 | | Erase from start of line to cursor | |
| | | | 2 | | Erase entire line | |
| `CSI n X` | X | ECH | n = count | 1 | Erase n Characters (replace with space, don't move cursor) | Must |

### 3.3 Insert/Delete

| Sequence | Final | Name | Params | Default | Behavior | Priority |
|----------|-------|------|--------|---------|----------|----------|
| `CSI n @` | @ | ICH | n = count | 1 | Insert n blank Characters at cursor | Must |
| `CSI n P` | P | DCH | n = count | 1 | Delete n Characters at cursor, shift remaining left | Must |
| `CSI n L` | L | IL | n = count | 1 | Insert n blank Lines at cursor row, push down | Must |
| `CSI n M` | M | DL | n = count | 1 | Delete n Lines at cursor row, pull up | Must |

### 3.4 Scroll

| Sequence | Final | Name | Params | Default | Behavior | Priority |
|----------|-------|------|--------|---------|----------|----------|
| `CSI n S` | S | SU | n = count | 1 | Scroll Up n lines (content moves up, new blank lines at bottom) | Must |
| `CSI n T` | T | SD | n = count | 1 | Scroll Down n lines (content moves down, new blank lines at top) | Must |

### 3.5 SGR - Select Graphic Rendition (CSI n m)

Format: `CSI n1;n2;...;nk m` - set multiple attributes at once.

#### Basic Attributes

| Code | Name | Behavior | Priority |
|------|------|----------|----------|
| 0 | Reset | Reset all attributes to default | Must |
| 1 | Bold | Bold / increased intensity | Must |
| 2 | Dim | Faint / decreased intensity | Must |
| 3 | Italic | Italic | Must |
| 4 | Underline | Underline | Must |
| 5 | Slow Blink | Blink (< 150/min) | Should |
| 6 | Rapid Blink | Blink (>= 150/min), rarely supported | Future |
| 7 | Inverse | Swap foreground/background | Must |
| 8 | Hidden | Invisible text (still selectable) | Must |
| 9 | Strikethrough | Crossed out | Must |
| 21 | Double Underline | Double underline (or bold off in some) | Should |
| 22 | Normal Intensity | Neither bold nor dim | Must |
| 23 | Not Italic | Disable italic | Must |
| 24 | Not Underline | Disable underline | Must |
| 25 | Not Blink | Disable blink | Must |
| 27 | Not Inverse | Disable inverse | Must |
| 28 | Not Hidden | Disable hidden | Must |
| 29 | Not Strikethrough | Disable strikethrough | Must |

#### Foreground Colors

| Code | Color | Priority |
|------|-------|----------|
| 30 | Black | Must |
| 31 | Red | Must |
| 32 | Green | Must |
| 33 | Yellow | Must |
| 34 | Blue | Must |
| 35 | Magenta | Must |
| 36 | Cyan | Must |
| 37 | White | Must |
| 38 | Extended (see below) | Must |
| 39 | Default foreground | Must |
| 90 | Bright Black (Gray) | Must |
| 91 | Bright Red | Must |
| 92 | Bright Green | Must |
| 93 | Bright Yellow | Must |
| 94 | Bright Blue | Must |
| 95 | Bright Magenta | Must |
| 96 | Bright Cyan | Must |
| 97 | Bright White | Must |

#### Background Colors

| Code | Color | Priority |
|------|-------|----------|
| 40-47 | Standard (same as 30-37) | Must |
| 48 | Extended (see below) | Must |
| 49 | Default background | Must |
| 100-107 | Bright (same as 90-97) | Must |

#### Extended Color (256-color and True Color)

```
256-color foreground:  CSI 38;5;N m     (N = 0-255)
256-color background:  CSI 48;5;N m     (N = 0-255)
True color foreground: CSI 38;2;R;G;B m (R,G,B = 0-255)
True color background: CSI 48;2;R;G;B m (R,G,B = 0-255)
```

**256-color palette layout:**
| Range | Count | Description |
|-------|-------|-------------|
| 0-7 | 8 | Standard colors (same as SGR 30-37) |
| 8-15 | 8 | Bright colors (same as SGR 90-97) |
| 16-231 | 216 | 6 x 6 x 6 color cube: `16 + 36*r + 6*g + b` (r,g,b = 0-5) |
| 232-255 | 24 | Grayscale: darkest (232) to lightest (255) |

#### Underline Color (Kitty extension)

```
CSI 58;5;N m       - 256-color underline color
CSI 58;2;R;G;B m   - True color underline color
CSI 59 m           - Default underline color
```

Priority: Future

#### Underline Style (Kitty extension)

```
CSI 4:0 m  - No underline
CSI 4:1 m  - Single underline
CSI 4:2 m  - Double underline
CSI 4:3 m  - Curly underline
CSI 4:4 m  - Dotted underline
CSI 4:5 m  - Dashed underline
```

Priority: Future

### 3.6 Mode Setting

| Sequence | Name | Behavior | Priority |
|----------|------|----------|----------|
| `CSI n h` | SM | Set Mode (ANSI mode n) | Should |
| `CSI n l` | RM | Reset Mode (ANSI mode n) | Should |
| `CSI ? n h` | DECSET | Set DEC Private Mode n | Must |
| `CSI ? n l` | DECRST | Reset DEC Private Mode n | Must |

**DEC Private Modes - see Section 6 below.**

### 3.7 Device Status

| Sequence | Final | Name | Behavior | Response | Priority |
|----------|-------|------|----------|----------|----------|
| `CSI 5 n` | n | DSR | Device Status Report | `CSI 0 n` (OK) | Must |
| `CSI 6 n` | n | CPR | Cursor Position Report | `CSI r;c R` (row;col) | Must |
| `CSI c` | c | DA1 | Primary Device Attributes | `CSI ? 62;c... c` | Must |
| `CSI > c` | c | DA2 | Secondary Device Attributes | `CSI > Pp;Pv;Pc c` | Should |
| `CSI = c` | c | DA3 | Tertiary Device Attributes | `DCS ! | XXXXXXXX ST` | Future |

### 3.8 Scroll Region

| Sequence | Name | Params | Behavior | Priority |
|----------|------|--------|----------|----------|
| `CSI top;bottom r` | DECSTBM | top, bottom | Set scroll region (rows top through bottom) | Must |

Default: full screen. Affects: LF, RI, IL, DL, SU, SD within region.

### 3.9 Tab Stops

| Sequence | Name | Behavior | Priority |
|----------|------|----------|----------|
| `CSI 0 g` | TBC | Clear tab stop at current column | Should |
| `CSI 3 g` | TBC | Clear all tab stops | Should |

### 3.10 Miscellaneous CSI

| Sequence | Name | Behavior | Priority |
|----------|------|----------|----------|
| `CSI n b` | REP | Repeat preceding character n times | Should |
| `CSI n t` | XTWINOPS | Window manipulation (xterm) | Future |
| `CSI > n q` | XTVERSION | xterm version query | Future |
| `CSI n SP q` | DECSCUSR | Set Cursor Style | Must |

**Cursor Styles (DECSCUSR):**
| n | Style |
|---|-------|
| 0 | Default (usually blinking block) |
| 1 | Blinking block |
| 2 | Steady block |
| 3 | Blinking underline |
| 4 | Steady underline |
| 5 | Blinking bar |
| 6 | Steady bar |

---

## 4. OSC Sequences

Format: `ESC ]` + number + `;` + data + (`BEL` | `ESC \`)

| OSC | Name | Format | Behavior | Priority |
|-----|------|--------|----------|----------|
| 0 | Set Icon Name and Title | `OSC 0;text ST` | Set window/tab title | Must |
| 1 | Set Icon Name | `OSC 1;text ST` | Set icon name (usually ignored) | Should |
| 2 | Set Title | `OSC 2;text ST` | Set window title | Must |
| 4 | Color Palette | `OSC 4;index;spec ST` | Set/query color palette entry | Should |
| 7 | Current Working Directory | `OSC 7;file://host/path ST` | Report CWD to terminal | Must |
| 8 | Hyperlink | `OSC 8;params;uri ST` | Start/end hyperlink | Must |
| 9 | Notification (iTerm2) | `OSC 9;text ST` | System notification | Should |
| 10 | Foreground Color | `OSC 10;spec ST` | Set/query default foreground | Should |
| 11 | Background Color | `OSC 11;spec ST` | Set/query default background | Should |
| 12 | Cursor Color | `OSC 12;spec ST` | Set/query cursor color | Should |
| 52 | Clipboard | `OSC 52;selection;base64 ST` | Get/set clipboard content | Should |
| 104 | Reset Color | `OSC 104;index ST` | Reset color palette entry | Should |
| 110 | Reset Foreground | `OSC 110 ST` | Reset default foreground | Should |
| 111 | Reset Background | `OSC 111 ST` | Reset default background | Should |
| 112 | Reset Cursor Color | `OSC 112 ST` | Reset cursor color | Should |
| 133 | Shell Integration | `OSC 133;type ST` | Shell integration marks | Must |

### 4.1 OSC 7 - Working Directory (Details)

```
ESC ] 7 ; file://hostname/path/to/dir ESC \
```

- Shell sends this when changing directory
- Hostname can be an empty string
- Path must be URL-encoded
- **Critical for Wit:** Foundation for context-aware features - knowing which directory the user is in

### 4.2 OSC 8 - Hyperlinks (Details)

```
ESC ] 8 ; params ; uri ESC \    <- Start link
visible text
ESC ] 8 ; ; ESC \               <- End link (empty URI)
```

Params format: `key1=value1:key2=value2`
- `id=xxx` - group multiple link segments (spanning lines) as one link

Example:
```
\e]8;;https://example.com\e\\Click here\e]8;;\e\\
```

### 4.3 OSC 52 - Clipboard (Details)

```
ESC ] 52 ; selection ; base64-data ESC \    <- Set clipboard
ESC ] 52 ; selection ; ? ESC \              <- Query clipboard
```

Selection: `c` = clipboard, `p` = primary, `s` = secondary
Base64 data: Base64-encoded UTF-8 text

**Security concern:** Should have a setting to enable/disable, default disabled for query.

### 4.4 OSC 133 - Shell Integration (Details)

| Mark | Meaning | Context |
|------|---------|---------|
| `OSC 133;A ST` | **Prompt Start** | Shell begins displaying prompt |
| `OSC 133;B ST` | **Command Start** | User pressed Enter, command begins |
| `OSC 133;C ST` | **Output Start** | Command output begins |
| `OSC 133;D;exitcode ST` | **Command Complete** | Command finished with exit code |

> **Wit context engine will use these marks to:**
> - Identify command boundaries (blocks)
> - Track command exit codes
> - Enable click-to-rerun
> - Navigate between prompts

---

## 5. DCS Sequences

Format: `ESC P` + data + `ESC \`

| Sequence | Name | Behavior | Priority |
|----------|------|----------|----------|
| `DCS + p name ST` | XTGETTCAP | Request termcap/terminfo value | Future |
| `DCS + q hex ST` | XTGETXRES | Request xterm resource | Future |
| `DCS $ q param ST` | DECRQSS | Request setting | Future |
| `DCS q data ST` | Sixel | Sixel graphics data | Future |
| `DCS > \| data ST` | XTVERSION | xterm version response | Future |

> DCS sequences are complex and rarely used. Defer implementation for Wit.

---

## 6. DEC Private Modes

Set: `CSI ? n h` | Reset: `CSI ? n l`

| Mode | Name | Set (h) | Reset (l) | Priority |
|------|------|---------|-----------|----------|
| 1 | DECCKM | Application cursor keys (ESC O A/B/C/D) | Normal cursor keys (CSI A/B/C/D) | Must |
| 3 | DECCOLM | 132 column mode | 80 column mode | Future |
| 5 | DECSCNM | Reverse video (swap fg/bg for whole screen) | Normal video | Should |
| 6 | DECOM | Origin mode (cursor relative to scroll region) | Absolute positioning | Must |
| 7 | DECAWM | Auto-wrap mode (wrap at end of line) | No auto-wrap | Must |
| 12 | Cursor Blink | Start blinking | Stop blinking (steady) | Should |
| 25 | DECTCEM | Show cursor | Hide cursor | Must |
| 47 | Alt Screen | Switch to alternate screen buffer | Switch to normal screen buffer | Must |
| 66 | DECNKM | Application keypad | Normal keypad | Should |
| 69 | DECLRMM | Enable left/right margin mode | Disable | Future |
| 1000 | Mouse X10 | Enable X10 mouse reporting (button press) | Disable | Must |
| 1002 | Mouse Button | Enable button-event mouse tracking | Disable | Must |
| 1003 | Mouse Any | Enable any-event mouse tracking | Disable | Must |
| 1004 | Focus Events | Enable focus in/out events | Disable | Must |
| 1005 | Mouse UTF-8 | UTF-8 mouse coordinate encoding | Disable | Should |
| 1006 | Mouse SGR | SGR mouse coordinate encoding | Disable | Must |
| 1015 | Mouse URXVT | URXVT mouse coordinate encoding | Disable | Future |
| 1049 | Alt Screen + Cursor | Save cursor + switch to alt screen | Restore cursor + switch to normal | Must |
| 2004 | Bracketed Paste | Enable bracketed paste mode | Disable | Must |
| 2026 | Synchronized Rendering | Begin synchronized update | End synchronized update | Should |

### Mouse Encoding Comparison

| Mode | Format | Max Coords | Recommended |
|------|--------|------------|-------------|
| X10 (legacy) | `CSI M Cb Cx Cy` | 223 | No |
| UTF-8 (1005) | `CSI M Cb Cx Cy` (UTF-8 encoded) | 2047 | No |
| SGR (1006) | `CSI < Pb;Px;Py M/m` | Unlimited | **Yes** |
| URXVT (1015) | `CSI Pb;Px;Py M` | Unlimited | No |

> **Wit should use SGR mouse encoding** (mode 1006) as the default. It supports unlimited coordinates and distinguishes press/release.

### Bracketed Paste Mode (2004)

When enabled, terminal wraps pasted text:
```
ESC [ 200 ~    <- Paste start
...pasted text...
ESC [ 201 ~    <- Paste end
```

Applications use this to distinguish typed input vs pasted text (e.g., not auto-executing pasted commands).

### Synchronized Rendering (2026)

```
CSI ? 2026 h   <- Begin synchronized update (start buffering)
...terminal output...
CSI ? 2026 l   <- End synchronized update (flush/render)
```

Prevents flickering when an application sends many updates in succession. Terminal buffers all changes then renders once.

---

## 7. Special Key Encodings

### 7.1 Normal Mode vs Application Mode

| Key | Normal (DECCKM reset) | Application (DECCKM set) |
|-----|----------------------|--------------------------|
| Up | `CSI A` | `ESC O A` |
| Down | `CSI B` | `ESC O B` |
| Right | `CSI C` | `ESC O C` |
| Left | `CSI D` | `ESC O D` |
| Home | `CSI H` | `ESC O H` |
| End | `CSI F` | `ESC O F` |

### 7.2 Function Keys

| Key | Sequence |
|-----|----------|
| F1 | `ESC O P` |
| F2 | `ESC O Q` |
| F3 | `ESC O R` |
| F4 | `ESC O S` |
| F5 | `CSI 15 ~` |
| F6 | `CSI 17 ~` |
| F7 | `CSI 18 ~` |
| F8 | `CSI 19 ~` |
| F9 | `CSI 20 ~` |
| F10 | `CSI 21 ~` |
| F11 | `CSI 23 ~` |
| F12 | `CSI 24 ~` |

### 7.3 Modified Keys

Modifier encoding: `CSI 1;modifier code`

| Modifier | Code |
|----------|------|
| Shift | 2 |
| Alt | 3 |
| Shift+Alt | 4 |
| Ctrl | 5 |
| Shift+Ctrl | 6 |
| Alt+Ctrl | 7 |
| Shift+Alt+Ctrl | 8 |

Example: Ctrl+Right = `CSI 1;5 C`

---

## 8. Implementation Checklist for Wit

### Phase 1 - Core (MVP)
- [ ] C0: BEL, BS, HT, LF, VT, FF, CR, ESC, CAN, SUB
- [ ] ESC: DECSC(7), DECRC(8), IND(D), NEL(E), RI(M), RIS(c), CSI([), OSC(]), ST(\)
- [ ] CSI Cursor: CUU(A), CUD(B), CUF(C), CUB(D), CNL(E), CPL(F), CHA(G), CUP(H), HVP(f), VPA(d)
- [ ] CSI Erase: ED(J), EL(K), ECH(X)
- [ ] CSI Insert/Delete: ICH(@), DCH(P), IL(L), DL(M)
- [ ] CSI Scroll: SU(S), SD(T)
- [ ] CSI SGR(m): reset(0), bold(1), dim(2), italic(3), underline(4), blink(5), inverse(7), hidden(8), strike(9), disable(22-29), colors(30-37,40-47,90-97,100-107), extended(38,48), default(39,49)
- [ ] CSI DECSTBM(r), DECSCUSR(SP q)
- [ ] CSI DSR(n): status(5), cursor(6)
- [ ] CSI DA1(c)
- [ ] DEC modes: DECCKM(1), DECOM(6), DECAWM(7), DECTCEM(25), AltScreen(47/1049), Mouse(1000/1002/1003/1006), BracketedPaste(2004)
- [ ] OSC: title(0/2), CWD(7)

### Phase 2 - Enhanced
- [ ] OSC: hyperlinks(8), clipboard(52), shell integration(133)
- [ ] DEC modes: focus events(1004), synchronized rendering(2026)
- [ ] CSI: REP(b), CHT(I), CBT(Z), TBC(g)
- [ ] Character sets: G0/G1 (SO/SI, ESC(/))

### Phase 3 - Advanced
- [ ] Sixel graphics (DCS)
- [ ] Kitty keyboard protocol
- [ ] Kitty underline extensions
- [ ] Window manipulation (CSI t)

---

## References

1. XTerm Control Sequences: https://invisible-island.net/xterm/ctlseqs/ctlseqs.html
2. ECMA-48: https://ecma-international.org/publications-and-standards/standards/ecma-48/
3. console_codes(4) man page: `man 4 console_codes`
4. VT100 User Guide: https://vt100.net/docs/vt100-ug/
5. Wikipedia ANSI Escape Code: https://en.wikipedia.org/wiki/ANSI_escape_code
6. Kitty Protocol Extensions: https://sw.kovidgoyal.net/kitty/protocol-extensions/
