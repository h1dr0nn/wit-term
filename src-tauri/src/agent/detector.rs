//! Process-tree based agent detection using sysinfo.

use std::time::SystemTime;

use sysinfo::{Pid, System};

use super::types::{AgentIdentity, AgentKind, DetectionState};

/// Pattern for matching a known coding agent by process name or command-line args.
#[allow(dead_code)]
struct AgentPattern {
    kind: AgentKind,
    process_names: Vec<&'static str>,
    arg_patterns: Vec<&'static str>,
}

/// Scans the process tree to detect coding agents spawned under a shell.
pub struct AgentDetector {
    known_agents: Vec<AgentPattern>,
    state: DetectionState,
}

impl AgentDetector {
    pub fn new() -> Self {
        let known_agents = vec![
            AgentPattern {
                kind: AgentKind::ClaudeCode,
                process_names: vec!["claude", "claude-code"],
                arg_patterns: vec!["claude", "@anthropic-ai/claude-code"],
            },
            AgentPattern {
                kind: AgentKind::Aider,
                process_names: vec!["aider"],
                arg_patterns: vec!["aider", "-m aider"],
            },
            AgentPattern {
                kind: AgentKind::CodexCli,
                process_names: vec!["codex"],
                arg_patterns: vec!["codex", "@openai/codex"],
            },
            AgentPattern {
                kind: AgentKind::CopilotCli,
                process_names: vec!["github-copilot-cli", "ghcs"],
                arg_patterns: vec!["github-copilot-cli"],
            },
        ];

        Self {
            known_agents,
            state: DetectionState::Idle,
        }
    }

    /// Scan processes under `shell_pid` and return an `AgentIdentity` if a known
    /// coding agent is found as a direct child.
    pub fn scan(&mut self, shell_pid: u32) -> Option<AgentIdentity> {
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let parent_pid = Pid::from_u32(shell_pid);

        // Walk all processes, find direct children of the shell
        for (pid, process) in sys.processes() {
            let is_child = process
                .parent()
                .map(|p| p == parent_pid)
                .unwrap_or(false);

            if !is_child {
                continue;
            }

            let proc_name = process.name().to_string_lossy().to_lowercase();
            let cmd_args: Vec<String> = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_lowercase())
                .collect();
            let cmd_joined = cmd_args.join(" ");

            for pattern in &self.known_agents {
                let name_match = pattern
                    .process_names
                    .iter()
                    .any(|n| proc_name.contains(n));

                let arg_match = pattern
                    .arg_patterns
                    .iter()
                    .any(|a| cmd_joined.contains(a));

                if name_match || arg_match {
                    let display_name = match &pattern.kind {
                        AgentKind::ClaudeCode => "Claude Code".to_string(),
                        AgentKind::Aider => "Aider".to_string(),
                        AgentKind::CodexCli => "Codex CLI".to_string(),
                        AgentKind::CopilotCli => "Copilot CLI".to_string(),
                        AgentKind::Unknown(s) => s.clone(),
                    };

                    let identity = AgentIdentity {
                        kind: pattern.kind.clone(),
                        pid: pid.as_u32(),
                        name: display_name,
                        detected_at: SystemTime::now(),
                    };

                    self.state = DetectionState::Detected(identity.clone());
                    return Some(identity);
                }
            }
        }

        // If we were previously detected but no longer see the agent, mark ended
        if matches!(self.state, DetectionState::Detected(_)) {
            self.state = DetectionState::Ended;
        }

        None
    }

    #[allow(dead_code)]
    pub fn state(&self) -> &DetectionState {
        &self.state
    }
}

impl Default for AgentDetector {
    fn default() -> Self {
        Self::new()
    }
}
