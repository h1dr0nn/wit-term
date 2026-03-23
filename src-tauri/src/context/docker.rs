//! Docker context provider.

use std::collections::HashMap;
use std::path::Path;

use std::time::Duration;

use anyhow::Result;

use super::{find_upward, ContextInfo, ContextProvider, ContextValue};

pub struct DockerProvider;

const DOCKER_MARKERS: &[&str] = &[
    "Dockerfile",
    "docker-compose.yml",
    "docker-compose.yaml",
    "compose.yml",
    "compose.yaml",
    ".dockerignore",
];

impl ContextProvider for DockerProvider {
    fn name(&self) -> &str {
        "docker"
    }

    fn markers(&self) -> &[&str] {
        DOCKER_MARKERS
    }

    fn detect(&self, dir: &Path) -> bool {
        DOCKER_MARKERS
            .iter()
            .any(|marker| find_upward(dir, marker).is_some())
    }

    fn gather(&self, project_root: &Path, _cwd: &Path) -> Result<ContextInfo> {
        let mut data = HashMap::new();
        let mut detected_markers = Vec::new();

        let has_dockerfile = find_upward(project_root, "Dockerfile").is_some();
        if has_dockerfile {
            if let Some(df) = find_upward(project_root, "Dockerfile") {
                detected_markers.push(df.clone());

                // Parse base image and multi-stage from Dockerfile
                if let Ok(content) = std::fs::read_to_string(&df) {
                    let from_lines: Vec<&str> = content
                        .lines()
                        .filter(|l| l.trim_start().to_uppercase().starts_with("FROM "))
                        .collect();

                    if let Some(first_from) = from_lines.first() {
                        let parts: Vec<&str> = first_from.split_whitespace().collect();
                        if parts.len() >= 2 {
                            data.insert(
                                "base_image".into(),
                                ContextValue::String(parts[1].to_string()),
                            );
                        }
                    }

                    data.insert(
                        "multi_stage".into(),
                        ContextValue::Bool(from_lines.len() > 1),
                    );
                }
            }
        }
        data.insert("has_dockerfile".into(), ContextValue::Bool(has_dockerfile));

        // Compose file detection
        let compose_files = [
            "docker-compose.yml",
            "docker-compose.yaml",
            "compose.yml",
            "compose.yaml",
        ];
        let mut compose_file = None;
        for f in &compose_files {
            if let Some(path) = find_upward(project_root, f) {
                detected_markers.push(path.clone());
                compose_file = Some((f.to_string(), path));
                break;
            }
        }

        let has_compose = compose_file.is_some();
        data.insert("has_compose".into(), ContextValue::Bool(has_compose));

        if let Some((name, path)) = compose_file {
            data.insert("compose_file".into(), ContextValue::String(name));

            // Basic service extraction: look for lines like "  servicename:" under "services:"
            if let Ok(content) = std::fs::read_to_string(&path) {
                let mut in_services = false;
                let mut services = Vec::new();
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed == "services:" {
                        in_services = true;
                        continue;
                    }
                    if in_services {
                        // A service is a top-level key under services (2-space indent typically)
                        if !line.starts_with(' ') && !line.is_empty() {
                            break; // Left the services block
                        }
                        if let Some(stripped) = line.strip_prefix("  ") {
                            if !stripped.starts_with(' ')
                                && stripped.ends_with(':')
                                && !stripped.starts_with('#')
                            {
                                let svc = stripped.trim_end_matches(':').trim();
                                if !svc.is_empty() {
                                    services.push(svc.to_string());
                                }
                            }
                        }
                    }
                }
                if !services.is_empty() {
                    data.insert("compose_services".into(), ContextValue::List(services));
                }
            }
        }

        // Dockerignore
        let has_dockerignore = find_upward(project_root, ".dockerignore").is_some();
        data.insert(
            "has_dockerignore".into(),
            ContextValue::Bool(has_dockerignore),
        );

        Ok(ContextInfo {
            provider: "docker".into(),
            data,
            detected_markers,
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        vec!["docker".into()]
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec![
            "Dockerfile".into(),
            "docker-compose.yml".into(),
            "docker-compose.yaml".into(),
            "compose.yml".into(),
            "compose.yaml".into(),
        ]
    }

    fn priority(&self) -> u32 {
        80
    }

    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(120)
    }
}
