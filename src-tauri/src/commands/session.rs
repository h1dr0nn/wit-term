//! Tauri IPC command handlers for session management.

use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, State};

use crate::session::{SessionEvent, SessionInfo, SessionManager};
use crate::terminal::GridSnapshot;

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
                        let _ = app_handle.emit(
                            "grid_update",
                            GridUpdatePayload {
                                session_id,
                                snapshot,
                            },
                        );
                    }
                    SessionEvent::CwdChanged { session_id, cwd } => {
                        let _ = app_handle.emit(
                            "cwd_changed",
                            CwdChangedPayload { session_id, cwd },
                        );
                    }
                    SessionEvent::TitleChanged { session_id, title } => {
                        let _ = app_handle.emit(
                            "title_changed",
                            TitleChangedPayload { session_id, title },
                        );
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
                }
            }
        })
        .expect("Failed to spawn event forwarder thread");
}

#[tauri::command]
pub fn create_session(
    cwd: Option<String>,
    state: State<'_, SessionManagerState>,
) -> Result<String, String> {
    let mut manager = state.0.lock().unwrap();
    let config = cwd.map(|dir| {
        let mut c = crate::pty::PtyConfig::default();
        c.cwd = std::path::PathBuf::from(dir);
        c
    });
    manager.create_session(config)
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
