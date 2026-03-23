//! Tauri IPC command handlers for context engine.

use std::path::Path;

use tauri::State;

use crate::context::ProjectContext;
use crate::ContextEngineState;

use super::session::SessionManagerState;

#[tauri::command]
pub fn get_context(
    session_id: Option<String>,
    cwd: Option<String>,
    session_state: State<'_, SessionManagerState>,
    state: State<'_, ContextEngineState>,
) -> ProjectContext {
    // Resolve CWD: prefer session_id lookup, fallback to cwd parameter
    let resolved_cwd = if let Some(sid) = session_id {
        let manager = session_state.0.lock().unwrap();
        let sessions = manager.list_sessions();
        sessions
            .into_iter()
            .find(|s| s.id == sid)
            .map(|s| s.cwd)
            .unwrap_or_else(|| cwd.unwrap_or_else(|| ".".to_string()))
    } else {
        cwd.unwrap_or_else(|| ".".to_string())
    };

    let mut engine = state.0.lock().unwrap();
    engine.scan(Path::new(&resolved_cwd))
}

#[tauri::command]
pub fn get_providers(state: State<'_, ContextEngineState>) -> Vec<String> {
    let engine = state.0.lock().unwrap();
    engine.provider_names().into_iter().map(String::from).collect()
}
