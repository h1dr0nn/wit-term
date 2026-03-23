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

    /// Minimum compatible Wit version (semver, e.g. ">=0.1.0").
    #[serde(default)]
    pub wit_version: Option<String>,

    /// Plugin type: "completion", "context", "theme", or "all"
    #[serde(default = "default_plugin_type")]
    pub plugin_type: String,

    /// What this plugin provides.
    #[serde(default)]
    pub provides: PluginProvides,

    /// Completion TOML files provided by this plugin (legacy field).
    #[serde(default)]
    pub completions: Vec<String>,

    /// Path to the plugin directory.
    #[serde(skip)]
    pub path: PathBuf,
}

/// What a plugin provides.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PluginProvides {
    /// Completion TOML files.
    #[serde(default)]
    pub completions: Vec<String>,
    /// Context provider identifiers.
    #[serde(default)]
    pub contexts: Vec<String>,
    /// Theme TOML files.
    #[serde(default)]
    pub themes: Vec<String>,
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

        // Merge legacy completions field into provides.completions
        if !manifest.completions.is_empty() && manifest.provides.completions.is_empty() {
            manifest.provides.completions = manifest.completions.clone();
        }

        Ok(manifest)
    }

    /// Check if this plugin is compatible with the current Wit version.
    pub fn is_compatible(&self, wit_version: &str) -> bool {
        match &self.wit_version {
            None => true,
            Some(constraint) => {
                // Simple >=X.Y.Z check
                if let Some(min_ver) = constraint.strip_prefix(">=") {
                    version_gte(wit_version, min_ver.trim())
                } else {
                    true // Unknown constraint format — allow
                }
            }
        }
    }
}

/// Simple semver greater-than-or-equal comparison.
fn version_gte(current: &str, minimum: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.split('.')
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };
    let cur = parse(current);
    let min = parse(minimum);

    for i in 0..3 {
        let c = cur.get(i).copied().unwrap_or(0);
        let m = min.get(i).copied().unwrap_or(0);
        if c > m {
            return true;
        }
        if c < m {
            return false;
        }
    }
    true // Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_gte() {
        assert!(version_gte("0.2.0", "0.1.0"));
        assert!(version_gte("0.1.0", "0.1.0"));
        assert!(!version_gte("0.0.9", "0.1.0"));
        assert!(version_gte("1.0.0", "0.9.9"));
    }

    #[test]
    fn test_manifest_compatibility() {
        let manifest = PluginManifest {
            name: "test".into(),
            version: "1.0.0".into(),
            description: String::new(),
            author: String::new(),
            license: String::new(),
            wit_version: Some(">=0.1.0".into()),
            plugin_type: "completion".into(),
            provides: PluginProvides::default(),
            completions: vec![],
            path: PathBuf::new(),
        };
        assert!(manifest.is_compatible("0.1.0"));
        assert!(manifest.is_compatible("0.2.0"));
        assert!(!manifest.is_compatible("0.0.9"));
    }
}
