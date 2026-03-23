//! Tauri IPC command handlers for context engine.

use std::path::Path;

use tauri::State;

use crate::context::ProjectContext;
use crate::ContextEngineState;

#[tauri::command]
pub fn get_context(cwd: String, state: State<'_, ContextEngineState>) -> ProjectContext {
    let engine = state.0.lock().unwrap();
    engine.scan(Path::new(&cwd))
}

#[tauri::command]
pub fn get_providers(state: State<'_, ContextEngineState>) -> Vec<String> {
    let engine = state.0.lock().unwrap();
    engine.provider_names().into_iter().map(String::from).collect()
}
