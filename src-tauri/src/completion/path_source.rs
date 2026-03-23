//! Path completion source — completes filesystem paths.

use std::path::Path;

use super::parser::ParsedInput;
use super::{CompletionItem, CompletionKind, CompletionSource};

pub struct PathSource;

impl CompletionSource for PathSource {
    fn name(&self) -> &str {
        "path"
    }

    fn complete(&self, parsed: &ParsedInput, cwd: &Path) -> Vec<CompletionItem> {
        let word = &parsed.current_word;

        // Only complete if the word looks like a path
        if word.is_empty()
            || word.starts_with('-')
            || (parsed.word_index == 0 && !word.contains('/') && !word.contains('\\') && !word.starts_with('.'))
        {
            return Vec::new();
        }

        let (dir, prefix) = if let Some(sep_pos) = word.rfind(['/', '\\']) {
            let dir_part = &word[..=sep_pos];
            let prefix = &word[sep_pos + 1..];

            let dir_path = if dir_part.starts_with('/') || dir_part.starts_with('\\') || dir_part.contains(':') {
                std::path::PathBuf::from(dir_part)
            } else {
                cwd.join(dir_part)
            };

            (dir_path, prefix.to_string())
        } else {
            (cwd.to_path_buf(), word.to_string())
        };

        let mut items = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();

                // Skip hidden files unless prefix starts with .
                if name.starts_with('.') && !prefix.starts_with('.') {
                    continue;
                }

                if !prefix.is_empty() && !name.to_lowercase().starts_with(&prefix.to_lowercase()) {
                    continue;
                }

                let is_dir = entry.file_type().is_ok_and(|t| t.is_dir());
                let display_name = if is_dir {
                    format!("{name}/")
                } else {
                    name.clone()
                };

                // Build the full completion text
                let text = if let Some(sep_pos) = word.rfind(['/', '\\']) {
                    format!("{}{}", &word[..=sep_pos], display_name)
                } else {
                    display_name.clone()
                };

                let score = if name.to_lowercase().starts_with(&prefix.to_lowercase()) {
                    if is_dir { 0.8 } else { 0.7 }
                } else {
                    0.3
                };

                items.push(CompletionItem {
                    text,
                    display: display_name,
                    description: if is_dir {
                        "Directory".into()
                    } else {
                        "File".into()
                    },
                    kind: CompletionKind::Path,
                    score,
                });
            }
        }

        // Sort: directories first, then alphabetical
        items.sort_by(|a, b| {
            let a_dir = a.description == "Directory";
            let b_dir = b.description == "Directory";
            b_dir.cmp(&a_dir).then(a.display.cmp(&b.display))
        });

        items.truncate(20);
        items
    }
}
