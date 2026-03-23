//! Static completion source — loads TOML completion definitions.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use super::fuzzy::fuzzy_match;
use super::parser::ParsedInput;
use super::{CompletionItem, CompletionKind, CompletionSource};

/// A TOML-based completion definition.
#[derive(Debug, Deserialize)]
struct CompletionFile {
    command: CommandDef,
}

#[derive(Debug, Deserialize)]
struct CommandDef {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    subcommands: Vec<SubcommandDef>,
    #[serde(default)]
    flags: Vec<FlagDef>,
}

#[derive(Debug, Deserialize)]
struct SubcommandDef {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    flags: Vec<FlagDef>,
}

#[derive(Debug, Deserialize)]
struct FlagDef {
    name: String,
    #[serde(default)]
    short: String,
    #[serde(default)]
    description: String,
}

/// Static completion source backed by TOML files.
pub struct StaticSource {
    commands: HashMap<String, CommandDef>,
}

impl StaticSource {
    /// Load all TOML files from a directory.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let mut commands = HashMap::new();

        if !dir.exists() {
            return Ok(Self { commands });
        }

        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read completions dir: {e}"))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match toml::from_str::<CompletionFile>(&content) {
                        Ok(file) => {
                            commands.insert(file.command.name.clone(), file.command);
                        }
                        Err(e) => {
                            log::warn!("Failed to parse {}: {e}", path.display());
                        }
                    },
                    Err(e) => {
                        log::warn!("Failed to read {}: {e}", path.display());
                    }
                }
            }
        }

        Ok(Self { commands })
    }

    fn complete_subcommands(
        &self,
        cmd: &CommandDef,
        query: &str,
    ) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        for sub in &cmd.subcommands {
            if let Some(score) = fuzzy_match(query, &sub.name) {
                items.push(CompletionItem {
                    text: sub.name.clone(),
                    display: sub.name.clone(),
                    description: sub.description.clone(),
                    kind: CompletionKind::Subcommand,
                    score,
                });
            }
            // Check aliases
            for alias in &sub.aliases {
                if let Some(score) = fuzzy_match(query, alias) {
                    items.push(CompletionItem {
                        text: alias.clone(),
                        display: format!("{} (→ {})", alias, sub.name),
                        description: sub.description.clone(),
                        kind: CompletionKind::Subcommand,
                        score,
                    });
                }
            }
        }

        items
    }

    fn complete_flags(
        &self,
        flags: &[FlagDef],
        query: &str,
    ) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        for flag in flags {
            if let Some(score) = fuzzy_match(query, &flag.name) {
                items.push(CompletionItem {
                    text: flag.name.clone(),
                    display: if flag.short.is_empty() {
                        flag.name.clone()
                    } else {
                        format!("{}, {}", flag.short, flag.name)
                    },
                    description: flag.description.clone(),
                    kind: CompletionKind::Flag,
                    score,
                });
            }
            // Also match by short flag
            if !flag.short.is_empty() {
                if let Some(score) = fuzzy_match(query, &flag.short) {
                    items.push(CompletionItem {
                        text: flag.short.clone(),
                        display: format!("{}, {}", flag.short, flag.name),
                        description: flag.description.clone(),
                        kind: CompletionKind::Flag,
                        score,
                    });
                }
            }
        }

        items
    }
}

impl CompletionSource for StaticSource {
    fn name(&self) -> &str {
        "static"
    }

    fn complete(&self, parsed: &ParsedInput, _cwd: &Path) -> Vec<CompletionItem> {
        // If we're completing the command itself (word_index == 0)
        if parsed.word_index == 0 {
            let mut items = Vec::new();
            for (name, cmd) in &self.commands {
                if let Some(score) = fuzzy_match(&parsed.current_word, name) {
                    items.push(CompletionItem {
                        text: name.clone(),
                        display: name.clone(),
                        description: cmd.description.clone(),
                        kind: CompletionKind::Command,
                        score,
                    });
                }
            }
            return items;
        }

        // Look up the command
        let cmd = match self.commands.get(&parsed.command) {
            Some(cmd) => cmd,
            None => return Vec::new(),
        };

        // If current word is a flag
        if parsed.is_flag {
            // Check for subcommand-specific flags first
            if let Some(sub_name) = &parsed.subcommand {
                if let Some(sub) = cmd.subcommands.iter().find(|s| s.name == *sub_name) {
                    let mut items = self.complete_flags(&sub.flags, &parsed.current_word);
                    // Also include command-level flags
                    items.extend(self.complete_flags(&cmd.flags, &parsed.current_word));
                    return items;
                }
            }
            return self.complete_flags(&cmd.flags, &parsed.current_word);
        }

        // Complete subcommands (word_index == 1)
        if parsed.word_index == 1 {
            return self.complete_subcommands(cmd, &parsed.current_word);
        }

        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_source() -> StaticSource {
        let toml_content = r#"
[command]
name = "git"
description = "Version control"

[[command.subcommands]]
name = "commit"
description = "Record changes"
aliases = ["ci"]

[[command.subcommands.flags]]
name = "--message"
short = "-m"
description = "Commit message"

[[command.subcommands]]
name = "checkout"
description = "Switch branches"
aliases = ["co"]

[[command.flags]]
name = "--version"
description = "Show version"
"#;
        let file: CompletionFile = toml::from_str(toml_content).unwrap();
        let mut commands = HashMap::new();
        commands.insert(file.command.name.clone(), file.command);
        StaticSource { commands }
    }

    #[test]
    fn test_complete_command() {
        let source = make_test_source();
        let parsed = super::super::parser::parse_input("gi", 2);
        let items = source.complete(&parsed, Path::new("/tmp"));
        assert!(!items.is_empty());
        assert_eq!(items[0].text, "git");
    }

    #[test]
    fn test_complete_subcommand() {
        let source = make_test_source();
        let parsed = super::super::parser::parse_input("git co", 6);
        let items = source.complete(&parsed, Path::new("/tmp"));
        assert!(!items.is_empty());
        assert!(items.iter().any(|i| i.text == "commit" || i.text == "checkout"));
    }

    #[test]
    fn test_complete_flag() {
        let source = make_test_source();
        let parsed = super::super::parser::parse_input("git commit --m", 14);
        let items = source.complete(&parsed, Path::new("/tmp"));
        assert!(!items.is_empty());
        assert!(items.iter().any(|i| i.text == "--message"));
    }

    #[test]
    fn test_complete_alias() {
        let source = make_test_source();
        let parsed = super::super::parser::parse_input("git ci", 6);
        let items = source.complete(&parsed, Path::new("/tmp"));
        assert!(items.iter().any(|i| i.text == "ci"));
    }
}
