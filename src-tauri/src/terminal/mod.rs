//! Terminal emulation engine.
//!
//! Manages the terminal grid (cells, cursor, scrollback),
//! processes parsed ANSI actions, and tracks terminal modes.

mod emulator;
mod grid;

pub use emulator::Emulator;
pub use grid::{Cell, CellAttrs, Color, Grid, NamedColor};

use bitflags::bitflags;
use serde::Serialize;

bitflags! {
    /// Text attribute flags.
    #[derive(Debug, Clone, Copy, Default, PartialEq)]
    pub struct AttrFlags: u16 {
        const BOLD          = 0b0000_0001;
        const DIM           = 0b0000_0010;
        const ITALIC        = 0b0000_0100;
        const UNDERLINE     = 0b0000_1000;
        const BLINK         = 0b0001_0000;
        const INVERSE       = 0b0010_0000;
        const HIDDEN        = 0b0100_0000;
        const STRIKETHROUGH = 0b1000_0000;
    }
}

/// Cursor state.
#[derive(Debug, Clone)]
pub struct Cursor {
    pub col: usize,
    pub row: usize,
    pub visible: bool,
    pub shape: CursorShape,
    pub attrs: CellAttrs,
    pub pending_wrap: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            col: 0,
            row: 0,
            visible: true,
            shape: CursorShape::Block,
            attrs: CellAttrs::default(),
            pending_wrap: false,
        }
    }
}

/// Cursor visual shape.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum CursorShape {
    Block,
    Underline,
    Bar,
}

/// Terminal modes.
#[derive(Debug, Clone)]
pub struct TerminalModes {
    pub auto_wrap: bool,
    pub cursor_visible: bool,
    pub insert_mode: bool,
    pub linefeed_mode: bool,
    pub origin_mode: bool,
    pub alt_screen: bool,
    pub bracketed_paste: bool,
}

impl Default for TerminalModes {
    fn default() -> Self {
        Self {
            auto_wrap: true,
            cursor_visible: true,
            insert_mode: false,
            linefeed_mode: false,
            origin_mode: false,
            alt_screen: false,
            bracketed_paste: false,
        }
    }
}

/// Serializable grid snapshot for IPC.
#[derive(Debug, Clone, Serialize)]
pub struct GridSnapshot {
    pub rows: Vec<Vec<CellData>>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub cursor_visible: bool,
    pub cursor_shape: CursorShape,
    pub blocks: Vec<BlockInfo>,
    /// Number of scrollback rows at the beginning of `rows`.
    /// Total rows = scrollback_len + visible rows.
    pub scrollback_len: usize,
}

/// A command block — one prompt + command + output cycle.
#[derive(Debug, Clone, Serialize)]
pub struct BlockInfo {
    pub id: u32,
    pub prompt_row: usize,
    pub output_start_row: Option<usize>,
    pub output_end_row: Option<usize>,
    pub exit_code: Option<i32>,
    pub cwd: String,
    pub command: String,
}

/// Internal block tracking state.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockState {
    /// Waiting for next prompt.
    Idle,
    /// Prompt is being displayed (OSC 133;A received).
    PromptShown,
    /// User is typing a command (OSC 133;B received).
    CommandInput,
    /// Command is executing, output streaming (OSC 133;C received).
    Executing,
}

/// Internal command block with mutable state.
#[derive(Debug, Clone)]
pub struct CommandBlock {
    pub id: u32,
    pub prompt_row: usize,
    pub command_row: usize,
    pub output_start_row: Option<usize>,
    pub output_end_row: Option<usize>,
    pub exit_code: Option<i32>,
    pub cwd: String,
}

/// Serializable cell data for IPC.
#[derive(Debug, Clone, Serialize)]
pub struct CellData {
    pub content: String,
    pub fg: ColorData,
    pub bg: ColorData,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub inverse: bool,
}

/// Serializable color for IPC.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ColorData {
    Default,
    Named { name: String },
    Indexed { index: u8 },
    Rgb { r: u8, g: u8, b: u8 },
}

impl From<&Color> for ColorData {
    fn from(color: &Color) -> Self {
        match color {
            Color::Default => ColorData::Default,
            Color::Named(n) => ColorData::Named {
                name: format!("{n:?}"),
            },
            Color::Indexed(i) => ColorData::Indexed { index: *i },
            Color::Rgb(r, g, b) => ColorData::Rgb {
                r: *r,
                g: *g,
                b: *b,
            },
        }
    }
}

impl From<&Cell> for CellData {
    fn from(cell: &Cell) -> Self {
        CellData {
            content: cell.content.to_string(),
            fg: ColorData::from(&cell.attrs.fg),
            bg: ColorData::from(&cell.attrs.bg),
            bold: cell.attrs.flags.contains(AttrFlags::BOLD),
            dim: cell.attrs.flags.contains(AttrFlags::DIM),
            italic: cell.attrs.flags.contains(AttrFlags::ITALIC),
            underline: cell.attrs.flags.contains(AttrFlags::UNDERLINE),
            strikethrough: cell.attrs.flags.contains(AttrFlags::STRIKETHROUGH),
            inverse: cell.attrs.flags.contains(AttrFlags::INVERSE),
        }
    }
}
