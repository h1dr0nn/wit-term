//! Context detection engine.
//!
//! Scans the current working directory to detect project environments
//! (git, Node.js, Rust, Python, Docker, Go, Java) and gathers contextual info.
//! Supports caching with TTL and file system watching for invalidation.

mod docker;
mod generic;
mod git;
mod go;
mod java;
mod node;
mod python;
mod rust;

use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// Typed context value supporting multiple data types.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ContextValue {
    String(String),
    Bool(bool),
    Number(f64),
    List(Vec<String>),
    Map(HashMap<String, String>),
}

/// Information gathered by a single context provider.
#[derive(Debug, Clone, Serialize)]
pub struct ContextInfo {
    pub provider: String,
    pub data: HashMap<String, ContextValue>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub detected_markers: Vec<PathBuf>,
}

/// Aggregated project context for a directory.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectContext {
    pub project_root: Option<PathBuf>,
    pub cwd: PathBuf,
    pub providers: HashMap<String, ContextInfo>,
    #[serde(serialize_with = "serialize_system_time")]
    pub last_updated: SystemTime,
    pub completion_sets: Vec<String>,
}

fn serialize_system_time<S>(time: &SystemTime, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    serializer.serialize_u64(duration.as_secs())
}

/// Events emitted by the context engine.
#[derive(Debug, Clone)]
pub enum ContextEvent {
    /// Context has changed (new scan results).
    ContextChanged(ProjectContext),
    /// Working directory changed.
    CwdChanged(PathBuf),
    /// A provider was activated.
    ProviderActivated(String),
    /// A provider was deactivated.
    ProviderDeactivated(String),
}

/// Trait for context providers.
pub trait ContextProvider: Send + Sync {
    /// Provider name (e.g. "git", "node").
    fn name(&self) -> &str;

    /// File/directory markers that indicate this provider is relevant.
    fn markers(&self) -> &[&str] {
        &[]
    }

    /// Check if the provider is relevant for the given directory.
    fn detect(&self, project_root: &Path) -> bool {
        self.markers().iter().any(|m| project_root.join(m).exists())
    }

    /// Gather context information from the project.
    fn gather(&self, project_root: &Path, cwd: &Path) -> Result<ContextInfo>;

    /// Completion sets this provider activates (e.g. ["git"], ["npm", "node"]).
    fn completion_sets(&self) -> Vec<String> {
        vec![]
    }

    /// File patterns to watch for changes.
    fn watch_patterns(&self) -> Vec<String> {
        vec![]
    }

    /// Priority (higher = scanned first). Default 100.
    fn priority(&self) -> u32 {
        100
    }

    /// Cache TTL for this provider's results.
    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(60)
    }
}

/// Cached context result for a single provider.
struct CachedContext {
    info: ContextInfo,
    gathered_at: Instant,
    ttl: Duration,
}

impl CachedContext {
    fn is_valid(&self) -> bool {
        self.gathered_at.elapsed() < self.ttl
    }
}

/// The context engine holds all providers and scans directories with caching.
pub struct ContextEngine {
    providers: Vec<Box<dyn ContextProvider>>,
    cache: HashMap<String, CachedContext>,
    last_project_root: Option<PathBuf>,
}

impl ContextEngine {
    pub fn new() -> Self {
        let mut providers: Vec<Box<dyn ContextProvider>> = vec![
            Box::new(git::GitProvider),
            Box::new(node::NodeProvider),
            Box::new(rust::RustProvider),
            Box::new(python::PythonProvider),
            Box::new(docker::DockerProvider),
            Box::new(go::GoProvider),
            Box::new(java::JavaProvider),
            Box::new(generic::GenericProvider),
        ];
        // Sort by priority descending (higher priority first)
        providers.sort_by(|a, b| b.priority().cmp(&a.priority()));
        Self {
            providers,
            cache: HashMap::new(),
            last_project_root: None,
        }
    }

    /// Find the project root by walking up from `cwd` and checking all provider markers.
    fn find_project_root(&self, cwd: &Path) -> Option<PathBuf> {
        let all_markers: Vec<&str> = self
            .providers
            .iter()
            .flat_map(|p| p.markers().to_vec())
            .collect();

        // Walk up from cwd, find the nearest directory that contains any marker
        let mut current = cwd.to_path_buf();
        loop {
            for marker in &all_markers {
                if current.join(marker).exists() {
                    return Some(current);
                }
            }
            if !current.pop() {
                return None;
            }
        }
    }

    /// Invalidate cache for a specific provider.
    pub fn invalidate(&mut self, provider_name: &str) {
        self.cache.remove(provider_name);
    }

    /// Invalidate all cached contexts.
    pub fn invalidate_all(&mut self) {
        self.cache.clear();
    }

    /// Scan a directory and return the aggregated project context.
    /// Uses cached results when available and valid.
    pub fn scan(&mut self, cwd: &Path) -> ProjectContext {
        let project_root = self.find_project_root(cwd);
        let scan_dir = project_root.as_deref().unwrap_or(cwd);

        // If project root changed, invalidate all caches
        if self.last_project_root.as_deref() != project_root.as_deref() {
            self.invalidate_all();
            self.last_project_root = project_root.clone();
        }

        let mut providers = HashMap::new();
        let mut completion_sets = Vec::new();

        for provider in &self.providers {
            if provider.detect(scan_dir) {
                let name = provider.name().to_string();

                // Check cache first
                if let Some(cached) = self.cache.get(&name) {
                    if cached.is_valid() {
                        completion_sets.extend(provider.completion_sets());
                        providers.insert(name, cached.info.clone());
                        continue;
                    }
                }

                // Cache miss or expired — gather fresh data
                match provider.gather(scan_dir, cwd) {
                    Ok(info) => {
                        completion_sets.extend(provider.completion_sets());
                        providers.insert(name.clone(), info.clone());

                        // Store in cache
                        self.cache.insert(
                            name,
                            CachedContext {
                                info,
                                gathered_at: Instant::now(),
                                ttl: provider.cache_ttl(),
                            },
                        );
                    }
                    Err(e) => {
                        log::warn!("Provider {} failed: {}", provider.name(), e);
                    }
                }
            }
        }

        completion_sets.dedup();

        ProjectContext {
            project_root,
            cwd: cwd.to_path_buf(),
            providers,
            last_updated: SystemTime::now(),
            completion_sets,
        }
    }

    /// Get provider names.
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.iter().map(|p| p.name()).collect()
    }

    /// Get all watch patterns from active providers (for file watcher setup).
    pub fn watch_patterns(&self) -> Vec<String> {
        self.providers
            .iter()
            .flat_map(|p| p.watch_patterns())
            .collect()
    }
}

impl Default for ContextEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Walk up from `dir` to find a file/directory by name.
pub(crate) fn find_upward(dir: &Path, name: &str) -> Option<PathBuf> {
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
        assert_eq!(engine.provider_names().len(), 8);
    }

    #[test]
    fn test_scan_nonexistent_dir() {
        let mut engine = ContextEngine::new();
        let ctx = engine.scan(Path::new("/nonexistent/path/12345"));
        assert!(ctx.providers.is_empty());
    }

    #[test]
    fn test_project_context_has_required_fields() {
        let mut engine = ContextEngine::new();
        let ctx = engine.scan(Path::new("/nonexistent/path/12345"));
        assert!(ctx.project_root.is_none());
        assert!(ctx.completion_sets.is_empty());
        // last_updated should be recent
        assert!(ctx.last_updated.elapsed().unwrap().as_secs() < 1);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut engine = ContextEngine::new();
        engine.cache.insert(
            "test".into(),
            CachedContext {
                info: ContextInfo {
                    provider: "test".into(),
                    data: HashMap::new(),
                    detected_markers: vec![],
                },
                gathered_at: Instant::now(),
                ttl: Duration::from_secs(60),
            },
        );
        assert!(engine.cache.contains_key("test"));
        engine.invalidate("test");
        assert!(!engine.cache.contains_key("test"));
    }

    #[test]
    fn test_cache_expiry() {
        let cached = CachedContext {
            info: ContextInfo {
                provider: "test".into(),
                data: HashMap::new(),
                detected_markers: vec![],
            },
            gathered_at: Instant::now() - Duration::from_secs(120),
            ttl: Duration::from_secs(60),
        };
        assert!(!cached.is_valid());
    }

    #[test]
    fn test_watch_patterns() {
        let engine = ContextEngine::new();
        let patterns = engine.watch_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns.contains(&".git/HEAD".to_string()));
    }
}
