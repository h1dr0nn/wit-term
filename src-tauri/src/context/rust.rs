//! Rust/Cargo context provider.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use super::{find_upward, ContextInfo, ContextProvider, ContextValue};

pub struct RustProvider;

impl ContextProvider for RustProvider {
    fn name(&self) -> &str {
        "rust"
    }

    fn markers(&self) -> &[&str] {
        &["Cargo.toml"]
    }

    fn gather(&self, project_root: &Path, _cwd: &Path) -> Result<ContextInfo> {
        let mut data = HashMap::new();
        let mut detected_markers = Vec::new();

        if let Some(cargo_path) = find_upward(project_root, "Cargo.toml") {
            detected_markers.push(cargo_path.clone());

            if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                if let Ok(manifest) = content.parse::<toml::Table>() {
                    if let Some(package) = manifest.get("package").and_then(|v| v.as_table()) {
                        if let Some(name) = package.get("name").and_then(|v| v.as_str()) {
                            data.insert("name".into(), ContextValue::String(name.to_string()));
                        }
                        if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
                            data.insert(
                                "version".into(),
                                ContextValue::String(version.to_string()),
                            );
                        }
                        if let Some(edition) = package.get("edition").and_then(|v| v.as_str()) {
                            data.insert(
                                "edition".into(),
                                ContextValue::String(edition.to_string()),
                            );
                        }
                        if let Some(rust_version) =
                            package.get("rust-version").and_then(|v| v.as_str())
                        {
                            data.insert(
                                "rust_version".into(),
                                ContextValue::String(rust_version.to_string()),
                            );
                        }
                    }

                    // Check if it's a workspace
                    let is_workspace = manifest.contains_key("workspace");
                    data.insert("is_workspace".into(), ContextValue::Bool(is_workspace));

                    if is_workspace {
                        if let Some(workspace) =
                            manifest.get("workspace").and_then(|v| v.as_table())
                        {
                            if let Some(members) =
                                workspace.get("members").and_then(|v| v.as_array())
                            {
                                let member_list: Vec<String> = members
                                    .iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect();
                                data.insert(
                                    "workspace_members".into(),
                                    ContextValue::List(member_list),
                                );
                            }
                        }
                    }

                    // Dependencies
                    if let Some(deps) = manifest.get("dependencies").and_then(|v| v.as_table()) {
                        let dep_names: Vec<String> =
                            deps.keys().map(|s| s.to_string()).collect();
                        data.insert("dependencies".into(), ContextValue::List(dep_names));
                    }

                    // Features
                    if let Some(features) = manifest.get("features").and_then(|v| v.as_table()) {
                        let feature_names: Vec<String> =
                            features.keys().map(|s| s.to_string()).collect();
                        data.insert("features".into(), ContextValue::List(feature_names));
                    }

                    // Build script
                    let has_build_script = manifest
                        .get("package")
                        .and_then(|v| v.as_table())
                        .and_then(|p| p.get("build"))
                        .is_some();
                    data.insert(
                        "has_build_script".into(),
                        ContextValue::Bool(has_build_script),
                    );
                }
            }

            if let Some(root) = cargo_path.parent() {
                data.insert(
                    "root".into(),
                    ContextValue::String(root.to_string_lossy().into_owned()),
                );

                // Check for toolchain file
                let toolchain_path = root.join("rust-toolchain.toml");
                if toolchain_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&toolchain_path) {
                        if let Ok(table) = content.parse::<toml::Table>() {
                            if let Some(channel) = table
                                .get("toolchain")
                                .and_then(|v| v.as_table())
                                .and_then(|t| t.get("channel"))
                                .and_then(|v| v.as_str())
                            {
                                data.insert(
                                    "toolchain".into(),
                                    ContextValue::String(channel.to_string()),
                                );
                            }
                        }
                    }
                } else if root.join("rust-toolchain").exists() {
                    if let Ok(content) = std::fs::read_to_string(root.join("rust-toolchain")) {
                        data.insert(
                            "toolchain".into(),
                            ContextValue::String(content.trim().to_string()),
                        );
                    }
                }
            }
        }

        // Detect rustc runtime version
        if let Ok(output) = std::process::Command::new("rustc")
            .arg("--version")
            .output()
        {
            if output.status.success() {
                // "rustc 1.79.0 (129f3b996 2024-06-10)" -> "1.79.0"
                let out = String::from_utf8_lossy(&output.stdout);
                if let Some(ver) = out.split_whitespace().nth(1) {
                    data.insert(
                        "runtime_version".into(),
                        ContextValue::String(ver.to_string()),
                    );
                }
            }
        }

        Ok(ContextInfo {
            provider: "rust".into(),
            data,
            detected_markers,
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        vec!["cargo".into()]
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec![
            "Cargo.toml".into(),
            "Cargo.lock".into(),
            "rust-toolchain.toml".into(),
        ]
    }

    fn priority(&self) -> u32 {
        150
    }
}
