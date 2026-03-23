//! Rust/Cargo context provider.

use std::collections::HashMap;
use std::path::Path;

use super::{find_upward, ContextInfo, ContextProvider};

pub struct RustProvider;

impl ContextProvider for RustProvider {
    fn name(&self) -> &str {
        "rust"
    }

    fn detect(&self, dir: &Path) -> bool {
        find_upward(dir, "Cargo.toml").is_some()
    }

    fn gather(&self, dir: &Path) -> ContextInfo {
        let mut data = HashMap::new();

        if let Some(cargo_path) = find_upward(dir, "Cargo.toml") {
            if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                if let Ok(manifest) = content.parse::<toml::Table>() {
                    if let Some(package) = manifest.get("package").and_then(|v| v.as_table()) {
                        if let Some(name) = package.get("name").and_then(|v| v.as_str()) {
                            data.insert("name".into(), name.to_string());
                        }
                        if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
                            data.insert("version".into(), version.to_string());
                        }
                        if let Some(edition) = package.get("edition").and_then(|v| v.as_str()) {
                            data.insert("edition".into(), edition.to_string());
                        }
                    }

                    // Check if it's a workspace
                    if manifest.contains_key("workspace") {
                        data.insert("workspace".into(), "true".into());
                    }
                }
            }

            if let Some(root) = cargo_path.parent() {
                data.insert("root".into(), root.to_string_lossy().into_owned());
            }
        }

        ContextInfo {
            provider: "rust".into(),
            detected: true,
            data,
        }
    }
}
