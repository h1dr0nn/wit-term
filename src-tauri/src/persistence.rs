use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSession {
    pub id: String,
    pub title: String,
    pub cwd: String,
    pub created_at: u64,
    pub last_used_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistedState {
    pub sessions: Vec<PersistedSession>,
}

fn persistence_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("wit").join("sessions.json"))
}

pub fn save_sessions(sessions: &[PersistedSession]) -> Result<(), String> {
    let path = persistence_path().ok_or("No data directory")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let state = PersistedState {
        sessions: sessions.to_vec(),
    };
    let json = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn load_sessions() -> Result<Vec<PersistedSession>, String> {
    let path = persistence_path().ok_or("No data directory")?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let state: PersistedState = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(state.sessions)
}
