//! Plugin manifest format.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Plugin manifest (plugin.toml).
#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub license: String,

    /// Plugin type: "completion", "context", or "both"
    #[serde(default = "default_plugin_type")]
    pub plugin_type: String,

    /// Completion TOML files provided by this plugin
    #[serde(default)]
    pub completions: Vec<String>,

    /// Path to the plugin directory
    #[serde(skip)]
    pub path: PathBuf,
}

fn default_plugin_type() -> String {
    "completion".into()
}

impl PluginManifest {
    /// Load a plugin manifest from a directory.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let manifest_path = dir.join("plugin.toml");
        let content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read plugin manifest: {e}"))?;

        let mut manifest: Self = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse plugin manifest: {e}"))?;

        manifest.path = dir.to_path_buf();
        Ok(manifest)
    }
}
