//! Node.js context provider.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use super::{find_upward, ContextInfo, ContextProvider, ContextValue};

pub struct NodeProvider;

/// Frameworks detected from package.json dependencies.
const FRAMEWORK_DEPS: &[(&str, &str)] = &[
    ("next", "Next.js"),
    ("nuxt", "Nuxt"),
    ("@angular/core", "Angular"),
    ("vue", "Vue"),
    ("svelte", "Svelte"),
    ("react", "React"),
    ("express", "Express"),
    ("fastify", "Fastify"),
    ("@nestjs/core", "NestJS"),
    ("astro", "Astro"),
    ("@remix-run/react", "Remix"),
];

impl ContextProvider for NodeProvider {
    fn name(&self) -> &str {
        "node"
    }

    fn markers(&self) -> &[&str] {
        &["package.json"]
    }

    fn gather(&self, project_root: &Path, _cwd: &Path) -> Result<ContextInfo> {
        let mut data = HashMap::new();
        let mut detected_markers = Vec::new();

        if let Some(pkg_path) = find_upward(project_root, "package.json") {
            detected_markers.push(pkg_path.clone());

            if let Ok(content) = std::fs::read_to_string(&pkg_path) {
                if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(name) = pkg.get("name").and_then(|v| v.as_str()) {
                        data.insert("name".into(), ContextValue::String(name.to_string()));
                    }
                    if let Some(version) = pkg.get("version").and_then(|v| v.as_str()) {
                        data.insert("version".into(), ContextValue::String(version.to_string()));
                    }
                    if let Some(private) = pkg.get("private").and_then(|v| v.as_bool()) {
                        data.insert("private".into(), ContextValue::Bool(private));
                    }

                    // Collect script names
                    if let Some(scripts) = pkg.get("scripts").and_then(|v| v.as_object()) {
                        let script_names: Vec<String> =
                            scripts.keys().map(|s| s.to_string()).collect();
                        data.insert("scripts".into(), ContextValue::List(script_names));
                    }

                    // Dependencies list
                    if let Some(deps) = pkg.get("dependencies").and_then(|v| v.as_object()) {
                        let dep_names: Vec<String> =
                            deps.keys().map(|s| s.to_string()).collect();
                        data.insert("dependencies".into(), ContextValue::List(dep_names.clone()));

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
                    }

                    if let Some(dev_deps) =
                        pkg.get("devDependencies").and_then(|v| v.as_object())
                    {
                        let dev_dep_names: Vec<String> =
                            dev_deps.keys().map(|s| s.to_string()).collect();

                        // Check for TypeScript
                        let has_typescript = dev_dep_names.iter().any(|d| d == "typescript");
                        data.insert("has_typescript".into(), ContextValue::Bool(has_typescript));

                        // Check for ESLint
                        let has_eslint = dev_dep_names.iter().any(|d| d == "eslint");
                        data.insert("has_eslint".into(), ContextValue::Bool(has_eslint));

                        data.insert(
                            "dev_dependencies".into(),
                            ContextValue::List(dev_dep_names),
                        );
                    }

                    // Engine node version
                    if let Some(engines) = pkg.get("engines").and_then(|v| v.as_object()) {
                        if let Some(node_ver) = engines.get("node").and_then(|v| v.as_str()) {
                            data.insert(
                                "engine_node".into(),
                                ContextValue::String(node_ver.to_string()),
                            );
                        }
                    }
                }
            }

            // Detect package manager
            if let Some(parent) = pkg_path.parent() {
                if parent.join("pnpm-lock.yaml").exists() {
                    data.insert(
                        "package_manager".into(),
                        ContextValue::String("pnpm".into()),
                    );
                } else if parent.join("yarn.lock").exists() {
                    data.insert(
                        "package_manager".into(),
                        ContextValue::String("yarn".into()),
                    );
                } else if parent.join("bun.lockb").exists()
                    || parent.join("bun.lock").exists()
                {
                    data.insert(
                        "package_manager".into(),
                        ContextValue::String("bun".into()),
                    );
                } else {
                    data.insert(
                        "package_manager".into(),
                        ContextValue::String("npm".into()),
                    );
                }

                // Monorepo detection
                let is_monorepo = parent.join("pnpm-workspace.yaml").exists()
                    || parent.join("lerna.json").exists();
                data.insert("is_monorepo".into(), ContextValue::Bool(is_monorepo));

                data.insert(
                    "root".into(),
                    ContextValue::String(parent.to_string_lossy().into_owned()),
                );
            }
        }

        // Detect Node.js runtime version (node --version)
        if let Ok(output) = std::process::Command::new("node")
            .arg("--version")
            .output()
        {
            if output.status.success() {
                let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
                // node --version returns "v24.14.0", store without the leading 'v'
                let ver = ver.strip_prefix('v').unwrap_or(&ver).to_string();
                data.insert("runtime_version".into(), ContextValue::String(ver));
            }
        }

        Ok(ContextInfo {
            provider: "node".into(),
            data,
            detected_markers,
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        vec!["npm".into(), "yarn".into(), "pnpm".into()]
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec![
            "package.json".into(),
            "package-lock.json".into(),
            "pnpm-lock.yaml".into(),
            "yarn.lock".into(),
        ]
    }

    fn priority(&self) -> u32 {
        150
    }
}
