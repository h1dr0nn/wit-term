//! Tauri IPC command handlers for session management.

use std::path::Path;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, State};

use crate::context::ProjectContext;
use crate::session::{SessionEvent, SessionInfo, SessionManager};
use crate::terminal::GridSnapshot;
use crate::ContextEngineState;

/// Shared state wrapper for SessionManager.
pub struct SessionManagerState(pub Mutex<SessionManager>);

#[derive(Clone, serde::Serialize)]
struct GridUpdatePayload {
    session_id: String,
    snapshot: GridSnapshot,
}

#[derive(Clone, serde::Serialize)]
struct SessionExitedPayload {
    session_id: String,
    exit_code: i32,
}

#[derive(Clone, serde::Serialize)]
struct CwdChangedPayload {
    session_id: String,
    cwd: String,
}

#[derive(Clone, serde::Serialize)]
struct TitleChangedPayload {
    session_id: String,
    title: String,
}

#[derive(Clone, serde::Serialize)]
struct ContextChangedPayload {
    session_id: String,
    context: ProjectContext,
}

#[derive(Clone, serde::Serialize)]
struct BellPayload {
    session_id: String,
}

#[derive(Clone, serde::Serialize)]
struct CommandOutputPayload {
    session_id: String,
    command_id: u64,
    output: String,
    duration_ms: u64,
}

#[derive(Clone, serde::Serialize)]
struct CommandOutputChunkPayload {
    session_id: String,
    command_id: u64,
    output: String,
}

#[derive(Clone, serde::Serialize)]
struct AgentDetectedPayload {
    session_id: String,
    agent_name: String,
    agent_kind: String,
    pid: u32,
}

#[derive(Clone, serde::Serialize)]
struct AgentExitedPayload {
    session_id: String,
    agent_name: String,
}

#[derive(Clone, serde::Serialize)]
struct AgentEventPayload {
    session_id: String,
    event: crate::agent::types::AgentEvent,
}

#[derive(Clone, serde::Serialize)]
struct FileChangePayload {
    session_id: String,
    change: crate::agent::types::FileChange,
}

/// Initialize session event forwarding to the frontend.
pub fn init_event_forwarding(app: &AppHandle) {
    let manager = app.state::<SessionManagerState>();
    let event_rx = manager.0.lock().unwrap().event_rx();
    let app_handle = app.clone();

    std::thread::Builder::new()
        .name("event-forwarder".into())
        .spawn(move || {
            while let Ok(event) = event_rx.recv() {
                match event {
                    SessionEvent::GridUpdate {
                        session_id,
                        snapshot,
                    } => {
                        let payload = GridUpdatePayload {
                            session_id,
                            snapshot,
                        };
                        // Emit both documented and current event names
                        let _ = app_handle.emit("grid_update", payload.clone());
                        let _ = app_handle.emit("terminal_output", payload);
                    }
                    SessionEvent::CwdChanged { session_id, cwd } => {
                        let _ = app_handle.emit(
                            "cwd_changed",
                            CwdChangedPayload {
                                session_id: session_id.clone(),
                                cwd: cwd.clone(),
                            },
                        );

                        // Trigger context scan and emit context_changed
                        if let Some(context_state) =
                            app_handle.try_state::<ContextEngineState>()
                        {
                            let mut engine = context_state.0.lock().unwrap();
                            let context = engine.scan(Path::new(&cwd));
                            let _ = app_handle.emit(
                                "context_changed",
                                ContextChangedPayload {
                                    session_id,
                                    context,
                                },
                            );
                        }
                    }
                    SessionEvent::TitleChanged { session_id, title } => {
                        let payload = TitleChangedPayload {
                            session_id,
                            title,
                        };
                        // Emit both documented and current event names
                        let _ = app_handle.emit("title_changed", payload.clone());
                        let _ = app_handle.emit("terminal_title", payload);
                    }
                    SessionEvent::Exited {
                        session_id,
                        exit_code,
                    } => {
                        let _ = app_handle.emit(
                            "session_exited",
                            SessionExitedPayload {
                                session_id,
                                exit_code,
                            },
                        );
                    }
                    SessionEvent::Bell { session_id } => {
                        let _ = app_handle.emit(
                            "terminal_bell",
                            BellPayload { session_id },
                        );
                    }
                    SessionEvent::CommandOutput {
                        session_id,
                        command_id,
                        output,
                        duration_ms,
                    } => {
                        let _ = app_handle.emit(
                            "command_output",
                            CommandOutputPayload {
                                session_id,
                                command_id,
                                output,
                                duration_ms,
                            },
                        );
                    }
                    SessionEvent::CommandOutputChunk {
                        session_id,
                        command_id,
                        output,
                    } => {
                        let _ = app_handle.emit(
                            "command_output_chunk",
                            CommandOutputChunkPayload {
                                session_id,
                                command_id,
                                output,
                            },
                        );
                    }
                    SessionEvent::AgentDetected {
                        session_id,
                        agent_name,
                        agent_kind,
                        pid,
                    } => {
                        let _ = app_handle.emit(
                            "agent_detected",
                            AgentDetectedPayload {
                                session_id,
                                agent_name,
                                agent_kind,
                                pid,
                            },
                        );
                    }
                    SessionEvent::AgentExited {
                        session_id,
                        agent_name,
                    } => {
                        let _ = app_handle.emit(
                            "agent_exited",
                            AgentExitedPayload {
                                session_id,
                                agent_name,
                            },
                        );
                    }
                    SessionEvent::AgentEvent { session_id, event } => {
                        let _ = app_handle.emit(
                            "agent_event",
                            AgentEventPayload {
                                session_id,
                                event,
                            },
                        );
                    }
                    SessionEvent::FileChange { session_id, change } => {
                        let _ = app_handle.emit(
                            "file_change",
                            FileChangePayload {
                                session_id,
                                change,
                            },
                        );
                    }
                }
            }
        })
        .expect("Failed to spawn event forwarder thread");
}

#[derive(Clone, serde::Serialize)]
pub struct CreateSessionResult {
    id: String,
    cwd: String,
}

#[tauri::command]
pub fn create_session(
    cwd: Option<String>,
    cols: Option<u16>,
    rows: Option<u16>,
    state: State<'_, SessionManagerState>,
) -> Result<CreateSessionResult, String> {
    let mut manager = state.0.lock().unwrap();
    let mut c = crate::pty::PtyConfig::default();
    if let Some(dir) = &cwd {
        c.cwd = std::path::PathBuf::from(dir);
    }
    if let Some(cols) = cols {
        c.cols = cols;
    }
    if let Some(rows) = rows {
        c.rows = rows;
    }
    let config = Some(c);
    // Determine the actual CWD that will be used
    let actual_cwd = config
        .as_ref()
        .map(|c| c.cwd.to_string_lossy().into_owned())
        .unwrap_or_else(|| {
            crate::pty::PtyConfig::default()
                .cwd
                .to_string_lossy()
                .into_owned()
        });
    let id = manager.create_session(config)?;
    Ok(CreateSessionResult {
        id,
        cwd: actual_cwd,
    })
}

#[tauri::command]
pub fn destroy_session(
    session_id: String,
    state: State<'_, SessionManagerState>,
) -> Result<(), String> {
    let mut manager = state.0.lock().unwrap();
    manager.destroy_session(&session_id)
}

#[tauri::command]
pub fn list_sessions(state: State<'_, SessionManagerState>) -> Vec<SessionInfo> {
    let manager = state.0.lock().unwrap();
    manager.list_sessions()
}

#[tauri::command]
pub fn send_input(
    session_id: String,
    data: String,
    state: State<'_, SessionManagerState>,
) -> Result<(), String> {
    let manager = state.0.lock().unwrap();
    manager.send_input(&session_id, data.as_bytes())
}

#[tauri::command]
pub fn submit_command(
    session_id: String,
    command: String,
    command_id: u64,
    state: State<'_, SessionManagerState>,
) -> Result<(), String> {
    let manager = state.0.lock().unwrap();
    manager.submit_command(&session_id, &command, command_id)
}

#[tauri::command]
pub fn resize_session(
    session_id: String,
    cols: u16,
    rows: u16,
    state: State<'_, SessionManagerState>,
) -> Result<(), String> {
    let manager = state.0.lock().unwrap();
    manager.resize_session(&session_id, cols, rows)
}

#[tauri::command]
pub fn get_snapshot(
    session_id: String,
    state: State<'_, SessionManagerState>,
) -> Result<GridSnapshot, String> {
    let manager = state.0.lock().unwrap();
    manager.get_snapshot(&session_id)
}

#[tauri::command]
pub fn save_session_state(
    sessions: Vec<crate::persistence::PersistedSession>,
) -> Result<(), String> {
    crate::persistence::save_sessions(&sessions)
}

#[tauri::command]
pub fn load_session_state() -> Result<Vec<crate::persistence::PersistedSession>, String> {
    crate::persistence::load_sessions()
}

/// Alias for get_snapshot to match documented API.
#[tauri::command]
pub fn get_session_grid(
    session_id: String,
    state: State<'_, SessionManagerState>,
) -> Result<GridSnapshot, String> {
    let manager = state.0.lock().unwrap();
    manager.get_snapshot(&session_id)
}
