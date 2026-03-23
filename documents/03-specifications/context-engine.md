# Context Engine

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

The Context Engine is the module responsible for detecting and tracking the environment surrounding a terminal session. Instead of requiring the user to configure things manually, Wit automatically scans directory structure, reads config files, and builds a context model to provide relevant completions.

The module is located at `src-tauri/src/context/` and is the component that forms Wit's core differentiator compared to other terminal emulators.

**Principles:**
- Only reads the filesystem, **never writes/modifies** project files
- All operations must be non-blocking or run on a background thread
- Graceful degradation - if detection fails, context = empty (no crash)
- Zero network access - everything is local

---

## What is Context

Context in Wit includes the following information:

```rust
/// The complete context of a terminal session at the current point in time
pub struct ProjectContext {
    /// Root directory of the project (where marker files were found)
    pub project_root: Option<PathBuf>,

    /// Current working directory (may differ from project_root)
    pub cwd: PathBuf,

    /// Active context providers and their data
    pub providers: HashMap<String, ContextInfo>,

    /// Timestamp when context was last updated
    pub last_updated: Instant,

    /// Combined completion sets to load
    pub completion_sets: Vec<String>,
}

/// Information from a specific provider
pub struct ContextInfo {
    /// Provider name (e.g., "git", "node", "rust")
    pub provider: String,

    /// Key-value data collected
    pub data: HashMap<String, ContextValue>,

    /// Marker files detected
    pub detected_markers: Vec<PathBuf>,
}

/// Value types for context data
pub enum ContextValue {
    String(String),
    Bool(bool),
    Number(f64),
    List(Vec<String>),
    Map(HashMap<String, String>),
}
```

### Real-world context example

When the user `cd`s into `/home/user/projects/my-app`:

```
ProjectContext {
    project_root: "/home/user/projects/my-app",
    cwd: "/home/user/projects/my-app/src",
    providers: {
        "git": {
            branch: "feature/auth",
            status: "modified",
            remote: "origin - github.com:user/my-app.git",
            stash_count: 2,
            has_uncommitted: true,
        },
        "node": {
            package_manager: "pnpm",
            node_version: "20.11.0",
            scripts: ["dev", "build", "test", "lint"],
            dependencies: ["react", "next", "typescript"],
            dev_dependencies: ["vitest", "eslint", "prettier"],
            framework: "next",
        },
        "docker": {
            has_dockerfile: true,
            has_compose: true,
            compose_services: ["web", "db", "redis"],
        },
    },
    completion_sets: ["git", "node", "pnpm", "next", "docker", "docker-compose"],
}
```

---

## Context Provider Trait

### Interface Design

```rust
/// Trait for each context provider
pub trait ContextProvider: Send + Sync {
    /// Unique name of the provider (e.g., "git", "node")
    fn name(&self) -> &str;

    /// Marker files/directories used to detect the provider
    /// The engine will search for these markers when scanning a directory
    fn markers(&self) -> &[&str];

    /// Quick check whether the provider is active at this directory
    /// Based only on the existence of marker files
    fn detect(&self, project_root: &Path) -> bool {
        self.markers().iter().any(|marker| {
            project_root.join(marker).exists()
        })
    }

    /// Collect detailed context information
    /// Runs on a background thread, may take some time
    fn gather(&self, project_root: &Path, cwd: &Path) -> Result<ContextInfo>;

    /// List of completion set names to load when the provider is active
    fn completion_sets(&self) -> Vec<String>;

    /// Files to watch for cache invalidation
    /// The provider will be re-gathered when these files change
    fn watch_patterns(&self) -> Vec<String>;

    /// Priority - providers with higher priority run first
    /// Useful when providers have dependencies (e.g., node needs to know if it is a monorepo)
    fn priority(&self) -> u32 { 100 }
}
```

### Provider Registry

```rust
pub struct ContextEngine {
    /// Registered providers, sorted by priority
    providers: Vec<Box<dyn ContextProvider>>,

    /// File system watcher
    watcher: Option<RecommendedWatcher>,

    /// Current context (shared with other modules)
    current: Arc<RwLock<ProjectContext>>,

    /// Event channel for context changes
    event_tx: Sender<ContextEvent>,

    /// Cache of provider results
    cache: HashMap<String, CachedContext>,
}

struct CachedContext {
    info: ContextInfo,
    gathered_at: Instant,
    file_hashes: HashMap<PathBuf, u64>, // For change detection
}

pub enum ContextEvent {
    /// Context has changed (sent to frontend and completion engine)
    ContextChanged(ProjectContext),

    /// CWD has changed
    CwdChanged(PathBuf),

    /// A provider was detected/lost
    ProviderActivated(String),
    ProviderDeactivated(String),
}
```

---

## Built-in Providers

### GitProvider

**Marker files:** `.git`, `.git/` (directory)

**Information collected:**

| Key | Type | Description | Source |
|---|---|---|---|
| `branch` | String | Current branch name | `HEAD` file or `git rev-parse` |
| `detached` | Bool | HEAD is in detached state | `HEAD` content |
| `commit_short` | String | Short hash of HEAD commit | `git rev-parse --short HEAD` |
| `status` | String | "clean" / "modified" / "staged" / "conflict" | `git status --porcelain` |
| `modified_count` | Number | Number of modified files | `git status --porcelain` |
| `staged_count` | Number | Number of staged files | `git status --porcelain` |
| `untracked_count` | Number | Number of untracked files | `git status --porcelain` |
| `remote` | String | Remote URL | `git remote get-url origin` |
| `remote_name` | String | Remote name (usually "origin") | `git remote` |
| `ahead` | Number | Number of commits ahead of remote | `git rev-list @{u}..HEAD --count` |
| `behind` | Number | Number of commits behind remote | `git rev-list HEAD..@{u} --count` |
| `stash_count` | Number | Number of stash entries | `git stash list \| wc -l` |
| `tags` | List | Tags pointing to HEAD | `git tag --points-at HEAD` |
| `is_merge` | Bool | Currently in merge state | `.git/MERGE_HEAD` exists |
| `is_rebase` | Bool | Currently in rebase state | `.git/rebase-merge/` exists |
| `has_conflicts` | Bool | Has merge conflicts | `git status --porcelain` has `UU` |

**Implementation strategy: libgit2 vs git CLI**

| Approach | Advantages | Disadvantages |
|---|---|---|
| **libgit2** (via `git2` crate) | Fast (no process spawn), no git dependency | Large crate (~2MB), some operations harder than CLI |
| **git CLI** (`Command::new("git")`) | Simple, always up-to-date, handles edge cases | Slower (process spawn), git must be installed |

**Recommendation:** Use **git CLI** for MVP because:
1. Simple implementation
2. Handles all edge cases (worktrees, submodules, etc.)
3. The user certainly has git installed if there is a `.git` directory
4. Migrate to libgit2 when performance optimization is needed

```rust
pub struct GitProvider;

impl ContextProvider for GitProvider {
    fn name(&self) -> &str { "git" }

    fn markers(&self) -> &[&str] { &[".git"] }

    fn gather(&self, project_root: &Path, _cwd: &Path) -> Result<ContextInfo> {
        let mut data = HashMap::new();

        // Branch
        let branch = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(project_root)
            .output()?;
        data.insert("branch".into(),
            ContextValue::String(String::from_utf8_lossy(&branch.stdout).trim().into()));

        // Status (porcelain for machine parsing)
        let status = Command::new("git")
            .args(["status", "--porcelain=v1", "--branch"])
            .current_dir(project_root)
            .output()?;
        let (modified, staged, untracked) = parse_git_status(&status.stdout);
        data.insert("modified_count".into(), ContextValue::Number(modified as f64));
        data.insert("staged_count".into(), ContextValue::Number(staged as f64));
        data.insert("untracked_count".into(), ContextValue::Number(untracked as f64));

        // ... more fields

        Ok(ContextInfo {
            provider: "git".into(),
            data,
            detected_markers: vec![project_root.join(".git")],
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        vec!["git".into()]
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec![
            ".git/HEAD".into(),           // Branch changes
            ".git/index".into(),          // Staging area changes
            ".git/refs/".into(),          // Branch/tag updates
            ".git/MERGE_HEAD".into(),     // Merge state
            ".git/rebase-merge/".into(),  // Rebase state
        ]
    }

    fn priority(&self) -> u32 { 200 } // High priority - most common
}
```

### NodeProvider

**Marker files:** `package.json`

**Information collected:**

| Key | Type | Description | Source |
|---|---|---|---|
| `name` | String | Package name | `package.json - name` |
| `version` | String | Package version | `package.json - version` |
| `private` | Bool | Is private package | `package.json - private` |
| `package_manager` | String | "npm" / "yarn" / "pnpm" / "bun" | Lock file detection |
| `pm_version` | String | Package manager version | `npm --version` etc. |
| `node_version` | String | Node.js version | `node --version` |
| `scripts` | List | Available npm scripts | `package.json - scripts (keys)` |
| `dependencies` | List | Production dependency names | `package.json - dependencies (keys)` |
| `dev_dependencies` | List | Dev dependency names | `package.json - devDependencies (keys)` |
| `framework` | String | Detected framework | Dependency analysis |
| `is_monorepo` | Bool | Workspace root | `package.json - workspaces` |
| `has_typescript` | Bool | TypeScript configured | `tsconfig.json` exists |
| `has_eslint` | Bool | ESLint configured | `.eslintrc*` exists |
| `engine_node` | String | Required Node version | `package.json - engines.node` |

**Package manager detection:**

| Lock file | Package manager |
|---|---|
| `pnpm-lock.yaml` | pnpm |
| `yarn.lock` | yarn |
| `bun.lockb` / `bun.lock` | bun |
| `package-lock.json` | npm |
| (none of above) | npm (default) |

**Framework detection:**

| Dependencies | Framework |
|---|---|
| `next` | Next.js |
| `nuxt` | Nuxt |
| `@angular/core` | Angular |
| `vue` (without nuxt) | Vue.js |
| `svelte` or `@sveltejs/kit` | Svelte/SvelteKit |
| `react` (without next) | React |
| `express` | Express.js |
| `fastify` | Fastify |
| `@nestjs/core` | NestJS |
| `astro` | Astro |
| `remix` or `@remix-run/react` | Remix |

```rust
pub struct NodeProvider;

impl ContextProvider for NodeProvider {
    fn name(&self) -> &str { "node" }
    fn markers(&self) -> &[&str] { &["package.json"] }

    fn gather(&self, project_root: &Path, _cwd: &Path) -> Result<ContextInfo> {
        let pkg_path = project_root.join("package.json");
        let pkg: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&pkg_path)?
        )?;

        let mut data = HashMap::new();

        // Name & version
        if let Some(name) = pkg["name"].as_str() {
            data.insert("name".into(), ContextValue::String(name.into()));
        }

        // Scripts
        if let Some(scripts) = pkg["scripts"].as_object() {
            let script_names: Vec<String> = scripts.keys().cloned().collect();
            data.insert("scripts".into(), ContextValue::List(script_names));
        }

        // Package manager detection
        let pm = if project_root.join("pnpm-lock.yaml").exists() {
            "pnpm"
        } else if project_root.join("yarn.lock").exists() {
            "yarn"
        } else if project_root.join("bun.lockb").exists()
               || project_root.join("bun.lock").exists() {
            "bun"
        } else {
            "npm"
        };
        data.insert("package_manager".into(), ContextValue::String(pm.into()));

        // Dependencies
        if let Some(deps) = pkg["dependencies"].as_object() {
            let dep_names: Vec<String> = deps.keys().cloned().collect();

            // Framework detection
            let framework = detect_framework(&dep_names);
            if let Some(fw) = framework {
                data.insert("framework".into(), ContextValue::String(fw));
            }

            data.insert("dependencies".into(), ContextValue::List(dep_names));
        }

        // Monorepo
        let is_monorepo = pkg["workspaces"].is_array()
            || project_root.join("pnpm-workspace.yaml").exists()
            || project_root.join("lerna.json").exists();
        data.insert("is_monorepo".into(), ContextValue::Bool(is_monorepo));

        Ok(ContextInfo {
            provider: "node".into(),
            data,
            detected_markers: vec![pkg_path],
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        // Dynamic - depends on detected package manager and framework
        // Base: ["node"]
        // + package manager: ["npm"] / ["yarn"] / ["pnpm"]
        // + framework: ["next"] / ["nuxt"] / etc.
        vec!["node".into()] // Extended dynamically based on gather results
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec![
            "package.json".into(),
            "package-lock.json".into(),
            "yarn.lock".into(),
            "pnpm-lock.yaml".into(),
            "tsconfig.json".into(),
        ]
    }
}
```

### PythonProvider

**Marker files:** `pyproject.toml`, `setup.py`, `setup.cfg`, `requirements.txt`, `Pipfile`, `.python-version`

**Information collected:**

| Key | Type | Description | Source |
|---|---|---|---|
| `name` | String | Project name | `pyproject.toml - project.name` |
| `version` | String | Project version | `pyproject.toml - project.version` |
| `python_version` | String | Python version | `python --version` |
| `required_python` | String | Required Python version | `pyproject.toml - project.requires-python` |
| `build_system` | String | Build backend | `pyproject.toml - build-system.build-backend` |
| `package_manager` | String | "pip" / "poetry" / "pipenv" / "uv" / "pdm" / "hatch" | Lock file / tool detection |
| `venv_path` | String | Virtual environment path | `.venv/`, `venv/`, `$VIRTUAL_ENV` |
| `venv_active` | Bool | Virtualenv is active | `$VIRTUAL_ENV` set |
| `dependencies` | List | Project dependencies | `pyproject.toml - dependencies` |
| `scripts` | List | Entry points / scripts | `pyproject.toml - project.scripts` |
| `has_tests` | Bool | Test directory exists | `tests/`, `test/` |
| `test_framework` | String | "pytest" / "unittest" | Dependency detection |
| `framework` | String | "django" / "flask" / "fastapi" | Dependency detection |

**Package manager / tool detection:**

| Indicator | Tool |
|---|---|
| `poetry.lock` | Poetry |
| `Pipfile.lock` | Pipenv |
| `uv.lock` | uv |
| `pdm.lock` | PDM |
| `pyproject.toml` has `[tool.hatch]` | Hatch |
| `pyproject.toml` has `[tool.poetry]` | Poetry |
| `requirements.txt` only | pip |

**Virtualenv detection:**

```rust
fn detect_venv(project_root: &Path) -> Option<PathBuf> {
    // 1. Check environment variable
    if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
        return Some(PathBuf::from(venv));
    }

    // 2. Check common venv directories
    for dir_name in &[".venv", "venv", "env", ".env"] {
        let venv_dir = project_root.join(dir_name);
        let activate = if cfg!(windows) {
            venv_dir.join("Scripts").join("activate")
        } else {
            venv_dir.join("bin").join("activate")
        };
        if activate.exists() {
            return Some(venv_dir);
        }
    }

    // 3. Check pyenv
    if project_root.join(".python-version").exists() {
        // pyenv virtualenv
    }

    None
}
```

### RustProvider

**Marker files:** `Cargo.toml`

**Information collected:**

| Key | Type | Description | Source |
|---|---|---|---|
| `name` | String | Crate name | `Cargo.toml - package.name` |
| `version` | String | Crate version | `Cargo.toml - package.version` |
| `edition` | String | Rust edition (2021, 2024) | `Cargo.toml - package.edition` |
| `rust_version` | String | MSRV | `Cargo.toml - package.rust-version` |
| `is_workspace` | Bool | Is workspace root | `Cargo.toml - [workspace]` exists |
| `workspace_members` | List | Workspace member crates | `Cargo.toml - workspace.members` |
| `targets` | List | Binary/library targets | `Cargo.toml - [[bin]], [lib]` |
| `features` | List | Available features | `Cargo.toml - [features]` keys |
| `dependencies` | List | Dependency names | `Cargo.toml - [dependencies]` keys |
| `has_build_script` | Bool | build.rs exists | `build.rs` exists |
| `toolchain` | String | Active toolchain | `rustup show active-toolchain` |

```rust
pub struct RustProvider;

impl ContextProvider for RustProvider {
    fn name(&self) -> &str { "rust" }
    fn markers(&self) -> &[&str] { &["Cargo.toml"] }

    fn gather(&self, project_root: &Path, _cwd: &Path) -> Result<ContextInfo> {
        let cargo_path = project_root.join("Cargo.toml");
        let cargo: toml::Value = toml::from_str(
            &std::fs::read_to_string(&cargo_path)?
        )?;

        let mut data = HashMap::new();

        // Package info
        if let Some(pkg) = cargo.get("package") {
            if let Some(name) = pkg.get("name").and_then(|v| v.as_str()) {
                data.insert("name".into(), ContextValue::String(name.into()));
            }
            if let Some(edition) = pkg.get("edition").and_then(|v| v.as_str()) {
                data.insert("edition".into(), ContextValue::String(edition.into()));
            }
        }

        // Workspace detection
        let is_workspace = cargo.get("workspace").is_some();
        data.insert("is_workspace".into(), ContextValue::Bool(is_workspace));

        if is_workspace {
            if let Some(members) = cargo.get("workspace")
                .and_then(|w| w.get("members"))
                .and_then(|m| m.as_array())
            {
                let member_names: Vec<String> = members.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                data.insert("workspace_members".into(), ContextValue::List(member_names));
            }
        }

        // Features
        if let Some(features) = cargo.get("features").and_then(|f| f.as_table()) {
            let feature_names: Vec<String> = features.keys().cloned().collect();
            data.insert("features".into(), ContextValue::List(feature_names));
        }

        // Dependencies
        if let Some(deps) = cargo.get("dependencies").and_then(|d| d.as_table()) {
            let dep_names: Vec<String> = deps.keys().cloned().collect();
            data.insert("dependencies".into(), ContextValue::List(dep_names));
        }

        Ok(ContextInfo {
            provider: "rust".into(),
            data,
            detected_markers: vec![cargo_path],
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        vec!["cargo".into(), "rustup".into()]
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec![
            "Cargo.toml".into(),
            "Cargo.lock".into(),
            "rust-toolchain.toml".into(),
            "rust-toolchain".into(),
        ]
    }
}
```

### DockerProvider

**Marker files:** `Dockerfile`, `docker-compose.yml`, `docker-compose.yaml`, `compose.yml`, `compose.yaml`

**Information collected:**

| Key | Type | Description | Source |
|---|---|---|---|
| `has_dockerfile` | Bool | Dockerfile exists | File existence |
| `has_compose` | Bool | Compose file exists | File existence |
| `compose_services` | List | Service names in compose | Parse compose YAML |
| `compose_file` | String | Path to compose file | First match |
| `base_image` | String | FROM image in Dockerfile | Parse Dockerfile |
| `has_dockerignore` | Bool | .dockerignore exists | File existence |
| `multi_stage` | Bool | Multiple FROM instructions | Parse Dockerfile |

```rust
pub struct DockerProvider;

impl ContextProvider for DockerProvider {
    fn name(&self) -> &str { "docker" }

    fn markers(&self) -> &[&str] {
        &[
            "Dockerfile",
            "docker-compose.yml",
            "docker-compose.yaml",
            "compose.yml",
            "compose.yaml",
        ]
    }

    fn gather(&self, project_root: &Path, _cwd: &Path) -> Result<ContextInfo> {
        let mut data = HashMap::new();

        // Dockerfile
        let has_dockerfile = project_root.join("Dockerfile").exists();
        data.insert("has_dockerfile".into(), ContextValue::Bool(has_dockerfile));

        if has_dockerfile {
            let dockerfile = std::fs::read_to_string(project_root.join("Dockerfile"))?;
            let from_lines: Vec<&str> = dockerfile.lines()
                .filter(|l| l.trim_start().to_uppercase().starts_with("FROM "))
                .collect();

            if let Some(first_from) = from_lines.first() {
                let image = first_from.trim_start()
                    .strip_prefix("FROM ").unwrap_or("")
                    .split_whitespace().next().unwrap_or("");
                data.insert("base_image".into(), ContextValue::String(image.into()));
            }

            data.insert("multi_stage".into(),
                ContextValue::Bool(from_lines.len() > 1));
        }

        // Compose file
        let compose_files = [
            "docker-compose.yml", "docker-compose.yaml",
            "compose.yml", "compose.yaml",
        ];
        for compose_name in &compose_files {
            let compose_path = project_root.join(compose_name);
            if compose_path.exists() {
                data.insert("has_compose".into(), ContextValue::Bool(true));
                data.insert("compose_file".into(),
                    ContextValue::String(compose_name.to_string()));

                // Parse services (basic YAML parsing)
                if let Ok(content) = std::fs::read_to_string(&compose_path) {
                    let services = parse_compose_services(&content);
                    data.insert("compose_services".into(),
                        ContextValue::List(services));
                }
                break;
            }
        }

        // .dockerignore
        data.insert("has_dockerignore".into(),
            ContextValue::Bool(project_root.join(".dockerignore").exists()));

        Ok(ContextInfo {
            provider: "docker".into(),
            data,
            detected_markers: self.markers().iter()
                .map(|m| project_root.join(m))
                .filter(|p| p.exists())
                .collect(),
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        vec!["docker".into(), "docker-compose".into()]
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec![
            "Dockerfile".into(),
            "docker-compose.yml".into(),
            "docker-compose.yaml".into(),
            "compose.yml".into(),
            "compose.yaml".into(),
            ".dockerignore".into(),
        ]
    }
}
```

### GoProvider

**Marker files:** `go.mod`

**Information collected:**

| Key | Type | Description | Source |
|---|---|---|---|
| `module` | String | Module path | `go.mod - module` |
| `go_version` | String | Go version in go.mod | `go.mod - go` directive |
| `dependencies` | List | Direct dependency modules | `go.mod - require` |
| `has_vendor` | Bool | Vendor directory exists | `vendor/` exists |
| `has_go_sum` | Bool | go.sum exists | `go.sum` exists |
| `installed_go` | String | Installed Go version | `go version` |

```rust
pub struct GoProvider;

impl ContextProvider for GoProvider {
    fn name(&self) -> &str { "go" }
    fn markers(&self) -> &[&str] { &["go.mod"] }

    fn gather(&self, project_root: &Path, _cwd: &Path) -> Result<ContextInfo> {
        let go_mod = std::fs::read_to_string(project_root.join("go.mod"))?;
        let mut data = HashMap::new();

        // Parse module name
        for line in go_mod.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("module ") {
                let module = trimmed.strip_prefix("module ").unwrap().trim();
                data.insert("module".into(), ContextValue::String(module.into()));
            }
            if trimmed.starts_with("go ") {
                let version = trimmed.strip_prefix("go ").unwrap().trim();
                data.insert("go_version".into(), ContextValue::String(version.into()));
            }
        }

        // Dependencies (basic parsing of require block)
        let deps = parse_go_mod_require(&go_mod);
        data.insert("dependencies".into(), ContextValue::List(deps));

        data.insert("has_vendor".into(),
            ContextValue::Bool(project_root.join("vendor").is_dir()));
        data.insert("has_go_sum".into(),
            ContextValue::Bool(project_root.join("go.sum").exists()));

        Ok(ContextInfo {
            provider: "go".into(),
            data,
            detected_markers: vec![project_root.join("go.mod")],
        })
    }

    fn completion_sets(&self) -> Vec<String> {
        vec!["go".into()]
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec!["go.mod".into(), "go.sum".into()]
    }
}
```

### JavaProvider

**Marker files:** `pom.xml`, `build.gradle`, `build.gradle.kts`, `settings.gradle`, `settings.gradle.kts`

**Information collected:**

| Key | Type | Description | Source |
|---|---|---|---|
| `build_tool` | String | "maven" / "gradle" | Marker file detected |
| `group_id` | String | Maven groupId | `pom.xml` |
| `artifact_id` | String | Maven artifactId | `pom.xml` |
| `version` | String | Project version | `pom.xml` / `build.gradle` |
| `java_version` | String | Java version | `java -version` |
| `is_multi_module` | Bool | Multi-module project | `pom.xml` modules / `settings.gradle` includes |
| `has_wrapper` | Bool | Maven/Gradle wrapper | `mvnw` / `gradlew` exists |
| `framework` | String | "spring-boot" / "quarkus" / etc. | Dependency analysis |
| `kotlin` | Bool | Kotlin project | `.kt` files or `kotlin` plugin |

```rust
pub struct JavaProvider;

impl ContextProvider for JavaProvider {
    fn name(&self) -> &str { "java" }

    fn markers(&self) -> &[&str] {
        &["pom.xml", "build.gradle", "build.gradle.kts"]
    }

    fn completion_sets(&self) -> Vec<String> {
        // Dynamic based on build tool
        vec!["java".into()] // + "maven" or "gradle"
    }

    fn watch_patterns(&self) -> Vec<String> {
        vec![
            "pom.xml".into(),
            "build.gradle".into(),
            "build.gradle.kts".into(),
            "settings.gradle".into(),
            "settings.gradle.kts".into(),
        ]
    }
}
```

### GenericProvider

**Marker files:** `Makefile`, `.editorconfig`, `.env`, `.envrc`, `justfile`

GenericProvider detects common development files that do not belong to a specific language/framework.

**Information collected:**

| Key | Type | Description | Source |
|---|---|---|---|
| `has_makefile` | Bool | Makefile exists | File existence |
| `make_targets` | List | Available make targets | Parse Makefile |
| `has_justfile` | Bool | justfile exists | File existence |
| `just_recipes` | List | Available just recipes | Parse justfile |
| `has_editorconfig` | Bool | .editorconfig exists | File existence |
| `has_env` | Bool | .env file exists | File existence |
| `has_envrc` | Bool | .envrc (direnv) exists | File existence |
| `has_devcontainer` | Bool | .devcontainer/ exists | Directory existence |
| `has_vscode` | Bool | .vscode/ exists | Directory existence |
| `has_idea` | Bool | .idea/ exists | Directory existence |
| `ci_system` | String | CI system detected | `.github/workflows/`, `.gitlab-ci.yml`, etc. |

**Make target parsing:**

```rust
fn parse_makefile_targets(content: &str) -> Vec<String> {
    content.lines()
        .filter_map(|line| {
            // Match lines like "target: deps" but not variables or comments
            let line = line.trim();
            if line.starts_with('#') || line.starts_with('\t') || line.is_empty() {
                return None;
            }
            if let Some(colon_pos) = line.find(':') {
                let target = &line[..colon_pos];
                // Skip variable assignments (contain =)
                if target.contains('=') || target.contains('%') {
                    return None;
                }
                // Skip .PHONY and other special targets
                if target.starts_with('.') {
                    return None;
                }
                Some(target.trim().to_string())
            } else {
                None
            }
        })
        .collect()
}
```

**CI system detection:**

| Path | CI System |
|---|---|
| `.github/workflows/` | GitHub Actions |
| `.gitlab-ci.yml` | GitLab CI |
| `Jenkinsfile` | Jenkins |
| `.circleci/` | CircleCI |
| `.travis.yml` | Travis CI |
| `azure-pipelines.yml` | Azure DevOps |
| `bitbucket-pipelines.yml` | Bitbucket Pipelines |

---

## Directory Scanning Strategy

### Project Root Detection

When the user `cd`s into a directory, the Context Engine needs to find the project root. Strategy: **walk up from CWD**.

```rust
fn find_project_root(cwd: &Path, providers: &[Box<dyn ContextProvider>]) -> Option<PathBuf> {
    // Collect all marker files from all providers
    let all_markers: Vec<&str> = providers.iter()
        .flat_map(|p| p.markers().to_vec())
        .collect();

    // Walk up from CWD
    let mut current = cwd.to_path_buf();
    loop {
        for marker in &all_markers {
            if current.join(marker).exists() {
                return Some(current);
            }
        }

        // Move to parent directory
        if !current.pop() {
            break; // Reached filesystem root
        }
    }

    None // No project root found
}
```

**Example:**

```
CWD = /home/user/projects/my-app/src/components/

Scan order:
1. /home/user/projects/my-app/src/components/  <- no markers
2. /home/user/projects/my-app/src/             <- no markers
3. /home/user/projects/my-app/                 <- package.json found! -> project root
```

### Nested Projects

Some projects have nested structure (monorepo):

```
/workspace/              <- package.json (workspace root)
├── packages/
│   ├── frontend/        <- package.json (sub-package)
│   └── backend/         <- package.json + Cargo.toml
└── .git/
```

**Rules:**
1. Find the nearest project root first (closest to CWD)
2. Continue walking up to find the workspace root
3. The Git provider always uses the git root (where `.git/` is)
4. Context reports both project root and workspace root if they differ

```rust
pub struct ProjectRoots {
    /// Nearest project root (closest marker to CWD)
    pub project: PathBuf,

    /// Workspace root (furthest marker, if monorepo)
    /// None if there is no nested structure
    pub workspace: Option<PathBuf>,

    /// Git root (where .git/ is)
    /// May differ from project root and workspace root
    pub git_root: Option<PathBuf>,
}
```

### Performance Bounds

- Maximum walk-up depth: 20 levels (configurable)
- Skip scan if CWD is a system directory (`/`, `/usr`, `C:\Windows`, etc.)
- Cache negative results: if a directory has no markers, cache "no project" for 30 seconds

---

## File System Watching

### What to Watch

Each active provider registers watch patterns (see the `watch_patterns()` method). The Context Engine aggregates and watches all patterns.

```rust
fn setup_watcher(
    project_root: &Path,
    active_providers: &[&dyn ContextProvider],
) -> Result<RecommendedWatcher> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;

    // Watch project root recursively
    watcher.watch(project_root, RecursiveMode::Recursive)?;

    Ok(watcher)
}
```

### Debouncing

File system events often arrive in bursts (IDE save, git operations, package install). Debouncing is needed to avoid re-scanning too many times.

```rust
pub struct DebouncedWatcher {
    watcher: RecommendedWatcher,
    pending_events: HashMap<PathBuf, Instant>,
    debounce_duration: Duration,
}

impl DebouncedWatcher {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            watcher: /* ... */,
            pending_events: HashMap::new(),
            debounce_duration: Duration::from_millis(debounce_ms),
        }
    }

    /// Process pending events, return paths that are ready
    pub fn poll(&mut self) -> Vec<PathBuf> {
        let now = Instant::now();
        let ready: Vec<PathBuf> = self.pending_events.iter()
            .filter(|(_, &timestamp)| now.duration_since(timestamp) >= self.debounce_duration)
            .map(|(path, _)| path.clone())
            .collect();

        for path in &ready {
            self.pending_events.remove(path);
        }

        ready
    }
}
```

**Debounce settings:**
- Default: 300ms
- Git operations (many file changes): coalesce into 1 re-scan
- `package.json` / `Cargo.toml` changes: re-scan immediately (low debounce 50ms)

### Performance Considerations

| Concern | Mitigation |
|---|---|
| Too many watchers | Watch project root recursively, filter events by pattern |
| Large `node_modules/` | Exclude `node_modules/`, `target/`, `.git/objects/` from watch |
| High-frequency events | Debounce 300ms |
| Cross-platform differences | Use `notify` crate - abstracts inotify (Linux), FSEvents (macOS), ReadDirectoryChangesW (Windows) |

**Excluded directories (never watch):**

```rust
const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    ".git/objects",
    ".git/logs",
    "target",          // Rust build output
    "dist",
    "build",
    ".next",
    "__pycache__",
    ".venv",
    "venv",
    ".tox",
    "vendor",          // Go vendor
    ".gradle",
    ".idea",
    ".vscode",
];
```

---

## Context Caching and Invalidation

### Cache Strategy

```rust
pub struct ContextCache {
    entries: HashMap<String, CacheEntry>,
}

struct CacheEntry {
    info: ContextInfo,
    gathered_at: Instant,
    ttl: Duration,
    /// Hash of relevant files at gather time
    file_checksums: HashMap<PathBuf, u64>,
}

impl ContextCache {
    /// Get cached context if still valid
    fn get(&self, provider_name: &str) -> Option<&ContextInfo> {
        let entry = self.entries.get(provider_name)?;

        // TTL check
        if entry.gathered_at.elapsed() > entry.ttl {
            return None;
        }

        Some(&entry.info)
    }

    /// Invalidate cache for specific provider
    fn invalidate(&mut self, provider_name: &str) {
        self.entries.remove(provider_name);
    }

    /// Invalidate all providers affected by a file change
    fn invalidate_for_path(&mut self, changed_path: &Path) {
        let providers_to_invalidate: Vec<String> = self.entries.iter()
            .filter(|(_, entry)| {
                entry.file_checksums.keys().any(|watched| {
                    changed_path == watched || changed_path.starts_with(watched)
                })
            })
            .map(|(name, _)| name.clone())
            .collect();

        for name in providers_to_invalidate {
            self.entries.remove(&name);
        }
    }
}
```

### TTL per Provider

| Provider | TTL | Reason |
|---|---|---|
| Git | 5 seconds | Git status changes frequently |
| Node | 60 seconds | package.json changes rarely |
| Python | 60 seconds | pyproject.toml changes rarely |
| Rust | 60 seconds | Cargo.toml changes rarely |
| Docker | 120 seconds | Docker files change very rarely |
| Go | 60 seconds | go.mod changes rarely |
| Java | 120 seconds | pom.xml/build.gradle change rarely |
| Generic | 120 seconds | Static config files |

### Invalidation Triggers

1. **File system event** - `invalidate_for_path()` - re-gather affected providers
2. **CWD change** - invalidate all - re-detect + re-gather
3. **Manual refresh** - invalidate all - full re-scan
4. **TTL expired** - lazy invalidation on next access

---

## Context Event System

### Event Flow

```
File change / CWD change
    |
    v
ContextEngine::on_change()
    |
    +-- Invalidate cache
    +-- Re-detect providers
    +-- Re-gather (background thread)
    |
    v
ContextEvent::ContextChanged(new_context)
    |
    +---> Completion Engine: reload completion sets
    +---> Frontend (via Tauri event): update ContextSidebar
    +---> Session: update session context
```

### Event Types

```rust
pub enum ContextEvent {
    /// Full context update
    ContextChanged {
        context: ProjectContext,
    },

    /// Single provider updated (incremental)
    ProviderUpdated {
        provider: String,
        info: ContextInfo,
    },

    /// Provider activated (newly detected)
    ProviderActivated {
        provider: String,
    },

    /// Provider deactivated (markers no longer found)
    ProviderDeactivated {
        provider: String,
    },

    /// CWD changed
    DirectoryChanged {
        old_cwd: PathBuf,
        new_cwd: PathBuf,
        new_project_root: Option<PathBuf>,
    },

    /// Error during context gathering
    GatherError {
        provider: String,
        error: String,
    },
}
```

### Subscription

```rust
impl ContextEngine {
    /// Subscribe to context events
    pub fn subscribe(&self) -> Receiver<ContextEvent> {
        self.event_tx.subscribe()
    }

    /// Get current context snapshot (read lock)
    pub fn current(&self) -> ProjectContext {
        self.current.read().clone()
    }

    /// Force re-scan everything
    pub fn refresh(&self) {
        self.cache.clear();
        self.detect_and_gather();
    }

    /// Handle CWD change
    pub fn on_cwd_changed(&mut self, new_cwd: PathBuf) {
        let old_cwd = self.current.read().cwd.clone();
        let new_root = find_project_root(&new_cwd, &self.providers);

        // If project root changed, full re-scan
        if new_root != self.current.read().project_root {
            self.cache.clear();
            self.detect_and_gather();
        }

        self.event_tx.send(ContextEvent::DirectoryChanged {
            old_cwd,
            new_cwd,
            new_project_root: new_root,
        }).ok();
    }
}
```

---

## How Context Feeds into Completion Engine

### Connection Point

The Context Engine provides two types of information to the Completion Engine:

**1. Completion Set Selection:**

```rust
// Example: in a Node + Git project
let context = context_engine.current();
let sets = context.completion_sets;
// -> ["git", "node", "pnpm", "next", "docker"]

// Completion Engine loads the corresponding sets
for set_name in &sets {
    completion_engine.load_set(set_name);
}
```

**2. Dynamic Completion Data:**

```rust
// Context data is injected into completion matching
// Example: git branch names for "git checkout " completion
let git_context = context.providers.get("git");
if let Some(branches) = git_context.data.get("branches") {
    // Add branches as completions for "git checkout"
}

// npm scripts for "npm run " completion
let node_context = context.providers.get("node");
if let Some(scripts) = node_context.data.get("scripts") {
    // Add scripts as completions for "npm run"
}

// Make targets for "make " completion
let generic_context = context.providers.get("generic");
if let Some(targets) = generic_context.data.get("make_targets") {
    // Add targets as completions for "make"
}
```

### Completion Context Integration

```rust
/// When the user requests completions, CompletionRequest includes full context
pub struct CompletionRequest {
    pub input: String,
    pub cursor_pos: usize,
    pub cwd: PathBuf,
    pub context: ProjectContext,  // <- From Context Engine
}

/// Completion sources can use context to filter/enhance results
impl CompletionSource for GitCompletionSource {
    fn complete(&self, request: &CompletionRequest) -> Vec<Completion> {
        let git_context = request.context.providers.get("git");

        // If not in a git repo, return empty
        if git_context.is_none() {
            return vec![];
        }

        match parse_git_command(&request.input) {
            Some(GitCommand::Checkout { partial_branch }) => {
                // Use context's branch list for completion
                let branches = git_context.unwrap()
                    .data.get("branches")
                    .and_then(|v| v.as_list())
                    .unwrap_or_default();

                branches.iter()
                    .filter(|b| b.starts_with(&partial_branch))
                    .map(|b| Completion {
                        text: b.clone(),
                        kind: CompletionKind::GitBranch,
                        ..
                    })
                    .collect()
            }
            // ... other git subcommands
            _ => vec![],
        }
    }
}
```

---

## Extensibility

### Custom Provider Registration

In the future, users may create custom providers:

```toml
# ~/.config/wit/providers/terraform.toml
[provider]
name = "terraform"
markers = ["main.tf", "terraform.tf", "*.tf"]

[gather]
# Simple key-value extraction
version = { command = "terraform version -json", jq = ".terraform_version" }
workspace = { command = "terraform workspace show" }

[completions]
sets = ["terraform"]

[watch]
patterns = ["*.tf", "*.tfvars"]
```

However, this is a future feature (Level 4+), not in the MVP scope.

---

## References

- [direnv](https://direnv.net/) - Directory-specific environment (inspiration for project detection)
- [starship](https://starship.rs/) - Cross-shell prompt with context detection (reference implementation)
- [notify crate](https://docs.rs/notify) - Rust file system notification library
- [git2 crate](https://docs.rs/git2) - Rust libgit2 bindings
- [toml crate](https://docs.rs/toml) - TOML parser for Rust
