//! Java context provider.

use std::collections::HashMap;
use std::path::Path;

use std::time::Duration;

use anyhow::Result;

use super::{find_upward, ContextInfo, ContextProvider, ContextValue};

pub struct JavaProvider;

impl ContextProvider for JavaProvider {
    fn name(&self) -> &str {
        "java"
    }

    fn markers(&self) -> &[&str] {
        &["pom.xml", "build.gradle", "build.gradle.kts"]
    }

    fn gather(&self, project_root: &Path, _cwd: &Path) -> Result<ContextInfo> {
        let mut data = HashMap::new();
        let mut detected_markers = Vec::new();

        // Detect build tool
        if let Some(pom_path) = find_upward(project_root, "pom.xml") {
            detected_markers.push(pom_path.clone());
            data.insert(
                "build_tool".into(),
                ContextValue::String("maven".into()),
            );

            // Basic pom.xml parsing (extract groupId, artifactId, version)
            if let Ok(content) = std::fs::read_to_string(&pom_path) {
                if let Some(group_id) = extract_xml_value(&content, "groupId") {
                    data.insert(
                        "group_id".into(),
                        ContextValue::String(group_id),
                    );
                }
                if let Some(artifact_id) = extract_xml_value(&content, "artifactId") {
                    data.insert(
                        "artifact_id".into(),
                        ContextValue::String(artifact_id),
                    );
                }
                if let Some(version) = extract_xml_value(&content, "version") {
                    data.insert(
                        "version".into(),
                        ContextValue::String(version),
                    );
                }
                // Java version
                if let Some(java_ver) = extract_xml_value(&content, "maven.compiler.source")
                    .or_else(|| extract_xml_value(&content, "java.version"))
                {
                    data.insert(
                        "java_version".into(),
                        ContextValue::String(java_ver),
                    );
                }

                // Multi-module check
                let is_multi_module = content.contains("<modules>");
                data.insert("is_multi_module".into(), ContextValue::Bool(is_multi_module));

                // Framework detection
                if content.contains("spring-boot") {
                    data.insert(
                        "framework".into(),
                        ContextValue::String("Spring Boot".into()),
                    );
                } else if content.contains("quarkus") {
                    data.insert(
                        "framework".into(),
                        ContextValue::String("Quarkus".into()),
                    );
                }
            }

            // Check for Maven wrapper
            if let Some(root) = pom_path.parent() {
                let has_wrapper = root.join("mvnw").exists() || root.join("mvnw.cmd").exists();
                data.insert("has_wrapper".into(), ContextValue::Bool(has_wrapper));
            }
        } else if let Some(gradle_path) = find_upward(project_root, "build.gradle")
            .or_else(|| find_upward(project_root, "build.gradle.kts"))
        {
            detected_markers.push(gradle_path.clone());
            let is_kotlin = gradle_path
                .extension()
                .is_some_and(|ext| ext == "kts");
            data.insert(
                "build_tool".into(),
                ContextValue::String("gradle".into()),
            );
            data.insert("kotlin".into(), ContextValue::Bool(is_kotlin));

            // Check for Gradle wrapper
            if let Some(root) = gradle_path.parent() {
                let has_wrapper =
                    root.join("gradlew").exists() || root.join("gradlew.bat").exists();
                data.insert("has_wrapper".into(), ContextValue::Bool(has_wrapper));

                // Multi-module check
                let is_multi_module = root.join("settings.gradle").exists()
                    || root.join("settings.gradle.kts").exists();
                data.insert("is_multi_module".into(), ContextValue::Bool(is_multi_module));

                data.insert(
                    "root".into(),
                    ContextValue::String(root.to_string_lossy().into_owned()),
                );
            }

            // Framework detection from build file
            if let Ok(content) = std::fs::read_to_string(&gradle_path) {
                if content.contains("spring-boot") || content.contains("org.springframework.boot")
                {
                    data.insert(
                        "framework".into(),
                        ContextValue::String("Spring Boot".into()),
                    );
                }
            }
        }

        Ok(ContextInfo {
            provider: "java".into(),
            data,
            detected_markers,
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        vec!["java".into()]
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec![
            "pom.xml".into(),
            "build.gradle".into(),
            "build.gradle.kts".into(),
        ]
    }

    fn priority(&self) -> u32 {
        120
    }

    fn cache_ttl(&self) -> Duration {
        Duration::from_secs(120)
    }
}

/// Simple XML value extraction (finds first occurrence of <tag>value</tag>).
fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    if let Some(start) = xml.find(&open) {
        let value_start = start + open.len();
        if let Some(end) = xml[value_start..].find(&close) {
            let value = xml[value_start..value_start + end].trim();
            if !value.is_empty() && !value.starts_with("${") {
                return Some(value.to_string());
            }
        }
    }
    None
}
