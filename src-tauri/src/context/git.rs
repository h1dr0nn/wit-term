//! Git context provider.

use std::collections::HashMap;
use std::path::Path;

use super::{find_upward, ContextInfo, ContextProvider};

pub struct GitProvider;

impl ContextProvider for GitProvider {
    fn name(&self) -> &str {
        "git"
    }

    fn detect(&self, dir: &Path) -> bool {
        find_upward(dir, ".git").is_some()
    }

    fn gather(&self, dir: &Path) -> ContextInfo {
        let mut data = HashMap::new();

        if let Some(git_dir) = find_upward(dir, ".git") {
            let git_path = if git_dir.is_file() {
                // Worktree: .git is a file pointing to the actual git dir
                if let Ok(content) = std::fs::read_to_string(&git_dir) {
                    if let Some(path) = content.strip_prefix("gitdir: ") {
                        Some(git_dir.parent().unwrap().join(path.trim()))
                    } else {
                        Some(git_dir)
                    }
                } else {
                    Some(git_dir)
                }
            } else {
                Some(git_dir)
            };

            if let Some(git_path) = git_path {
                // Read branch from HEAD
                let head_path = git_path.join("HEAD");
                if let Ok(head) = std::fs::read_to_string(head_path) {
                    let head = head.trim();
                    if let Some(branch) = head.strip_prefix("ref: refs/heads/") {
                        data.insert("branch".into(), branch.to_string());
                    } else {
                        // Detached HEAD
                        data.insert("branch".into(), head[..8.min(head.len())].to_string());
                        data.insert("detached".into(), "true".into());
                    }
                }

                // Determine repo root
                if let Some(parent) = git_path.parent() {
                    // If .git is a directory, parent is the repo root
                    // If it was found via find_upward, the parent of .git is the root
                    data.insert("root".into(), parent.to_string_lossy().into_owned());
                }
            }

            // Check dirty status (simple: look for uncommitted changes marker)
            let status = std::process::Command::new("git")
                .args(["status", "--porcelain", "--short"])
                .current_dir(dir)
                .output();

            match status {
                Ok(output) if output.status.success() => {
                    let out = String::from_utf8_lossy(&output.stdout);
                    data.insert(
                        "status".into(),
                        if out.trim().is_empty() {
                            "clean".into()
                        } else {
                            "dirty".into()
                        },
                    );
                }
                _ => {
                    data.insert("status".into(), "unknown".into());
                }
            }
        }

        ContextInfo {
            provider: "git".into(),
            detected: true,
            data,
        }
    }
}
