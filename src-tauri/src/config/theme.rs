//! Theme definition and loading.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A complete terminal theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    #[serde(default)]
    pub author: String,
    pub colors: ThemeColors,
}

/// Theme color palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub foreground: String,
    pub background: String,
    pub cursor: String,
    #[serde(default = "default_selection_bg")]
    pub selection_bg: String,
    #[serde(default = "default_selection_fg")]
    pub selection_fg: String,

    // Standard 16 ANSI colors
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,

    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,
}

fn default_selection_bg() -> String {
    "#45475a".into()
}
fn default_selection_fg() -> String {
    "#cdd6f4".into()
}

/// Wrapper for TOML file structure.
#[derive(Debug, Deserialize)]
struct ThemeFile {
    theme: ThemeFileDef,
}

#[derive(Debug, Deserialize)]
struct ThemeFileDef {
    name: String,
    #[serde(default)]
    author: String,
    colors: ThemeColors,
}

impl Theme {
    /// Load a theme from a TOML file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read theme: {e}"))?;
        let file: ThemeFile =
            toml::from_str(&content).map_err(|e| format!("Failed to parse theme: {e}"))?;

        Ok(Theme {
            name: file.theme.name,
            author: file.theme.author,
            colors: file.theme.colors,
        })
    }

    /// Load a theme by name from the themes directory.
    pub fn load_by_name(themes_dir: &Path, name: &str) -> Result<Self, String> {
        let path = themes_dir.join(format!("{name}.toml"));
        Self::load(&path)
    }
}
