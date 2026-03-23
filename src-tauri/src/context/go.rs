//! Go context provider.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use super::{find_upward, ContextInfo, ContextProvider, ContextValue};

pub struct GoProvider;

impl ContextProvider for GoProvider {
    fn name(&self) -> &str {
        "go"
    }

    fn markers(&self) -> &[&str] {
        &["go.mod"]
    }

    fn gather(&self, project_root: &Path, _cwd: &Path) -> Result<ContextInfo> {
        let mut data = HashMap::new();
        let mut detected_markers = Vec::new();

        if let Some(go_mod_path) = find_upward(project_root, "go.mod") {
            detected_markers.push(go_mod_path.clone());

            if let Ok(content) = std::fs::read_to_string(&go_mod_path) {
                // Parse module name
                for line in content.lines() {
                    let line = line.trim();
                    if let Some(module) = line.strip_prefix("module ") {
                        data.insert(
                            "module".into(),
                            ContextValue::String(module.trim().to_string()),
                        );
                    }
                    if let Some(version) = line.strip_prefix("go ") {
                        data.insert(
                            "go_version".into(),
                            ContextValue::String(version.trim().to_string()),
                        );
                    }
                }

                // Parse dependencies from require block
                let mut in_require = false;
                let mut deps = Vec::new();
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed == "require (" {
                        in_require = true;
                        continue;
                    }
                    if trimmed == ")" && in_require {
                        in_require = false;
                        continue;
                    }
                    if in_require && !trimmed.starts_with("//") && !trimmed.is_empty() {
                        // "github.com/foo/bar v1.2.3" -> "github.com/foo/bar"
                        if let Some(dep) = trimmed.split_whitespace().next() {
                            deps.push(dep.to_string());
                        }
                    }
                    // Single-line require
                    if let Some(rest) = trimmed.strip_prefix("require ") {
                        if !rest.starts_with('(') {
                            if let Some(dep) = rest.split_whitespace().next() {
                                deps.push(dep.to_string());
                            }
                        }
                    }
                }
                if !deps.is_empty() {
                    data.insert("dependencies".into(), ContextValue::List(deps));
                }
            }

            if let Some(root) = go_mod_path.parent() {
                data.insert(
                    "root".into(),
                    ContextValue::String(root.to_string_lossy().into_owned()),
                );

                // Check for vendor directory
                data.insert(
                    "has_vendor".into(),
                    ContextValue::Bool(root.join("vendor").is_dir()),
                );

                // Check for go.sum
                data.insert(
                    "has_go_sum".into(),
                    ContextValue::Bool(root.join("go.sum").exists()),
                );
            }
        }

        Ok(ContextInfo {
            provider: "go".into(),
            data,
            detected_markers,
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        vec!["go".into()]
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec!["go.mod".into(), "go.sum".into()]
    }

    fn priority(&self) -> u32 {
        150
    }
}
