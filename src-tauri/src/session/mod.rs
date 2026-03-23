//! Session management.
//!
//! Each session owns a PTY, terminal emulator, and associated state.
//! The SessionManager handles session lifecycle (create, destroy, list).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam_channel::{Receiver, Sender};
use serde::Serialize;

use crate::pty::{self, PtyBackend, PtyConfig};
use crate::terminal::{Emulator, GridSnapshot};

/// Unique session identifier.
pub type SessionId = String;

/// Events emitted by sessions.
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// Grid snapshot after processing PTY output.
    GridUpdate {
        session_id: SessionId,
        snapshot: GridSnapshot,
    },
    /// Window title changed via OSC.
    TitleChanged {
        session_id: SessionId,
        title: String,
    },
    /// Session process exited.
    Exited {
        session_id: SessionId,
        exit_code: i32,
    },
}

/// Manages all active terminal sessions.
pub struct SessionManager {
    sessions: HashMap<SessionId, Session>,
    next_id: AtomicU32,
    event_tx: Sender<SessionEvent>,
    event_rx: Receiver<SessionEvent>,
}

/// Serializable session info for the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub cwd: String,
}

/// A single terminal session.
struct Session {
    id: SessionId,
    title: String,
    cwd: std::path::PathBuf,
    pty: Arc<Mutex<Box<dyn PtyBackend>>>,
    emulator: Arc<Mutex<Emulator>>,
    _read_thread: thread::JoinHandle<()>,
    shutdown_tx: Sender<()>,
}

impl SessionManager {
    pub fn new() -> Self {
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        Self {
            sessions: HashMap::new(),
            next_id: AtomicU32::new(1),
            event_tx,
            event_rx,
        }
    }

    pub fn event_rx(&self) -> Receiver<SessionEvent> {
        self.event_rx.clone()
    }

    pub fn create_session(&mut self, config: Option<PtyConfig>) -> Result<SessionId, String> {
        let config = config.unwrap_or_default();
        let id = {
            let n = self.next_id.fetch_add(1, Ordering::Relaxed);
            format!("session-{n}")
        };

        let pty = pty::spawn_pty(&config).map_err(|e| format!("Failed to spawn PTY: {e}"))?;
        let pty = Arc::new(Mutex::new(pty));

        let emulator = Arc::new(Mutex::new(Emulator::new(
            config.cols as usize,
            config.rows as usize,
        )));

        let (shutdown_tx, shutdown_rx) = crossbeam_channel::bounded(1);
        let event_tx = self.event_tx.clone();
        let session_id = id.clone();
        let pty_reader = Arc::clone(&pty);
        let emu_ref = Arc::clone(&emulator);

        let read_thread = thread::Builder::new()
            .name(format!("pty-reader-{id}"))
            .spawn(move || {
                let mut buf = [0u8; 8192];
                let mut last_title: Option<String> = None;

                loop {
                    if shutdown_rx.try_recv().is_ok() {
                        break;
                    }

                    let n = {
                        let mut pty = pty_reader.lock().unwrap();
                        match pty.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => n,
                            Err(e) => {
                                log::debug!("PTY read error for {session_id}: {e}");
                                break;
                            }
                        }
                    };

                    // Process through emulator
                    let snapshot = {
                        let mut emu = emu_ref.lock().unwrap();
                        emu.process(&buf[..n]);

                        // Check for title changes
                        if emu.title != last_title {
                            if let Some(title) = &emu.title {
                                let _ = event_tx.send(SessionEvent::TitleChanged {
                                    session_id: session_id.clone(),
                                    title: title.clone(),
                                });
                            }
                            last_title = emu.title.clone();
                        }

                        if emu.take_dirty() {
                            Some(emu.snapshot())
                        } else {
                            None
                        }
                    };

                    if let Some(snapshot) = snapshot {
                        let _ = event_tx.send(SessionEvent::GridUpdate {
                            session_id: session_id.clone(),
                            snapshot,
                        });
                    }
                }

                let exit_code = {
                    let pty = pty_reader.lock().unwrap();
                    if pty.is_alive() { -1 } else { 0 }
                };
                let _ = event_tx.send(SessionEvent::Exited {
                    session_id,
                    exit_code,
                });
            })
            .map_err(|e| format!("Failed to spawn reader thread: {e}"))?;

        let session = Session {
            id: id.clone(),
            title: format!("Terminal {}", self.sessions.len() + 1),
            cwd: config.cwd,
            pty,
            emulator,
            _read_thread: read_thread,
            shutdown_tx,
        };

        self.sessions.insert(id.clone(), session);
        Ok(id)
    }

    pub fn destroy_session(&mut self, id: &str) -> Result<(), String> {
        let session = self
            .sessions
            .remove(id)
            .ok_or_else(|| format!("Session {id} not found"))?;

        let _ = session.shutdown_tx.send(());
        drop(session);
        Ok(())
    }

    pub fn send_input(&self, id: &str, data: &[u8]) -> Result<(), String> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| format!("Session {id} not found"))?;

        let mut pty = session.pty.lock().unwrap();
        pty.write(data)
            .map_err(|e| format!("Failed to write to PTY: {e}"))?;
        Ok(())
    }

    pub fn resize_session(&self, id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| format!("Session {id} not found"))?;

        // Resize PTY
        {
            let pty = session.pty.lock().unwrap();
            pty.resize(cols, rows)
                .map_err(|e| format!("Failed to resize PTY: {e}"))?;
        }

        // Resize emulator grid
        {
            let mut emu = session.emulator.lock().unwrap();
            emu.resize(cols as usize, rows as usize);
        }

        Ok(())
    }

    pub fn get_snapshot(&self, id: &str) -> Result<GridSnapshot, String> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| format!("Session {id} not found"))?;

        let emu = session.emulator.lock().unwrap();
        Ok(emu.snapshot())
    }

    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions
            .values()
            .map(|s| SessionInfo {
                id: s.id.clone(),
                title: s.title.clone(),
                cwd: s.cwd.to_string_lossy().into_owned(),
            })
            .collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
