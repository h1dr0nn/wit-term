//! Tauri IPC command handlers for completions.

use std::sync::Mutex;

use tauri::State;

use crate::completion::{CompletionEngine, CompletionItem, CompletionRequest};

/// Shared completion engine state.
pub struct CompletionEngineState(pub Mutex<CompletionEngine>);

#[tauri::command]
pub fn request_completions(
    input: String,
    cursor_pos: usize,
    cwd: String,
    state: State<'_, CompletionEngineState>,
) -> Vec<CompletionItem> {
    let engine = state.0.lock().unwrap();
    engine.complete(&CompletionRequest {
        input,
        cursor_pos,
        cwd,
    })
}
