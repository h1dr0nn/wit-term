# Context Sidebar (Right)

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Overview

The right sidebar displays contextual information related to the current session. This information is automatically updated as the user works, helping them quickly grasp the state of the project, git, and environment without needing to type commands.

**Design principles:**
- Minimal, info-dense: lots of information in little space
- Read-only (mostly): display only, not an input form
- Auto-update: updates when context changes
- Non-intrusive: hidden by default, only shown when the user needs it

---

## 2. Layout & Dimensions

### 2.1. Dimensions

| Property        | Value                          |
|-----------------|--------------------------------|
| Default width   | 280px                          |
| Min width       | 200px (when resizing)          |
| Max width       | 400px (when resizing)          |
| Collapsed width | 0px (completely hidden)        |

### 2.2. Default State

- **Hidden by default**: right sidebar is not shown when the app is first opened
- **User preference**: state (open/closed) is saved and restored on restart
- **Auto-hide**: does not auto-close while user is working; only closes when user toggles

### 2.3. Resize

- **Drag handle**: left border of sidebar, 4px hit area
- **Cursor**: `col-resize` on hover
- **Persist**: width is saved to user preferences

### 2.4. Overall Structure

```
+------------------------------+
|  Context           [>]       |  <- Header
+------------------------------+
|  v Project Info              |  <- Section 1
|    Name: my-project          |
|    Type: Node.js             |
|    Path: ~/projects/my-proj  |
+------------------------------+
|  v Git                       |  <- Section 2
|    Branch: main              |
|    Status: clean             |
|    Ahead: 0 / Behind: 0     |
|    Recent commits...         |
+------------------------------+
|  v Environment               |  <- Section 3
|    Shell: bash 5.2           |
|    Node: v20.11.0            |
|    Python: 3.12.1            |
+------------------------------+
|  v Active Providers          |  <- Section 4
|    [*] Git Provider          |
|    [*] Node Provider         |
|    [ ] Python Provider       |
+------------------------------+
|  v Quick Actions (future)    |  <- Section 5
|    [Run Tests]               |
|    [Build]                   |
+------------------------------+
```

---

## 3. Toggle (Collapse/Expand)

### 3.1. Toggle Methods

| Method              | Description                      |
|---------------------|----------------------------------|
| Ctrl+Shift+B        | Toggle sidebar visibility        |
| Toggle button       | Icon button [>] in header        |
| Status bar button   | (Optional) icon in status bar    |

### 3.2. Animation

- Same as left sidebar: slide 200ms ease-out
- Terminal view resizes accordingly
- Reduce motion: instant

---

## 4. Header

```
+------------------------------+
|  Context              [>]    |
+------------------------------+
```

- **Title**: "Context", font `--text-md`, weight 600
- **Collapse button**: icon CaretRight (collapse to right), 28x28px, ghost style
- **Height**: 48px
- **Padding**: 0 `--sp-3`
- **Background**: `--color-surface`
- **Border bottom**: 1px solid `--color-border-muted`

---

## 5. Sections

Each section is a collapsible panel with a header and content.

### 5.1. Section Header

- **Height**: 32px
- **Padding**: `--sp-1` vertical, `--sp-3` horizontal
- **Icon**: CaretDown (expanded) / CaretRight (collapsed), 16px
- **Title**: `--text-sm`, weight 500, `--color-text`
- **Hover**: background `--color-surface-hover`
- **Click**: toggles section expand/collapse
- **Keyboard**: Enter/Space to toggle

### 5.2. Section Content

- **Padding**: `--sp-2` vertical, `--sp-3` horizontal (left indent adds `--sp-4` to align with title)
- **Background**: transparent (same `--color-surface` as sidebar)
- **Text**: `--text-sm`, `--color-text-secondary`
- **Labels**: `--color-text-muted`, right-side values `--color-text`

---

## 6. Section: Project Info

### 6.1. When Displayed

- Always displayed (or hidden if no project is detected)

### 6.2. Content

| Field       | Description                    | Example                  |
|-------------|--------------------------------|--------------------------|
| Name        | Project name (from package.json, Cargo.toml, etc.) | `my-project` |
| Type        | Project type with icon         | Node.js, Rust, Python    |
| Root path   | Root path of the project       | `~/projects/my-project`  |

### 6.3. Layout

```
Project Info
  Name     my-project
  Type     [icon] Node.js
  Path     ~/projects/my-proj...
```

- **Path**: truncated from the middle if too long, tooltip shows full path
- **Type icon**: small icon (16px) by language/framework
  - Node.js: green
  - Rust: orange
  - Python: blue/yellow
  - Go: sky blue
  - Generic: Folder icon

### 6.4. Empty State

When no project is detected:
- Text: "No project detected" colored `--color-text-muted`
- Displays current working directory instead of project info

---

## 7. Section: Git

### 7.1. When Displayed

- Only displayed when inside a git repository
- Section is completely hidden if not a git repo

### 7.2. Content

| Field           | Description                    | Example              |
|-----------------|--------------------------------|----------------------|
| Branch          | Current branch name            | `main`, `feature/x`  |
| Status          | Clean/dirty indicator          | Clean / 3 modified   |
| Ahead/Behind    | Number of commits ahead/behind remote | Ahead 2 / Behind 0  |
| Recent commits  | 3-5 most recent commits        | List of commits      |

### 7.3. Layout

```
Git
  Branch    [icon] main
  Status    [dot] Clean
  Remote    Ahead 2 / Behind 0

  Recent Commits
    a1b2c3  Fix login bug         2h ago
    d4e5f6  Add user model        5h ago
    g7h8i9  Initial commit        1d ago
```

### 7.4. Status Indicators

| Status    | Color               | Icon/Indicator           |
|-----------|---------------------|--------------------------|
| Clean     | `--color-success`   | Green dot                |
| Dirty     | `--color-warning`   | Yellow dot + number of modified files |
| Conflict  | `--color-error`     | Red dot                  |
| Detached  | `--color-info`      | Blue info icon           |

### 7.5. Branch Name

- Icon: GitBranch (16px)
- Max width: truncate with ellipsis if branch name is too long
- Tooltip: shows full branch name

### 7.6. Recent Commits

- Shows 3 commits by default, click "Show more" to show 5
- Each commit: `short_hash  message  relative_time`
- Hash: `--color-primary`, monospace font
- Message: `--color-text`, truncated to 1 line
- Time: `--color-text-muted`, right-aligned

---

## 8. Section: Environment

### 8.1. When Displayed

- Always displayed (at least has shell info)

### 8.2. Content

| Field           | Description                    | Example              |
|-----------------|--------------------------------|----------------------|
| Shell           | Shell name and version         | bash 5.2.21          |
| Node            | Node.js version (if present)   | v20.11.0             |
| Python          | Python version (if present)    | 3.12.1               |
| Rust            | Rust toolchain (if present)    | 1.75.0 (stable)      |
| Go              | Go version (if present)        | 1.22.0               |
| Java            | Java version (if present)      | 21.0.1               |
| Ruby            | Ruby version (if present)      | 3.3.0                |

### 8.3. Layout

```
Environment
  Shell     bash 5.2.21
  Node      v20.11.0
  Python    3.12.1
  Rust      1.75.0
```

### 8.4. Detection

- Only shows a runtime/language if detected in PATH or project config
- Version is retrieved when the session starts and when cwd changes
- Caches version info, does not re-query on every render

---

## 9. Section: Active Providers

### 9.1. When Displayed

- Always displayed (even if empty)

### 9.2. Content

List of context providers currently active for the current session.

### 9.3. Layout

```
Active Providers
  [*] Git Provider           running
  [*] Node Provider          running
  [ ] Python Provider        inactive
  [!] Docker Provider        error
```

### 9.4. Provider States

| State      | Icon/Indicator | Color              | Description                 |
|------------|----------------|--------------------|---------------------------|
| Running    | Filled circle  | `--color-success`  | Provider is running normally |
| Inactive   | Empty circle   | `--color-text-muted`| Provider is not activated  |
| Error      | Warning icon   | `--color-error`    | Provider encountered an error |
| Loading    | Spinner        | `--color-primary`  | Provider is initializing   |

### 9.5. Interactions

- Hover: tooltip with provider details (name, state, last update time)
- Click (future): toggle provider on/off
- Error state: tooltip shows error message

---

## 10. Section: Quick Actions (Future)

> **Note**: This is a future feature, not implemented in v1.

### 10.1. Concept

Quick action buttons based on the current context:
- **Node.js project**: "Run Tests" (`npm test`), "Build" (`npm run build`), "Dev" (`npm run dev`)
- **Rust project**: "Build" (`cargo build`), "Test" (`cargo test`), "Run" (`cargo run`)
- **Git**: "Pull" (`git pull`), "Push" (`git push`), "Stash" (`git stash`)

### 10.2. Layout

```
Quick Actions
  [> Run Tests]    [> Build]
  [> Dev Server]   [> Lint]
```

- Buttons: secondary style, compact (28px height)
- Grid: 2 columns
- Auto-detect: actions are automatically created based on project type and scripts in config files

---

## 11. Auto-Update Behavior

### 11.1. Update Triggers

| Event                    | Sections updated          |
|--------------------------|---------------------------|
| Session switch           | All sections              |
| CWD change               | Project Info, Git, Environment |
| Git operation            | Git section               |
| File system change       | Project Info (if related) |
| Provider status change   | Active Providers          |

### 11.2. Update Strategy

- **Debounce**: 500ms after the last event
- **Non-blocking**: update runs in background, does not affect terminal performance
- **Incremental**: only updates the affected section, does not re-render the entire sidebar
- **Loading state**: shows a subtle spinner (or fade) when fetching new data

---

## 12. Accessibility

- **Sidebar role**: `role="complementary"`, `aria-label="Context information"`
- **Sections**: `role="region"` with corresponding `aria-label`
- **Section headers**: `role="button"`, `aria-expanded="true/false"`
- **Collapse button**: `aria-label="Collapse context sidebar"` / `aria-label="Expand context sidebar"`
- **Status indicators**: do not rely solely on color, include text/icon
- **Keyboard**: Tab through section headers, Enter to toggle, Escape to return focus to terminal

---

## 13. Colors & Theme Integration

Sidebar must use design tokens from `design-system.md`:
- Background: `--color-surface`
- Text: `--color-text`, `--color-text-secondary`, `--color-text-muted`
- Borders: `--color-border-muted`
- Status colors: use semantic colors (`--color-success`, `--color-warning`, `--color-error`)

When switching themes (dark/light), the sidebar automatically updates via CSS custom properties.
