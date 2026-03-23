//! Tauri IPC command handlers for completions.

use std::sync::Mutex;

use tauri::State;

use crate::completion::{CompletionEngine, CompletionItem, CompletionRequest};
use crate::session::SessionManager;

use super::session::SessionManagerState;

/// Shared completion engine state.
pub struct CompletionEngineState(pub Mutex<CompletionEngine>);

#[tauri::command]
pub fn request_completions(
    input: String,
    cursor_pos: usize,
    session_id: Option<String>,
    cwd: Option<String>,
    session_state: State<'_, SessionManagerState>,
    state: State<'_, CompletionEngineState>,
) -> Vec<CompletionItem> {
    // Resolve CWD: prefer session_id lookup, fallback to cwd parameter
    let resolved_cwd = resolve_cwd(session_id, cwd, &session_state);

    let engine = state.0.lock().unwrap();
    engine.complete(&CompletionRequest {
        input,
        cursor_pos,
        cwd: resolved_cwd,
    })
}

#[tauri::command]
pub fn accept_completion(
    session_id: String,
    text: String,
    session_state: State<'_, SessionManagerState>,
) -> Result<(), String> {
    let manager = session_state.0.lock().unwrap();
    manager.send_input(&session_id, text.as_bytes())
}

/// Resolve CWD from session_id or direct cwd parameter.
fn resolve_cwd(
    session_id: Option<String>,
    cwd: Option<String>,
    session_state: &State<'_, SessionManagerState>,
) -> String {
    if let Some(sid) = session_id {
        let manager = session_state.0.lock().unwrap();
        if let Some(session_cwd) = get_session_cwd(&manager, &sid) {
            return session_cwd;
        }
    }
    cwd.unwrap_or_else(|| ".".to_string())
}

/// Get the CWD of a session.
fn get_session_cwd(manager: &SessionManager, session_id: &str) -> Option<String> {
    let sessions = manager.list_sessions();
    sessions
        .into_iter()
        .find(|s| s.id == session_id)
        .map(|s| s.cwd)
}
