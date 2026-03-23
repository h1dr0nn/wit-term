//! ANSI parser state machine.
//!
//! Implements the Paul Williams VT parser model with 14 states.
//! Converts raw byte streams into structured terminal actions.

mod params;
mod state_machine;

pub use params::Params;
pub use state_machine::Parser;

/// Parser states following the VT500 state machine model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParserState {
    #[default]
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

/// Actions produced by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Print a visible character to the terminal.
    Print(char),
    /// Execute a C0 control code.
    Execute(u8),
    /// Dispatch a CSI sequence.
    CsiDispatch {
        params: Vec<u16>,
        intermediates: Vec<u8>,
        final_byte: u8,
        private_marker: Option<u8>,
    },
    /// Dispatch an ESC sequence.
    EscDispatch {
        intermediates: Vec<u8>,
        final_byte: u8,
    },
    /// Dispatch an OSC sequence.
    OscDispatch(Vec<Vec<u8>>),
}
