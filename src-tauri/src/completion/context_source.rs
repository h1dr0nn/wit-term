//! Dynamic context-aware completion source.
//!
//! Provides completions based on the current project context:
//! git branches/tags, npm scripts, cargo targets, docker containers/images.

use std::path::Path;
use std::process::Command;

use super::fuzzy::fuzzy_match;
use super::parser::ParsedInput;
use super::{CompletionItem, CompletionKind, CompletionSource};

pub struct ContextSource;

impl CompletionSource for ContextSource {
    fn name(&self) -> &str {
        "context"
    }

    fn complete(&self, parsed: &ParsedInput, cwd: &Path) -> Vec<CompletionItem> {
        // Only provide dynamic completions for arguments (word_index >= 2)
        // or subcommand position for specific commands
        match parsed.command.as_str() {
            "git" => complete_git(parsed, cwd),
            "npm" | "yarn" | "pnpm" | "bun" => complete_node_scripts(parsed, cwd),
            "cargo" => complete_cargo(parsed, cwd),
            "docker" => complete_docker(parsed, cwd),
            "make" => complete_make_targets(parsed, cwd),
            _ => Vec::new(),
        }
    }
}

/// Git dynamic completions: branches, tags, remotes.
fn complete_git(parsed: &ParsedInput, cwd: &Path) -> Vec<CompletionItem> {
    let sub = match &parsed.subcommand {
        Some(s) => s.as_str(),
        None => return Vec::new(),
    };

    // Only provide branch/tag completions for relevant subcommands
    let needs_branch = matches!(
        sub,
        "checkout" | "switch" | "branch" | "merge" | "rebase" | "diff" | "log" | "cherry-pick"
    );
    let needs_tag = matches!(sub, "checkout" | "diff" | "log");
    let needs_remote = matches!(sub, "push" | "pull" | "fetch");

    if !needs_branch && !needs_tag && !needs_remote {
        return Vec::new();
    }

    // Don't complete flags
    if parsed.is_flag {
        return Vec::new();
    }

    let query = &parsed.current_word;
    let mut items = Vec::new();

    if needs_branch {
        if let Some(branches) = git_branches(cwd) {
            for branch in branches {
                if let Some(score) = fuzzy_match(query, &branch) {
                    items.push(CompletionItem {
                        text: branch.clone(),
                        display: branch,
                        description: "Branch".into(),
                        kind: CompletionKind::Argument,
                        score: score * 0.95, // Slightly below static completions
                    });
                }
            }
        }
    }

    if needs_tag {
        if let Some(tags) = git_tags(cwd) {
            for tag in tags {
                if let Some(score) = fuzzy_match(query, &tag) {
                    items.push(CompletionItem {
                        text: tag.clone(),
                        display: tag,
                        description: "Tag".into(),
                        kind: CompletionKind::Argument,
                        score: score * 0.9,
                    });
                }
            }
        }
    }

    if needs_remote {
        if let Some(remotes) = git_remotes(cwd) {
            for remote in remotes {
                if let Some(score) = fuzzy_match(query, &remote) {
                    items.push(CompletionItem {
                        text: remote.clone(),
                        display: remote,
                        description: "Remote".into(),
                        kind: CompletionKind::Argument,
                        score: score * 0.9,
                    });
                }
            }
        }
    }

    items
}

/// npm/yarn/pnpm dynamic completions: scripts from package.json.
fn complete_node_scripts(parsed: &ParsedInput, cwd: &Path) -> Vec<CompletionItem> {
    let sub = match &parsed.subcommand {
        Some(s) => s.as_str(),
        None => return Vec::new(),
    };

    if sub != "run" && sub != "run-script" {
        return Vec::new();
    }

    if parsed.is_flag {
        return Vec::new();
    }

    let query = &parsed.current_word;
    let mut items = Vec::new();

    if let Some(scripts) = read_npm_scripts(cwd) {
        for (name, script_cmd) in scripts {
            if let Some(score) = fuzzy_match(query, &name) {
                items.push(CompletionItem {
                    text: name.clone(),
                    display: name,
                    description: truncate_str(&script_cmd, 50),
                    kind: CompletionKind::Argument,
                    score: score * 0.95,
                });
            }
        }
    }

    items
}

/// Cargo dynamic completions: binary/example targets.
fn complete_cargo(parsed: &ParsedInput, cwd: &Path) -> Vec<CompletionItem> {
    let sub = match &parsed.subcommand {
        Some(s) => s.as_str(),
        None => return Vec::new(),
    };

    // For `cargo run --bin <name>` or `cargo run --example <name>`
    let needs_bin = matches!(sub, "run" | "build" | "test" | "bench" | "install");
    if !needs_bin || parsed.is_flag {
        return Vec::new();
    }

    // Check if previous word was --bin or --example
    let prev_word = if parsed.word_index > 0 {
        parsed.words.get(parsed.word_index - 1).map(|s| s.as_str())
    } else {
        None
    };

    let needs_bin_names = prev_word == Some("--bin");
    let needs_example_names = prev_word == Some("--example");

    if !needs_bin_names && !needs_example_names {
        return Vec::new();
    }

    let query = &parsed.current_word;
    let mut items = Vec::new();

    if let Some(targets) = read_cargo_targets(cwd, needs_example_names) {
        for target in targets {
            if let Some(score) = fuzzy_match(query, &target) {
                let desc = if needs_example_names {
                    "Example"
                } else {
                    "Binary"
                };
                items.push(CompletionItem {
                    text: target.clone(),
                    display: target,
                    description: desc.into(),
                    kind: CompletionKind::Argument,
                    score: score * 0.95,
                });
            }
        }
    }

    items
}

/// Docker dynamic completions: containers, images.
fn complete_docker(parsed: &ParsedInput, cwd: &Path) -> Vec<CompletionItem> {
    let sub = match &parsed.subcommand {
        Some(s) => s.as_str(),
        None => return Vec::new(),
    };

    if parsed.is_flag {
        return Vec::new();
    }

    let query = &parsed.current_word;
    let mut items = Vec::new();

    let needs_container = matches!(
        sub,
        "start" | "stop" | "restart" | "rm" | "exec" | "logs" | "inspect" | "attach" | "kill"
    );
    let needs_image = matches!(sub, "run" | "rmi" | "inspect" | "push" | "pull" | "tag");

    if needs_container {
        if let Some(containers) = docker_containers(cwd) {
            for (name, status) in containers {
                if let Some(score) = fuzzy_match(query, &name) {
                    items.push(CompletionItem {
                        text: name.clone(),
                        display: name,
                        description: status,
                        kind: CompletionKind::Argument,
                        score: score * 0.95,
                    });
                }
            }
        }
    }

    if needs_image {
        if let Some(images) = docker_images(cwd) {
            for (repo_tag, size) in images {
                if let Some(score) = fuzzy_match(query, &repo_tag) {
                    items.push(CompletionItem {
                        text: repo_tag.clone(),
                        display: repo_tag,
                        description: size,
                        kind: CompletionKind::Argument,
                        score: score * 0.9,
                    });
                }
            }
        }
    }

    items
}

/// Make dynamic completions: targets from Makefile.
fn complete_make_targets(parsed: &ParsedInput, cwd: &Path) -> Vec<CompletionItem> {
    // Make targets are at word_index 1+ (positional args)
    if parsed.word_index == 0 || parsed.is_flag {
        return Vec::new();
    }

    let query = &parsed.current_word;
    let mut items = Vec::new();

    if let Some(targets) = read_makefile_targets(cwd) {
        for target in targets {
            if let Some(score) = fuzzy_match(query, &target) {
                items.push(CompletionItem {
                    text: target.clone(),
                    display: target,
                    description: "Make target".into(),
                    kind: CompletionKind::Argument,
                    score: score * 0.95,
                });
            }
        }
    }

    items
}

// ─── Helper functions ──────────────────────────────────────────────

fn git_branches(cwd: &Path) -> Option<Vec<String>> {
    let output = Command::new("git")
        .args(["branch", "--list", "--format=%(refname:short)"])
        .current_dir(cwd)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

fn git_tags(cwd: &Path) -> Option<Vec<String>> {
    let output = Command::new("git")
        .args(["tag", "--list"])
        .current_dir(cwd)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

fn git_remotes(cwd: &Path) -> Option<Vec<String>> {
    let output = Command::new("git")
        .args(["remote"])
        .current_dir(cwd)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

fn read_npm_scripts(cwd: &Path) -> Option<Vec<(String, String)>> {
    // Walk up to find package.json
    let mut dir = cwd.to_path_buf();
    loop {
        let pkg_path = dir.join("package.json");
        if pkg_path.exists() {
            let content = std::fs::read_to_string(&pkg_path).ok()?;
            let pkg: serde_json::Value = serde_json::from_str(&content).ok()?;
            let scripts = pkg.get("scripts")?.as_object()?;
            return Some(
                scripts
                    .iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect(),
            );
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn read_cargo_targets(cwd: &Path, examples: bool) -> Option<Vec<String>> {
    // Walk up to find Cargo.toml
    let mut dir = cwd.to_path_buf();
    loop {
        let cargo_path = dir.join("Cargo.toml");
        if cargo_path.exists() {
            let content = std::fs::read_to_string(&cargo_path).ok()?;
            let manifest: toml::Value = toml::from_str(&content).ok()?;

            let section = if examples { "example" } else { "bin" };
            let targets = manifest.get(section)?.as_array()?;
            return Some(
                targets
                    .iter()
                    .filter_map(|t| t.get("name")?.as_str().map(|s| s.to_string()))
                    .collect(),
            );
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn docker_containers(cwd: &Path) -> Option<Vec<(String, String)>> {
    let output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.Names}}\t{{.Status}}"])
        .current_dir(cwd)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, '\t');
                let name = parts.next()?.trim().to_string();
                let status = parts.next().unwrap_or("").trim().to_string();
                if name.is_empty() {
                    None
                } else {
                    Some((name, status))
                }
            })
            .collect(),
    )
}

fn docker_images(cwd: &Path) -> Option<Vec<(String, String)>> {
    let output = Command::new("docker")
        .args(["images", "--format", "{{.Repository}}:{{.Tag}}\t{{.Size}}"])
        .current_dir(cwd)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, '\t');
                let repo_tag = parts.next()?.trim().to_string();
                let size = parts.next().unwrap_or("").trim().to_string();
                if repo_tag.is_empty() || repo_tag == "<none>:<none>" {
                    None
                } else {
                    Some((repo_tag, size))
                }
            })
            .collect(),
    )
}

fn read_makefile_targets(cwd: &Path) -> Option<Vec<String>> {
    let makefile = cwd.join("Makefile");
    if !makefile.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&makefile).ok()?;
    let mut targets = Vec::new();

    for line in content.lines() {
        // Match lines like `target_name:` (not starting with tab/space, not .PHONY etc)
        if let Some(colon_pos) = line.find(':') {
            let target = line[..colon_pos].trim();
            if !target.is_empty()
                && !target.starts_with('#')
                && !target.starts_with('.')
                && !target.starts_with('\t')
                && !target.contains(' ')
                && !target.contains('$')
            {
                targets.push(target.to_string());
            }
        }
    }

    Some(targets)
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world!", 8), "hello...");
    }

    #[test]
    fn test_complete_git_no_subcommand() {
        let parsed = super::super::parser::parse_input("git ", 4);
        let items = complete_git(&parsed, Path::new("/tmp"));
        assert!(items.is_empty());
    }

    #[test]
    fn test_complete_git_irrelevant_subcommand() {
        let parsed = super::super::parser::parse_input("git add ", 8);
        let items = complete_git(&parsed, Path::new("/tmp"));
        assert!(items.is_empty());
    }

    #[test]
    fn test_complete_npm_no_run() {
        let parsed = super::super::parser::parse_input("npm install ", 12);
        let items = complete_node_scripts(&parsed, Path::new("/tmp"));
        assert!(items.is_empty());
    }

    #[test]
    fn test_complete_cargo_no_bin_flag() {
        let parsed = super::super::parser::parse_input("cargo run ", 10);
        let items = complete_cargo(&parsed, Path::new("/tmp"));
        assert!(items.is_empty());
    }

    #[test]
    fn test_complete_docker_flags_ignored() {
        let parsed = super::super::parser::parse_input("docker stop --", 14);
        let items = complete_docker(&parsed, Path::new("/tmp"));
        assert!(items.is_empty());
    }

    #[test]
    fn test_read_makefile_targets() {
        let _dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let temp = std::env::temp_dir().join("wit-test-makefile");
        std::fs::create_dir_all(&temp).unwrap();
        std::fs::write(
            temp.join("Makefile"),
            "build:\n\techo build\ntest:\n\techo test\n.PHONY: build test\n",
        )
        .unwrap();

        let targets = read_makefile_targets(&temp).unwrap();
        assert!(targets.contains(&"build".to_string()));
        assert!(targets.contains(&"test".to_string()));

        std::fs::remove_dir_all(&temp).ok();
    }

    #[test]
    fn test_complete_make_at_command_position() {
        let parsed = super::super::parser::parse_input("make", 4);
        let items = complete_make_targets(&parsed, Path::new("/tmp"));
        assert!(items.is_empty()); // word_index == 0, should not complete
    }
}
