//! Agent output adapters that parse terminal output into structured events.

mod claude_code;

use super::types::{AgentEvent, AgentKind};
use claude_code::ClaudeCodeAdapter;

/// Trait for parsing raw PTY output bytes into structured agent events.
pub trait AgentAdapter: Send {
    /// Human-readable adapter name.
    fn name(&self) -> &str;

    /// Parse a chunk of raw PTY output bytes and return any events found.
    fn parse_output(&mut self, data: &[u8]) -> Vec<AgentEvent>;

    /// Reset internal parser state (e.g. between agent sessions).
    fn reset(&mut self);
}

/// Create the appropriate adapter for a given agent kind.
/// Falls back to a generic adapter for unknown agents.
pub fn create_adapter(kind: &AgentKind) -> Box<dyn AgentAdapter> {
    match kind {
        AgentKind::ClaudeCode => Box::new(ClaudeCodeAdapter::new()),
        // Other agents use a generic pass-through for now
        _ => Box::new(GenericAdapter {
            name: format!("{kind}"),
        }),
    }
}

/// A generic adapter that emits StatusText for every non-empty line.
struct GenericAdapter {
    name: String,
}

impl AgentAdapter for GenericAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn parse_output(&mut self, data: &[u8]) -> Vec<AgentEvent> {
        let text = String::from_utf8_lossy(data);
        let mut events = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                events.push(AgentEvent::StatusText {
                    text: trimmed.to_string(),
                });
            }
        }
        events
    }

    fn reset(&mut self) {}
}
