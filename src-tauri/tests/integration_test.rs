//! Integration tests for the terminal emulator.
//!
//! These tests simulate realistic terminal usage by feeding raw ANSI output
//! through the emulator and verifying grid state.

use wit_lib::terminal::Emulator;

/// Helper to extract visible text from a row.
fn row_text(emu: &Emulator, row: usize) -> String {
    let grid_row = emu.grid.row(row);
    grid_row.iter().map(|c| c.content.as_str()).collect::<String>()
}

/// Helper to extract visible text from a row, trimmed.
fn row_text_trimmed(emu: &Emulator, row: usize) -> String {
    row_text(emu, row).trim_end().to_string()
}

#[test]
fn test_basic_prompt_and_echo() {
    let mut emu = Emulator::new(40, 10);
    // Simulate: prompt "$ " then user types "echo hello" then output "hello"
    emu.process(b"$ echo hello\r\nhello\r\n$ ");

    assert_eq!(row_text_trimmed(&emu, 0), "$ echo hello");
    assert_eq!(row_text_trimmed(&emu, 1), "hello");
    assert_eq!(row_text_trimmed(&emu, 2), "$");
}

#[test]
fn test_colored_output() {
    let mut emu = Emulator::new(40, 5);
    // Red text "error" followed by reset and normal text
    emu.process(b"\x1b[31merror\x1b[0m: something failed");

    // Verify the text is correct
    assert_eq!(row_text_trimmed(&emu, 0), "error: something failed");

    // Verify "error" has red foreground
    let cell = emu.grid.cell(0, 0);
    assert_eq!(
        cell.attrs.fg,
        wit_lib::terminal::Color::Named(wit_lib::terminal::NamedColor::Red)
    );

    // Verify ": something failed" has default color
    let cell_after = emu.grid.cell(0, 5);
    assert_eq!(cell_after.attrs.fg, wit_lib::terminal::Color::Default);
}

#[test]
fn test_cursor_positioning() {
    let mut emu = Emulator::new(20, 5);
    // Write "ABCDE", move cursor to col 3, overwrite with "X"
    emu.process(b"ABCDE\x1b[1;3HX");

    assert_eq!(row_text_trimmed(&emu, 0), "ABXDE");
}

#[test]
fn test_erase_in_line() {
    let mut emu = Emulator::new(20, 5);
    emu.process(b"Hello World");
    // Move to col 6, erase to end of line
    emu.process(b"\x1b[1;6H\x1b[K");

    assert_eq!(row_text_trimmed(&emu, 0), "Hello");
}

#[test]
fn test_erase_in_display() {
    let mut emu = Emulator::new(20, 5);
    emu.process(b"Line 1\r\nLine 2\r\nLine 3");
    // Clear entire display
    emu.process(b"\x1b[2J");

    assert_eq!(row_text_trimmed(&emu, 0), "");
    assert_eq!(row_text_trimmed(&emu, 1), "");
    assert_eq!(row_text_trimmed(&emu, 2), "");
}

#[test]
fn test_scroll_region() {
    let mut emu = Emulator::new(20, 5);
    // Set scroll region to rows 2-4
    emu.process(b"\x1b[2;4r");
    // Move cursor to bottom of scroll region and write lines
    emu.process(b"\x1b[2;1HA\r\nB\r\nC\r\nD");

    // The scroll region should have scrolled, row 1 should be untouched
}

#[test]
fn test_wrap_long_line() {
    let mut emu = Emulator::new(10, 3);
    emu.process(b"1234567890ABCDE");

    assert_eq!(row_text_trimmed(&emu, 0), "1234567890");
    assert_eq!(row_text_trimmed(&emu, 1), "ABCDE");
}

#[test]
fn test_backspace() {
    let mut emu = Emulator::new(20, 5);
    emu.process(b"Hello\x08\x08\x08lo");

    assert_eq!(row_text_trimmed(&emu, 0), "Heloo");
}

#[test]
fn test_tab_stops() {
    let mut emu = Emulator::new(40, 5);
    emu.process(b"A\tB\tC");

    // Tab stops every 8 columns
    assert_eq!(emu.grid.cell(0, 0).content.as_str(), "A");
    assert_eq!(emu.grid.cell(0, 8).content.as_str(), "B");
    assert_eq!(emu.grid.cell(0, 16).content.as_str(), "C");
}

#[test]
fn test_insert_delete_chars() {
    let mut emu = Emulator::new(20, 5);
    emu.process(b"ABCDE");
    // Move to B, insert 2 chars
    emu.process(b"\x1b[1;2H\x1b[2@");

    assert_eq!(row_text_trimmed(&emu, 0), "A  BCDE");

    // Delete 1 char at cursor
    emu.process(b"\x1b[P");
    assert_eq!(row_text_trimmed(&emu, 0), "A BCDE");
}

#[test]
fn test_insert_delete_lines() {
    let mut emu = Emulator::new(20, 5);
    emu.process(b"Line1\r\nLine2\r\nLine3");
    // Move to line 2, insert a blank line
    emu.process(b"\x1b[2;1H\x1b[L");

    assert_eq!(row_text_trimmed(&emu, 0), "Line1");
    assert_eq!(row_text_trimmed(&emu, 1), "");
    assert_eq!(row_text_trimmed(&emu, 2), "Line2");
    assert_eq!(row_text_trimmed(&emu, 3), "Line3");
}

#[test]
fn test_256_colors() {
    let mut emu = Emulator::new(20, 5);
    // Foreground: 256-color index 196 (red), Background: 256-color index 21 (blue)
    emu.process(b"\x1b[38;5;196;48;5;21mX\x1b[0m");

    let cell = emu.grid.cell(0, 0);
    assert_eq!(cell.attrs.fg, wit_lib::terminal::Color::Indexed(196));
    assert_eq!(cell.attrs.bg, wit_lib::terminal::Color::Indexed(21));
}

#[test]
fn test_rgb_colors() {
    let mut emu = Emulator::new(20, 5);
    emu.process(b"\x1b[38;2;255;128;0mX\x1b[0m");

    let cell = emu.grid.cell(0, 0);
    assert_eq!(cell.attrs.fg, wit_lib::terminal::Color::Rgb(255, 128, 0));
}

#[test]
fn test_multiple_sgr_attributes() {
    let mut emu = Emulator::new(20, 5);
    // Bold + Italic + Red foreground
    emu.process(b"\x1b[1;3;31mX\x1b[0m");

    let cell = emu.grid.cell(0, 0);
    assert!(cell
        .attrs
        .flags
        .contains(wit_lib::terminal::AttrFlags::BOLD));
    assert!(cell
        .attrs
        .flags
        .contains(wit_lib::terminal::AttrFlags::ITALIC));
    assert_eq!(
        cell.attrs.fg,
        wit_lib::terminal::Color::Named(wit_lib::terminal::NamedColor::Red)
    );
}

#[test]
fn test_osc_window_title() {
    let mut emu = Emulator::new(80, 24);
    emu.process(b"\x1b]0;My Terminal Title\x07");
    assert_eq!(emu.title.as_deref(), Some("My Terminal Title"));

    emu.process(b"\x1b]2;New Title\x07");
    assert_eq!(emu.title.as_deref(), Some("New Title"));
}

#[test]
fn test_cursor_save_restore_esc() {
    let mut emu = Emulator::new(80, 24);
    emu.process(b"\x1b[10;20H"); // Move to 10,20
    emu.process(b"\x1b7"); // Save
    emu.process(b"\x1b[1;1H"); // Move to origin
    assert_eq!(emu.cursor.row, 0);
    assert_eq!(emu.cursor.col, 0);
    emu.process(b"\x1b8"); // Restore
    assert_eq!(emu.cursor.row, 9);
    assert_eq!(emu.cursor.col, 19);
}

#[test]
fn test_cursor_save_restore_csi() {
    let mut emu = Emulator::new(80, 24);
    emu.process(b"\x1b[10;20H"); // Move to 10,20
    emu.process(b"\x1b[s"); // Save
    emu.process(b"\x1b[1;1H"); // Move to origin
    emu.process(b"\x1b[u"); // Restore
    assert_eq!(emu.cursor.row, 9);
    assert_eq!(emu.cursor.col, 19);
}

#[test]
fn test_full_reset() {
    let mut emu = Emulator::new(80, 24);
    emu.process(b"\x1b[1;31mBold Red Text");
    emu.process(b"\x1bc"); // Full reset (RIS)

    assert_eq!(emu.cursor.row, 0);
    assert_eq!(emu.cursor.col, 0);
    assert_eq!(row_text_trimmed(&emu, 0), "");
}

#[test]
fn test_dec_cursor_visibility() {
    let mut emu = Emulator::new(80, 24);
    assert!(emu.modes.cursor_visible);

    emu.process(b"\x1b[?25l"); // Hide
    assert!(!emu.modes.cursor_visible);

    emu.process(b"\x1b[?25h"); // Show
    assert!(emu.modes.cursor_visible);
}

#[test]
fn test_alt_screen_buffer() {
    let mut emu = Emulator::new(20, 5);
    emu.process(b"Main screen content");

    // Enter alt screen
    emu.process(b"\x1b[?1049h");
    assert!(emu.modes.alt_screen);
    assert_eq!(row_text_trimmed(&emu, 0), ""); // Alt screen is cleared

    // Leave alt screen
    emu.process(b"\x1b[?1049l");
    assert!(!emu.modes.alt_screen);
}

#[test]
fn test_snapshot_structure() {
    let mut emu = Emulator::new(5, 3);
    emu.process(b"\x1b[1;31mAB\x1b[0mCD");

    let snap = emu.snapshot();
    assert_eq!(snap.rows.len(), 3);
    assert_eq!(snap.rows[0].len(), 5);
    assert_eq!(snap.rows[0][0].content, "A");
    assert!(snap.rows[0][0].bold);
    assert!(!snap.rows[0][2].bold); // "C" is after reset
    assert_eq!(snap.cursor_col, 4);
    assert_eq!(snap.cursor_row, 0);
    assert!(snap.cursor_visible);
}

#[test]
fn test_resize() {
    let mut emu = Emulator::new(20, 5);
    emu.process(b"Hello");
    emu.resize(40, 10);

    assert_eq!(emu.grid.cols, 40);
    assert_eq!(emu.grid.rows, 10);
    assert_eq!(row_text_trimmed(&emu, 0), "Hello");
}

#[test]
fn test_scrollback_overflow() {
    let mut emu = Emulator::new(10, 3);
    // Write enough lines to fill scrollback
    for i in 0..10 {
        emu.process(format!("Line {i}\r\n").as_bytes());
    }
    // Grid should still have the latest content
    assert!(emu.cursor.row <= 2);
}

#[test]
fn test_rapid_sgr_changes() {
    let mut emu = Emulator::new(80, 24);
    // Rapid attribute changes like compiler output
    emu.process(b"\x1b[0;1;31merror\x1b[0m\x1b[1m: \x1b[0m\x1b[0;1;37mfailed\x1b[0m");

    assert_eq!(row_text_trimmed(&emu, 0), "error: failed");
}

#[test]
fn test_partial_sequences_across_reads() {
    let mut emu = Emulator::new(80, 24);

    // Send escape sequence split across two reads
    emu.process(b"A\x1b[31"); // Partial CSI
    // At this point 'A' should be printed
    assert_eq!(emu.grid.cell(0, 0).content.as_str(), "A");

    emu.process(b"mB"); // Complete the CSI and print B
    assert_eq!(emu.grid.cell(0, 1).content.as_str(), "B");
    assert_eq!(
        emu.grid.cell(0, 1).attrs.fg,
        wit_lib::terminal::Color::Named(wit_lib::terminal::NamedColor::Red)
    );
}
