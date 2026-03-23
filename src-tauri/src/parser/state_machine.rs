//! ANSI parser state machine implementation.
//!
//! Based on the Paul Williams VT parser model.

use super::{Action, ParserState};

/// The ANSI parser processes a byte stream and produces Actions.
pub struct Parser {
    state: ParserState,
    params: Vec<u16>,
    current_param: u16,
    intermediates: Vec<u8>,
    osc_data: Vec<u8>,
    private_marker: Option<u8>,
    has_param: bool,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: ParserState::Ground,
            params: Vec::with_capacity(16),
            current_param: 0,
            intermediates: Vec::with_capacity(2),
            osc_data: Vec::with_capacity(256),
            private_marker: None,
            has_param: false,
        }
    }

    /// Process a chunk of bytes and return all actions produced.
    pub fn process(&mut self, data: &[u8]) -> Vec<Action> {
        let mut actions = Vec::new();
        for &byte in data {
            self.advance(byte, &mut actions);
        }
        actions
    }

    /// Process a single byte through the state machine.
    fn advance(&mut self, byte: u8, actions: &mut Vec<Action>) {
        // Anywhere transitions: these override state-specific handling
        match byte {
            // CAN and SUB cancel the current sequence
            0x18 | 0x1A => {
                self.state = ParserState::Ground;
                return;
            }
            // ESC starts a new escape sequence from any state
            0x1B => {
                self.clear();
                self.state = ParserState::Escape;
                return;
            }
            _ => {}
        }

        match self.state {
            ParserState::Ground => self.state_ground(byte, actions),
            ParserState::Escape => self.state_escape(byte, actions),
            ParserState::EscapeIntermediate => self.state_escape_intermediate(byte, actions),
            ParserState::CsiEntry => self.state_csi_entry(byte, actions),
            ParserState::CsiParam => self.state_csi_param(byte, actions),
            ParserState::CsiIntermediate => self.state_csi_intermediate(byte, actions),
            ParserState::CsiIgnore => self.state_csi_ignore(byte),
            ParserState::OscString => self.state_osc_string(byte, actions),
            ParserState::DcsEntry => self.state_dcs_entry(byte),
            ParserState::DcsParam => self.state_dcs_param(byte),
            ParserState::DcsIntermediate => self.state_dcs_intermediate(byte),
            ParserState::DcsPassthrough => self.state_dcs_passthrough(byte),
            ParserState::DcsIgnore => self.state_dcs_ignore(byte),
            ParserState::SosPmApc => self.state_sos_pm_apc(byte),
        }
    }

    /// Clear parser state for a new sequence.
    fn clear(&mut self) {
        self.params.clear();
        self.current_param = 0;
        self.intermediates.clear();
        self.osc_data.clear();
        self.private_marker = None;
        self.has_param = false;
    }

    /// Finalize the current parameter being built.
    fn finish_param(&mut self) {
        if self.has_param {
            self.params.push(self.current_param);
            self.current_param = 0;
            self.has_param = false;
        }
    }

    // ── State handlers ──

    fn state_ground(&mut self, byte: u8, actions: &mut Vec<Action>) {
        match byte {
            // C0 controls
            0x00..=0x1F => {
                // Execute C0 control codes (except the ones handled as anywhere transitions)
                actions.push(Action::Execute(byte));
            }
            // Printable ASCII
            0x20..=0x7E => {
                actions.push(Action::Print(byte as char));
            }
            // DEL - ignored
            0x7F => {}
            // UTF-8 multibyte or Latin-1 supplement
            0x80..=0xFF => {
                // For now, handle as single-byte print (will be extended with full UTF-8)
                if let Some(ch) = char::from_u32(byte as u32) {
                    actions.push(Action::Print(ch));
                }
            }
        }
    }

    fn state_escape(&mut self, byte: u8, actions: &mut Vec<Action>) {
        match byte {
            // C0 controls in escape - execute them
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                actions.push(Action::Execute(byte));
            }
            // Intermediate bytes
            0x20..=0x2F => {
                self.intermediates.push(byte);
                self.state = ParserState::EscapeIntermediate;
            }
            // ESC [ -> CSI
            0x5B => {
                self.state = ParserState::CsiEntry;
            }
            // ESC ] -> OSC
            0x5D => {
                self.osc_data.clear();
                self.state = ParserState::OscString;
            }
            // ESC P -> DCS
            0x50 => {
                self.state = ParserState::DcsEntry;
            }
            // ESC X (SOS), ESC ^ (PM), ESC _ (APC)
            0x58 | 0x5E | 0x5F => {
                self.state = ParserState::SosPmApc;
            }
            // Final bytes -> dispatch ESC sequence
            0x30..=0x4F | 0x51..=0x57 | 0x59..=0x5A | 0x5C | 0x60..=0x7E => {
                actions.push(Action::EscDispatch {
                    intermediates: self.intermediates.clone(),
                    final_byte: byte,
                });
                self.state = ParserState::Ground;
            }
            // DEL - ignored
            0x7F => {}
            _ => {
                self.state = ParserState::Ground;
            }
        }
    }

    fn state_escape_intermediate(&mut self, byte: u8, actions: &mut Vec<Action>) {
        match byte {
            // C0 controls
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                actions.push(Action::Execute(byte));
            }
            // More intermediate bytes
            0x20..=0x2F => {
                self.intermediates.push(byte);
            }
            // Final byte
            0x30..=0x7E => {
                actions.push(Action::EscDispatch {
                    intermediates: self.intermediates.clone(),
                    final_byte: byte,
                });
                self.state = ParserState::Ground;
            }
            // DEL - ignored
            0x7F => {}
            _ => {
                self.state = ParserState::Ground;
            }
        }
    }

    fn state_csi_entry(&mut self, byte: u8, actions: &mut Vec<Action>) {
        match byte {
            // C0 controls
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                actions.push(Action::Execute(byte));
            }
            // Private marker (?, >, <, =)
            0x3C..=0x3F => {
                self.private_marker = Some(byte);
                self.state = ParserState::CsiParam;
            }
            // Parameter digit
            0x30..=0x39 => {
                self.current_param = (byte - b'0') as u16;
                self.has_param = true;
                self.state = ParserState::CsiParam;
            }
            // Semicolon (parameter separator) with no preceding param
            0x3B => {
                self.params.push(0);
                self.state = ParserState::CsiParam;
            }
            // Colon (sub-parameter separator)
            0x3A => {
                self.state = ParserState::CsiIgnore;
            }
            // Intermediate bytes
            0x20..=0x2F => {
                self.intermediates.push(byte);
                self.state = ParserState::CsiIntermediate;
            }
            // Final byte -> dispatch with no params
            0x40..=0x7E => {
                actions.push(Action::CsiDispatch {
                    params: self.params.clone(),
                    intermediates: self.intermediates.clone(),
                    final_byte: byte,
                    private_marker: self.private_marker,
                });
                self.state = ParserState::Ground;
            }
            // DEL - ignored
            0x7F => {}
            _ => {
                self.state = ParserState::Ground;
            }
        }
    }

    fn state_csi_param(&mut self, byte: u8, actions: &mut Vec<Action>) {
        match byte {
            // C0 controls
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                actions.push(Action::Execute(byte));
            }
            // Parameter digit
            0x30..=0x39 => {
                self.current_param = self.current_param.saturating_mul(10).saturating_add((byte - b'0') as u16);
                self.has_param = true;
            }
            // Semicolon - parameter separator
            0x3B => {
                // Push current param (0 if empty) and reset
                self.params.push(self.current_param);
                self.current_param = 0;
                self.has_param = false;
            }
            // Colon - sub-parameter separator (treat like semicolon for now)
            0x3A => {
                self.params.push(self.current_param);
                self.current_param = 0;
                self.has_param = false;
            }
            // Private markers in wrong position
            0x3C..=0x3F => {
                self.state = ParserState::CsiIgnore;
            }
            // Intermediate bytes
            0x20..=0x2F => {
                self.finish_param();
                self.intermediates.push(byte);
                self.state = ParserState::CsiIntermediate;
            }
            // Final byte -> dispatch
            0x40..=0x7E => {
                self.finish_param();
                actions.push(Action::CsiDispatch {
                    params: self.params.clone(),
                    intermediates: self.intermediates.clone(),
                    final_byte: byte,
                    private_marker: self.private_marker,
                });
                self.state = ParserState::Ground;
            }
            // DEL - ignored
            0x7F => {}
            _ => {
                self.state = ParserState::Ground;
            }
        }
    }

    fn state_csi_intermediate(&mut self, byte: u8, actions: &mut Vec<Action>) {
        match byte {
            // C0 controls
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                actions.push(Action::Execute(byte));
            }
            // More intermediate bytes
            0x20..=0x2F => {
                self.intermediates.push(byte);
            }
            // Parameter bytes in wrong position
            0x30..=0x3F => {
                self.state = ParserState::CsiIgnore;
            }
            // Final byte -> dispatch
            0x40..=0x7E => {
                actions.push(Action::CsiDispatch {
                    params: self.params.clone(),
                    intermediates: self.intermediates.clone(),
                    final_byte: byte,
                    private_marker: self.private_marker,
                });
                self.state = ParserState::Ground;
            }
            // DEL - ignored
            0x7F => {}
            _ => {
                self.state = ParserState::Ground;
            }
        }
    }

    fn state_csi_ignore(&mut self, byte: u8) {
        if (0x40..=0x7E).contains(&byte) {
            self.state = ParserState::Ground;
        }
    }

    fn state_osc_string(&mut self, byte: u8, actions: &mut Vec<Action>) {
        match byte {
            // BEL terminates OSC (xterm-style)
            0x07 => {
                self.dispatch_osc(actions);
                self.state = ParserState::Ground;
            }
            // ST (via ESC \) is handled by the ESC anywhere transition + checking for \
            // C0 controls except BEL are ignored in OSC
            0x00..=0x06 | 0x08..=0x1F => {}
            // Regular bytes collected
            _ => {
                self.osc_data.push(byte);
            }
        }
    }

    fn dispatch_osc(&mut self, actions: &mut Vec<Action>) {
        // Split OSC data on ';'
        let parts: Vec<Vec<u8>> = self
            .osc_data
            .split(|&b| b == b';')
            .map(|s| s.to_vec())
            .collect();
        actions.push(Action::OscDispatch(parts));
    }

    // DCS states - minimal handling for now

    fn state_dcs_entry(&mut self, byte: u8) {
        match byte {
            0x30..=0x39 | 0x3B => self.state = ParserState::DcsParam,
            0x3C..=0x3F => self.state = ParserState::DcsParam,
            0x20..=0x2F => self.state = ParserState::DcsIntermediate,
            0x40..=0x7E => self.state = ParserState::DcsPassthrough,
            0x3A => self.state = ParserState::DcsIgnore,
            _ => {}
        }
    }

    fn state_dcs_param(&mut self, byte: u8) {
        match byte {
            0x30..=0x3B => {} // collect params
            0x20..=0x2F => self.state = ParserState::DcsIntermediate,
            0x40..=0x7E => self.state = ParserState::DcsPassthrough,
            0x3C..=0x3F => self.state = ParserState::DcsIgnore,
            _ => {}
        }
    }

    fn state_dcs_intermediate(&mut self, byte: u8) {
        match byte {
            0x20..=0x2F => {} // collect
            0x40..=0x7E => self.state = ParserState::DcsPassthrough,
            0x30..=0x3F => self.state = ParserState::DcsIgnore,
            _ => {}
        }
    }

    fn state_dcs_passthrough(&mut self, byte: u8) {
        if byte == 0x9C {
            self.state = ParserState::Ground;
        }
    }

    fn state_dcs_ignore(&mut self, byte: u8) {
        if byte == 0x9C {
            self.state = ParserState::Ground;
        }
    }

    fn state_sos_pm_apc(&mut self, byte: u8) {
        if byte == 0x9C {
            self.state = ParserState::Ground;
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_ascii() {
        let mut parser = Parser::new();
        let actions = parser.process(b"hello");
        assert_eq!(
            actions,
            vec![
                Action::Print('h'),
                Action::Print('e'),
                Action::Print('l'),
                Action::Print('l'),
                Action::Print('o'),
            ]
        );
    }

    #[test]
    fn test_c0_controls() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\r\n");
        assert_eq!(
            actions,
            vec![Action::Execute(0x0D), Action::Execute(0x0A)]
        );
    }

    #[test]
    fn test_bell() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x07");
        assert_eq!(actions, vec![Action::Execute(0x07)]);
    }

    #[test]
    fn test_backspace() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x08");
        assert_eq!(actions, vec![Action::Execute(0x08)]);
    }

    #[test]
    fn test_tab() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\t");
        assert_eq!(actions, vec![Action::Execute(0x09)]);
    }

    #[test]
    fn test_csi_cursor_up() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x1b[5A");
        assert_eq!(
            actions,
            vec![Action::CsiDispatch {
                params: vec![5],
                intermediates: vec![],
                final_byte: b'A',
                private_marker: None,
            }]
        );
    }

    #[test]
    fn test_csi_cursor_position() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x1b[10;20H");
        assert_eq!(
            actions,
            vec![Action::CsiDispatch {
                params: vec![10, 20],
                intermediates: vec![],
                final_byte: b'H',
                private_marker: None,
            }]
        );
    }

    #[test]
    fn test_csi_no_params() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x1b[H");
        assert_eq!(
            actions,
            vec![Action::CsiDispatch {
                params: vec![],
                intermediates: vec![],
                final_byte: b'H',
                private_marker: None,
            }]
        );
    }

    #[test]
    fn test_csi_sgr_bold() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x1b[1m");
        assert_eq!(
            actions,
            vec![Action::CsiDispatch {
                params: vec![1],
                intermediates: vec![],
                final_byte: b'm',
                private_marker: None,
            }]
        );
    }

    #[test]
    fn test_csi_sgr_multiple() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x1b[1;31;42m");
        assert_eq!(
            actions,
            vec![Action::CsiDispatch {
                params: vec![1, 31, 42],
                intermediates: vec![],
                final_byte: b'm',
                private_marker: None,
            }]
        );
    }

    #[test]
    fn test_csi_256_color() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x1b[38;5;196m");
        assert_eq!(
            actions,
            vec![Action::CsiDispatch {
                params: vec![38, 5, 196],
                intermediates: vec![],
                final_byte: b'm',
                private_marker: None,
            }]
        );
    }

    #[test]
    fn test_csi_rgb_color() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x1b[38;2;255;128;0m");
        assert_eq!(
            actions,
            vec![Action::CsiDispatch {
                params: vec![38, 2, 255, 128, 0],
                intermediates: vec![],
                final_byte: b'm',
                private_marker: None,
            }]
        );
    }

    #[test]
    fn test_csi_private_mode() {
        let mut parser = Parser::new();
        // DECSET: show cursor
        let actions = parser.process(b"\x1b[?25h");
        assert_eq!(
            actions,
            vec![Action::CsiDispatch {
                params: vec![25],
                intermediates: vec![],
                final_byte: b'h',
                private_marker: Some(b'?'),
            }]
        );
    }

    #[test]
    fn test_csi_erase_display() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x1b[2J");
        assert_eq!(
            actions,
            vec![Action::CsiDispatch {
                params: vec![2],
                intermediates: vec![],
                final_byte: b'J',
                private_marker: None,
            }]
        );
    }

    #[test]
    fn test_esc_sequence() {
        let mut parser = Parser::new();
        // ESC D = Index (scroll up)
        let actions = parser.process(b"\x1bD");
        assert_eq!(
            actions,
            vec![Action::EscDispatch {
                intermediates: vec![],
                final_byte: b'D',
            }]
        );
    }

    #[test]
    fn test_esc_with_intermediate() {
        let mut parser = Parser::new();
        // ESC ( 0 = DEC Special Graphics charset
        let actions = parser.process(b"\x1b(0");
        assert_eq!(
            actions,
            vec![Action::EscDispatch {
                intermediates: vec![b'('],
                final_byte: b'0',
            }]
        );
    }

    #[test]
    fn test_osc_title() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x1b]0;My Title\x07");
        assert_eq!(
            actions,
            vec![Action::OscDispatch(vec![
                b"0".to_vec(),
                b"My Title".to_vec(),
            ])]
        );
    }

    #[test]
    fn test_mixed_content() {
        let mut parser = Parser::new();
        let actions = parser.process(b"A\x1b[1mB\x1b[0mC");
        assert_eq!(
            actions,
            vec![
                Action::Print('A'),
                Action::CsiDispatch {
                    params: vec![1],
                    intermediates: vec![],
                    final_byte: b'm',
                    private_marker: None,
                },
                Action::Print('B'),
                Action::CsiDispatch {
                    params: vec![0],
                    intermediates: vec![],
                    final_byte: b'm',
                    private_marker: None,
                },
                Action::Print('C'),
            ]
        );
    }

    #[test]
    fn test_cancel_sequence() {
        let mut parser = Parser::new();
        // Start a CSI but cancel with CAN
        let actions = parser.process(b"\x1b[5\x18hello");
        // CAN cancels the sequence, then "hello" prints normally
        assert_eq!(
            actions,
            vec![
                Action::Print('h'),
                Action::Print('e'),
                Action::Print('l'),
                Action::Print('l'),
                Action::Print('o'),
            ]
        );
    }

    #[test]
    fn test_partial_sequence() {
        let mut parser = Parser::new();

        // Feed the first part
        let actions1 = parser.process(b"\x1b[31");
        assert!(actions1.is_empty()); // No actions yet, still parsing

        // Feed the rest
        let actions2 = parser.process(b"m");
        assert_eq!(
            actions2,
            vec![Action::CsiDispatch {
                params: vec![31],
                intermediates: vec![],
                final_byte: b'm',
                private_marker: None,
            }]
        );
    }

    #[test]
    fn test_reset_sgr() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x1b[m");
        assert_eq!(
            actions,
            vec![Action::CsiDispatch {
                params: vec![],
                intermediates: vec![],
                final_byte: b'm',
                private_marker: None,
            }]
        );
    }

    #[test]
    fn test_del_ignored() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x7F");
        assert!(actions.is_empty());
    }

    #[test]
    fn test_scroll_up() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x1b[3S");
        assert_eq!(
            actions,
            vec![Action::CsiDispatch {
                params: vec![3],
                intermediates: vec![],
                final_byte: b'S',
                private_marker: None,
            }]
        );
    }

    #[test]
    fn test_insert_lines() {
        let mut parser = Parser::new();
        let actions = parser.process(b"\x1b[2L");
        assert_eq!(
            actions,
            vec![Action::CsiDispatch {
                params: vec![2],
                intermediates: vec![],
                final_byte: b'L',
                private_marker: None,
            }]
        );
    }
}
