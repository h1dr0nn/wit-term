pub mod commands;
pub mod parser;
pub mod pty;
pub mod session;
pub mod terminal;

use commands::session::SessionManagerState;
use session::SessionManager;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .manage(SessionManagerState(Mutex::new(SessionManager::new())))
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
