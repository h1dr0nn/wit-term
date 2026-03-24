//! Shared types for agent detection and event parsing.

use serde::Serialize;
use std::time::SystemTime;

/// The kind of coding agent detected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AgentKind {
    ClaudeCode,
    Aider,
    CodexCli,
    CopilotCli,
    Unknown(String),
}

impl std::fmt::Display for AgentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentKind::ClaudeCode => write!(f, "claude_code"),
            AgentKind::Aider => write!(f, "aider"),
            AgentKind::CodexCli => write!(f, "codex_cli"),
            AgentKind::CopilotCli => write!(f, "copilot_cli"),
            AgentKind::Unknown(name) => write!(f, "unknown:{name}"),
        }
    }
}

/// Identity of a detected agent process.
#[derive(Debug, Clone, Serialize)]
pub struct AgentIdentity {
    pub kind: AgentKind,
    pub pid: u32,
    /// Human-readable display name (e.g. "Claude Code").
    pub name: String,
    pub detected_at: SystemTime,
}

/// Events parsed from agent terminal output.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
    ThinkingStart,
    ThinkingEnd,
    ToolUse {
        tool_name: String,
        description: String,
    },
    FileEdit {
        path: String,
        action: FileAction,
    },
    TokenUpdate {
        input_tokens: u64,
        output_tokens: u64,
    },
    CostUpdate {
        total_cost: f64,
    },
    ModelInfo {
        model_name: String,
    },
    StatusText {
        text: String,
    },
    Error {
        message: String,
    },
}

/// The action performed on a file.
#[derive(Debug, Clone, Serialize)]
pub enum FileAction {
    Created,
    Modified,
    Deleted,
}

/// A file change detected by the file watcher.
#[derive(Debug, Clone, Serialize)]
pub struct FileChange {
    pub path: String,
    pub action: FileAction,
    pub timestamp: SystemTime,
}

/// The current state of agent detection for a session.
#[derive(Debug, Clone)]
pub enum DetectionState {
    Idle,
    Detected(AgentIdentity),
    Ended,
}
