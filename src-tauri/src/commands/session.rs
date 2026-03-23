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
                }
            }
        })
        .expect("Failed to spawn event forwarder thread");
}

#[derive(Clone, serde::Serialize)]
struct CreateSessionResult {
    id: String,
    cwd: String,
}

#[tauri::command]
pub fn create_session(
    cwd: Option<String>,
    state: State<'_, SessionManagerState>,
) -> Result<CreateSessionResult, String> {
    let mut manager = state.0.lock().unwrap();
    let config = if let Some(dir) = &cwd {
        let mut c = crate::pty::PtyConfig::default();
        c.cwd = std::path::PathBuf::from(dir);
        Some(c)
    } else {
        None
    };
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

/// Alias for get_snapshot to match documented API.
#[tauri::command]
pub fn get_session_grid(
    session_id: String,
    state: State<'_, SessionManagerState>,
) -> Result<GridSnapshot, String> {
    let manager = state.0.lock().unwrap();
    manager.get_snapshot(&session_id)
}
