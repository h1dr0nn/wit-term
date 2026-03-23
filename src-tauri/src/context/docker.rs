//! Docker context provider.

use std::collections::HashMap;
use std::path::Path;

use super::{find_upward, ContextInfo, ContextProvider};

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

    fn detect(&self, dir: &Path) -> bool {
        DOCKER_MARKERS
            .iter()
            .any(|marker| find_upward(dir, marker).is_some())
    }

    fn gather(&self, dir: &Path) -> ContextInfo {
        let mut data = HashMap::new();

        let has_dockerfile = find_upward(dir, "Dockerfile").is_some();
        let has_compose = ["docker-compose.yml", "docker-compose.yaml", "compose.yml", "compose.yaml"]
            .iter()
            .any(|f| find_upward(dir, f).is_some());

        if has_dockerfile {
            data.insert("dockerfile".into(), "true".into());
        }
        if has_compose {
            data.insert("compose".into(), "true".into());
        }

        ContextInfo {
            provider: "docker".into(),
            detected: true,
            data,
        }
    }
}
