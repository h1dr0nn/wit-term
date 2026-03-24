//! Strip ANSI escape sequences from raw PTY output bytes.

/// Strip ALL ANSI escape sequences from raw bytes and return plain text.
pub fn strip_ansi(input: &[u8]) -> String {
    process_ansi(input, false)
}

/// Strip non-SGR ANSI sequences but KEEP color/style codes (`\x1b[...m`).
/// Removes cursor movement, screen clears, OSC, etc. Normalizes \r\n.
pub fn strip_non_sgr(input: &[u8]) -> String {
    process_ansi(input, true)
}

/// Core ANSI processor. If `keep_sgr` is true, SGR sequences are preserved.
fn process_ansi(input: &[u8], keep_sgr: bool) -> String {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;

    while i < input.len() {
        let b = input[i];

        if b == 0x1b {
            i += 1;
            if i >= input.len() {
                break;
            }
            match input[i] {
                b'[' => {
                    // CSI sequence: scan until final byte (0x40..=0x7E)
                    let csi_start = i - 1; // index of \x1b
                    i += 1;
                    while i < input.len() {
                        if (0x40..=0x7E).contains(&input[i]) {
                            let final_byte = input[i];
                            i += 1;
                            if keep_sgr && final_byte == b'm' {
                                // SGR ends with 'm' — keep color codes
                                out.extend_from_slice(&input[csi_start..i]);
                            } else if matches!(final_byte, b'B' | b'E' | b'F') {
                                // Cursor Down (B), Cursor Next Line (E), Cursor Prev Line (F)
                                // Replace with newline to preserve line structure
                                out.push(b'\n');
                            }
                            break;
                        }
                        i += 1;
                    }
                }
                b']' => {
                    // OSC sequence: skip until ST (\x1b\\ or \x07)
                    i += 1;
                    while i < input.len() {
                        if input[i] == 0x07 {
                            i += 1;
                            break;
                        }
                        if input[i] == 0x1b && i + 1 < input.len() && input[i + 1] == b'\\' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                }
                b'(' | b')' => {
                    i += 1;
                    if i < input.len() {
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        } else if b == b'\r' {
            if i + 1 < input.len() && input[i + 1] == b'\n' {
                // \r\n → \n
                out.push(b'\n');
                i += 2;
            } else {
                // Lone \r → \n (Windows ConPTY uses \r between lines)
                out.push(b'\n');
                i += 1;
            }
        } else if b == 0x07 {
            i += 1;
        } else {
            out.push(b);
            i += 1;
        }
    }

    String::from_utf8_lossy(&out).into_owned()
}

/// Key for comparing cell style (only the attributes that affect rendering).
#[derive(PartialEq)]
struct CellStyle<'a> {
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    inverse: bool,
    fg: &'a super::ColorData,
    bg: &'a super::ColorData,
}

impl<'a> CellStyle<'a> {
    fn from_cell(cell: &'a super::CellData) -> Self {
        Self {
            bold: cell.bold,
            dim: cell.dim,
            italic: cell.italic,
            underline: cell.underline,
            strikethrough: cell.strikethrough,
            inverse: cell.inverse,
            fg: &cell.fg,
            bg: &cell.bg,
        }
    }

    fn is_default(&self) -> bool {
        !self.bold
            && !self.dim
            && !self.italic
            && !self.underline
            && !self.strikethrough
            && !self.inverse
            && matches!(self.fg, super::ColorData::Default)
            && matches!(self.bg, super::ColorData::Default)
    }

    fn write_sgr(&self, out: &mut String) {
        let mut params: Vec<String> = Vec::new();
        if self.bold { params.push("1".into()); }
        if self.dim { params.push("2".into()); }
        if self.italic { params.push("3".into()); }
        if self.underline { params.push("4".into()); }
        if self.inverse { params.push("7".into()); }
        if self.strikethrough { params.push("9".into()); }
        push_color_sgr(self.fg, true, &mut params);
        push_color_sgr(self.bg, false, &mut params);
        if !params.is_empty() {
            out.push_str(&format!("\x1b[{}m", params.join(";")));
        }
    }
}

/// Convert grid rows (with cell attributes) to ANSI-colored text.
/// Each row becomes a line. SGR codes are only emitted when style changes
/// between adjacent cells (coalesced for efficiency).
pub fn grid_to_ansi_text(
    rows: &[Vec<super::CellData>],
    start: usize,
    end: usize,
) -> String {
    let start = start.min(rows.len());
    let end = end.min(rows.len());
    let mut out = String::new();

    for row in &rows[start..end] {
        let mut line = String::new();
        let mut current_styled = false;
        let mut prev_style: Option<CellStyle<'_>> = None;

        for cell in row {
            // Skip spacer cells (from wide characters — they have empty content
            // but are different from regular empty cells which we treat as spaces)
            if cell.content.is_empty() {
                continue;
            }
            let ch = &cell.content;
            let style = CellStyle::from_cell(cell);

            // Only emit SGR when style changes
            let style_changed = prev_style.as_ref() != Some(&style);
            if style_changed {
                if style.is_default() {
                    if current_styled {
                        line.push_str("\x1b[0m");
                        current_styled = false;
                    }
                } else {
                    // Reset then apply new style
                    if current_styled {
                        line.push_str("\x1b[0m");
                    }
                    style.write_sgr(&mut line);
                    current_styled = true;
                }
            }

            line.push_str(ch);
            prev_style = Some(style);
        }

        if current_styled {
            line.push_str("\x1b[0m");
        }

        let trimmed = line.trim_end();
        if !trimmed.is_empty() {
            out.push_str(trimmed);
        }
        out.push('\n');
    }

    out
}

fn push_color_sgr(color: &super::ColorData, is_fg: bool, params: &mut Vec<String>) {
    match color {
        super::ColorData::Default => {}
        super::ColorData::Named { name } => {
            let base = if is_fg { 30 } else { 40 };
            let code = match name.as_str() {
                "Black" => base,
                "Red" => base + 1,
                "Green" => base + 2,
                "Yellow" => base + 3,
                "Blue" => base + 4,
                "Magenta" => base + 5,
                "Cyan" => base + 6,
                "White" => base + 7,
                "BrightBlack" => base + 60,
                "BrightRed" => base + 61,
                "BrightGreen" => base + 62,
                "BrightYellow" => base + 63,
                "BrightBlue" => base + 64,
                "BrightMagenta" => base + 65,
                "BrightCyan" => base + 66,
                "BrightWhite" => base + 67,
                _ => return,
            };
            params.push(code.to_string());
        }
        super::ColorData::Indexed { index } => {
            let prefix = if is_fg { "38;5" } else { "48;5" };
            params.push(format!("{prefix};{index}"));
        }
        super::ColorData::Rgb { r, g, b } => {
            let prefix = if is_fg { "38;2" } else { "48;2" };
            params.push(format!("{prefix};{r};{g};{b}"));
        }
    }
}

/// Extract plain text from a single grid row (no ANSI codes).
pub fn grid_row_to_text(row: &[super::CellData]) -> String {
    let line: String = row
        .iter()
        .map(|c| if c.content.is_empty() { " " } else { c.content.as_str() })
        .collect();
    line.trim_end().to_string()
}

/// Helper: strip ANSI from a &str (for line-level matching).
fn strip_ansi_str(s: &str) -> String {
    strip_ansi(s.as_bytes())
}

/// Strip the echoed command line from output.
/// Works on text that may contain ANSI color codes — uses line-based matching.
pub fn strip_echo(output: &str, command: &str) -> String {
    let lines: Vec<&str> = output.split('\n').collect();
    for (i, line) in lines.iter().enumerate() {
        let plain = strip_ansi_str(line);
        if plain.contains(command) {
            // Skip everything up to and including this line
            let rest = lines[i + 1..].join("\n");
            return rest;
        }
    }
    output.to_string()
}

/// Strip a trailing shell prompt from output.
/// Works on text that may contain ANSI color codes.
pub fn strip_trailing_prompt(output: &str) -> String {
    let trimmed = output.trim_end();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Some(last_nl) = trimmed.rfind('\n') {
        let last_line = &trimmed[last_nl + 1..];
        let plain = strip_ansi_str(last_line);
        if is_prompt_line(plain.trim()) {
            return trimmed[..last_nl].trim_end().to_string();
        }
    } else {
        let plain = strip_ansi_str(trimmed);
        if is_prompt_line(plain.trim()) {
            return String::new();
        }
    }
    trimmed.to_string()
}

/// Extract the CWD from a shell prompt line, if it looks like one.
/// Returns the path portion, e.g. `"PS C:\Users\test>"` → `"C:\Users\test"`.
pub fn extract_cwd_from_prompt(line: &str) -> Option<String> {
    let plain = strip_ansi_str(line).trim().to_string();
    // PowerShell: "PS C:\path>"
    if plain.starts_with("PS ") && plain.ends_with('>') {
        let path = plain[3..plain.len() - 1].trim();
        if !path.is_empty() {
            return Some(path.to_string());
        }
    }
    // CMD: "C:\path>"
    if plain.len() >= 3 && plain.as_bytes().get(1) == Some(&b':') && plain.ends_with('>') {
        let path = plain[..plain.len() - 1].trim();
        if !path.is_empty() {
            return Some(path.to_string());
        }
    }
    None
}

fn is_prompt_line(line: &str) -> bool {
    let line = line.trim();
    if line.is_empty() {
        return false;
    }
    // PowerShell: "PS C:\path>"
    if line.starts_with("PS ") && line.ends_with('>') {
        return true;
    }
    // CMD: "C:\path>"
    if line.len() >= 3 && line.as_bytes()[1] == b':' && line.ends_with('>') {
        return true;
    }
    // Common Unix prompts ending with $ # %
    if line.ends_with('$') || line.ends_with('#') || line.ends_with('%') {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_plain_text() {
        assert_eq!(strip_ansi(b"hello world"), "hello world");
    }

    #[test]
    fn test_strip_ansi_csi() {
        assert_eq!(strip_ansi(b"\x1b[1mhello\x1b[0m"), "hello");
    }

    #[test]
    fn test_strip_ansi_color() {
        assert_eq!(strip_ansi(b"\x1b[32mgreen\x1b[0m"), "green");
    }

    #[test]
    fn test_strip_non_sgr_keeps_colors() {
        let input = b"\x1b[32mgreen\x1b[0m \x1b[Hcursor";
        let result = strip_non_sgr(input);
        assert_eq!(result, "\x1b[32mgreen\x1b[0m cursor");
    }

    #[test]
    fn test_strip_non_sgr_removes_osc() {
        let input = b"\x1b]0;title\x07\x1b[31mred\x1b[0m";
        let result = strip_non_sgr(input);
        assert_eq!(result, "\x1b[31mred\x1b[0m");
    }

    #[test]
    fn test_strip_ansi_osc() {
        assert_eq!(strip_ansi(b"\x1b]0;title\x07hello"), "hello");
    }

    #[test]
    fn test_strip_crlf() {
        assert_eq!(strip_ansi(b"line1\r\nline2\r\n"), "line1\nline2\n");
    }

    #[test]
    fn test_strip_lone_cr() {
        assert_eq!(strip_ansi(b"hello\rworld"), "hello\nworld");
    }

    #[test]
    fn test_strip_echo() {
        let output = "PS C:\\Users> git --version\ngit version 2.53.0\n";
        assert_eq!(strip_echo(output, "git --version"), "git version 2.53.0\n");
    }

    #[test]
    fn test_strip_echo_with_ansi() {
        let output = "\x1b[32mPS C:\\Users>\x1b[0m git --version\ngit version 2.53.0\n";
        assert_eq!(strip_echo(output, "git --version"), "git version 2.53.0\n");
    }

    #[test]
    fn test_strip_echo_no_output() {
        assert_eq!(strip_echo("git --version", "git --version"), "");
    }

    #[test]
    fn test_strip_echo_not_found() {
        let output = "some other text\n";
        assert_eq!(strip_echo(output, "git --version"), "some other text\n");
    }

    #[test]
    fn test_strip_trailing_prompt_ps() {
        let output = "git version 2.53.0\nPS C:\\Users\\test>";
        assert_eq!(strip_trailing_prompt(output), "git version 2.53.0");
    }

    #[test]
    fn test_strip_trailing_prompt_cmd() {
        let output = "git version 2.53.0\nC:\\Users\\test>";
        assert_eq!(strip_trailing_prompt(output), "git version 2.53.0");
    }

    #[test]
    fn test_strip_trailing_prompt_with_ansi() {
        let output = "output line\n\x1b[32mPS C:\\Users>\x1b[0m";
        assert_eq!(strip_trailing_prompt(output), "output line");
    }

    #[test]
    fn test_strip_trailing_prompt_unix() {
        let output = "git version 2.53.0\nuser@host:~$";
        assert_eq!(strip_trailing_prompt(output), "git version 2.53.0");
    }

    #[test]
    fn test_strip_trailing_prompt_no_prompt() {
        let output = "git version 2.53.0";
        assert_eq!(strip_trailing_prompt(output), "git version 2.53.0");
    }
}
