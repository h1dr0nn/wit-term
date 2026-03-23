//! Python context provider.

use std::collections::HashMap;
use std::path::Path;

use super::{find_upward, ContextInfo, ContextProvider};

pub struct PythonProvider;

const PYTHON_MARKERS: &[&str] = &[
    "pyproject.toml",
    "setup.py",
    "setup.cfg",
    "requirements.txt",
    "Pipfile",
    "poetry.lock",
    "uv.lock",
];

impl ContextProvider for PythonProvider {
    fn name(&self) -> &str {
        "python"
    }

    fn detect(&self, dir: &Path) -> bool {
        PYTHON_MARKERS
            .iter()
            .any(|marker| find_upward(dir, marker).is_some())
    }

    fn gather(&self, dir: &Path) -> ContextInfo {
        let mut data = HashMap::new();

        // Detect tool
        if find_upward(dir, "pyproject.toml").is_some() {
            // Check if it's Poetry, PDM, or standard
            if let Some(pyproject) = find_upward(dir, "pyproject.toml") {
                if let Ok(content) = std::fs::read_to_string(&pyproject) {
                    if let Ok(table) = content.parse::<toml::Table>() {
                        if let Some(project) =
                            table.get("project").and_then(|v| v.as_table())
                        {
                            if let Some(name) = project.get("name").and_then(|v| v.as_str()) {
                                data.insert("name".into(), name.to_string());
                            }
                            if let Some(version) =
                                project.get("version").and_then(|v| v.as_str())
                            {
                                data.insert("version".into(), version.to_string());
                            }
                        }
                    }
                }
                if let Some(root) = pyproject.parent() {
                    data.insert("root".into(), root.to_string_lossy().into_owned());
                }
            }

            if find_upward(dir, "poetry.lock").is_some() {
                data.insert("tool".into(), "poetry".into());
            } else if find_upward(dir, "uv.lock").is_some() {
                data.insert("tool".into(), "uv".into());
            } else {
                data.insert("tool".into(), "pip".into());
            }
        } else if find_upward(dir, "Pipfile").is_some() {
            data.insert("tool".into(), "pipenv".into());
        } else if find_upward(dir, "requirements.txt").is_some() {
            data.insert("tool".into(), "pip".into());
        }

        // Check for virtual env
        for venv_name in &[".venv", "venv", "env", ".env"] {
            if dir.join(venv_name).join("pyvenv.cfg").exists() {
                data.insert("venv".into(), venv_name.to_string());
                break;
            }
        }

        ContextInfo {
            provider: "python".into(),
            detected: true,
            data,
        }
    }
}
