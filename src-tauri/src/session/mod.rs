//! Session management.
//!
//! Each session owns a PTY, terminal emulator, and associated state.
//! The SessionManager handles session lifecycle (create, destroy, list).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};
use serde::Serialize;

use crate::agent::adapters::{self as agent_adapters};
use crate::agent::detector::AgentDetector;
use crate::agent::types::DetectionState;
use crate::agent::watcher::FileWatcher;
use crate::pty::{self, PtyBackend, PtyConfig};
use crate::terminal::strip;
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
    /// Current working directory changed via OSC 7.
    CwdChanged {
        session_id: SessionId,
        cwd: String,
    },
    /// Session process exited.
    Exited {
        session_id: SessionId,
        exit_code: i32,
    },
    /// Terminal bell (BEL character).
    Bell {
        session_id: SessionId,
    },
    /// Finalized command output (emitted when the next command starts).
    CommandOutput {
        session_id: SessionId,
        command_id: u64,
        output: String,
        duration_ms: u64,
    },
    /// Live streaming output chunk (emitted as PTY produces output).
    CommandOutputChunk {
        session_id: SessionId,
        command_id: u64,
        output: String,
    },
    /// A coding agent was detected as a child of the session shell.
    AgentDetected {
        session_id: SessionId,
        agent_name: String,
        agent_kind: String,
        pid: u32,
    },
    /// A previously detected agent has exited.
    AgentExited {
        session_id: SessionId,
        agent_name: String,
    },
    /// A structured event parsed from agent output.
    AgentEvent {
        session_id: SessionId,
        event: crate::agent::types::AgentEvent,
    },
    /// A file change detected by the project file watcher.
    FileChange {
        session_id: SessionId,
        change: crate::agent::types::FileChange,
    },
}

/// Per-command output capture state, shared between IPC and PTY reader threads.
struct CaptureState {
    /// Currently active command ID (None = idle).
    active_command_id: Option<u64>,
    /// The command string (for echo stripping).
    active_command: Option<String>,
    /// Grid cursor row when the command was submitted (absolute, includes scrollback).
    start_cursor_row: Option<usize>,
    /// When this command was submitted.
    started_at: Option<Instant>,
}

impl CaptureState {
    fn new() -> Self {
        Self {
            active_command_id: None,
            active_command: None,
            start_cursor_row: None,
            started_at: None,
        }
    }
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
    pty: Arc<Box<dyn PtyBackend>>,
    emulator: Arc<Mutex<Emulator>>,
    _read_thread: thread::JoinHandle<()>,
    shutdown_tx: Sender<()>,
    /// Shared capture state for per-command output buffering.
    capture: Arc<Mutex<CaptureState>>,
    /// Channel to forward PTY data to the agent adapter (set when agent is detected).
    #[allow(dead_code)]
    agent_data_tx: Arc<Mutex<Option<Sender<Vec<u8>>>>>,
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
        let pty = Arc::new(pty);

        let emulator = Arc::new(Mutex::new(Emulator::new(
            config.cols as usize,
            config.rows as usize,
        )));

        let capture = Arc::new(Mutex::new(CaptureState::new()));

        let (shutdown_tx, shutdown_rx) = crossbeam_channel::bounded(1);
        let event_tx = self.event_tx.clone();
        let session_id = id.clone();
        let pty_reader = Arc::clone(&pty);
        let pty_for_agent = Arc::clone(&pty);
        let config_cwd = config.cwd.clone();
        let emu_ref = Arc::clone(&emulator);
        let capture_ref = Arc::clone(&capture);
        let agent_data_tx_shared: Arc<Mutex<Option<Sender<Vec<u8>>>>> =
            Arc::new(Mutex::new(None));
        let agent_data_tx_for_reader = Arc::clone(&agent_data_tx_shared);

        let read_thread = thread::Builder::new()
            .name(format!("pty-reader-{id}"))
            .spawn(move || {
                let mut buf = [0u8; 8192];
                let mut last_title: Option<String> = None;
                let mut last_cwd: Option<std::path::PathBuf> = None;

                loop {
                    if shutdown_rx.try_recv().is_ok() {
                        break;
                    }

                    let n = match pty_reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(e) => {
                            log::debug!("PTY read error for {session_id}: {e}");
                            break;
                        }
                    };

                    // Forward PTY data to agent adapter if one is active
                    if let Ok(guard) = agent_data_tx_for_reader.lock() {
                        if let Some(tx) = guard.as_ref() {
                            // Non-blocking: drop data if channel is full
                            let _ = tx.try_send(buf[..n].to_vec());
                        }
                    }

                    // Process through emulator FIRST (so grid has correct layout)
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

                        // Check for CWD changes (OSC 7)
                        if emu.take_cwd_dirty() {
                            if let Some(cwd) = &emu.cwd {
                                if emu.cwd != last_cwd {
                                    let _ = event_tx.send(SessionEvent::CwdChanged {
                                        session_id: session_id.clone(),
                                        cwd: cwd.to_string_lossy().into_owned(),
                                    });
                                    last_cwd = emu.cwd.clone();
                                }
                            }
                        }

                        if emu.take_dirty() {
                            Some(emu.snapshot())
                        } else {
                            None
                        }
                    };

                    // Extract output from grid for live streaming chunk
                    if let Some(ref snap) = snapshot {
                        let cap = capture_ref.lock().unwrap();
                        if let Some(cmd_id) = cap.active_command_id {
                            if let Some(start_row) = cap.start_cursor_row {
                                // Detect CWD from the current prompt line (at cursor_row).
                                // This line is NOT included in the output range (exclusive),
                                // so we check it separately.
                                if snap.cursor_row < snap.rows.len() {
                                    let prompt_line = strip::grid_row_to_text(
                                        &snap.rows[snap.cursor_row],
                                    );
                                    if let Some(cwd) =
                                        strip::extract_cwd_from_prompt(&prompt_line)
                                    {
                                        let _ = event_tx.send(SessionEvent::CwdChanged {
                                            session_id: session_id.clone(),
                                            cwd,
                                        });
                                    }
                                }

                                let output = strip::grid_to_ansi_text(
                                    &snap.rows,
                                    start_row,
                                    snap.cursor_row,
                                );
                                let output = if let Some(cmd) = &cap.active_command {
                                    strip::strip_echo(&output, cmd)
                                } else {
                                    output
                                };
                                let output = strip::strip_trailing_prompt(&output);
                                let _ = event_tx.send(SessionEvent::CommandOutputChunk {
                                    session_id: session_id.clone(),
                                    command_id: cmd_id,
                                    output,
                                });
                            }
                        }
                    }

                    if let Some(snapshot) = snapshot {
                        let _ = event_tx.send(SessionEvent::GridUpdate {
                            session_id: session_id.clone(),
                            snapshot,
                        });
                    }
                }

                let exit_code = if pty_reader.is_alive() { -1 } else { 0 };
                let _ = event_tx.send(SessionEvent::Exited {
                    session_id,
                    exit_code,
                });
            })
            .map_err(|e| format!("Failed to spawn reader thread: {e}"))?;

        // Emit initial CWD so frontend has it immediately
        // (Windows shells don't send OSC 7)
        let initial_cwd = config.cwd.to_string_lossy().into_owned();
        let _ = self.event_tx.send(SessionEvent::CwdChanged {
            session_id: id.clone(),
            cwd: initial_cwd,
        });

        let session = Session {
            id: id.clone(),
            title: format!("Terminal {}", self.sessions.len() + 1),
            cwd: config.cwd,
            pty,
            emulator,
            _read_thread: read_thread,
            shutdown_tx,
            capture,
            agent_data_tx: Arc::clone(&agent_data_tx_shared),
        };

        self.sessions.insert(id.clone(), session);

        // Spawn agent detection thread
        let agent_event_tx = self.event_tx.clone();
        let agent_session_id = id.clone();
        let shell_pid = pty_for_agent.child_pid();
        let agent_cwd = config_cwd;

        let agent_data_tx_for_detector = Arc::clone(&agent_data_tx_shared);

        thread::Builder::new()
            .name(format!("agent-detector-{}", &id))
            .spawn(move || {
                let mut detector = AgentDetector::new();
                let mut agent_active = false;
                let mut _file_watcher: Option<FileWatcher> = None;
                let mut detected_name: Option<String> = None;

                loop {
                    // Check for agent every 2 seconds
                    thread::sleep(std::time::Duration::from_secs(2));

                    let identity = detector.scan(shell_pid);

                    match (identity, agent_active) {
                        (Some(ident), false) => {
                            // New agent detected
                            log::info!(
                                "Agent detected in session {}: {} (pid {})",
                                agent_session_id,
                                ident.name,
                                ident.pid
                            );

                            let _ = agent_event_tx.send(SessionEvent::AgentDetected {
                                session_id: agent_session_id.clone(),
                                agent_name: ident.name.clone(),
                                agent_kind: ident.kind.to_string(),
                                pid: ident.pid,
                            });

                            detected_name = Some(ident.name.clone());
                            agent_active = true;

                            // Create a channel for PTY data and set it in the
                            // shared slot so the reader thread starts forwarding.
                            let (pty_data_tx, pty_data_rx) =
                                crossbeam_channel::bounded::<Vec<u8>>(64);
                            {
                                let mut slot =
                                    agent_data_tx_for_detector.lock().unwrap();
                                *slot = Some(pty_data_tx);
                            }

                            // Start file watcher
                            let (fw_tx, fw_rx) = crossbeam_channel::bounded(256);
                            match FileWatcher::new(agent_cwd.clone(), fw_tx) {
                                Ok(fw) => {
                                    _file_watcher = Some(fw);

                                    // Spawn a thread to forward file changes
                                    let fw_event_tx = agent_event_tx.clone();
                                    let fw_session_id = agent_session_id.clone();
                                    thread::Builder::new()
                                        .name(format!(
                                            "file-watcher-fwd-{fw_session_id}"
                                        ))
                                        .spawn(move || {
                                            while let Ok(change) = fw_rx.recv() {
                                                let _ = fw_event_tx.send(
                                                    SessionEvent::FileChange {
                                                        session_id: fw_session_id
                                                            .clone(),
                                                        change,
                                                    },
                                                );
                                            }
                                        })
                                        .ok();
                                }
                                Err(e) => {
                                    log::warn!(
                                        "Failed to start file watcher: {e}"
                                    );
                                }
                            }

                            // Start processing PTY data through the adapter
                            let adapter_event_tx = agent_event_tx.clone();
                            let adapter_session_id = agent_session_id.clone();
                            let mut adapter_inner =
                                agent_adapters::create_adapter(&ident.kind);

                            thread::Builder::new()
                                .name(format!(
                                    "agent-adapter-{adapter_session_id}"
                                ))
                                .spawn(move || {
                                    while let Ok(data) = pty_data_rx.recv() {
                                        let events =
                                            adapter_inner.parse_output(&data);
                                        for event in events {
                                            let _ = adapter_event_tx.send(
                                                SessionEvent::AgentEvent {
                                                    session_id:
                                                        adapter_session_id.clone(),
                                                    event,
                                                },
                                            );
                                        }
                                    }
                                })
                                .ok();
                        }
                        (None, true) => {
                            // Agent exited
                            if matches!(detector.state(), DetectionState::Ended) {
                                log::info!(
                                    "Agent exited in session {}",
                                    agent_session_id
                                );

                                let name =
                                    detected_name.take().unwrap_or_default();
                                let _ =
                                    agent_event_tx.send(SessionEvent::AgentExited {
                                        session_id: agent_session_id.clone(),
                                        agent_name: name,
                                    });

                                agent_active = false;
                                _file_watcher = None;

                                // Clear the data channel so reader stops sending
                                if let Ok(mut slot) =
                                    agent_data_tx_for_detector.lock()
                                {
                                    *slot = None;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
            .ok();

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

        session
            .pty
            .write(data)
            .map_err(|e| format!("Failed to write to PTY: {e}"))?;
        Ok(())
    }

    /// Submit a command from the InputBar. This atomically:
    /// 1. Finalizes the previous command's captured output
    /// 2. Starts capturing for the new command
    /// 3. Writes the command to the PTY
    pub fn submit_command(
        &self,
        id: &str,
        command: &str,
        command_id: u64,
    ) -> Result<(), String> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| format!("Session {id} not found"))?;

        // Get current cursor position from emulator
        let current_cursor_row = {
            let emu = session.emulator.lock().unwrap();
            let snap = emu.snapshot();
            snap.cursor_row
        };

        // Finalize previous capture and start new one
        {
            let mut cap = session.capture.lock().unwrap();

            // Finalize previous command if any
            if let Some(prev_id) = cap.active_command_id.take() {
                // Extract text from grid rows (correct layout + colors)
                let emu = session.emulator.lock().unwrap();
                let snap = emu.snapshot();
                let start = cap.start_cursor_row.unwrap_or(0);
                let output = strip::grid_to_ansi_text(&snap.rows, start, current_cursor_row);
                drop(emu);

                let output = if let Some(cmd) = &cap.active_command {
                    strip::strip_echo(&output, cmd)
                } else {
                    output
                };
                let output = strip::strip_trailing_prompt(&output);

                let duration_ms = cap
                    .started_at
                    .map(|t| t.elapsed().as_millis() as u64)
                    .unwrap_or(0);

                let _ = self.event_tx.send(SessionEvent::CommandOutput {
                    session_id: session.id.clone(),
                    command_id: prev_id,
                    output,
                    duration_ms,
                });
            }

            // Start new capture (record cursor position for grid extraction)
            cap.active_command_id = Some(command_id);
            cap.active_command = Some(command.to_string());
            cap.start_cursor_row = Some(current_cursor_row);
            cap.started_at = Some(Instant::now());
        }

        // Write command to PTY
        let data = format!("{command}\r");
        session
            .pty
            .write(data.as_bytes())
            .map_err(|e| format!("Failed to write to PTY: {e}"))?;

        Ok(())
    }

    pub fn resize_session(&self, id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| format!("Session {id} not found"))?;

        // Resize PTY (no mutex needed — resize is thread-safe)
        session
            .pty
            .resize(cols, rows)
            .map_err(|e| format!("Failed to resize PTY: {e}"))?;

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
