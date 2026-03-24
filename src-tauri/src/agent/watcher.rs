//! File system watcher for detecting file changes during agent sessions.

use std::path::PathBuf;
use std::time::SystemTime;

use crossbeam_channel::Sender;
use notify::{Event, EventKind, RecursiveMode, Watcher};

use super::types::{FileAction, FileChange};

/// Directories to ignore when watching for file changes.
const IGNORE_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "__pycache__",
    ".next",
    ".venv",
    "venv",
    "dist",
    "build",
];

/// Watches a project directory for file changes and sends `FileChange` events.
pub struct FileWatcher {
    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,
}

impl FileWatcher {
    /// Create a new file watcher on `project_dir`, sending events through `sender`.
    pub fn new(
        project_dir: PathBuf,
        sender: Sender<FileChange>,
    ) -> Result<Self, notify::Error> {
        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            let event = match res {
                Ok(e) => e,
                Err(err) => {
                    log::warn!("File watcher error: {err}");
                    return;
                }
            };

            // Map notify event kinds to our FileAction
            let action = match event.kind {
                EventKind::Create(_) => FileAction::Created,
                EventKind::Modify(_) => FileAction::Modified,
                EventKind::Remove(_) => FileAction::Deleted,
                _ => return, // Ignore access, other events
            };

            for path in event.paths {
                // Skip ignored directories
                let path_str = path.to_string_lossy();
                if IGNORE_DIRS
                    .iter()
                    .any(|dir| path_str.contains(&format!("{}{}", std::path::MAIN_SEPARATOR, dir)))
                {
                    continue;
                }

                let change = FileChange {
                    path: path_str.into_owned(),
                    action: action.clone(),
                    timestamp: SystemTime::now(),
                };

                // Non-blocking send; drop the event if the receiver is full/gone
                let _ = sender.try_send(change);
            }
        })?;

        let mut fw = Self { watcher };

        fw.watcher
            .watch(&project_dir, RecursiveMode::Recursive)?;

        Ok(fw)
    }

    /// Stop watching. The watcher is dropped, which removes all watches.
    #[allow(dead_code)]
    pub fn stop(self) {
        drop(self.watcher);
    }
}
