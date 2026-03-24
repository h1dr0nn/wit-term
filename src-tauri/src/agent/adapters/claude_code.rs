//! Claude Code output adapter.
//!
//! Parses the terminal output of Claude Code and extracts structured events
//! such as cost updates, token counts, tool uses, file edits, and thinking state.

use crate::agent::types::{AgentEvent, FileAction};

use super::AgentAdapter;

/// Adapter for parsing Claude Code terminal output.
pub struct ClaudeCodeAdapter {
    line_buffer: String,
    in_thinking: bool,
}

impl ClaudeCodeAdapter {
    pub fn new() -> Self {
        Self {
            line_buffer: String::new(),
            in_thinking: false,
        }
    }

    /// Parse a single complete line of Claude Code output into events.
    fn parse_line(&mut self, line: &str) -> Vec<AgentEvent> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let mut events = Vec::new();

        // Strip ANSI escape codes for pattern matching
        let plain = strip_ansi_codes(trimmed);
        let plain = plain.trim();

        if plain.is_empty() {
            return events;
        }

        // Cost pattern: look for "$" followed by digits (e.g. "$0.42", "Cost: $1.23")
        if let Some(cost) = extract_cost(plain) {
            events.push(AgentEvent::CostUpdate { total_cost: cost });
            return events;
        }

        // Token pattern: look for "tokens" with nearby numbers
        if let Some((input, output)) = extract_tokens(plain) {
            events.push(AgentEvent::TokenUpdate {
                input_tokens: input,
                output_tokens: output,
            });
            return events;
        }

        // Model pattern: "claude-" followed by model identifier
        if let Some(model) = extract_model(plain) {
            events.push(AgentEvent::ModelInfo {
                model_name: model.to_string(),
            });
            return events;
        }

        // Thinking indicators
        if plain.contains("Thinking")
            || plain.contains("thinking")
            || plain.contains('\u{280B}') // braille spinner chars
            || plain.contains('\u{2839}')
            || plain.contains('\u{2834}')
            || plain.contains('\u{2826}')
        {
            if !self.in_thinking {
                self.in_thinking = true;
                events.push(AgentEvent::ThinkingStart);
            }
            return events;
        }

        // If we were thinking and hit a non-thinking line, end thinking
        if self.in_thinking {
            self.in_thinking = false;
            events.push(AgentEvent::ThinkingEnd);
        }

        // File edit patterns
        if let Some((path, action)) = extract_file_edit(plain) {
            events.push(AgentEvent::FileEdit {
                path: path.to_string(),
                action,
            });
            return events;
        }

        // Tool use patterns
        if let Some((tool, desc)) = extract_tool_use(plain) {
            events.push(AgentEvent::ToolUse {
                tool_name: tool.to_string(),
                description: desc.to_string(),
            });
            return events;
        }

        // Fallback: emit as status text
        events.push(AgentEvent::StatusText {
            text: plain.to_string(),
        });

        events
    }
}

impl AgentAdapter for ClaudeCodeAdapter {
    fn name(&self) -> &str {
        "Claude Code"
    }

    fn parse_output(&mut self, data: &[u8]) -> Vec<AgentEvent> {
        let text = String::from_utf8_lossy(data);
        self.line_buffer.push_str(&text);

        let mut events = Vec::new();

        // Process all complete lines (those ending with \n)
        while let Some(newline_pos) = self.line_buffer.find('\n') {
            let line: String = self.line_buffer.drain(..=newline_pos).collect();
            let line = line.trim_end_matches('\n').trim_end_matches('\r');
            events.extend(self.parse_line(line));
        }

        events
    }

    fn reset(&mut self) {
        self.line_buffer.clear();
        self.in_thinking = false;
    }
}

/// Strip ANSI escape sequences from a string for plain-text pattern matching.
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // ESC sequence — consume until end
            if let Some(&next) = chars.peek() {
                if next == '[' {
                    // CSI sequence: consume until letter
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_alphabetic() {
                            chars.next();
                            break;
                        }
                        chars.next();
                    }
                } else if next == ']' {
                    // OSC sequence: consume until BEL or ST
                    chars.next();
                    while let Some(c) = chars.next() {
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' {
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                                break;
                            }
                        }
                    }
                } else {
                    chars.next(); // consume single char after ESC
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Extract a dollar cost from a line (e.g., "$0.42" or "Cost: $1.23").
fn extract_cost(line: &str) -> Option<f64> {
    let dollar_pos = line.find('$')?;
    let rest = &line[dollar_pos + 1..];
    // Collect digits and dots
    let num_str: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if num_str.is_empty() {
        return None;
    }
    num_str.parse::<f64>().ok()
}

/// Extract input/output token counts from a line.
fn extract_tokens(line: &str) -> Option<(u64, u64)> {
    let lower = line.to_lowercase();
    if !lower.contains("token") {
        return None;
    }

    // Look for patterns like "1234 input ... 5678 output" or "input: 1234 output: 5678"
    let numbers: Vec<u64> = line
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();

    if numbers.len() >= 2 {
        Some((numbers[0], numbers[1]))
    } else if numbers.len() == 1 {
        Some((numbers[0], 0))
    } else {
        None
    }
}

/// Extract a model name (e.g., "claude-3-opus-20240229").
fn extract_model(line: &str) -> Option<&str> {
    let lower = line.to_lowercase();
    // Find "claude-" in the lowercased version, then extract the token from the original
    let start = lower.find("claude-")?;
    let rest = &line[start..];
    let end = rest
        .find(|c: char| c.is_whitespace() || c == ',' || c == ')' || c == ']')
        .unwrap_or(rest.len());
    let model = &rest[..end];
    if model.len() > 7 {
        // more than just "claude-"
        Some(model)
    } else {
        None
    }
}

/// Extract file edit info from a line.
fn extract_file_edit(line: &str) -> Option<(&str, FileAction)> {
    let patterns: &[(&str, FileAction)] = &[
        ("Create ", FileAction::Created),
        ("Created ", FileAction::Created),
        ("Write ", FileAction::Modified),
        ("Wrote ", FileAction::Modified),
        ("Edit ", FileAction::Modified),
        ("Edited ", FileAction::Modified),
        ("Delete ", FileAction::Deleted),
        ("Deleted ", FileAction::Deleted),
    ];

    for (prefix, action) in patterns {
        if let Some(rest) = line.strip_prefix(prefix) {
            let path = rest.split_whitespace().next()?;
            if path.contains('/') || path.contains('\\') || path.contains('.') {
                return Some((path, action.clone()));
            }
        }
    }

    None
}

/// Extract tool use info from a line.
fn extract_tool_use(line: &str) -> Option<(&str, &str)> {
    let tools = [
        "Read", "Bash", "Search", "Glob", "Grep", "Write", "Edit", "ListDir",
    ];

    for tool in &tools {
        // Match patterns like "Read(file.rs)" or "Bash: ls -la" or "⠙ Read file.rs"
        if line.starts_with(tool) || line.contains(&format!(" {tool}")) {
            let desc = line
                .find(tool)
                .map(|i| &line[i..])
                .unwrap_or(line)
                .trim();
            return Some((tool, desc));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cost() {
        assert_eq!(extract_cost("Cost: $0.42"), Some(0.42));
        assert_eq!(extract_cost("Total: $1.23 USD"), Some(1.23));
        assert_eq!(extract_cost("no cost here"), None);
    }

    #[test]
    fn test_extract_model() {
        assert_eq!(
            extract_model("Using claude-3-opus-20240229"),
            Some("claude-3-opus-20240229")
        );
        assert_eq!(extract_model("no model here"), None);
    }

    #[test]
    fn test_strip_ansi() {
        let input = "\x1b[32mHello\x1b[0m World";
        assert_eq!(strip_ansi_codes(input), "Hello World");
    }

    #[test]
    fn test_parse_output_buffering() {
        let mut adapter = ClaudeCodeAdapter::new();

        // Partial line — should not produce events yet
        let events = adapter.parse_output(b"Cost: $0.");
        assert!(events.is_empty());

        // Complete the line
        let events = adapter.parse_output(b"42\n");
        assert_eq!(events.len(), 1);
        match &events[0] {
            AgentEvent::CostUpdate { total_cost } => {
                assert!((total_cost - 0.42).abs() < f64::EPSILON);
            }
            other => panic!("Expected CostUpdate, got {:?}", other),
        }
    }
}
