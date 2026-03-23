//! Tauri IPC commands for configuration and theming.

use std::path::PathBuf;

use crate::config::{self, AppConfig, Theme};

#[tauri::command]
pub fn get_config() -> AppConfig {
    AppConfig::load()
}

#[tauri::command]
pub fn set_config(config: AppConfig) -> Result<(), String> {
    config.save()
}

#[tauri::command]
pub fn list_themes() -> Vec<String> {
    config::list_themes(&PathBuf::from("themes"))
}

#[tauri::command]
pub fn get_theme(name: String) -> Result<Theme, String> {
    Theme::load_by_name(&PathBuf::from("themes"), &name)
}
