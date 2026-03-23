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
    #[serde(default)]
    wit_completion_version: Option<String>,
    command: CommandDef,
}

#[derive(Debug, Deserialize)]
struct CommandDef {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    subcommands: Vec<SubcommandDef>,
    #[serde(default)]
    flags: Vec<FlagDef>,
    #[serde(default)]
    args: Vec<ArgDef>,
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
    #[serde(default)]
    args: Vec<ArgDef>,
    #[serde(default)]
    subcommands: Vec<SubcommandDef>,
    #[serde(default)]
    hidden: bool,
}

#[derive(Debug, Deserialize)]
struct FlagDef {
    name: String,
    #[serde(default)]
    short: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    takes_value: bool,
    #[serde(default)]
    value_hint: Option<String>,
    #[serde(default)]
    value_enum: Vec<String>,
    #[serde(default)]
    default_value: Option<String>,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    repeatable: bool,
    #[serde(default)]
    deprecated: bool,
    #[serde(default)]
    deprecated_message: Option<String>,
    #[serde(default)]
    conflicts_with: Vec<String>,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    hidden: bool,
}

#[derive(Debug, Deserialize)]
struct ArgDef {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    value_hint: Option<String>,
    #[serde(default)]
    value_enum: Vec<String>,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    repeatable: bool,
}

/// Static completion source backed by TOML files.
pub struct StaticSource {
    commands: HashMap<String, CommandDef>,
}

impl StaticSource {
    /// Load all TOML files from a directory, searching bundled/, community/, user/ subdirs.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let mut commands = HashMap::new();

        // Search order: dir itself, then bundled/, community/, user/ subdirs
        let search_dirs = [
            dir.to_path_buf(),
            dir.join("bundled"),
            dir.join("community"),
            dir.join("user"),
        ];

        for search_dir in &search_dirs {
            if !search_dir.exists() {
                continue;
            }
            Self::load_from_dir(search_dir, &mut commands);
        }

        Ok(Self { commands })
    }

    fn load_from_dir(dir: &Path, commands: &mut HashMap<String, CommandDef>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

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
    }

    fn complete_subcommands(
        &self,
        cmd: &CommandDef,
        query: &str,
    ) -> Vec<CompletionItem> {
        Self::complete_subcommand_list(&cmd.subcommands, query)
    }

    fn complete_subcommand_list(
        subcommands: &[SubcommandDef],
        query: &str,
    ) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        for sub in subcommands {
            if sub.hidden {
                continue;
            }
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
            // Skip hidden and deprecated flags unless query is an exact prefix match
            if flag.hidden && !flag.name.starts_with(query) {
                continue;
            }
            if flag.deprecated && !flag.name.starts_with(query) {
                continue;
            }

            if let Some(score) = fuzzy_match(query, &flag.name) {
                let display = if flag.short.is_empty() {
                    flag.name.clone()
                } else {
                    format!("{}, {}", flag.short, flag.name)
                };
                let description = if flag.deprecated {
                    flag.deprecated_message
                        .as_deref()
                        .unwrap_or("[deprecated]")
                        .to_string()
                } else {
                    flag.description.clone()
                };

                items.push(CompletionItem {
                    text: flag.name.clone(),
                    display,
                    description,
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

    fn complete_value_enum(values: &[String], query: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        for value in values {
            if let Some(score) = fuzzy_match(query, value) {
                items.push(CompletionItem {
                    text: value.clone(),
                    display: value.clone(),
                    description: String::new(),
                    kind: CompletionKind::Argument,
                    score,
                });
            }
        }
        items
    }

    fn complete_args(args: &[ArgDef], query: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        for arg in args {
            // If the arg has value_enum, suggest those values
            if !arg.value_enum.is_empty() {
                items.extend(Self::complete_value_enum(&arg.value_enum, query));
            }
        }
        items
    }

    /// Find a subcommand chain for nested subcommands (e.g. "git remote add").
    fn find_subcommand<'a>(
        cmd: &'a CommandDef,
        sub_names: &[&str],
    ) -> Option<&'a SubcommandDef> {
        if sub_names.is_empty() {
            return None;
        }

        let mut current_subs = &cmd.subcommands;
        let mut found: Option<&SubcommandDef> = None;

        for name in sub_names {
            match current_subs.iter().find(|s| {
                s.name == *name || s.aliases.iter().any(|a| a == *name)
            }) {
                Some(sub) => {
                    found = Some(sub);
                    current_subs = &sub.subcommands;
                }
                None => break,
            }
        }

        found
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
                // Also match command aliases
                for alias in &cmd.aliases {
                    if let Some(score) = fuzzy_match(&parsed.current_word, alias) {
                        items.push(CompletionItem {
                            text: alias.clone(),
                            display: format!("{} (→ {})", alias, name),
                            description: cmd.description.clone(),
                            kind: CompletionKind::Command,
                            score,
                        });
                    }
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
            // Check if the previous flag takes a value_enum
            if parsed.word_index >= 2 {
                let words: Vec<&str> = parsed.raw_words();
                if let Some(prev_word) = words.get(parsed.word_index.saturating_sub(1)) {
                    // Find the flag def for prev_word
                    let all_flags = self.collect_applicable_flags(cmd, parsed);
                    if let Some(flag) = all_flags.iter().find(|f| {
                        f.name == *prev_word || f.short == *prev_word
                    }) {
                        if flag.takes_value && !flag.value_enum.is_empty() {
                            return Self::complete_value_enum(
                                &flag.value_enum,
                                &parsed.current_word,
                            );
                        }
                    }
                }
            }

            // Subcommand-specific flags
            if let Some(sub_name) = &parsed.subcommand {
                // Try nested subcommands
                let sub_chain: Vec<&str> = vec![sub_name.as_str()];
                if let Some(sub) = Self::find_subcommand(cmd, &sub_chain) {
                    let mut items = self.complete_flags(&sub.flags, &parsed.current_word);
                    items.extend(self.complete_flags(&cmd.flags, &parsed.current_word));
                    return items;
                }
            }
            return self.complete_flags(&cmd.flags, &parsed.current_word);
        }

        // Complete subcommands
        if parsed.word_index == 1 {
            return self.complete_subcommands(cmd, &parsed.current_word);
        }

        // For word_index >= 2, check for nested subcommands
        if parsed.word_index >= 2 {
            if let Some(sub_name) = &parsed.subcommand {
                let sub_chain: Vec<&str> = vec![sub_name.as_str()];
                if let Some(sub) = Self::find_subcommand(cmd, &sub_chain) {
                    // Try completing nested subcommands
                    if !sub.subcommands.is_empty() {
                        let nested = Self::complete_subcommand_list(
                            &sub.subcommands,
                            &parsed.current_word,
                        );
                        if !nested.is_empty() {
                            return nested;
                        }
                    }
                    // Try completing args
                    if !sub.args.is_empty() {
                        let arg_items = Self::complete_args(&sub.args, &parsed.current_word);
                        if !arg_items.is_empty() {
                            return arg_items;
                        }
                    }
                }
            }

            // Try command-level args
            if !cmd.args.is_empty() {
                return Self::complete_args(&cmd.args, &parsed.current_word);
            }
        }

        Vec::new()
    }
}

impl StaticSource {
    fn collect_applicable_flags<'a>(
        &'a self,
        cmd: &'a CommandDef,
        parsed: &ParsedInput,
    ) -> Vec<&'a FlagDef> {
        let mut flags: Vec<&FlagDef> = cmd.flags.iter().collect();
        if let Some(sub_name) = &parsed.subcommand {
            if let Some(sub) = cmd.subcommands.iter().find(|s| s.name == *sub_name) {
                flags.extend(sub.flags.iter());
            }
        }
        flags
    }
}

impl ParsedInput {
    fn raw_words(&self) -> Vec<&str> {
        // Simple split for accessing previous words
        self.line.split_whitespace().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_source() -> StaticSource {
        let toml_content = r#"
wit_completion_version = "1.0"

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
takes_value = true

[[command.subcommands]]
name = "checkout"
description = "Switch branches"
aliases = ["co"]

[[command.subcommands]]
name = "remote"
description = "Manage remotes"

[[command.subcommands.subcommands]]
name = "add"
description = "Add a remote"

[[command.subcommands.subcommands]]
name = "remove"
description = "Remove a remote"

[[command.flags]]
name = "--version"
description = "Show version"

[[command.flags]]
name = "--help"
short = "-h"
description = "Show help"
hidden = true
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

    #[test]
    fn test_hidden_flag_filtered() {
        let source = make_test_source();
        let parsed = super::super::parser::parse_input("git --ver", 9);
        let items = source.complete(&parsed, Path::new("/tmp"));
        // --version should appear, --help should be filtered (hidden)
        assert!(items.iter().any(|i| i.text == "--version"));
        assert!(!items.iter().any(|i| i.text == "--help"));
    }

    #[test]
    fn test_wit_completion_version_parsed() {
        let toml_content = r#"
wit_completion_version = "1.0"

[command]
name = "test"
description = "Test command"
"#;
        let file: CompletionFile = toml::from_str(toml_content).unwrap();
        assert_eq!(file.wit_completion_version, Some("1.0".to_string()));
    }

    #[test]
    fn test_nested_subcommands() {
        let source = make_test_source();
        // "git remote a" should suggest "add"
        let parsed = super::super::parser::parse_input("git remote a", 12);
        let items = source.complete(&parsed, Path::new("/tmp"));
        assert!(items.iter().any(|i| i.text == "add"));
    }
}
