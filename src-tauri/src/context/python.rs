//! Python context provider.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use super::{find_upward, ContextInfo, ContextProvider, ContextValue};

pub struct PythonProvider;

const PYTHON_MARKERS: &[&str] = &[
    "pyproject.toml",
    "setup.py",
    "setup.cfg",
    "requirements.txt",
    "Pipfile",
    "poetry.lock",
    "uv.lock",
    ".python-version",
];

/// Frameworks detected from dependencies.
const FRAMEWORK_DEPS: &[(&str, &str)] = &[
    ("django", "Django"),
    ("flask", "Flask"),
    ("fastapi", "FastAPI"),
];

impl ContextProvider for PythonProvider {
    fn name(&self) -> &str {
        "python"
    }

    fn markers(&self) -> &[&str] {
        PYTHON_MARKERS
    }

    fn detect(&self, dir: &Path) -> bool {
        PYTHON_MARKERS
            .iter()
            .any(|marker| find_upward(dir, marker).is_some())
    }

    fn gather(&self, project_root: &Path, cwd: &Path) -> Result<ContextInfo> {
        let mut data = HashMap::new();
        let mut detected_markers = Vec::new();

        // Detect tool and parse pyproject.toml
        if let Some(pyproject) = find_upward(project_root, "pyproject.toml") {
            detected_markers.push(pyproject.clone());

            if let Ok(content) = std::fs::read_to_string(&pyproject) {
                if let Ok(table) = content.parse::<toml::Table>() {
                    // [project] section
                    if let Some(project) = table.get("project").and_then(|v| v.as_table()) {
                        if let Some(name) = project.get("name").and_then(|v| v.as_str()) {
                            data.insert("name".into(), ContextValue::String(name.to_string()));
                        }
                        if let Some(version) = project.get("version").and_then(|v| v.as_str()) {
                            data.insert(
                                "version".into(),
                                ContextValue::String(version.to_string()),
                            );
                        }
                        if let Some(requires_python) =
                            project.get("requires-python").and_then(|v| v.as_str())
                        {
                            data.insert(
                                "required_python".into(),
                                ContextValue::String(requires_python.to_string()),
                            );
                        }

                        // Dependencies
                        if let Some(deps) = project.get("dependencies").and_then(|v| v.as_array())
                        {
                            let dep_names: Vec<String> = deps
                                .iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| {
                                    // Strip version specifiers: "flask>=2.0" -> "flask"
                                    s.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                                        .next()
                                        .unwrap_or(s)
                                        .to_lowercase()
                                })
                                .collect();

                            // Framework detection
                            for (dep, framework) in FRAMEWORK_DEPS {
                                if dep_names.iter().any(|d| d == dep) {
                                    data.insert(
                                        "framework".into(),
                                        ContextValue::String(framework.to_string()),
                                    );
                                    break;
                                }
                            }

                            data.insert("dependencies".into(), ContextValue::List(dep_names));
                        }

                        // Scripts
                        if let Some(scripts) =
                            project.get("scripts").and_then(|v| v.as_table())
                        {
                            let script_names: Vec<String> =
                                scripts.keys().map(|s| s.to_string()).collect();
                            data.insert("scripts".into(), ContextValue::List(script_names));
                        }
                    }

                    // Build system
                    if let Some(build_system) =
                        table.get("build-system").and_then(|v| v.as_table())
                    {
                        if let Some(backend) =
                            build_system.get("build-backend").and_then(|v| v.as_str())
                        {
                            data.insert(
                                "build_system".into(),
                                ContextValue::String(backend.to_string()),
                            );
                        }
                    }
                }
            }

            // Package manager detection
            if find_upward(project_root, "poetry.lock").is_some() {
                data.insert(
                    "package_manager".into(),
                    ContextValue::String("poetry".into()),
                );
            } else if find_upward(project_root, "uv.lock").is_some() {
                data.insert(
                    "package_manager".into(),
                    ContextValue::String("uv".into()),
                );
            } else if find_upward(project_root, "pdm.lock").is_some() {
                data.insert(
                    "package_manager".into(),
                    ContextValue::String("pdm".into()),
                );
            } else {
                data.insert(
                    "package_manager".into(),
                    ContextValue::String("pip".into()),
                );
            }

            if let Some(root) = pyproject.parent() {
                data.insert(
                    "root".into(),
                    ContextValue::String(root.to_string_lossy().into_owned()),
                );
            }
        } else if find_upward(project_root, "Pipfile").is_some() {
            if let Some(pipfile) = find_upward(project_root, "Pipfile") {
                detected_markers.push(pipfile);
            }
            data.insert(
                "package_manager".into(),
                ContextValue::String("pipenv".into()),
            );
        } else if let Some(req_path) = find_upward(project_root, "requirements.txt") {
            detected_markers.push(req_path);
            data.insert(
                "package_manager".into(),
                ContextValue::String("pip".into()),
            );
        }

        // Check for virtual env
        let scan_dir = if cwd.starts_with(project_root) {
            cwd
        } else {
            project_root
        };
        for venv_name in &[".venv", "venv", "env"] {
            let venv_path = scan_dir.join(venv_name);
            if venv_path.join("pyvenv.cfg").exists() {
                data.insert(
                    "venv_path".into(),
                    ContextValue::String(venv_path.to_string_lossy().into_owned()),
                );
                // Check if active
                let venv_active = std::env::var("VIRTUAL_ENV")
                    .map(|v| v.contains(venv_name))
                    .unwrap_or(false);
                data.insert("venv_active".into(), ContextValue::Bool(venv_active));
                break;
            }
        }

        // Python version from .python-version
        if let Some(pv_path) = find_upward(project_root, ".python-version") {
            if let Ok(content) = std::fs::read_to_string(&pv_path) {
                data.insert(
                    "python_version".into(),
                    ContextValue::String(content.trim().to_string()),
                );
            }
        }

        // Test framework detection
        let has_pytest = project_root.join("pytest.ini").exists()
            || project_root.join("conftest.py").exists();
        let has_tests = has_pytest
            || project_root.join("tests").is_dir()
            || project_root.join("test").is_dir();
        data.insert("has_tests".into(), ContextValue::Bool(has_tests));
        if has_pytest {
            data.insert(
                "test_framework".into(),
                ContextValue::String("pytest".into()),
            );
        }

        Ok(ContextInfo {
            provider: "python".into(),
            data,
            detected_markers,
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        vec!["pip".into()]
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec![
            "pyproject.toml".into(),
            "requirements.txt".into(),
            "Pipfile".into(),
            "poetry.lock".into(),
            "uv.lock".into(),
        ]
    }

    fn priority(&self) -> u32 {
        120
    }
}
