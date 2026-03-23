//! Context detection engine.
//!
//! Scans the current working directory to detect project environments
//! (git, Node.js, Rust, Python, Docker) and gathers contextual info.

mod docker;
mod git;
mod node;
mod python;
mod rust;

use serde::Serialize;
use std::path::{Path, PathBuf};

/// Information gathered by a single context provider.
#[derive(Debug, Clone, Serialize)]
pub struct ContextInfo {
    pub provider: String,
    pub detected: bool,
    #[serde(flatten)]
    pub data: std::collections::HashMap<String, String>,
}

/// Aggregated project context for a directory.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectContext {
    pub cwd: String,
    pub providers: Vec<ContextInfo>,
}

/// Trait for context providers.
pub trait ContextProvider: Send + Sync {
    fn name(&self) -> &str;
    fn detect(&self, dir: &Path) -> bool;
    fn gather(&self, dir: &Path) -> ContextInfo;
}

/// The context engine holds all providers and scans directories.
pub struct ContextEngine {
    providers: Vec<Box<dyn ContextProvider>>,
}

impl ContextEngine {
    pub fn new() -> Self {
        let providers: Vec<Box<dyn ContextProvider>> = vec![
            Box::new(git::GitProvider),
            Box::new(node::NodeProvider),
            Box::new(rust::RustProvider),
            Box::new(python::PythonProvider),
            Box::new(docker::DockerProvider),
        ];
        Self { providers }
    }

    /// Scan a directory and return the aggregated project context.
    pub fn scan(&self, dir: &Path) -> ProjectContext {
        let providers: Vec<ContextInfo> = self
            .providers
            .iter()
            .filter(|p| p.detect(dir))
            .map(|p| p.gather(dir))
            .collect();

        ProjectContext {
            cwd: dir.to_string_lossy().into_owned(),
            providers,
        }
    }

    /// Get provider names.
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.iter().map(|p| p.name()).collect()
    }
}

impl Default for ContextEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Walk up from `dir` to find a file/directory by name.
fn find_upward(dir: &Path, name: &str) -> Option<PathBuf> {
    let mut current = dir.to_path_buf();
    loop {
        let candidate = current.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_engine_creation() {
        let engine = ContextEngine::new();
        assert_eq!(engine.provider_names().len(), 5);
    }

    #[test]
    fn test_scan_nonexistent_dir() {
        let engine = ContextEngine::new();
        let ctx = engine.scan(Path::new("/nonexistent/path/12345"));
        assert!(ctx.providers.is_empty());
    }
}
