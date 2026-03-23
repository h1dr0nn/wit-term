//! Completion engine.
//!
//! Provides context-aware tab completions from multiple sources:
//! static TOML definitions, filesystem paths, and fuzzy matching.

mod context_source;
mod fuzzy;
pub mod parser;
mod path_source;
mod static_source;

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A completion request from the frontend.
#[derive(Debug, Clone, Deserialize)]
pub struct CompletionRequest {
    pub input: String,
    pub cursor_pos: usize,
    pub cwd: String,
}

/// A single completion item.
#[derive(Debug, Clone, Serialize)]
pub struct CompletionItem {
    pub text: String,
    pub display: String,
    pub description: String,
    pub kind: CompletionKind,
    pub score: f64,
}

/// The type of completion.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum CompletionKind {
    Command,
    Subcommand,
    Flag,
    Argument,
    Path,
}

/// Trait for completion sources.
pub trait CompletionSource: Send + Sync {
    fn name(&self) -> &str;
    fn complete(&self, parsed: &parser::ParsedInput, cwd: &Path) -> Vec<CompletionItem>;
}

/// The main completion engine.
pub struct CompletionEngine {
    sources: Vec<Box<dyn CompletionSource>>,
}

impl CompletionEngine {
    pub fn new(completions_dir: &Path) -> Self {
        let mut sources: Vec<Box<dyn CompletionSource>> = Vec::new();

        // Load static TOML completions
        if let Ok(static_src) = static_source::StaticSource::load(completions_dir) {
            sources.push(Box::new(static_src));
        }

        // Path completion source
        sources.push(Box::new(path_source::PathSource));

        // Dynamic context-aware completions (git branches, npm scripts, etc.)
        sources.push(Box::new(context_source::ContextSource));

        Self { sources }
    }

    /// Process a completion request and return ranked results.
    pub fn complete(&self, request: &CompletionRequest) -> Vec<CompletionItem> {
        let parsed = parser::parse_input(&request.input, request.cursor_pos);
        let cwd = Path::new(&request.cwd);

        let mut items: Vec<CompletionItem> = self
            .sources
            .iter()
            .flat_map(|source| source.complete(&parsed, cwd))
            .collect();

        // Sort by score descending
        items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit to top 20
        items.truncate(20);
        items
    }
}

impl Default for CompletionEngine {
    fn default() -> Self {
        Self::new(Path::new("completions"))
    }
}
