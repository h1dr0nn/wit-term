pub mod commands;
pub mod completion;
pub mod config;
pub mod context;
pub mod parser;
pub mod plugin;
pub mod pty;
pub mod session;
pub mod terminal;

use commands::completion::CompletionEngineState;
use commands::session::SessionManagerState;
use completion::CompletionEngine;
use context::ContextEngine;
use session::SessionManager;
use std::path::PathBuf;
use std::sync::Mutex;

/// Shared context engine state.
pub struct ContextEngineState(pub Mutex<ContextEngine>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(SessionManagerState(Mutex::new(SessionManager::new())))
        .manage(ContextEngineState(Mutex::new(ContextEngine::new())))
        .manage(CompletionEngineState(Mutex::new(CompletionEngine::new(
            &PathBuf::from("completions"),
        ))))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Start forwarding session events to the frontend
            commands::session::init_event_forwarding(app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session::create_session,
            commands::session::destroy_session,
            commands::session::list_sessions,
            commands::session::send_input,
            commands::session::resize_session,
            commands::session::get_snapshot,
            commands::session::get_session_grid,
            commands::context::get_context,
            commands::context::get_providers,
            commands::completion::request_completions,
            commands::completion::accept_completion,
            commands::config::get_config,
            commands::config::set_config,
            commands::config::list_themes,
            commands::config::get_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
