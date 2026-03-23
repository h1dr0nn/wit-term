# Completion Data Format

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

This document defines the format for completion data files - files that describe commands, subcommands, flags, and arguments that the Completion Engine uses for suggestions.

**Primary format:** TOML - human-readable, easy to write, easy to contribute, good for version control. Each file describes one command group (e.g., `git.toml`, `docker.toml`).

**Principles:**

- One file = one top-level command (or one group of related commands)
- Declarative - describes "what the command has", not "how to complete"
- Human-first - new contributors should be able to understand it in 5 minutes
- No code required, just write TOML

---

## File Organization

### Directory Structure

```
completions/
├── bundled/                    # Shipped with the app, maintained by core team
│   ├── git.toml
│   ├── docker.toml
│   ├── npm.toml
│   ├── cargo.toml
│   ├── kubectl.toml
│   ├── systemd.toml
│   └── ...
├── community/                  # Community-contributed (optional install)
│   ├── terraform.toml
│   ├── aws.toml
│   └── ...
└── user/                       # User-defined overrides
    ├── my-tool.toml
    └── git.toml                # Overrides/extends bundled git.toml
```

**Load order (highest to lowest priority):**

1. `~/.config/wit/completions/` (user overrides)
2. `$APP_DATA/completions/community/`
3. `$APP_DATA/completions/bundled/`

If the same command name exists in multiple locations, the higher-priority file **merges** into the lower-priority file (does not fully replace). Users can override descriptions, add flags, or disable flags.

### File Naming

- File name = command name (lowercase): `git.toml`, `docker.toml`
- If the command has a hyphen: `docker-compose.toml`
- Each file must contain exactly one `[command]` section at the root

---

## Schema Version

Each file begins with a version declaration:

```toml
wit_completion_version = "1.0"
```

**Version rules:**

- `1.x` - backward compatible: newer engines can read older files
- `2.0` - breaking change: engine must support both v1 and v2 during the transition period
- Engine ignores files with versions it does not understand (logs a warning)

---

## Schema Definition

### Top-level Command

```toml
wit_completion_version = "1.0"

[command]
name = "git"
description = "Distributed version control system"
# Aliases the user might use
aliases = ["g"]
```

### Flags (Top-level)

Flags that apply to the main command (before subcommand):

```toml
[[command.flags]]
name = "--version"
short = "-v"
description = "Print git version"
# Flag does not take an argument
takes_value = false

[[command.flags]]
name = "--help"
short = "-h"
description = "Show help message"
takes_value = false

[[command.flags]]
name = "--git-dir"
description = "Set path to the repository (.git directory)"
takes_value = true
value_hint = "directory"   # Hints to the completion engine to use PathSource (dirs only)

[[command.flags]]
name = "--work-tree"
description = "Set path to the working tree"
takes_value = true
value_hint = "directory"
```

### Flag Properties

| Property | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | Long flag name (including `--`) |
| `short` | string | no | Short flag (including `-`) |
| `description` | string | no | Short description |
| `takes_value` | bool | no | Whether the flag takes an argument (default: false) |
| `value_hint` | string | no | Hint for argument type (see table below) |
| `value_enum` | string[] | no | List of fixed values |
| `default_value` | string | no | Default value |
| `required` | bool | no | Whether the flag is required (default: false) |
| `repeatable` | bool | no | Can be used multiple times (default: false) |
| `deprecated` | bool | no | Has been deprecated (default: false) |
| `deprecated_message` | string | no | Message when flag is deprecated |
| `conflicts_with` | string[] | no | Mutually exclusive with which flags |
| `requires` | string[] | no | Requires which flags to be present too |
| `hidden` | bool | no | Hidden from completion (default: false) |

### Value Hints

Value hints tell the engine which source to use for completing the argument:

| Hint | Description | Source |
|---|---|---|
| `"file"` | Any file path | PathSource (files + dirs) |
| `"directory"` | Directory only | PathSource (dirs only) |
| `"executable"` | Executable file | PathSource (filter executable) |
| `"url"` | URL | No auto-complete |
| `"hostname"` | Hostname | No auto-complete |
| `"username"` | Username | OS users |
| `"git_branch"` | Git branch name | ContextSource (git) |
| `"git_tag"` | Git tag name | ContextSource (git) |
| `"git_remote"` | Git remote name | ContextSource (git) |
| `"git_ref"` | Any git ref | ContextSource (git) |
| `"git_file"` | Git-tracked files | ContextSource (git) |
| `"git_modified_file"` | Modified files | ContextSource (git) |
| `"docker_image"` | Docker image name | ContextSource (docker) |
| `"docker_container"` | Running container | ContextSource (docker) |
| `"npm_script"` | npm script name | ContextSource (node) |
| `"cargo_target"` | Cargo target | ContextSource (cargo) |
| `"environment_variable"` | Env var name | System env vars |
| `"process"` | Running process | System processes |
| `"custom"` | Custom (see `value_source`) | DynamicSource |

### Subcommands

```toml
[[command.subcommands]]
name = "add"
description = "Add file contents to the index"
aliases = []

    [[command.subcommands.flags]]
    name = "--all"
    short = "-A"
    description = "Add all changes (tracked and untracked)"
    takes_value = false

    [[command.subcommands.flags]]
    name = "--force"
    short = "-f"
    description = "Allow adding otherwise ignored files"
    takes_value = false

    [[command.subcommands.flags]]
    name = "--patch"
    short = "-p"
    description = "Interactively choose hunks to add"
    takes_value = false

    # Arguments (positional)
    [[command.subcommands.args]]
    name = "pathspec"
    description = "Files to add"
    value_hint = "git_file"
    repeatable = true        # Can accept multiple files
    required = false
```

### Nested Subcommands

```toml
[[command.subcommands]]
name = "remote"
description = "Manage set of tracked repositories"

    [[command.subcommands.subcommands]]
    name = "add"
    description = "Add a new remote"

        [[command.subcommands.subcommands.args]]
        name = "name"
        description = "Name for the remote"
        required = true

        [[command.subcommands.subcommands.args]]
        name = "url"
        description = "URL of the remote repository"
        value_hint = "url"
        required = true

    [[command.subcommands.subcommands]]
    name = "remove"
    description = "Remove a remote"
    aliases = ["rm"]

        [[command.subcommands.subcommands.args]]
        name = "name"
        description = "Name of the remote to remove"
        value_hint = "git_remote"
        required = true

    [[command.subcommands.subcommands]]
    name = "set-url"
    description = "Change URL for a remote"

        [[command.subcommands.subcommands.args]]
        name = "name"
        description = "Name of the remote"
        value_hint = "git_remote"
        required = true

        [[command.subcommands.subcommands.args]]
        name = "url"
        description = "New URL"
        value_hint = "url"
        required = true
```

### Flags with Enum Values

```toml
[[command.subcommands]]
name = "log"
description = "Show commit logs"

    [[command.subcommands.flags]]
    name = "--format"
    description = "Pretty-print format"
    takes_value = true
    value_enum = ["oneline", "short", "medium", "full", "fuller", "email", "raw"]

    [[command.subcommands.flags]]
    name = "--diff-filter"
    description = "Select only files matching filter"
    takes_value = true
    value_enum = ["A", "C", "D", "M", "R", "T", "U", "X", "B"]

    [[command.subcommands.flags]]
    name = "--sort"
    description = "Sort order for refs"
    takes_value = true
    # Too many or dynamic values - do not use value_enum
    # Instead use value_hint or free-form
```

### Dynamic Arguments

When arguments need to come from context rather than a static list:

```toml
[[command.subcommands]]
name = "checkout"
description = "Switch branches or restore working tree files"

    [[command.subcommands.args]]
    name = "branch_or_path"
    description = "Branch name, tag, commit, or file path"
    # Multiple value_hints - engine queries all corresponding sources
    value_hint = "git_ref"
    required = false

    [[command.subcommands.flags]]
    name = "--branch"
    short = "-b"
    description = "Create and checkout a new branch"
    takes_value = true
    # No value_hint - free-form input (user types a new branch name)
```

### Conditional Completions

Some flags are only valid when used with a specific subcommand or another flag:

```toml
[[command.subcommands]]
name = "commit"
description = "Record changes to the repository"

    [[command.subcommands.flags]]
    name = "--message"
    short = "-m"
    description = "Commit message"
    takes_value = true
    required = false

    [[command.subcommands.flags]]
    name = "--amend"
    description = "Amend the previous commit"
    takes_value = false

    [[command.subcommands.flags]]
    name = "--no-edit"
    description = "Use selected commit message without editing"
    takes_value = false
    # Only relevant when --amend is used
    requires = ["--amend"]

    [[command.subcommands.flags]]
    name = "--all"
    short = "-a"
    description = "Automatically stage modified and deleted files"
    takes_value = false

    [[command.subcommands.flags]]
    name = "--fixup"
    description = "Create a fixup commit for given commit"
    takes_value = true
    value_hint = "git_ref"
    # Cannot be used with --squash
    conflicts_with = ["--squash"]

    [[command.subcommands.flags]]
    name = "--squash"
    description = "Create a squash commit for given commit"
    takes_value = true
    value_hint = "git_ref"
    conflicts_with = ["--fixup"]
```

### Mutually Exclusive Flags

```toml
[[command.subcommands]]
name = "merge"
description = "Join two or more development histories together"

    [[command.subcommands.flags]]
    name = "--ff"
    description = "Fast-forward if possible (default)"
    takes_value = false
    conflicts_with = ["--no-ff", "--ff-only"]

    [[command.subcommands.flags]]
    name = "--no-ff"
    description = "Always create a merge commit"
    takes_value = false
    conflicts_with = ["--ff", "--ff-only"]

    [[command.subcommands.flags]]
    name = "--ff-only"
    description = "Abort if fast-forward is not possible"
    takes_value = false
    conflicts_with = ["--ff", "--no-ff"]
```

### Repeatable Flags

```toml
[[command.subcommands.flags]]
name = "--verbose"
short = "-v"
description = "Increase verbosity (can be repeated: -vvv)"
takes_value = false
repeatable = true

[[command.subcommands.flags]]
name = "--exec"
short = "-e"
description = "Execute command for each commit (repeatable)"
takes_value = true
repeatable = true
```

---

## Complete Example Files

### git.toml (Comprehensive)

```toml
wit_completion_version = "1.0"

[command]
name = "git"
description = "Distributed version control system"
aliases = ["g"]

# -- Global flags -------------------------------------------------------------

[[command.flags]]
name = "--version"
description = "Print git version"

[[command.flags]]
name = "--help"
short = "-h"
description = "Show help"

[[command.flags]]
name = "--git-dir"
description = "Set the path to the repository (.git directory)"
takes_value = true
value_hint = "directory"

[[command.flags]]
name = "--work-tree"
description = "Set the path to the working tree"
takes_value = true
value_hint = "directory"

[[command.flags]]
name = "--no-pager"
description = "Do not pipe output into a pager"

[[command.flags]]
name = "--config"
short = "-c"
description = "Set a configuration variable"
takes_value = true
repeatable = true

# -- Subcommand: init ---------------------------------------------------------

[[command.subcommands]]
name = "init"
description = "Create an empty Git repository"

    [[command.subcommands.flags]]
    name = "--bare"
    description = "Create a bare repository"

    [[command.subcommands.flags]]
    name = "--initial-branch"
    short = "-b"
    description = "Name of the initial branch"
    takes_value = true

    [[command.subcommands.args]]
    name = "directory"
    description = "Directory to initialize"
    value_hint = "directory"
    required = false

# -- Subcommand: clone --------------------------------------------------------

[[command.subcommands]]
name = "clone"
description = "Clone a repository into a new directory"

    [[command.subcommands.flags]]
    name = "--branch"
    short = "-b"
    description = "Checkout specific branch"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--depth"
    description = "Create a shallow clone with N commits"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--single-branch"
    description = "Clone only one branch"

    [[command.subcommands.flags]]
    name = "--recurse-submodules"
    description = "Initialize submodules"

    [[command.subcommands.args]]
    name = "repository"
    description = "Repository URL to clone"
    value_hint = "url"
    required = true

    [[command.subcommands.args]]
    name = "directory"
    description = "Target directory"
    value_hint = "directory"
    required = false

# -- Subcommand: add ----------------------------------------------------------

[[command.subcommands]]
name = "add"
description = "Add file contents to the index"

    [[command.subcommands.flags]]
    name = "--all"
    short = "-A"
    description = "Add all changes"

    [[command.subcommands.flags]]
    name = "--force"
    short = "-f"
    description = "Allow adding ignored files"

    [[command.subcommands.flags]]
    name = "--patch"
    short = "-p"
    description = "Interactive staging"

    [[command.subcommands.flags]]
    name = "--dry-run"
    short = "-n"
    description = "Dry run"

    [[command.subcommands.args]]
    name = "pathspec"
    description = "Files to add"
    value_hint = "git_modified_file"
    repeatable = true

# -- Subcommand: commit -------------------------------------------------------

[[command.subcommands]]
name = "commit"
description = "Record changes to the repository"

    [[command.subcommands.flags]]
    name = "--message"
    short = "-m"
    description = "Commit message"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--all"
    short = "-a"
    description = "Stage all modified files"

    [[command.subcommands.flags]]
    name = "--amend"
    description = "Amend previous commit"

    [[command.subcommands.flags]]
    name = "--no-edit"
    description = "Don't edit commit message"
    requires = ["--amend"]

    [[command.subcommands.flags]]
    name = "--allow-empty"
    description = "Allow empty commit"

    [[command.subcommands.flags]]
    name = "--fixup"
    description = "Fixup commit"
    takes_value = true
    value_hint = "git_ref"
    conflicts_with = ["--squash"]

    [[command.subcommands.flags]]
    name = "--squash"
    description = "Squash commit"
    takes_value = true
    value_hint = "git_ref"
    conflicts_with = ["--fixup"]

    [[command.subcommands.flags]]
    name = "--signoff"
    short = "-s"
    description = "Add Signed-off-by trailer"

# -- Subcommand: push ---------------------------------------------------------

[[command.subcommands]]
name = "push"
description = "Update remote refs along with associated objects"

    [[command.subcommands.flags]]
    name = "--force"
    short = "-f"
    description = "Force push"

    [[command.subcommands.flags]]
    name = "--force-with-lease"
    description = "Safe force push"

    [[command.subcommands.flags]]
    name = "--set-upstream"
    short = "-u"
    description = "Set upstream tracking"

    [[command.subcommands.flags]]
    name = "--tags"
    description = "Push all tags"

    [[command.subcommands.flags]]
    name = "--delete"
    short = "-d"
    description = "Delete remote branch"

    [[command.subcommands.flags]]
    name = "--dry-run"
    short = "-n"
    description = "Dry run"

    [[command.subcommands.args]]
    name = "remote"
    description = "Remote name"
    value_hint = "git_remote"
    required = false

    [[command.subcommands.args]]
    name = "refspec"
    description = "Branch or refspec to push"
    value_hint = "git_branch"
    required = false

# -- Subcommand: pull ---------------------------------------------------------

[[command.subcommands]]
name = "pull"
description = "Fetch from and integrate with another repository or branch"

    [[command.subcommands.flags]]
    name = "--rebase"
    short = "-r"
    description = "Rebase instead of merge"

    [[command.subcommands.flags]]
    name = "--no-rebase"
    description = "Merge (default)"
    conflicts_with = ["--rebase"]

    [[command.subcommands.flags]]
    name = "--ff-only"
    description = "Abort if fast-forward not possible"

    [[command.subcommands.args]]
    name = "remote"
    description = "Remote name"
    value_hint = "git_remote"
    required = false

    [[command.subcommands.args]]
    name = "branch"
    description = "Branch to pull"
    value_hint = "git_branch"
    required = false

# -- Subcommand: checkout -----------------------------------------------------

[[command.subcommands]]
name = "checkout"
description = "Switch branches or restore working tree files"

    [[command.subcommands.flags]]
    name = "--branch"
    short = "-b"
    description = "Create and checkout new branch"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--force"
    short = "-f"
    description = "Force checkout (discard local changes)"

    [[command.subcommands.flags]]
    name = "--track"
    short = "-t"
    description = "Set up tracking for new branch"

    [[command.subcommands.args]]
    name = "target"
    description = "Branch, tag, or commit"
    value_hint = "git_ref"
    required = false

# -- Subcommand: branch -------------------------------------------------------

[[command.subcommands]]
name = "branch"
description = "List, create, or delete branches"

    [[command.subcommands.flags]]
    name = "--all"
    short = "-a"
    description = "List both remote and local branches"

    [[command.subcommands.flags]]
    name = "--delete"
    short = "-d"
    description = "Delete branch"

    [[command.subcommands.flags]]
    name = "--force"
    short = "-D"
    description = "Force delete branch"

    [[command.subcommands.flags]]
    name = "--move"
    short = "-m"
    description = "Rename branch"

    [[command.subcommands.flags]]
    name = "--remote"
    short = "-r"
    description = "List remote branches"

    [[command.subcommands.args]]
    name = "branch_name"
    description = "Branch name"
    value_hint = "git_branch"
    required = false

# -- Subcommand: merge --------------------------------------------------------

[[command.subcommands]]
name = "merge"
description = "Join development histories together"

    [[command.subcommands.flags]]
    name = "--no-ff"
    description = "Always create merge commit"
    conflicts_with = ["--ff-only"]

    [[command.subcommands.flags]]
    name = "--ff-only"
    description = "Fast-forward only"
    conflicts_with = ["--no-ff"]

    [[command.subcommands.flags]]
    name = "--squash"
    description = "Squash merge"

    [[command.subcommands.flags]]
    name = "--abort"
    description = "Abort current merge"

    [[command.subcommands.args]]
    name = "branch"
    description = "Branch to merge"
    value_hint = "git_branch"
    required = false

# -- Subcommand: log ----------------------------------------------------------

[[command.subcommands]]
name = "log"
description = "Show commit logs"

    [[command.subcommands.flags]]
    name = "--oneline"
    description = "One line per commit"

    [[command.subcommands.flags]]
    name = "--graph"
    description = "Draw ASCII graph"

    [[command.subcommands.flags]]
    name = "--all"
    description = "Show all refs"

    [[command.subcommands.flags]]
    name = "--format"
    description = "Pretty-print format"
    takes_value = true
    value_enum = ["oneline", "short", "medium", "full", "fuller", "email", "raw"]

    [[command.subcommands.flags]]
    name = "--author"
    description = "Filter by author"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--since"
    description = "Show commits since date"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--until"
    description = "Show commits until date"
    takes_value = true

    [[command.subcommands.flags]]
    name = "-n"
    description = "Limit number of commits"
    takes_value = true

# -- Subcommand: stash --------------------------------------------------------

[[command.subcommands]]
name = "stash"
description = "Stash changes in a dirty working directory"

    [[command.subcommands.subcommands]]
    name = "push"
    description = "Save changes to stash"

        [[command.subcommands.subcommands.flags]]
        name = "--message"
        short = "-m"
        description = "Stash message"
        takes_value = true

        [[command.subcommands.subcommands.flags]]
        name = "--keep-index"
        description = "Keep staged changes"

    [[command.subcommands.subcommands]]
    name = "pop"
    description = "Apply and remove top stash"

    [[command.subcommands.subcommands]]
    name = "apply"
    description = "Apply top stash without removing"

    [[command.subcommands.subcommands]]
    name = "list"
    description = "List stashes"

    [[command.subcommands.subcommands]]
    name = "drop"
    description = "Remove a stash entry"

    [[command.subcommands.subcommands]]
    name = "clear"
    description = "Remove all stash entries"

# -- Subcommand: remote -------------------------------------------------------

[[command.subcommands]]
name = "remote"
description = "Manage set of tracked repositories"

    [[command.subcommands.flags]]
    name = "--verbose"
    short = "-v"
    description = "Show remote URL"

    [[command.subcommands.subcommands]]
    name = "add"
    description = "Add a new remote"

        [[command.subcommands.subcommands.args]]
        name = "name"
        description = "Remote name"
        required = true

        [[command.subcommands.subcommands.args]]
        name = "url"
        description = "Remote URL"
        value_hint = "url"
        required = true

    [[command.subcommands.subcommands]]
    name = "remove"
    description = "Remove a remote"
    aliases = ["rm"]

        [[command.subcommands.subcommands.args]]
        name = "name"
        description = "Remote to remove"
        value_hint = "git_remote"
        required = true

    [[command.subcommands.subcommands]]
    name = "rename"
    description = "Rename a remote"

        [[command.subcommands.subcommands.args]]
        name = "old"
        description = "Current name"
        value_hint = "git_remote"
        required = true

        [[command.subcommands.subcommands.args]]
        name = "new"
        description = "New name"
        required = true

    [[command.subcommands.subcommands]]
    name = "set-url"
    description = "Change URL for a remote"

        [[command.subcommands.subcommands.args]]
        name = "name"
        description = "Remote name"
        value_hint = "git_remote"
        required = true

        [[command.subcommands.subcommands.args]]
        name = "url"
        description = "New URL"
        value_hint = "url"
        required = true

# -- Subcommand: diff ---------------------------------------------------------

[[command.subcommands]]
name = "diff"
description = "Show changes between commits, commit and working tree, etc."

    [[command.subcommands.flags]]
    name = "--staged"
    description = "Show staged changes"

    [[command.subcommands.flags]]
    name = "--cached"
    description = "Synonym for --staged"

    [[command.subcommands.flags]]
    name = "--stat"
    description = "Show diffstat"

    [[command.subcommands.flags]]
    name = "--name-only"
    description = "Show only names of changed files"

    [[command.subcommands.flags]]
    name = "--name-status"
    description = "Show names and status of changed files"

# -- Subcommand: status -------------------------------------------------------

[[command.subcommands]]
name = "status"
description = "Show the working tree status"

    [[command.subcommands.flags]]
    name = "--short"
    short = "-s"
    description = "Short format"

    [[command.subcommands.flags]]
    name = "--branch"
    short = "-b"
    description = "Show branch info in short format"

    [[command.subcommands.flags]]
    name = "--porcelain"
    description = "Machine-readable output"
    takes_value = true
    value_enum = ["v1", "v2"]

# -- Subcommand: rebase -------------------------------------------------------

[[command.subcommands]]
name = "rebase"
description = "Reapply commits on top of another base tip"

    [[command.subcommands.flags]]
    name = "--interactive"
    short = "-i"
    description = "Interactive rebase"

    [[command.subcommands.flags]]
    name = "--onto"
    description = "Rebase onto a different base"
    takes_value = true
    value_hint = "git_ref"

    [[command.subcommands.flags]]
    name = "--continue"
    description = "Continue rebase after resolving conflicts"
    conflicts_with = ["--abort", "--skip"]

    [[command.subcommands.flags]]
    name = "--abort"
    description = "Abort rebase"
    conflicts_with = ["--continue", "--skip"]

    [[command.subcommands.flags]]
    name = "--skip"
    description = "Skip current patch"
    conflicts_with = ["--continue", "--abort"]

    [[command.subcommands.args]]
    name = "upstream"
    description = "Upstream branch"
    value_hint = "git_ref"
    required = false

# -- Subcommand: tag ----------------------------------------------------------

[[command.subcommands]]
name = "tag"
description = "Create, list, delete, or verify tags"

    [[command.subcommands.flags]]
    name = "--annotate"
    short = "-a"
    description = "Create annotated tag"

    [[command.subcommands.flags]]
    name = "--message"
    short = "-m"
    description = "Tag message"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--delete"
    short = "-d"
    description = "Delete tag"

    [[command.subcommands.flags]]
    name = "--list"
    short = "-l"
    description = "List tags"

    [[command.subcommands.args]]
    name = "tagname"
    description = "Tag name"
    required = false

    [[command.subcommands.args]]
    name = "commit"
    description = "Commit to tag"
    value_hint = "git_ref"
    required = false
```

### docker.toml

```toml
wit_completion_version = "1.0"

[command]
name = "docker"
description = "Container platform"

[[command.flags]]
name = "--help"
description = "Show help"

[[command.flags]]
name = "--version"
description = "Print version"

[[command.flags]]
name = "--host"
short = "-H"
description = "Daemon socket to connect to"
takes_value = true

# -- run ----------------------------------------------------------------------

[[command.subcommands]]
name = "run"
description = "Create and run a new container"

    [[command.subcommands.flags]]
    name = "--name"
    description = "Assign a name to the container"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--detach"
    short = "-d"
    description = "Run container in background"

    [[command.subcommands.flags]]
    name = "--interactive"
    short = "-i"
    description = "Keep STDIN open"

    [[command.subcommands.flags]]
    name = "--tty"
    short = "-t"
    description = "Allocate a pseudo-TTY"

    [[command.subcommands.flags]]
    name = "--rm"
    description = "Automatically remove container when it exits"

    [[command.subcommands.flags]]
    name = "--publish"
    short = "-p"
    description = "Publish port (host:container)"
    takes_value = true
    repeatable = true

    [[command.subcommands.flags]]
    name = "--volume"
    short = "-v"
    description = "Bind mount a volume"
    takes_value = true
    repeatable = true

    [[command.subcommands.flags]]
    name = "--env"
    short = "-e"
    description = "Set environment variable"
    takes_value = true
    repeatable = true

    [[command.subcommands.flags]]
    name = "--network"
    description = "Connect to a network"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--restart"
    description = "Restart policy"
    takes_value = true
    value_enum = ["no", "always", "unless-stopped", "on-failure"]

    [[command.subcommands.args]]
    name = "image"
    description = "Image to run"
    value_hint = "docker_image"
    required = true

    [[command.subcommands.args]]
    name = "command"
    description = "Command to run in container"
    required = false

# -- build --------------------------------------------------------------------

[[command.subcommands]]
name = "build"
description = "Build an image from a Dockerfile"

    [[command.subcommands.flags]]
    name = "--tag"
    short = "-t"
    description = "Name and optionally a tag (name:tag)"
    takes_value = true
    repeatable = true

    [[command.subcommands.flags]]
    name = "--file"
    short = "-f"
    description = "Path to Dockerfile"
    takes_value = true
    value_hint = "file"

    [[command.subcommands.flags]]
    name = "--no-cache"
    description = "Do not use cache when building"

    [[command.subcommands.flags]]
    name = "--platform"
    description = "Set platform"
    takes_value = true
    value_enum = ["linux/amd64", "linux/arm64", "linux/arm/v7"]

    [[command.subcommands.args]]
    name = "path"
    description = "Build context path"
    value_hint = "directory"
    required = true

# -- ps, stop, rm, exec, logs, images, pull, push ----------------------------

[[command.subcommands]]
name = "ps"
description = "List containers"

    [[command.subcommands.flags]]
    name = "--all"
    short = "-a"
    description = "Show all containers"

    [[command.subcommands.flags]]
    name = "--quiet"
    short = "-q"
    description = "Only display container IDs"

[[command.subcommands]]
name = "stop"
description = "Stop running containers"

    [[command.subcommands.args]]
    name = "container"
    description = "Container to stop"
    value_hint = "docker_container"
    repeatable = true
    required = true

[[command.subcommands]]
name = "rm"
description = "Remove containers"

    [[command.subcommands.flags]]
    name = "--force"
    short = "-f"
    description = "Force removal"

    [[command.subcommands.args]]
    name = "container"
    description = "Container to remove"
    value_hint = "docker_container"
    repeatable = true
    required = true

[[command.subcommands]]
name = "exec"
description = "Execute a command in a running container"

    [[command.subcommands.flags]]
    name = "--interactive"
    short = "-i"
    description = "Keep STDIN open"

    [[command.subcommands.flags]]
    name = "--tty"
    short = "-t"
    description = "Allocate a pseudo-TTY"

    [[command.subcommands.args]]
    name = "container"
    description = "Container to exec in"
    value_hint = "docker_container"
    required = true

    [[command.subcommands.args]]
    name = "command"
    description = "Command to execute"
    required = true

[[command.subcommands]]
name = "logs"
description = "Fetch logs of a container"

    [[command.subcommands.flags]]
    name = "--follow"
    short = "-f"
    description = "Follow log output"

    [[command.subcommands.flags]]
    name = "--tail"
    description = "Number of lines to show from end"
    takes_value = true

    [[command.subcommands.args]]
    name = "container"
    description = "Container"
    value_hint = "docker_container"
    required = true

[[command.subcommands]]
name = "images"
description = "List images"

    [[command.subcommands.flags]]
    name = "--all"
    short = "-a"
    description = "Show all images"

[[command.subcommands]]
name = "pull"
description = "Download an image from a registry"

    [[command.subcommands.args]]
    name = "image"
    description = "Image name"
    value_hint = "docker_image"
    required = true

[[command.subcommands]]
name = "push"
description = "Upload an image to a registry"

    [[command.subcommands.args]]
    name = "image"
    description = "Image name"
    value_hint = "docker_image"
    required = true

# -- compose ------------------------------------------------------------------

[[command.subcommands]]
name = "compose"
description = "Docker Compose commands"

    [[command.subcommands.flags]]
    name = "--file"
    short = "-f"
    description = "Compose file path"
    takes_value = true
    value_hint = "file"
    repeatable = true

    [[command.subcommands.subcommands]]
    name = "up"
    description = "Create and start containers"

        [[command.subcommands.subcommands.flags]]
        name = "--detach"
        short = "-d"
        description = "Run in background"

        [[command.subcommands.subcommands.flags]]
        name = "--build"
        description = "Build images before starting"

    [[command.subcommands.subcommands]]
    name = "down"
    description = "Stop and remove containers"

        [[command.subcommands.subcommands.flags]]
        name = "--volumes"
        short = "-v"
        description = "Remove volumes"

        [[command.subcommands.subcommands.flags]]
        name = "--rmi"
        description = "Remove images"
        takes_value = true
        value_enum = ["all", "local"]

    [[command.subcommands.subcommands]]
    name = "logs"
    description = "View output from containers"

        [[command.subcommands.subcommands.flags]]
        name = "--follow"
        short = "-f"
        description = "Follow log output"

    [[command.subcommands.subcommands]]
    name = "ps"
    description = "List containers"

    [[command.subcommands.subcommands]]
    name = "build"
    description = "Build or rebuild services"

    [[command.subcommands.subcommands]]
    name = "restart"
    description = "Restart services"

    [[command.subcommands.subcommands]]
    name = "exec"
    description = "Execute a command in a running container"
```

### npm.toml

```toml
wit_completion_version = "1.0"

[command]
name = "npm"
description = "Node.js package manager"

[[command.flags]]
name = "--help"
short = "-h"
description = "Show help"

[[command.flags]]
name = "--version"
short = "-v"
description = "Print version"

[[command.subcommands]]
name = "install"
description = "Install packages"
aliases = ["i", "add"]

    [[command.subcommands.flags]]
    name = "--save-dev"
    short = "-D"
    description = "Save as devDependency"

    [[command.subcommands.flags]]
    name = "--save-exact"
    short = "-E"
    description = "Save exact version"

    [[command.subcommands.flags]]
    name = "--global"
    short = "-g"
    description = "Install globally"

    [[command.subcommands.flags]]
    name = "--legacy-peer-deps"
    description = "Ignore peer dependency conflicts"

    [[command.subcommands.flags]]
    name = "--force"
    short = "-f"
    description = "Force installation"

[[command.subcommands]]
name = "run"
description = "Run a script defined in package.json"
aliases = ["run-script"]

    [[command.subcommands.args]]
    name = "script"
    description = "Script name from package.json"
    value_hint = "npm_script"
    required = true

[[command.subcommands]]
name = "test"
description = "Run test script"
aliases = ["t", "tst"]

[[command.subcommands]]
name = "start"
description = "Run start script"

[[command.subcommands]]
name = "build"
description = "Run build script"

[[command.subcommands]]
name = "uninstall"
description = "Remove a package"
aliases = ["un", "remove", "rm"]

    [[command.subcommands.flags]]
    name = "--save-dev"
    short = "-D"
    description = "Remove from devDependencies"

    [[command.subcommands.flags]]
    name = "--global"
    short = "-g"
    description = "Remove globally installed package"

[[command.subcommands]]
name = "update"
description = "Update packages"
aliases = ["up", "upgrade"]

[[command.subcommands]]
name = "init"
description = "Create a package.json file"

    [[command.subcommands.flags]]
    name = "--yes"
    short = "-y"
    description = "Accept all defaults"

[[command.subcommands]]
name = "publish"
description = "Publish a package"

    [[command.subcommands.flags]]
    name = "--access"
    description = "Package access level"
    takes_value = true
    value_enum = ["public", "restricted"]

    [[command.subcommands.flags]]
    name = "--tag"
    description = "Publish with a dist-tag"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--dry-run"
    description = "Do everything except publish"

[[command.subcommands]]
name = "outdated"
description = "Check for outdated packages"

    [[command.subcommands.flags]]
    name = "--global"
    short = "-g"
    description = "Check globally installed packages"

[[command.subcommands]]
name = "ls"
description = "List installed packages"
aliases = ["list"]

    [[command.subcommands.flags]]
    name = "--all"
    description = "Show all packages"

    [[command.subcommands.flags]]
    name = "--depth"
    description = "Dependency depth"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--global"
    short = "-g"
    description = "List globally installed packages"

[[command.subcommands]]
name = "ci"
description = "Clean install (from package-lock.json)"

[[command.subcommands]]
name = "cache"
description = "Manage npm cache"

    [[command.subcommands.subcommands]]
    name = "clean"
    description = "Clean cache"

        [[command.subcommands.subcommands.flags]]
        name = "--force"
        description = "Force cache clean"

    [[command.subcommands.subcommands]]
    name = "verify"
    description = "Verify cache contents"

[[command.subcommands]]
name = "config"
description = "Manage npm configuration"

    [[command.subcommands.subcommands]]
    name = "set"
    description = "Set a config key"

    [[command.subcommands.subcommands]]
    name = "get"
    description = "Get a config key"

    [[command.subcommands.subcommands]]
    name = "list"
    description = "List all config"
```

### cargo.toml (Note: named `cargo-cli.toml` to avoid conflict with Rust's Cargo.toml)

```toml
wit_completion_version = "1.0"

[command]
name = "cargo"
description = "Rust package manager and build system"

[[command.flags]]
name = "--version"
short = "-V"
description = "Print version"

[[command.flags]]
name = "--list"
description = "List installed commands"

[[command.flags]]
name = "--verbose"
short = "-v"
description = "Use verbose output"
repeatable = true

[[command.flags]]
name = "--quiet"
short = "-q"
description = "Suppress output"

[[command.flags]]
name = "--color"
description = "Coloring"
takes_value = true
value_enum = ["auto", "always", "never"]

[[command.subcommands]]
name = "build"
description = "Compile the current package"
aliases = ["b"]

    [[command.subcommands.flags]]
    name = "--release"
    short = "-r"
    description = "Build in release mode"

    [[command.subcommands.flags]]
    name = "--target"
    description = "Build for the target triple"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--features"
    short = "-F"
    description = "Space or comma separated list of features"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--all-features"
    description = "Activate all features"

    [[command.subcommands.flags]]
    name = "--no-default-features"
    description = "Do not activate default feature"

    [[command.subcommands.flags]]
    name = "--package"
    short = "-p"
    description = "Package to build"
    takes_value = true
    value_hint = "cargo_target"

    [[command.subcommands.flags]]
    name = "--jobs"
    short = "-j"
    description = "Number of parallel jobs"
    takes_value = true

[[command.subcommands]]
name = "run"
description = "Run a binary or example of the local package"
aliases = ["r"]

    [[command.subcommands.flags]]
    name = "--release"
    short = "-r"
    description = "Run in release mode"

    [[command.subcommands.flags]]
    name = "--bin"
    description = "Run the specified binary"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--example"
    description = "Run the specified example"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--features"
    short = "-F"
    description = "Features to activate"
    takes_value = true

[[command.subcommands]]
name = "test"
description = "Run the tests"
aliases = ["t"]

    [[command.subcommands.flags]]
    name = "--release"
    description = "Test in release mode"

    [[command.subcommands.flags]]
    name = "--no-run"
    description = "Compile but don't run tests"

    [[command.subcommands.flags]]
    name = "--doc"
    description = "Test documentation"

    [[command.subcommands.flags]]
    name = "--lib"
    description = "Test only library"

    [[command.subcommands.flags]]
    name = "--features"
    short = "-F"
    description = "Features to activate"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--package"
    short = "-p"
    description = "Package to test"
    takes_value = true
    value_hint = "cargo_target"

[[command.subcommands]]
name = "check"
description = "Check the current package for errors"
aliases = ["c"]

    [[command.subcommands.flags]]
    name = "--release"
    description = "Check in release mode"

    [[command.subcommands.flags]]
    name = "--all-targets"
    description = "Check all targets"

    [[command.subcommands.flags]]
    name = "--features"
    short = "-F"
    description = "Features to activate"
    takes_value = true

[[command.subcommands]]
name = "clean"
description = "Remove the target directory"

    [[command.subcommands.flags]]
    name = "--release"
    description = "Clean release artifacts"

    [[command.subcommands.flags]]
    name = "--package"
    short = "-p"
    description = "Package to clean"
    takes_value = true
    value_hint = "cargo_target"

[[command.subcommands]]
name = "new"
description = "Create a new Cargo package"

    [[command.subcommands.flags]]
    name = "--bin"
    description = "Create binary package (default)"
    conflicts_with = ["--lib"]

    [[command.subcommands.flags]]
    name = "--lib"
    description = "Create library package"
    conflicts_with = ["--bin"]

    [[command.subcommands.flags]]
    name = "--name"
    description = "Set the package name"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--edition"
    description = "Rust edition"
    takes_value = true
    value_enum = ["2015", "2018", "2021", "2024"]

    [[command.subcommands.args]]
    name = "path"
    description = "Directory for the new package"
    value_hint = "directory"
    required = true

[[command.subcommands]]
name = "init"
description = "Create a new Cargo package in an existing directory"

    [[command.subcommands.flags]]
    name = "--bin"
    description = "Create binary package (default)"
    conflicts_with = ["--lib"]

    [[command.subcommands.flags]]
    name = "--lib"
    description = "Create library package"
    conflicts_with = ["--bin"]

[[command.subcommands]]
name = "add"
description = "Add dependencies to a Cargo.toml manifest"

    [[command.subcommands.flags]]
    name = "--dev"
    short = "-D"
    description = "Add as dev dependency"

    [[command.subcommands.flags]]
    name = "--build"
    short = "-B"
    description = "Add as build dependency"

    [[command.subcommands.flags]]
    name = "--features"
    short = "-F"
    description = "Features to enable"
    takes_value = true

    [[command.subcommands.flags]]
    name = "--no-default-features"
    description = "Disable default features"

    [[command.subcommands.args]]
    name = "crate"
    description = "Crate name to add"
    required = true

[[command.subcommands]]
name = "remove"
description = "Remove dependencies from a Cargo.toml manifest"
aliases = ["rm"]

    [[command.subcommands.flags]]
    name = "--dev"
    short = "-D"
    description = "Remove from dev dependencies"

    [[command.subcommands.flags]]
    name = "--build"
    short = "-B"
    description = "Remove from build dependencies"

    [[command.subcommands.args]]
    name = "crate"
    description = "Crate name to remove"
    required = true

[[command.subcommands]]
name = "update"
description = "Update dependencies as recorded in the lock file"

    [[command.subcommands.flags]]
    name = "--dry-run"
    description = "Don't actually update"

[[command.subcommands]]
name = "publish"
description = "Publish this package to a registry"

    [[command.subcommands.flags]]
    name = "--dry-run"
    description = "Perform all checks without publishing"

    [[command.subcommands.flags]]
    name = "--allow-dirty"
    description = "Allow publishing from a dirty working directory"

[[command.subcommands]]
name = "clippy"
description = "Run Clippy lints"

    [[command.subcommands.flags]]
    name = "--fix"
    description = "Automatically apply lint suggestions"

    [[command.subcommands.flags]]
    name = "--all-targets"
    description = "Lint all targets"

    [[command.subcommands.flags]]
    name = "--features"
    short = "-F"
    description = "Features to activate"
    takes_value = true

[[command.subcommands]]
name = "fmt"
description = "Format the current package's source code"

    [[command.subcommands.flags]]
    name = "--check"
    description = "Check formatting without applying changes"

    [[command.subcommands.flags]]
    name = "--all"
    description = "Format all packages in workspace"

[[command.subcommands]]
name = "doc"
description = "Build documentation"

    [[command.subcommands.flags]]
    name = "--open"
    description = "Open docs in browser"

    [[command.subcommands.flags]]
    name = "--no-deps"
    description = "Don't build documentation for dependencies"

[[command.subcommands]]
name = "bench"
description = "Run benchmarks"

    [[command.subcommands.flags]]
    name = "--features"
    short = "-F"
    description = "Features to activate"
    takes_value = true
```

---

## Validation Rules

### Required Fields

| Field | Required | Default |
|---|---|---|
| `wit_completion_version` | yes | - |
| `command.name` | yes | - |
| `command.description` | no | `""` |
| `flag.name` | yes | - |
| `flag.takes_value` | no | `false` |
| `arg.name` | yes | - |
| `arg.required` | no | `false` |
| `subcommand.name` | yes | - |

### Validation Checks

The engine performs validation when loading a file:

1. **Version check:** `wit_completion_version` must be a version the engine supports
2. **Name uniqueness:** No two flags with the same `name` in the same scope
3. **Short uniqueness:** No two flags with the same `short` in the same scope
4. **conflicts_with valid:** Every flag in `conflicts_with` must exist
5. **requires valid:** Every flag in `requires` must exist
6. **No circular requires:** `A requires B requires A` - error
7. **value_enum non-empty:** If `value_enum` is present, it must have at least 1 value
8. **value_hint valid:** `value_hint` must be one of the values in the supported hints table
9. **Subcommand name valid:** Must not contain whitespace
10. **Flag name format:** Long flags must start with `--`, short flags must start with `-` followed by exactly 1 character

### Error Handling on Validation Failure

- Log warning with file path and error line
- Skip the erroneous item, still load the rest of the file
- Do not crash the app because of one faulty completion file

---

## Rust Schema Types

Completion data files are deserialized into the following Rust types:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CompletionFile {
    pub wit_completion_version: String,
    pub command: CommandDef,
}

#[derive(Deserialize)]
pub struct CommandDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub flags: Vec<FlagDef>,
    #[serde(default)]
    pub subcommands: Vec<SubcommandDef>,
    #[serde(default)]
    pub args: Vec<ArgDef>,
}

#[derive(Deserialize)]
pub struct SubcommandDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub flags: Vec<FlagDef>,
    #[serde(default)]
    pub subcommands: Vec<SubcommandDef>,  // Nested subcommands
    #[serde(default)]
    pub args: Vec<ArgDef>,
}

#[derive(Deserialize)]
pub struct FlagDef {
    pub name: String,
    #[serde(default)]
    pub short: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub takes_value: bool,
    #[serde(default)]
    pub value_hint: Option<String>,
    #[serde(default)]
    pub value_enum: Option<Vec<String>>,
    #[serde(default)]
    pub default_value: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub repeatable: bool,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub deprecated_message: Option<String>,
    #[serde(default)]
    pub conflicts_with: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Deserialize)]
pub struct ArgDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub value_hint: Option<String>,
    #[serde(default)]
    pub value_enum: Option<Vec<String>>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub repeatable: bool,
}
```

---

## Versioning & Compatibility

### Schema Versioning

| Version | Status | Changes |
|---|---|---|
| `1.0` | Current | Initial schema |
| `1.1` (planned) | - | Add `platforms` field (OS-specific flags) |
| `1.2` (planned) | - | Add `examples` field (usage examples) |
| `2.0` (planned) | - | Restructured format (TBD) |

### Backward Compatibility Rules

- Engine v1.x **must** be able to read any file v1.x (including higher minor versions - ignore unknown fields)
- Engine v2.x **must** be able to read file v1.x (migration path)
- File v2.x is **not** required to be readable by engine v1.x

---

## Contributor Guide

### Writing a new completion file

1. Create a file `<command-name>.toml` in `completions/community/`
2. Start with `wit_completion_version = "1.0"` and a `[command]` section
3. Refer to `man <command>` or `<command> --help` to get flags and subcommands
4. **No need to cover 100% of flags** - focus on the most commonly used flags
5. Test with the validation tool: `wit completion validate <file.toml>`
6. Submit a pull request

### Checklist for contributors

- [ ] File starts with `wit_completion_version = "1.0"`
- [ ] `[command]` has `name` and `description`
- [ ] Every flag has `name` (including `--` prefix)
- [ ] Short flags use `-` prefix + 1 character
- [ ] `description` is concise, starts with a verb (e.g., "Show...", "Set...", "Enable...")
- [ ] `conflicts_with` references correct flag names
- [ ] `value_hint` uses a valid value from the supported hints table
- [ ] `value_enum` lists enough common values (does not need to be exhaustive)
- [ ] File parses successfully (run the validation tool)

### Validation Tool

```bash
# Validate a single file
wit completion validate path/to/command.toml

# Validate all files in a directory
wit completion validate-all path/to/completions/

# Preview completions from a file (dry-run)
wit completion preview path/to/command.toml "git commit --"
```

---

## Performance Considerations

- All completion files are loaded into memory at startup (lazy loading per file if needed)
- Estimated memory: ~1KB per command definition - 1000 commands ~ 1MB
- File I/O only occurs during load and hot-reload
- Completion files are parsed into optimized in-memory data structures (raw TOML is not retained)
- Subcommand lookup uses `HashMap` for O(1) access
- Flag lookup uses `HashMap` for O(1) access

---

## Known Limitations and Future Work

1. **v1.0:** Does not yet support platform-specific flags (e.g., `--color` only on Linux)
2. **v1.1:** Add `platforms` field to filter by OS
3. **Future:** Auto-import from man pages (`wit completion generate <command>`)
4. **Future:** Import from shell completion scripts (bash/zsh/fish)
5. **Future:** Web-based editor for creating completion files (visual tool)
