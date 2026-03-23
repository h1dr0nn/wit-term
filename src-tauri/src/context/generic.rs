//! Generic context provider.
//!
//! Detects general project tooling: Makefile, justfile, editorconfig,
//! environment files, CI systems, and IDE configurations.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use super::{ContextInfo, ContextProvider, ContextValue};

pub struct GenericProvider;

impl ContextProvider for GenericProvider {
    fn name(&self) -> &str {
        "generic"
    }

    fn markers(&self) -> &[&str] {
        &[
            "Makefile",
            ".editorconfig",
            ".env",
            ".envrc",
            "justfile",
            ".devcontainer",
        ]
    }

    fn detect(&self, dir: &Path) -> bool {
        self.markers().iter().any(|m| dir.join(m).exists())
    }

    fn gather(&self, project_root: &Path, _cwd: &Path) -> Result<ContextInfo> {
        let mut data = HashMap::new();
        let detected_markers = Vec::new();

        // Makefile
        let has_makefile = project_root.join("Makefile").exists()
            || project_root.join("makefile").exists()
            || project_root.join("GNUmakefile").exists();
        data.insert("has_makefile".into(), ContextValue::Bool(has_makefile));

        if has_makefile {
            let makefile_path = if project_root.join("Makefile").exists() {
                project_root.join("Makefile")
            } else if project_root.join("makefile").exists() {
                project_root.join("makefile")
            } else {
                project_root.join("GNUmakefile")
            };
            if let Ok(content) = std::fs::read_to_string(&makefile_path) {
                let targets: Vec<String> = content
                    .lines()
                    .filter(|line| {
                        !line.starts_with('\t')
                            && !line.starts_with('#')
                            && !line.starts_with('.')
                            && line.contains(':')
                            && !line.contains(":=")
                            && !line.contains("?=")
                    })
                    .filter_map(|line| {
                        line.split(':').next().map(|t| t.trim().to_string())
                    })
                    .filter(|t| !t.is_empty() && !t.contains(' '))
                    .collect();
                if !targets.is_empty() {
                    data.insert("make_targets".into(), ContextValue::List(targets));
                }
            }
        }

        // Justfile
        let has_justfile = project_root.join("justfile").exists()
            || project_root.join("Justfile").exists();
        data.insert("has_justfile".into(), ContextValue::Bool(has_justfile));

        if has_justfile {
            let justfile_path = if project_root.join("justfile").exists() {
                project_root.join("justfile")
            } else {
                project_root.join("Justfile")
            };
            if let Ok(content) = std::fs::read_to_string(&justfile_path) {
                let recipes: Vec<String> = content
                    .lines()
                    .filter(|line| {
                        !line.starts_with(' ')
                            && !line.starts_with('\t')
                            && !line.starts_with('#')
                            && !line.is_empty()
                            && line.contains(':')
                    })
                    .filter_map(|line| {
                        let name = line.split(':').next()?.trim();
                        if name.is_empty() || name.contains(' ') {
                            None
                        } else {
                            Some(name.to_string())
                        }
                    })
                    .collect();
                if !recipes.is_empty() {
                    data.insert("just_recipes".into(), ContextValue::List(recipes));
                }
            }
        }

        // Editorconfig
        data.insert(
            "has_editorconfig".into(),
            ContextValue::Bool(project_root.join(".editorconfig").exists()),
        );

        // Environment files
        data.insert(
            "has_env".into(),
            ContextValue::Bool(project_root.join(".env").exists()),
        );
        data.insert(
            "has_envrc".into(),
            ContextValue::Bool(project_root.join(".envrc").exists()),
        );

        // Devcontainer
        data.insert(
            "has_devcontainer".into(),
            ContextValue::Bool(
                project_root.join(".devcontainer").is_dir()
                    || project_root.join(".devcontainer.json").exists(),
            ),
        );

        // IDE detection
        data.insert(
            "has_vscode".into(),
            ContextValue::Bool(project_root.join(".vscode").is_dir()),
        );
        data.insert(
            "has_idea".into(),
            ContextValue::Bool(project_root.join(".idea").is_dir()),
        );

        // CI system detection
        let ci_system = if project_root.join(".github/workflows").is_dir() {
            Some("github-actions")
        } else if project_root.join(".gitlab-ci.yml").exists() {
            Some("gitlab-ci")
        } else if project_root.join(".circleci").is_dir() {
            Some("circleci")
        } else if project_root.join("Jenkinsfile").exists() {
            Some("jenkins")
        } else if project_root.join(".travis.yml").exists() {
            Some("travis")
        } else if project_root.join("azure-pipelines.yml").exists() {
            Some("azure-devops")
        } else if project_root.join("bitbucket-pipelines.yml").exists() {
            Some("bitbucket")
        } else {
            None
        };
        if let Some(ci) = ci_system {
            data.insert(
                "ci_system".into(),
                ContextValue::String(ci.to_string()),
            );
        }

        Ok(ContextInfo {
            provider: "generic".into(),
            data,
            detected_markers,
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        vec!["make".into()]
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec!["Makefile".into(), "justfile".into()]
    }

    fn priority(&self) -> u32 {
        50
    }
}
