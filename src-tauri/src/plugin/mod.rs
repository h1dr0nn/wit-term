//! Plugin system.
//!
//! Provides a trait-based plugin API for extending Wit with custom
//! completion sources, context providers, and event hooks.

pub mod loader;
pub mod manifest;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::completion::{CompletionItem, CompletionSource};
use crate::completion::parser::ParsedInput;
use crate::context::ContextProvider;

/// Plugin lifecycle trait.
pub trait WitPlugin: Send + Sync {
    /// Plugin name (must match manifest).
    fn name(&self) -> &str;

    /// Plugin version.
    fn version(&self) -> &str;

    /// Called when the plugin is loaded.
    fn on_load(&mut self, _api: &PluginApi) {}

    /// Called when the plugin is unloaded.
    fn on_unload(&mut self) {}

    /// Return completion sources provided by this plugin.
    fn completion_sources(&self) -> Vec<Box<dyn CompletionSource>> {
        Vec::new()
    }

    /// Return context providers provided by this plugin.
    fn context_providers(&self) -> Vec<Box<dyn ContextProvider>> {
        Vec::new()
    }
}

/// API available to plugins.
pub struct PluginApi {
    pub config_dir: PathBuf,
    pub themes_dir: PathBuf,
    pub completions_dir: PathBuf,
}

impl PluginApi {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wit");
        Self {
            themes_dir: PathBuf::from("themes"),
            completions_dir: PathBuf::from("completions"),
            config_dir,
        }
    }
}

impl Default for PluginApi {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages loaded plugins.
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn WitPlugin>>,
    api: PluginApi,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            api: PluginApi::new(),
        }
    }

    /// Load all plugins from the plugins directory.
    pub fn load_plugins(&mut self) -> Vec<String> {
        let plugin_dir = self.api.config_dir.join("plugins");
        if !plugin_dir.exists() {
            return Vec::new();
        }

        let manifests = loader::discover_plugins(&plugin_dir);
        let mut loaded = Vec::new();

        for manifest in manifests {
            log::info!("Found plugin: {} v{}", manifest.name, manifest.version);
            // For now, plugins are TOML-only completion/context providers
            if let Some(plugin) = loader::load_toml_plugin(&manifest) {
                let name = manifest.name.clone();
                self.register_plugin(Box::new(plugin));
                loaded.push(name);
            }
        }

        loaded
    }

    /// Register a plugin.
    pub fn register_plugin(&mut self, mut plugin: Box<dyn WitPlugin>) {
        plugin.on_load(&self.api);
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    /// Unload all plugins.
    pub fn unload_all(&mut self) {
        for (_, plugin) in self.plugins.drain() {
            let mut p = plugin;
            p.on_unload();
        }
    }

    /// Get all completion sources from loaded plugins.
    pub fn completion_sources(&self) -> Vec<Box<dyn CompletionSource>> {
        self.plugins
            .values()
            .flat_map(|p| p.completion_sources())
            .collect()
    }

    /// Get all context providers from loaded plugins.
    pub fn context_providers(&self) -> Vec<Box<dyn ContextProvider>> {
        self.plugins
            .values()
            .flat_map(|p| p.context_providers())
            .collect()
    }

    /// List loaded plugins.
    pub fn list_plugins(&self) -> Vec<(&str, &str)> {
        self.plugins
            .values()
            .map(|p| (p.name(), p.version()))
            .collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A TOML-based plugin that provides additional completions.
pub struct TomlPlugin {
    pub name: String,
    pub version: String,
    completion_items: Vec<(String, Vec<CompletionItem>)>,
}

impl WitPlugin for TomlPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn completion_sources(&self) -> Vec<Box<dyn CompletionSource>> {
        vec![Box::new(TomlPluginSource {
            name: self.name.clone(),
            items: self.completion_items.clone(),
        })]
    }
}

struct TomlPluginSource {
    name: String,
    items: Vec<(String, Vec<CompletionItem>)>,
}

impl CompletionSource for TomlPluginSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn complete(&self, parsed: &ParsedInput, _cwd: &Path) -> Vec<CompletionItem> {
        // Find matching command
        for (cmd, items) in &self.items {
            if *cmd == parsed.command {
                return items.clone();
            }
        }
        Vec::new()
    }
}
