//! Input parser — extracts command structure from terminal input.

/// Parsed representation of the user's input at the cursor.
#[derive(Debug, Clone)]
pub struct ParsedInput {
    /// The full input line.
    pub line: String,
    /// The command (first word).
    pub command: String,
    /// The subcommand (second word if applicable).
    pub subcommand: Option<String>,
    /// All words parsed so far.
    pub words: Vec<String>,
    /// The word currently being typed (at cursor).
    pub current_word: String,
    /// Index of the current word in the words list.
    pub word_index: usize,
    /// Whether the current word starts with '-' (flag).
    pub is_flag: bool,
}

/// Parse input text at cursor position into structured form.
pub fn parse_input(input: &str, cursor_pos: usize) -> ParsedInput {
    let line = &input[..cursor_pos.min(input.len())];

    // Split into words, preserving the last word even if empty
    let words: Vec<String> = shell_split(line);

    let (current_word, word_index) = if line.ends_with(' ') {
        // Cursor is after a space — starting a new word
        (String::new(), words.len())
    } else if let Some(last) = words.last() {
        (last.clone(), words.len() - 1)
    } else {
        (String::new(), 0)
    };

    let command = words.first().cloned().unwrap_or_default();
    let subcommand = if words.len() > 1 && !words[1].starts_with('-') {
        Some(words[1].clone())
    } else {
        None
    };

    let is_flag = current_word.starts_with('-');

    ParsedInput {
        line: line.to_string(),
        command,
        subcommand,
        words,
        current_word,
        word_index,
        is_flag,
    }
}

/// Simple shell-like word splitting (handles basic quoting).
fn shell_split(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    for ch in input.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' if !in_single_quote => {
                escaped = true;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    words.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let parsed = parse_input("", 0);
        assert_eq!(parsed.command, "");
        assert_eq!(parsed.current_word, "");
        assert_eq!(parsed.word_index, 0);
    }

    #[test]
    fn test_parse_command_only() {
        let parsed = parse_input("git", 3);
        assert_eq!(parsed.command, "git");
        assert_eq!(parsed.current_word, "git");
        assert_eq!(parsed.word_index, 0);
    }

    #[test]
    fn test_parse_command_space() {
        let parsed = parse_input("git ", 4);
        assert_eq!(parsed.command, "git");
        assert_eq!(parsed.current_word, "");
        assert_eq!(parsed.word_index, 1);
    }

    #[test]
    fn test_parse_subcommand() {
        let parsed = parse_input("git co", 6);
        assert_eq!(parsed.command, "git");
        assert_eq!(parsed.subcommand, Some("co".into()));
        assert_eq!(parsed.current_word, "co");
        assert_eq!(parsed.word_index, 1);
    }

    #[test]
    fn test_parse_flag() {
        let parsed = parse_input("git commit --am", 15);
        assert_eq!(parsed.command, "git");
        assert_eq!(parsed.current_word, "--am");
        assert!(parsed.is_flag);
    }

    #[test]
    fn test_shell_split_quotes() {
        let words = shell_split(r#"echo "hello world" foo"#);
        assert_eq!(words, vec!["echo", "hello world", "foo"]);
    }

    #[test]
    fn test_shell_split_single_quotes() {
        let words = shell_split("echo 'hello world'");
        assert_eq!(words, vec!["echo", "hello world"]);
    }
}
