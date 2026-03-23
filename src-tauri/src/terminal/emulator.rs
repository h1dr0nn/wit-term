//! Terminal emulator - processes ANSI parser actions and updates the grid.

use crate::parser::{Action, Parser};

use super::grid::{CellAttrs, Color, Grid, NamedColor};
use super::{AttrFlags, BlockInfo, BlockState, CellData, CommandBlock, Cursor, GridSnapshot, TerminalModes};

/// The terminal emulator owns the grid, cursor, parser, and modes.
pub struct Emulator {
    pub grid: Grid,
    pub cursor: Cursor,
    pub modes: TerminalModes,
    parser: Parser,
    saved_cursor: Option<Cursor>,
    /// Title set by OSC sequences.
    pub title: Option<String>,
    /// Current working directory from OSC 7.
    pub cwd: Option<std::path::PathBuf>,
    /// Whether the grid has changed since the last snapshot.
    dirty: bool,
    /// Whether the CWD changed since last check.
    cwd_dirty: bool,
    /// Command blocks (Warp-style).
    pub blocks: Vec<CommandBlock>,
    /// Current block tracking state.
    pub block_state: BlockState,
    /// Next block ID.
    next_block_id: u32,
}

impl Emulator {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            grid: Grid::new(cols, rows),
            cursor: Cursor::default(),
            modes: TerminalModes::default(),
            parser: Parser::new(),
            saved_cursor: None,
            title: None,
            cwd: None,
            dirty: true,
            cwd_dirty: false,
            blocks: Vec::new(),
            block_state: BlockState::Idle,
            next_block_id: 1,
        }
    }

    /// Process raw bytes from the PTY.
    pub fn process(&mut self, data: &[u8]) {
        let actions = self.parser.process(data);
        for action in actions {
            self.handle_action(action);
        }
    }

    /// Check and clear the dirty flag.
    pub fn take_dirty(&mut self) -> bool {
        let was_dirty = self.dirty;
        self.dirty = false;
        was_dirty
    }

    /// Check and clear the CWD dirty flag.
    pub fn take_cwd_dirty(&mut self) -> bool {
        let was = self.cwd_dirty;
        self.cwd_dirty = false;
        was
    }

    /// Get a full grid snapshot for the frontend.
    pub fn snapshot(&self) -> GridSnapshot {
        let rows: Vec<Vec<CellData>> = (0..self.grid.rows)
            .map(|r| {
                self.grid
                    .row(r)
                    .iter()
                    .map(CellData::from)
                    .collect()
            })
            .collect();

        let blocks: Vec<BlockInfo> = self
            .blocks
            .iter()
            .map(|b| {
                // Extract command text from grid rows between command_row and output_start_row
                let command = self.extract_row_text(b.command_row);
                BlockInfo {
                    id: b.id,
                    prompt_row: b.prompt_row,
                    output_start_row: b.output_start_row,
                    output_end_row: b.output_end_row,
                    exit_code: b.exit_code,
                    cwd: b.cwd.clone(),
                    command,
                }
            })
            .collect();

        GridSnapshot {
            rows,
            cursor_row: self.cursor.row,
            cursor_col: self.cursor.col,
            cursor_visible: self.cursor.visible && self.modes.cursor_visible,
            cursor_shape: self.cursor.shape,
            blocks,
        }
    }

    /// Resize the terminal.
    pub fn resize(&mut self, cols: usize, rows: usize) {
        // Clamp cursor to new dimensions
        self.grid.resize(cols, rows);
        self.cursor.col = self.cursor.col.min(cols - 1);
        self.cursor.row = self.cursor.row.min(rows - 1);
        self.dirty = true;
    }

    /// Extract text content from a grid row.
    fn extract_row_text(&self, row: usize) -> String {
        if row >= self.grid.rows {
            return String::new();
        }
        self.grid
            .row(row)
            .iter()
            .map(|c| c.content.as_str())
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    fn handle_action(&mut self, action: Action) {
        self.dirty = true;
        match action {
            Action::Print(ch) => self.handle_print(ch),
            Action::Execute(byte) => self.handle_execute(byte),
            Action::CsiDispatch {
                params,
                intermediates: _,
                final_byte,
                private_marker,
            } => self.handle_csi(&params, final_byte, private_marker),
            Action::EscDispatch {
                intermediates,
                final_byte,
            } => self.handle_esc(&intermediates, final_byte),
            Action::OscDispatch(parts) => self.handle_osc(&parts),
        }
    }

    fn handle_print(&mut self, ch: char) {
        // Handle pending wrap
        if self.cursor.pending_wrap {
            if self.modes.auto_wrap {
                self.cursor.col = 0;
                if self.cursor.row == self.grid.scroll_bottom {
                    self.grid.scroll_up(1);
                } else if self.cursor.row < self.grid.rows - 1 {
                    self.cursor.row += 1;
                }
            }
            self.cursor.pending_wrap = false;
        }

        // Write character at cursor position
        self.grid
            .cell_mut(self.cursor.row, self.cursor.col)
            .set(ch, &self.cursor.attrs);

        // Advance cursor
        if self.cursor.col < self.grid.cols - 1 {
            self.cursor.col += 1;
        } else {
            // At the right margin: set pending wrap
            self.cursor.pending_wrap = true;
        }
    }

    fn handle_execute(&mut self, byte: u8) {
        match byte {
            // BEL
            0x07 => {} // Could emit bell event
            // BS - Backspace
            0x08 => {
                if self.cursor.col > 0 {
                    self.cursor.col -= 1;
                    self.cursor.pending_wrap = false;
                }
            }
            // HT - Horizontal Tab
            0x09 => {
                let next_tab = ((self.cursor.col / 8) + 1) * 8;
                self.cursor.col = next_tab.min(self.grid.cols - 1);
                self.cursor.pending_wrap = false;
            }
            // LF, VT, FF - Line Feed (and vertical tab, form feed)
            0x0A..=0x0C => {
                if self.cursor.row == self.grid.scroll_bottom {
                    self.grid.scroll_up(1);
                } else if self.cursor.row < self.grid.rows - 1 {
                    self.cursor.row += 1;
                }
                if self.modes.linefeed_mode {
                    self.cursor.col = 0;
                }
            }
            // CR - Carriage Return
            0x0D => {
                self.cursor.col = 0;
                self.cursor.pending_wrap = false;
            }
            // SO - Shift Out (activate G1)
            0x0E => {} // Character set switching not implemented yet
            // SI - Shift In (activate G0)
            0x0F => {} // Character set switching not implemented yet
            _ => {}
        }
    }

    fn handle_csi(&mut self, params: &[u16], final_byte: u8, private_marker: Option<u8>) {
        if private_marker == Some(b'?') {
            self.handle_dec_mode(params, final_byte);
            return;
        }

        let p = |i: usize, default: u16| -> u16 {
            params.get(i).copied().filter(|&v| v > 0).unwrap_or(default)
        };

        match final_byte {
            // CUU - Cursor Up
            b'A' => {
                let n = p(0, 1) as usize;
                self.cursor.row = self.cursor.row.saturating_sub(n).max(self.grid.scroll_top);
                self.cursor.pending_wrap = false;
            }
            // CUD - Cursor Down
            b'B' => {
                let n = p(0, 1) as usize;
                self.cursor.row = (self.cursor.row + n).min(self.grid.scroll_bottom);
                self.cursor.pending_wrap = false;
            }
            // CUF - Cursor Forward
            b'C' => {
                let n = p(0, 1) as usize;
                self.cursor.col = (self.cursor.col + n).min(self.grid.cols - 1);
                self.cursor.pending_wrap = false;
            }
            // CUB - Cursor Backward
            b'D' => {
                let n = p(0, 1) as usize;
                self.cursor.col = self.cursor.col.saturating_sub(n);
                self.cursor.pending_wrap = false;
            }
            // CNL - Cursor Next Line
            b'E' => {
                let n = p(0, 1) as usize;
                self.cursor.row = (self.cursor.row + n).min(self.grid.scroll_bottom);
                self.cursor.col = 0;
                self.cursor.pending_wrap = false;
            }
            // CPL - Cursor Previous Line
            b'F' => {
                let n = p(0, 1) as usize;
                self.cursor.row = self.cursor.row.saturating_sub(n).max(self.grid.scroll_top);
                self.cursor.col = 0;
                self.cursor.pending_wrap = false;
            }
            // CHA - Cursor Horizontal Absolute
            b'G' => {
                let col = p(0, 1) as usize;
                self.cursor.col = (col - 1).min(self.grid.cols - 1);
                self.cursor.pending_wrap = false;
            }
            // CUP / HVP - Cursor Position
            b'H' | b'f' => {
                let row = p(0, 1) as usize;
                let col = p(1, 1) as usize;
                self.cursor.row = (row - 1).min(self.grid.rows - 1);
                self.cursor.col = (col - 1).min(self.grid.cols - 1);
                self.cursor.pending_wrap = false;
            }
            // ED - Erase in Display
            b'J' => {
                let mode = p(0, 0);
                match mode {
                    0 => self.grid.erase_below(self.cursor.row, self.cursor.col),
                    1 => self.grid.erase_above(self.cursor.row, self.cursor.col),
                    2 => self.grid.erase_all(),
                    3 => {
                        self.grid.erase_all();
                        self.grid.erase_scrollback();
                    }
                    _ => {}
                }
            }
            // EL - Erase in Line
            b'K' => {
                let mode = p(0, 0);
                match mode {
                    0 => self.grid.erase_line_right(self.cursor.row, self.cursor.col),
                    1 => self.grid.erase_line_left(self.cursor.row, self.cursor.col),
                    2 => self.grid.erase_line_all(self.cursor.row),
                    _ => {}
                }
            }
            // IL - Insert Lines
            b'L' => {
                let n = p(0, 1) as usize;
                self.grid
                    .insert_lines(self.cursor.row, n, self.grid.scroll_bottom);
            }
            // DL - Delete Lines
            b'M' => {
                let n = p(0, 1) as usize;
                self.grid
                    .delete_lines(self.cursor.row, n, self.grid.scroll_bottom);
            }
            // DCH - Delete Characters
            b'P' => {
                let n = p(0, 1) as usize;
                self.grid.delete_chars(self.cursor.row, self.cursor.col, n);
            }
            // SU - Scroll Up
            b'S' => {
                let n = p(0, 1) as usize;
                self.grid.scroll_up(n);
            }
            // SD - Scroll Down
            b'T' => {
                let n = p(0, 1) as usize;
                self.grid.scroll_down(n);
            }
            // ECH - Erase Characters
            b'X' => {
                let n = p(0, 1) as usize;
                for i in 0..n {
                    let col = self.cursor.col + i;
                    if col >= self.grid.cols {
                        break;
                    }
                    self.grid.cell_mut(self.cursor.row, col).reset();
                }
            }
            // ICH - Insert Characters
            b'@' => {
                let n = p(0, 1) as usize;
                self.grid
                    .insert_chars(self.cursor.row, self.cursor.col, n);
            }
            // VPA - Vertical Line Position Absolute
            b'd' => {
                let row = p(0, 1) as usize;
                self.cursor.row = (row - 1).min(self.grid.rows - 1);
                self.cursor.pending_wrap = false;
            }
            // SGR - Select Graphic Rendition
            b'm' => self.handle_sgr(params),
            // DSR - Device Status Report
            b'n' => {
                // We can't respond in the current architecture, ignore for now
            }
            // DECSTBM - Set Scroll Region
            b'r' => {
                let top = p(0, 1) as usize;
                let bottom = p(1, self.grid.rows as u16) as usize;
                self.grid.scroll_top = (top - 1).min(self.grid.rows - 1);
                self.grid.scroll_bottom = (bottom - 1).min(self.grid.rows - 1);
                // Move cursor to origin
                self.cursor.row = 0;
                self.cursor.col = 0;
                self.cursor.pending_wrap = false;
            }
            // DECSC-like via CSI s - Save Cursor Position
            b's' => {
                self.saved_cursor = Some(self.cursor.clone());
            }
            // DECRC-like via CSI u - Restore Cursor Position
            b'u' => {
                if let Some(saved) = &self.saved_cursor {
                    self.cursor = saved.clone();
                }
            }
            _ => {
                log::trace!("Unhandled CSI: params={params:?}, final={final_byte:#x}");
            }
        }
    }

    fn handle_sgr(&mut self, params: &[u16]) {
        if params.is_empty() {
            // Reset all attributes
            self.cursor.attrs = CellAttrs::default();
            return;
        }

        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => self.cursor.attrs = CellAttrs::default(),
                1 => self.cursor.attrs.flags |= AttrFlags::BOLD,
                2 => self.cursor.attrs.flags |= AttrFlags::DIM,
                3 => self.cursor.attrs.flags |= AttrFlags::ITALIC,
                4 => self.cursor.attrs.flags |= AttrFlags::UNDERLINE,
                5 => self.cursor.attrs.flags |= AttrFlags::BLINK,
                7 => self.cursor.attrs.flags |= AttrFlags::INVERSE,
                8 => self.cursor.attrs.flags |= AttrFlags::HIDDEN,
                9 => self.cursor.attrs.flags |= AttrFlags::STRIKETHROUGH,
                21 => self.cursor.attrs.flags.remove(AttrFlags::BOLD),
                22 => {
                    self.cursor.attrs.flags.remove(AttrFlags::BOLD);
                    self.cursor.attrs.flags.remove(AttrFlags::DIM);
                }
                23 => self.cursor.attrs.flags.remove(AttrFlags::ITALIC),
                24 => self.cursor.attrs.flags.remove(AttrFlags::UNDERLINE),
                25 => self.cursor.attrs.flags.remove(AttrFlags::BLINK),
                27 => self.cursor.attrs.flags.remove(AttrFlags::INVERSE),
                28 => self.cursor.attrs.flags.remove(AttrFlags::HIDDEN),
                29 => self.cursor.attrs.flags.remove(AttrFlags::STRIKETHROUGH),
                // Foreground colors (8 standard)
                30 => self.cursor.attrs.fg = Color::Named(NamedColor::Black),
                31 => self.cursor.attrs.fg = Color::Named(NamedColor::Red),
                32 => self.cursor.attrs.fg = Color::Named(NamedColor::Green),
                33 => self.cursor.attrs.fg = Color::Named(NamedColor::Yellow),
                34 => self.cursor.attrs.fg = Color::Named(NamedColor::Blue),
                35 => self.cursor.attrs.fg = Color::Named(NamedColor::Magenta),
                36 => self.cursor.attrs.fg = Color::Named(NamedColor::Cyan),
                37 => self.cursor.attrs.fg = Color::Named(NamedColor::White),
                // 256-color / RGB foreground
                38 => {
                    i += 1;
                    if let Some(color) = self.parse_extended_color(params, &mut i) {
                        self.cursor.attrs.fg = color;
                    }
                    continue; // i already advanced
                }
                39 => self.cursor.attrs.fg = Color::Default,
                // Background colors (8 standard)
                40 => self.cursor.attrs.bg = Color::Named(NamedColor::Black),
                41 => self.cursor.attrs.bg = Color::Named(NamedColor::Red),
                42 => self.cursor.attrs.bg = Color::Named(NamedColor::Green),
                43 => self.cursor.attrs.bg = Color::Named(NamedColor::Yellow),
                44 => self.cursor.attrs.bg = Color::Named(NamedColor::Blue),
                45 => self.cursor.attrs.bg = Color::Named(NamedColor::Magenta),
                46 => self.cursor.attrs.bg = Color::Named(NamedColor::Cyan),
                47 => self.cursor.attrs.bg = Color::Named(NamedColor::White),
                // 256-color / RGB background
                48 => {
                    i += 1;
                    if let Some(color) = self.parse_extended_color(params, &mut i) {
                        self.cursor.attrs.bg = color;
                    }
                    continue;
                }
                49 => self.cursor.attrs.bg = Color::Default,
                // Bright foreground colors
                90 => self.cursor.attrs.fg = Color::Named(NamedColor::BrightBlack),
                91 => self.cursor.attrs.fg = Color::Named(NamedColor::BrightRed),
                92 => self.cursor.attrs.fg = Color::Named(NamedColor::BrightGreen),
                93 => self.cursor.attrs.fg = Color::Named(NamedColor::BrightYellow),
                94 => self.cursor.attrs.fg = Color::Named(NamedColor::BrightBlue),
                95 => self.cursor.attrs.fg = Color::Named(NamedColor::BrightMagenta),
                96 => self.cursor.attrs.fg = Color::Named(NamedColor::BrightCyan),
                97 => self.cursor.attrs.fg = Color::Named(NamedColor::BrightWhite),
                // Bright background colors
                100 => self.cursor.attrs.bg = Color::Named(NamedColor::BrightBlack),
                101 => self.cursor.attrs.bg = Color::Named(NamedColor::BrightRed),
                102 => self.cursor.attrs.bg = Color::Named(NamedColor::BrightGreen),
                103 => self.cursor.attrs.bg = Color::Named(NamedColor::BrightYellow),
                104 => self.cursor.attrs.bg = Color::Named(NamedColor::BrightBlue),
                105 => self.cursor.attrs.bg = Color::Named(NamedColor::BrightMagenta),
                106 => self.cursor.attrs.bg = Color::Named(NamedColor::BrightCyan),
                107 => self.cursor.attrs.bg = Color::Named(NamedColor::BrightWhite),
                _ => {}
            }
            i += 1;
        }
    }

    fn parse_extended_color(&self, params: &[u16], i: &mut usize) -> Option<Color> {
        if *i >= params.len() {
            return None;
        }
        match params[*i] {
            // 256-color: 38;5;N or 48;5;N
            5 => {
                *i += 1;
                if *i < params.len() {
                    let idx = params[*i] as u8;
                    *i += 1;
                    Some(Color::Indexed(idx))
                } else {
                    None
                }
            }
            // RGB: 38;2;R;G;B or 48;2;R;G;B
            2 => {
                if *i + 3 < params.len() {
                    let r = params[*i + 1] as u8;
                    let g = params[*i + 2] as u8;
                    let b = params[*i + 3] as u8;
                    *i += 4;
                    Some(Color::Rgb(r, g, b))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn handle_dec_mode(&mut self, params: &[u16], final_byte: u8) {
        let set = final_byte == b'h';
        for &mode in params {
            match mode {
                // DECCKM - Cursor Keys Mode
                1 => {} // TODO: cursor keys application mode
                // DECSCNM - Screen Mode (reverse video)
                5 => {} // TODO: reverse video
                // DECOM - Origin Mode
                6 => self.modes.origin_mode = set,
                // DECAWM - Auto-wrap Mode
                7 => self.modes.auto_wrap = set,
                // DECTCEM - Text Cursor Enable Mode
                25 => self.modes.cursor_visible = set,
                // Alt Screen Buffer
                47 | 1047 => self.modes.alt_screen = set,
                // Alt Screen Buffer with save/restore cursor
                1049 => {
                    if set {
                        self.saved_cursor = Some(self.cursor.clone());
                        self.modes.alt_screen = true;
                        self.grid.erase_all();
                    } else {
                        self.modes.alt_screen = false;
                        if let Some(saved) = self.saved_cursor.take() {
                            self.cursor = saved;
                        }
                    }
                }
                // Bracketed paste
                2004 => self.modes.bracketed_paste = set,
                _ => {
                    log::trace!("Unhandled DEC mode: {mode}, set={set}");
                }
            }
        }
    }

    fn handle_esc(&mut self, intermediates: &[u8], final_byte: u8) {
        match (intermediates, final_byte) {
            // IND - Index (move down, scroll if at bottom)
            ([], b'D') => {
                if self.cursor.row == self.grid.scroll_bottom {
                    self.grid.scroll_up(1);
                } else if self.cursor.row < self.grid.rows - 1 {
                    self.cursor.row += 1;
                }
            }
            // NEL - Next Line
            ([], b'E') => {
                self.cursor.col = 0;
                if self.cursor.row == self.grid.scroll_bottom {
                    self.grid.scroll_up(1);
                } else if self.cursor.row < self.grid.rows - 1 {
                    self.cursor.row += 1;
                }
            }
            // RI - Reverse Index (move up, scroll down if at top)
            ([], b'M') => {
                if self.cursor.row == self.grid.scroll_top {
                    self.grid.scroll_down(1);
                } else if self.cursor.row > 0 {
                    self.cursor.row -= 1;
                }
            }
            // DECSC - Save Cursor
            ([], b'7') => {
                self.saved_cursor = Some(self.cursor.clone());
            }
            // DECRC - Restore Cursor
            ([], b'8') => {
                if let Some(saved) = &self.saved_cursor {
                    self.cursor = saved.clone();
                }
            }
            // RIS - Full Reset
            ([], b'c') => {
                self.grid.erase_all();
                self.grid.erase_scrollback();
                self.cursor = Cursor::default();
                self.modes = TerminalModes::default();
                self.grid.scroll_top = 0;
                self.grid.scroll_bottom = self.grid.rows - 1;
            }
            _ => {
                log::trace!(
                    "Unhandled ESC: intermediates={intermediates:?}, final={final_byte:#x}"
                );
            }
        }
    }

    fn handle_osc(&mut self, parts: &[Vec<u8>]) {
        if parts.is_empty() {
            return;
        }

        let cmd = std::str::from_utf8(&parts[0]).unwrap_or("");
        match cmd {
            // Set window title
            "0" | "2" => {
                if parts.len() > 1 {
                    self.title =
                        Some(String::from_utf8_lossy(&parts[1]).into_owned());
                }
            }
            // CWD reporting: OSC 7 ; file://hostname/path ST
            "7" => {
                if parts.len() > 1 {
                    let uri = String::from_utf8_lossy(&parts[1]);
                    if let Some(path) = parse_file_uri(&uri) {
                        self.cwd = Some(std::path::PathBuf::from(path));
                        self.cwd_dirty = true;
                    }
                }
            }
            // OSC 133 - Shell integration / semantic prompt
            "133" => {
                if parts.len() > 1 {
                    let sub = std::str::from_utf8(&parts[1]).unwrap_or("");
                    self.handle_osc133(sub);
                }
            }
            _ => {
                log::trace!("Unhandled OSC: cmd={cmd}");
            }
        }
    }

    /// Handle OSC 133 semantic prompt sequences.
    fn handle_osc133(&mut self, sub: &str) {
        match sub.chars().next() {
            // A = Prompt start
            Some('A') => {
                // Close previous block if it was executing
                if let Some(last) = self.blocks.last_mut() {
                    if last.output_end_row.is_none() {
                        last.output_end_row = Some(self.cursor.row.saturating_sub(1));
                    }
                }

                let cwd = self
                    .cwd
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default();

                let block = CommandBlock {
                    id: self.next_block_id,
                    prompt_row: self.cursor.row,
                    command_row: self.cursor.row, // will be updated on B
                    output_start_row: None,
                    output_end_row: None,
                    exit_code: None,
                    cwd,
                };
                self.next_block_id += 1;
                self.blocks.push(block);
                self.block_state = BlockState::PromptShown;
            }
            // B = Command start (user typing after prompt)
            Some('B') => {
                if let Some(last) = self.blocks.last_mut() {
                    last.command_row = self.cursor.row;
                }
                self.block_state = BlockState::CommandInput;
            }
            // C = Command executed (output begins)
            Some('C') => {
                if let Some(last) = self.blocks.last_mut() {
                    last.output_start_row = Some(self.cursor.row);
                }
                self.block_state = BlockState::Executing;
            }
            // D = Command finished (exit code follows)
            Some('D') => {
                let exit_code = sub
                    .get(1..)
                    .and_then(|s| s.trim_start_matches(';').parse::<i32>().ok());

                if let Some(last) = self.blocks.last_mut() {
                    last.output_end_row = Some(self.cursor.row);
                    last.exit_code = exit_code;
                }
                self.block_state = BlockState::Idle;
            }
            _ => {}
        }
    }
}

/// Parse a file:// URI into a filesystem path.
/// Handles: `file://hostname/path` and `file:///path`
fn parse_file_uri(uri: &str) -> Option<String> {
    let uri = uri.trim();
    if let Some(rest) = uri.strip_prefix("file://") {
        // Skip hostname (everything before the first / after file://)
        if let Some(slash_pos) = rest.find('/') {
            let path = &rest[slash_pos..];
            // URL-decode percent-encoded characters
            Some(url_decode(path))
        } else {
            None
        }
    } else {
        // Bare path
        Some(uri.to_string())
    }
}

/// Simple URL percent-decoding.
fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let hi = chars.next().and_then(hex_val);
            let lo = chars.next().and_then(hex_val);
            if let (Some(h), Some(l)) = (hi, lo) {
                result.push((h << 4 | l) as char);
            }
        } else {
            result.push(b as char);
        }
    }
    result
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::ColorData;

    #[test]
    fn test_print_basic() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"Hello");
        assert_eq!(emu.grid.cell(0, 0).content.as_str(), "H");
        assert_eq!(emu.grid.cell(0, 4).content.as_str(), "o");
        assert_eq!(emu.cursor.col, 5);
    }

    #[test]
    fn test_newline() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"A\r\nB");
        assert_eq!(emu.grid.cell(0, 0).content.as_str(), "A");
        assert_eq!(emu.grid.cell(1, 0).content.as_str(), "B");
    }

    #[test]
    fn test_cursor_movement() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"\x1b[5;10H"); // Move to row 5, col 10
        assert_eq!(emu.cursor.row, 4); // 0-indexed
        assert_eq!(emu.cursor.col, 9);
    }

    #[test]
    fn test_sgr_bold() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"\x1b[1mA");
        assert!(emu.grid.cell(0, 0).attrs.flags.contains(AttrFlags::BOLD));
    }

    #[test]
    fn test_sgr_fg_color() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"\x1b[31mA"); // Red foreground
        assert_eq!(
            emu.grid.cell(0, 0).attrs.fg,
            Color::Named(NamedColor::Red)
        );
    }

    #[test]
    fn test_sgr_256_color() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"\x1b[38;5;196mA"); // 256-color foreground
        assert_eq!(emu.grid.cell(0, 0).attrs.fg, Color::Indexed(196));
    }

    #[test]
    fn test_sgr_rgb_color() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"\x1b[38;2;255;128;0mA"); // RGB foreground
        assert_eq!(emu.grid.cell(0, 0).attrs.fg, Color::Rgb(255, 128, 0));
    }

    #[test]
    fn test_sgr_reset() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"\x1b[1;31mA\x1b[0mB");
        assert!(emu.grid.cell(0, 0).attrs.flags.contains(AttrFlags::BOLD));
        assert!(!emu.grid.cell(0, 1).attrs.flags.contains(AttrFlags::BOLD));
        assert_eq!(emu.grid.cell(0, 1).attrs.fg, Color::Default);
    }

    #[test]
    fn test_erase_display() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"Hello World");
        emu.process(b"\x1b[2J"); // Erase all
        assert_eq!(emu.grid.cell(0, 0).content.as_str(), " ");
    }

    #[test]
    fn test_erase_line() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"Hello");
        emu.process(b"\x1b[3G"); // Move to col 3
        emu.process(b"\x1b[K"); // Erase to end of line
        assert_eq!(emu.grid.cell(0, 0).content.as_str(), "H");
        assert_eq!(emu.grid.cell(0, 1).content.as_str(), "e");
        assert_eq!(emu.grid.cell(0, 2).content.as_str(), " ");
    }

    #[test]
    fn test_scroll_at_bottom() {
        let mut emu = Emulator::new(80, 3);
        emu.process(b"A\r\nB\r\nC\r\nD");
        // A should be in scrollback, B on row 0, C on row 1, D on row 2
        assert_eq!(emu.grid.cell(0, 0).content.as_str(), "B");
        assert_eq!(emu.grid.cell(1, 0).content.as_str(), "C");
        assert_eq!(emu.grid.cell(2, 0).content.as_str(), "D");
    }

    #[test]
    fn test_auto_wrap() {
        let mut emu = Emulator::new(5, 3);
        emu.process(b"ABCDE"); // Fill row
        assert_eq!(emu.cursor.col, 4); // At last column
        assert!(emu.cursor.pending_wrap);
        emu.process(b"F"); // Should wrap
        assert_eq!(emu.cursor.row, 1);
        assert_eq!(emu.cursor.col, 1);
        assert_eq!(emu.grid.cell(1, 0).content.as_str(), "F");
    }

    #[test]
    fn test_backspace() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"AB\x08C");
        assert_eq!(emu.grid.cell(0, 0).content.as_str(), "A");
        assert_eq!(emu.grid.cell(0, 1).content.as_str(), "C");
    }

    #[test]
    fn test_tab() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"A\tB");
        assert_eq!(emu.grid.cell(0, 0).content.as_str(), "A");
        assert_eq!(emu.cursor.col, 9); // 'B' at col 8, cursor at 9
    }

    #[test]
    fn test_osc_title() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"\x1b]0;My Title\x07");
        assert_eq!(emu.title.as_deref(), Some("My Title"));
    }

    #[test]
    fn test_cursor_save_restore() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"\x1b[5;10H"); // Move to row 5, col 10
        emu.process(b"\x1b7"); // Save cursor
        emu.process(b"\x1b[1;1H"); // Move to origin
        emu.process(b"\x1b8"); // Restore cursor
        assert_eq!(emu.cursor.row, 4);
        assert_eq!(emu.cursor.col, 9);
    }

    #[test]
    fn test_snapshot() {
        let mut emu = Emulator::new(3, 2);
        emu.process(b"\x1b[31mAB");
        let snap = emu.snapshot();
        assert_eq!(snap.rows.len(), 2);
        assert_eq!(snap.rows[0].len(), 3);
        assert_eq!(snap.rows[0][0].content, "A");
        assert!(matches!(snap.rows[0][0].fg, ColorData::Named { .. }));
    }

    #[test]
    fn test_dec_cursor_visibility() {
        let mut emu = Emulator::new(80, 24);
        emu.process(b"\x1b[?25l"); // Hide cursor
        assert!(!emu.modes.cursor_visible);
        emu.process(b"\x1b[?25h"); // Show cursor
        assert!(emu.modes.cursor_visible);
    }
}
