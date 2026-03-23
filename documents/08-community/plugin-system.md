# Plugin System Architecture

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

The plugin system allows the community to extend Wit without modifying core code. The design prioritizes **simplicity and safety** - plugins are data files (TOML), not executable code.

### Design Principles

- **File-based, not compiled** - plugins are TOML files, no build toolchain needed
- **Safe by design** - no code execution, only declarative data
- **Zero config** - drop a file into the right directory and it works
- **Backwards compatible** - plugin format is versioned

---

## Plugin Types

### 1. Completion Plugins

Add completion rules for new commands. This is the most common plugin type and the primary way for the community to contribute.

**Format**: TOML file following the completion schema.

```toml
# completions/httpie.toml
[command]
name = "http"
description = "HTTPie - modern HTTP client"

[[command.flags]]
name = "--json"
short = "-j"
description = "Serialize data as JSON (default)"

[[command.flags]]
name = "--form"
short = "-f"
description = "Serialize data as form fields"
```

**See details**: [Completion Data Format](../06-reference/completion-data-format.md)

### 2. Context Provider Plugins

Detect new project types and provide context information to the completion engine.

**Format**: TOML config file with detection rules.

```toml
# contexts/flutter.toml
[context]
name = "flutter"
description = "Flutter mobile/web framework"

[detection]
files = ["pubspec.yaml", "lib/main.dart"]
directories = [".dart_tool", "android", "ios"]

[metadata]
# Information extracted from project
version_file = "pubspec.yaml"
version_pattern = 'version:\s*(.+)'

[completions]
# Commands relevant when in a Flutter project
boost = ["flutter", "dart", "pub"]
```

### 3. Theme Plugins

Custom themes for terminal appearance. Just a TOML file, no code needed.

**Format**: TOML theme definition.

```toml
# themes/nord.toml
[theme]
name = "Nord"
author = "Community"
version = "1.0.0"
description = "Nord color scheme for Wit terminal"

[colors]
background = "#2E3440"
foreground = "#D8DEE9"
cursor = "#D8DEE9"
selection_bg = "#434C5E"
selection_fg = "#ECEFF4"

black = "#3B4252"
red = "#BF616A"
green = "#A3BE8C"
yellow = "#EBCB8B"
blue = "#81A1C1"
magenta = "#B48EAD"
cyan = "#88C0D0"
white = "#E5E9F0"

bright_black = "#4C566A"
bright_red = "#BF616A"
bright_green = "#A3BE8C"
bright_yellow = "#EBCB8B"
bright_blue = "#81A1C1"
bright_magenta = "#B48EAD"
bright_cyan = "#8FBCBB"
bright_white = "#ECEFF4"
```

### 4. Action Plugins (Future)

Custom terminal actions - a later phase, when WASM runtime is available.

```toml
# actions/snippet-manager.toml
[action]
name = "snippet-manager"
description = "Save and recall command snippets"
trigger = "Ctrl+Shift+S"
type = "wasm"  # future
```

---

## Plugin Format

All plugins are **file-based**, no compilation needed:

| Plugin Type | File Format | Requires code? | Complexity |
|------------|-------------|----------------|------------|
| Completion | `.toml` | No | Low |
| Context | `.toml` | No | Low-Medium |
| Theme | `.toml` | No | Low |
| Action | `.toml` + `.wasm` | Yes (future) | High |

### Directory Structure

```
plugin-name/
├── plugin.toml          # Plugin manifest
├── completions/         # Completion TOML files (if any)
│   ├── command1.toml
│   └── command2.toml
├── contexts/            # Context detection rules (if any)
│   └── context.toml
└── themes/              # Theme definitions (if any)
    └── theme.toml
```

### Plugin Manifest (plugin.toml)

```toml
[plugin]
name = "kubernetes-pack"
version = "1.0.0"
description = "Completions for Kubernetes tools"
author = "contributor-name"
license = "MIT"
wit_version = ">=0.1.0"   # compatible Wit versions

[provides]
completions = ["kubectl", "helm", "kustomize"]
contexts = ["kubernetes"]
themes = []
```

---

## Plugin Discovery

Plugins are found in order of priority (high -> low):

### 1. Built-in Plugins

Bundled with the Wit app, located in the application bundle.

```
<app-directory>/plugins/
├── completions/
│   ├── git.toml
│   ├── docker.toml
│   └── ...
├── contexts/
│   ├── node.toml
│   ├── rust.toml
│   └── ...
└── themes/
    ├── default-dark.toml
    └── default-light.toml
```

### 2. User Plugins

Installed by the user, located in the config directory.

```
~/.config/wit/plugins/
├── my-completions/
│   ├── plugin.toml
│   └── completions/
│       └── my-tool.toml
└── my-theme/
    ├── plugin.toml
    └── themes/
        └── custom-dark.toml
```

Per OS:
- **Linux/macOS**: `~/.config/wit/plugins/`
- **Windows**: `%APPDATA%\wit\plugins\`

### 3. Community Plugins (Future)

Install via CLI:

```bash
# Install from registry (future)
wit plugin install kubernetes-pack

# Install from GitHub repo
wit plugin install github:user/wit-plugin-k8s

# List installed plugins
wit plugin list

# Update plugins
wit plugin update

# Remove plugin
wit plugin remove kubernetes-pack
```

---

## Plugin Registry (Future)

### GitHub-based Registry

The plugin registry will be a GitHub repository: **`wit-term/wit-plugins`**

```
wit-plugins/
├── registry.toml              # Index of all plugins
├── plugins/
│   ├── kubernetes-pack/
│   │   ├── metadata.toml      # Plugin info, download URL
│   │   └── README.md
│   ├── aws-completions/
│   │   ├── metadata.toml
│   │   └── README.md
│   └── ...
└── CONTRIBUTING.md
```

### Registry Entry Format

```toml
# plugins/kubernetes-pack/metadata.toml
[plugin]
name = "kubernetes-pack"
description = "Completions for kubectl, helm, and kustomize"
author = "contributor-name"
repository = "https://github.com/user/wit-plugin-k8s"
version = "1.2.0"
downloads = 1542
stars = 23

[compatibility]
wit_version = ">=0.1.0"
platforms = ["linux", "macos", "windows"]

[provides]
completions = ["kubectl", "helm", "kustomize"]
contexts = ["kubernetes"]
```

### Submission Process

1. Create a plugin following the specified format
2. Test locally
3. Submit a PR to the `wit-plugins` repository
4. Review by maintainers
5. Merge -> plugin appears in the registry

---

## Plugin API

### CompletionPlugin Trait

Completion plugins do not need to implement code - just TOML files following the schema. The engine will auto-load them.

Internally, the engine uses a trait:

```rust
/// Internal trait - plugin authors do not need to implement directly.
/// Just provide TOML files in the correct format.
trait CompletionPlugin {
    /// Plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Load completion rules from TOML files
    fn load_completions(&self) -> Result<Vec<CompletionRule>>;

    /// Validate completion data
    fn validate(&self) -> Result<Vec<ValidationError>>;
}
```

**For plugin authors**: you only need to write a TOML file following the schema. The engine will automatically parse and load it.

### ContextPlugin Trait

Context plugins are also TOML-based:

```rust
/// Internal trait for context detection.
trait ContextPlugin {
    /// Plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Detection rules - loaded from TOML
    fn detection_rules(&self) -> DetectionRules;

    /// Extract metadata from detected project
    fn extract_metadata(&self, project_path: &Path) -> Result<ProjectMetadata>;
}
```

**For plugin authors**: write a TOML file with detection rules (files, directories, patterns). The engine will handle the rest.

### ThemePlugin

Theme plugins are the simplest - just a TOML file defining colors. No trait or code, just a file in the correct format.

---

## Plugin Lifecycle

```
┌──────────┐    ┌──────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│ Discover │───>│ Load │───>│ Validate │───>│ Activate │───>│ (Unload) │
└──────────┘    └──────┘    └──────────┘    └──────────┘    └──────────┘
```

### 1. Discover

Engine scans plugin directories in priority order. Looks for `plugin.toml` manifests.

### 2. Load

Parse TOML files, read completion rules, context definitions, theme colors.

### 3. Validate

Check:
- Schema compliance - TOML is in correct format
- Version compatibility - plugin is compatible with current Wit version
- No conflicts - no duplicates with other plugins (or handle via priority)
- Data integrity - no references to undefined values

### 4. Activate

Plugin data is fed into the engine:
- Completion rules are merged into the completion index
- Context rules are added to the context detector
- Themes are added to the theme selector

### 5. Unload (when needed)

Plugin is disabled or removed:
- Data is removed from the engine
- Does not affect other plugins
- Hot-reload: can unload/reload without restarting the app

---

## Plugin Sandboxing

### Safe by Design

Wit plugins (currently) are **completely safe** because:

1. **TOML only** - plugins are just data files, no executable code
2. **No file system access** - plugins do not read/write files outside their scope
3. **No network access** - plugins do not connect to the internet
4. **No process execution** - plugins do not run processes
5. **Declarative** - plugins describe "what", not "how"

```
┌─────────────────────────────────────┐
│            Wit Engine               │
│                                     │
│  ┌─────────────┐  ┌──────────────┐  │
│  │ TOML Parser │  │ Plugin Loader│  │
│  └──────┬──────┘  └──────┬───────┘  │
│         │                │          │
│         v                v          │
│  ┌─────────────────────────────┐    │
│  │    Validated Data Only      │    │
│  │  (no code execution path)   │    │
│  └─────────────────────────────┘    │
│                                     │
└─────────────────────────────────────┘
         ^
         | Read-only
┌────────┴────────┐
│   TOML Files    │
│  (plugin data)  │
└─────────────────┘
```

### Comparison with executable plugins

| Aspect | Wit (TOML plugins) | Traditional (code plugins) |
|--------|-------------------|--------------------------|
| Security | Completely safe | Needs sandbox (VM, containers) |
| Complexity | Very low | High |
| Performance | Fast (just parse data) | Slower (load runtime) |
| Capability | Limited (data only) | Unlimited |
| Author skill | TOML knowledge | Programming language |

---

## WASM Plugins (Future Consideration)

For use cases that require more complex logic (e.g., dynamic completions based on API calls), Wit may support **WebAssembly plugins** in the future.

### Why WASM?

- **Sandboxed** - WASM runs in a sandbox, no direct system access
- **Cross-platform** - compile once, run everywhere
- **Performance** - near-native speed
- **Language agnostic** - write in Rust, Go, C, AssemblyScript, etc.

### WASM Plugin Interface (draft)

```rust
// Plugin authors will implement this interface (future)
#[wit_plugin]
trait DynamicCompletionProvider {
    /// Return completions based on runtime context
    fn complete(input: &str, context: &Context) -> Vec<Completion>;
}
```

### Use Cases for WASM Plugins

- **API-based completions**: query Docker registry for image names
- **Database completions**: suggest table/column names from schema
- **Dynamic file completions**: parse project-specific config files
- **Custom scoring**: special ranking algorithm for completion results

### Timeline

WASM plugin support is **Phase 5+** - only when TOML-based plugins are not sufficient to meet community needs.

---

## Versioning

### Plugin Version Format

Plugins use **Semantic Versioning** (semver):

```
MAJOR.MINOR.PATCH
```

- **MAJOR**: breaking changes in plugin format
- **MINOR**: new features added, backwards compatible
- **PATCH**: bug fixes, additional completions

### Compatibility with Wit Versions

```toml
[plugin]
wit_version = ">=0.1.0"        # Minimum Wit 0.1.0
wit_version = ">=0.1.0,<1.0.0" # From 0.1.0 to before 1.0.0
wit_version = ">=0.2.0"        # Requires features from 0.2.0
```

### Plugin Format Versioning

The plugin TOML format is also versioned:

```toml
[plugin]
format_version = "1"  # Plugin format version
name = "my-plugin"
version = "1.2.3"     # Plugin content version
```

When Wit updates the plugin format:
- **Minor format changes**: backwards compatible, old plugins still work
- **Major format changes**: migration tool is provided to convert plugins

---

## Summary

| Aspect | Current (Phase 1-3) | Future (Phase 4+) |
|--------|---------------------|---------------------|
| Plugin format | TOML files | TOML + WASM |
| Distribution | Manual / Git | Plugin registry + CLI |
| Discovery | File system scan | Registry search |
| Security | Safe (data only) | Sandboxed (WASM) |
| Capabilities | Static completions | Dynamic + static |

The plugin system is designed to **start simple** and **scale gradually**. The first and most important step is building a rich library of completion rules through community contributions.

> Get started: [Completion Contribution Guide](./completion-contribution.md)
