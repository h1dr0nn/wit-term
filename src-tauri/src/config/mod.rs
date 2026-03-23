//! Configuration and theming.

mod theme;

pub use theme::{Theme, ThemeColors};

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_cursor_style")]
    pub cursor_style: String,
    #[serde(default = "default_true")]
    pub cursor_blink: bool,
    #[serde(default = "default_scrollback")]
    pub scrollback_size: usize,
    #[serde(default = "default_true")]
    pub sidebar_visible: bool,
}

fn default_font_family() -> String {
    "Cascadia Code, JetBrains Mono, Fira Code, Consolas, monospace".into()
}
fn default_font_size() -> f32 {
    14.0
}
fn default_theme() -> String {
    "wit-dark".into()
}
fn default_cursor_style() -> String {
    "block".into()
}
fn default_true() -> bool {
    true
}
fn default_scrollback() -> usize {
    10000
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            font_family: default_font_family(),
            font_size: default_font_size(),
            theme: default_theme(),
            cursor_style: default_cursor_style(),
            cursor_blink: default_true(),
            scrollback_size: default_scrollback(),
            sidebar_visible: default_true(),
        }
    }
}

impl AppConfig {
    /// Load config from the default path.
    pub fn load() -> Self {
        let path = config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => toml::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Save config to the default path.
    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {e}"))?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {e}"))?;
        std::fs::write(&path, content).map_err(|e| format!("Failed to write config: {e}"))?;
        Ok(())
    }
}

/// Get the config file path.
fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("wit")
        .join("config.toml")
}

/// List available themes from the themes directory.
pub fn list_themes(themes_dir: &Path) -> Vec<String> {
    let mut themes = Vec::new();
    if let Ok(entries) = std::fs::read_dir(themes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                if let Some(stem) = path.file_stem() {
                    themes.push(stem.to_string_lossy().into_owned());
                }
            }
        }
    }
    themes.sort();
    themes
}
