//! Tauri IPC commands for configuration and theming.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::config::{self, AppConfig, Theme};

#[tauri::command]
pub fn get_config() -> AppConfig {
    AppConfig::load()
}

#[tauri::command]
pub fn set_config(config: AppConfig) -> Result<(), String> {
    config.save()
}

fn resolve_themes_dir(handle: &AppHandle) -> PathBuf {
    // 1. Try resource directory (bundled)
    if let Ok(dir) = handle.path().resource_dir() {
        let path = dir.join("themes");
        if path.exists() {
            return path;
        }
    }
    // 2. Try relative to CWD (dev mode)
    let path = PathBuf::from("themes");
    if path.exists() {
        return path;
    }
    // 3. Try parent then themes (tauri dev mode from src-tauri)
    let path = PathBuf::from("..").join("themes");
    if path.exists() {
        return path;
    }
    
    PathBuf::from("themes") // Fallback
}

#[tauri::command]
pub fn list_themes(handle: AppHandle) -> Vec<crate::config::ThemeInfo> {
    config::list_themes(&resolve_themes_dir(&handle))
}


#[tauri::command]
pub fn get_theme(handle: AppHandle, name: String) -> Result<Theme, String> {
    Theme::load_by_name(&resolve_themes_dir(&handle), &name)
}


