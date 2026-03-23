//! Plugin discovery and loading.

use std::path::Path;

use super::manifest::PluginManifest;
use super::TomlPlugin;
use crate::completion::CompletionItem;

/// Current Wit version for compatibility checking.
const WIT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
                        Ok(manifest) => {
                            // Check version compatibility
                            if manifest.is_compatible(WIT_VERSION) {
                                manifests.push(manifest);
                            } else {
                                log::warn!(
                                    "Plugin {} v{} requires wit {}, current is {}",
                                    manifest.name,
                                    manifest.version,
                                    manifest.wit_version.as_deref().unwrap_or("?"),
                                    WIT_VERSION,
                                );
                            }
                        }
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

    // Use provides.completions or legacy completions field
    let completion_files = if !manifest.provides.completions.is_empty() {
        &manifest.provides.completions
    } else {
        &manifest.completions
    };

    for completion_file in completion_files {
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

/// Discover theme files provided by a plugin.
pub fn load_plugin_themes(manifest: &PluginManifest) -> Vec<std::path::PathBuf> {
    manifest
        .provides
        .themes
        .iter()
        .map(|t| manifest.path.join(t))
        .filter(|p| p.exists())
        .collect()
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
