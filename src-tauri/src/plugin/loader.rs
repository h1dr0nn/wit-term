//! Plugin discovery and loading.

use std::path::Path;

use super::manifest::PluginManifest;
use super::TomlPlugin;
use crate::completion::CompletionItem;

/// Discover plugins in a directory.
pub fn discover_plugins(dir: &Path) -> Vec<PluginManifest> {
    let mut manifests = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("plugin.toml");
                if manifest_path.exists() {
                    match PluginManifest::load(&path) {
                        Ok(manifest) => manifests.push(manifest),
                        Err(e) => {
                            log::warn!("Failed to load plugin from {}: {e}", path.display());
                        }
                    }
                }
            }
        }
    }

    manifests
}

/// Load a TOML-based completion plugin.
pub fn load_toml_plugin(manifest: &PluginManifest) -> Option<TomlPlugin> {
    let mut completion_items: Vec<(String, Vec<CompletionItem>)> = Vec::new();

    for completion_file in &manifest.completions {
        let path = manifest.path.join(completion_file);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Parse as a simple completion definition
                if let Ok(file) = toml::from_str::<SimpleCompletionFile>(&content) {
                    let items: Vec<CompletionItem> = file
                        .completions
                        .into_iter()
                        .map(|c| CompletionItem {
                            text: c.text,
                            display: c.display.unwrap_or_default(),
                            description: c.description.unwrap_or_default(),
                            kind: crate::completion::CompletionKind::Command,
                            score: 0.5,
                        })
                        .collect();
                    completion_items.push((file.command, items));
                }
            }
        }
    }

    Some(TomlPlugin {
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        completion_items,
    })
}

#[derive(serde::Deserialize)]
struct SimpleCompletionFile {
    command: String,
    #[serde(default)]
    completions: Vec<SimpleCompletion>,
}

#[derive(serde::Deserialize)]
struct SimpleCompletion {
    text: String,
    display: Option<String>,
    description: Option<String>,
}
