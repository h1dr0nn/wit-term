//! Git context provider.

use std::collections::HashMap;
use std::path::Path;

use std::time::Duration;

use anyhow::Result;

use super::{find_upward, ContextInfo, ContextProvider, ContextValue};

pub struct GitProvider;

impl ContextProvider for GitProvider {
    fn name(&self) -> &str {
        "git"
    }

    fn markers(&self) -> &[&str] {
        &[".git"]
    }

    fn gather(&self, project_root: &Path, _cwd: &Path) -> Result<ContextInfo> {
        let mut data = HashMap::new();
        let mut detected_markers = Vec::new();

        if let Some(git_dir) = find_upward(project_root, ".git") {
            detected_markers.push(git_dir.clone());

            let git_path = if git_dir.is_file() {
                // Worktree: .git is a file pointing to the actual git dir
                if let Ok(content) = std::fs::read_to_string(&git_dir) {
                    if let Some(path) = content.strip_prefix("gitdir: ") {
                        git_dir.parent().unwrap().join(path.trim())
                    } else {
                        git_dir
                    }
                } else {
                    git_dir
                }
            } else {
                git_dir
            };

            // Read branch from HEAD
            let head_path = git_path.join("HEAD");
            if let Ok(head) = std::fs::read_to_string(head_path) {
                let head = head.trim();
                if let Some(branch) = head.strip_prefix("ref: refs/heads/") {
                    data.insert("branch".into(), ContextValue::String(branch.to_string()));
                    data.insert("detached".into(), ContextValue::Bool(false));
                } else {
                    // Detached HEAD
                    let short = &head[..8.min(head.len())];
                    data.insert("branch".into(), ContextValue::String(short.to_string()));
                    data.insert("detached".into(), ContextValue::Bool(true));
                    data.insert(
                        "commit_short".into(),
                        ContextValue::String(short.to_string()),
                    );
                }
            }

            // Determine repo root
            if let Some(parent) = git_path.parent() {
                data.insert(
                    "root".into(),
                    ContextValue::String(parent.to_string_lossy().into_owned()),
                );
            }

            // Check status with porcelain
            let status = std::process::Command::new("git")
                .args(["status", "--porcelain", "--short"])
                .current_dir(project_root)
                .output();

            match status {
                Ok(output) if output.status.success() => {
                    let out = String::from_utf8_lossy(&output.stdout);
                    if out.trim().is_empty() {
                        data.insert("status".into(), ContextValue::String("clean".into()));
                        data.insert("modified_count".into(), ContextValue::Number(0.0));
                        data.insert("staged_count".into(), ContextValue::Number(0.0));
                        data.insert("untracked_count".into(), ContextValue::Number(0.0));
                    } else {
                        data.insert("status".into(), ContextValue::String("dirty".into()));

                        let mut modified = 0u32;
                        let mut staged = 0u32;
                        let mut untracked = 0u32;
                        for line in out.lines() {
                            if line.len() < 2 {
                                continue;
                            }
                            let index = line.as_bytes()[0];
                            let worktree = line.as_bytes()[1];

                            if line.starts_with("??") {
                                untracked += 1;
                            } else {
                                if index != b' ' && index != b'?' {
                                    staged += 1;
                                }
                                if worktree != b' ' && worktree != b'?' {
                                    modified += 1;
                                }
                            }
                        }
                        data.insert("modified_count".into(), ContextValue::Number(modified as f64));
                        data.insert("staged_count".into(), ContextValue::Number(staged as f64));
                        data.insert(
                            "untracked_count".into(),
                            ContextValue::Number(untracked as f64),
                        );
                    }
                }
                _ => {
                    data.insert("status".into(), ContextValue::String("unknown".into()));
                }
            }

            // Remote info
            let remote_out = std::process::Command::new("git")
                .args(["remote", "-v"])
                .current_dir(project_root)
                .output();
            if let Ok(output) = remote_out {
                if output.status.success() {
                    let out = String::from_utf8_lossy(&output.stdout);
                    if let Some(first_line) = out.lines().next() {
                        let parts: Vec<&str> = first_line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            data.insert(
                                "remote_name".into(),
                                ContextValue::String(parts[0].to_string()),
                            );
                            data.insert(
                                "remote".into(),
                                ContextValue::String(parts[1].to_string()),
                            );
                        }
                    }
                }
            }

            // Ahead/behind
            let ab_out = std::process::Command::new("git")
                .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
                .current_dir(project_root)
                .output();
            if let Ok(output) = ab_out {
                if output.status.success() {
                    let out = String::from_utf8_lossy(&output.stdout);
                    let parts: Vec<&str> = out.trim().split('\t').collect();
                    if parts.len() == 2 {
                        if let Ok(ahead) = parts[0].parse::<f64>() {
                            data.insert("ahead".into(), ContextValue::Number(ahead));
                        }
                        if let Ok(behind) = parts[1].parse::<f64>() {
                            data.insert("behind".into(), ContextValue::Number(behind));
                        }
                    }
                }
            }

            // Stash count
            let stash_out = std::process::Command::new("git")
                .args(["stash", "list"])
                .current_dir(project_root)
                .output();
            if let Ok(output) = stash_out {
                if output.status.success() {
                    let count = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .count();
                    data.insert("stash_count".into(), ContextValue::Number(count as f64));
                }
            }

            // Merge/rebase state
            data.insert(
                "is_merge".into(),
                ContextValue::Bool(git_path.join("MERGE_HEAD").exists()),
            );
            data.insert(
                "is_rebase".into(),
                ContextValue::Bool(
                    git_path.join("rebase-merge").exists()
                        || git_path.join("rebase-apply").exists(),
                ),
            );
            data.insert(
                "has_conflicts".into(),
                ContextValue::Bool(git_path.join("MERGE_HEAD").exists()),
            );
        }

        Ok(ContextInfo {
            provider: "git".into(),
            data,
            detected_markers,
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        vec!["git".into()]
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec![
            ".git/HEAD".into(),
            ".git/index".into(),
            ".git/refs".into(),
        ]
    }

    fn priority(&self) -> u32 {
        200
    }

    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(5)
    }
}
