//! Node.js context provider.

use std::collections::HashMap;
use std::path::Path;

use super::{find_upward, ContextInfo, ContextProvider};

pub struct NodeProvider;

impl ContextProvider for NodeProvider {
    fn name(&self) -> &str {
        "node"
    }

    fn detect(&self, dir: &Path) -> bool {
        find_upward(dir, "package.json").is_some()
    }

    fn gather(&self, dir: &Path) -> ContextInfo {
        let mut data = HashMap::new();

        if let Some(pkg_path) = find_upward(dir, "package.json") {
            if let Ok(content) = std::fs::read_to_string(&pkg_path) {
                if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(name) = pkg.get("name").and_then(|v| v.as_str()) {
                        data.insert("name".into(), name.to_string());
                    }
                    if let Some(version) = pkg.get("version").and_then(|v| v.as_str()) {
                        data.insert("version".into(), version.to_string());
                    }

                    // Collect script names
                    if let Some(scripts) = pkg.get("scripts").and_then(|v| v.as_object()) {
                        let script_names: Vec<&str> = scripts.keys().map(|s| s.as_str()).collect();
                        data.insert("scripts".into(), script_names.join(","));
                    }

                    // Detect package manager
                    if let Some(parent) = pkg_path.parent() {
                        if parent.join("pnpm-lock.yaml").exists() {
                            data.insert("package_manager".into(), "pnpm".into());
                        } else if parent.join("yarn.lock").exists() {
                            data.insert("package_manager".into(), "yarn".into());
                        } else if parent.join("bun.lockb").exists() {
                            data.insert("package_manager".into(), "bun".into());
                        } else {
                            data.insert("package_manager".into(), "npm".into());
                        }
                    }
                }
            }

            if let Some(root) = pkg_path.parent() {
                data.insert("root".into(), root.to_string_lossy().into_owned());
            }
        }

        ContextInfo {
            provider: "node".into(),
            detected: true,
            data,
        }
    }
}
