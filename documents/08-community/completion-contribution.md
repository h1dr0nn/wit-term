# Completion Rules Contribution Guide

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Why Contribute Completion Rules?

Completion rules are the **heart** of Wit. Every TOML file you write helps thousands of terminal users work faster - typing less, remembering less, making fewer mistakes.

**And you can absolutely do it.** You just need:

- To know how to write TOML (simpler than JSON or YAML)
- To understand a CLI tool that you use daily
- To read man pages or `--help` output

No Rust knowledge required. No TypeScript knowledge required. No need to build the project. Just write a TOML file.

---

## What Are Completion Rules?

Completion rules are TOML files that describe the structure of a CLI command:

- **Command name** - the command name (e.g., `curl`, `git`, `docker`)
- **Subcommands** - sub-commands (e.g., `git commit`, `docker build`)
- **Flags/Options** - flags and options (e.g., `--verbose`, `-o <file>`)
- **Arguments** - the type of argument the command accepts (file, URL, hostname, etc.)
- **Descriptions** - brief descriptions to help users understand each option

When a user types a command in Wit terminal and presses Tab, the engine reads these files and displays relevant suggestions.

> Full schema: [Completion Data Format](../06-reference/completion-data-format.md)

---

## Step-by-Step Guide

### Step 1: Choose a command

See the [list of commands needing completions](#commands-needing-completions) at the bottom of the page. Choose a command you are familiar with.

Before starting, check if someone is already working on it:
- Look in the `completions/` directory in the repo
- Search GitHub Issues to see if someone has already claimed it
- If not, create a new issue or comment "I'll work on this" on an existing issue

### Step 2: Research the command

Gather information about the command:

```bash
# Read man page
man curl

# View help output
curl --help
curl --help all      # if extended help is available

# Check version (to note in the file)
curl --version
```

Reference sources:
- Man pages (`man <command>`)
- `--help` / `-h` output
- Official documentation website
- `tldr <command>` (brief summary)

You don't need to cover 100% of flags. Focus on the **most common flags** first.

### Step 3: Create the TOML file

Create a file in the `completions/` directory:

```
completions/
├── git.toml       # already exists
├── docker.toml    # already exists
└── curl.toml      # <- your new file
```

Start with the basic structure:

```toml
# Completion rules for curl
# Source: man curl, curl --help all
# Author: <your-name>
# Date: 2026-03-23

[command]
name = "curl"
description = "Transfer data from or to a server"
```

### Step 4: Test locally

```bash
# Validate TOML syntax
wit completions validate completions/curl.toml

# Test completions in Wit
pnpm tauri dev
# Then type "curl " and press Tab to see completions
```

If you haven't set up the dev environment, you can still validate TOML syntax online or with any TOML linter.

### Step 5: Submit PR

```bash
git checkout -b completions/add-curl
git add completions/curl.toml
git commit -m "feat(completions): add curl completion rules"
git push origin completions/add-curl
```

Create a Pull Request with the description:
- Which command you added
- Reference source (man page version, official docs URL)
- Number of flags/subcommands covered
- Any edge cases worth noting

---

## Complete Example: Writing Completions for `curl`

Here is a detailed walkthrough so you can see the process from start to finish.

### 3.1 - Start with the command definition

```toml
# Completion rules for curl
# Source: man curl (curl 8.7), https://curl.se/docs/manpage.html
# Author: your-name
# Date: 2026-03-23

[command]
name = "curl"
description = "Transfer data from or to a server using URLs"
```

### 3.2 - Add the most common flags

Start with flags that everyone uses:

```toml
[[command.flags]]
name = "--output"
short = "-o"
description = "Write output to file instead of stdout"
argument = "file"

[[command.flags]]
name = "--silent"
short = "-s"
description = "Silent mode, don't show progress or errors"

[[command.flags]]
name = "--verbose"
short = "-v"
description = "Make the operation more talkative"

[[command.flags]]
name = "--location"
short = "-L"
description = "Follow redirects"

[[command.flags]]
name = "--head"
short = "-I"
description = "Fetch headers only"

[[command.flags]]
name = "--include"
short = "-i"
description = "Include HTTP response headers in output"

[[command.flags]]
name = "--data"
short = "-d"
description = "Send data in POST request body"
argument = "data"

[[command.flags]]
name = "--header"
short = "-H"
description = "Add custom header to request"
argument = "header"
repeatable = true
```

### 3.3 - Add argument types

For flags that accept arguments with fixed values:

```toml
[[command.flags]]
name = "--request"
short = "-X"
description = "Specify HTTP method to use"
argument = "method"
argument_values = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]

[[command.flags]]
name = "--compressed"
description = "Request compressed response (using deflate or gzip)"

[[command.flags]]
name = "--connect-timeout"
description = "Maximum time in seconds for connection"
argument = "seconds"
argument_type = "number"

[[command.flags]]
name = "--max-time"
short = "-m"
description = "Maximum time in seconds for the whole operation"
argument = "seconds"
argument_type = "number"
```

### 3.4 - Add advanced flags

```toml
[[command.flags]]
name = "--user"
short = "-u"
description = "Server user and password"
argument = "user:password"

[[command.flags]]
name = "--cookie"
short = "-b"
description = "Send cookies from string/file"
argument = "data|filename"

[[command.flags]]
name = "--cookie-jar"
short = "-c"
description = "Write cookies to file after operation"
argument = "filename"

[[command.flags]]
name = "--cert"
short = "-E"
description = "Client certificate file"
argument = "certificate[:password]"

[[command.flags]]
name = "--insecure"
short = "-k"
description = "Allow insecure server connections (skip TLS verification)"

[[command.flags]]
name = "--proxy"
short = "-x"
description = "Use specified proxy"
argument = "[protocol://]host[:port]"

[[command.flags]]
name = "--upload-file"
short = "-T"
description = "Transfer local file to remote URL"
argument = "file"

[[command.flags]]
name = "--form"
short = "-F"
description = "Specify multipart form data"
argument = "name=content"
repeatable = true

[[command.flags]]
name = "--retry"
description = "Retry request if transient problems occur"
argument = "num"
argument_type = "number"

[[command.flags]]
name = "--user-agent"
short = "-A"
description = "Set User-Agent header"
argument = "name"

[[command.flags]]
name = "--write-out"
short = "-w"
description = "Display information on completion"
argument = "format"
```

### 3.5 - Command arguments (positional)

```toml
[[command.arguments]]
name = "url"
description = "URL(s) to transfer"
type = "url"
required = true
repeatable = true
```

### 3.6 - Complete file

Combine everything into the file `completions/curl.toml`. Ensure flags are sorted alphabetically (by long name).

---

## Style Guide

### File naming rules

| Type | Rule | Example |
|------|------|---------|
| Single command | `<command>.toml` | `curl.toml`, `wget.toml` |
| Command group | `<group>.toml` | `git.toml` (covers all git subcommands) |
| Scoped commands | `<scope>-<command>.toml` | `npm.toml`, `yarn.toml` (separate files) |

### General principles

**One file per command (or command group)**:
- `curl.toml` - one command, one file
- `git.toml` - git and all subcommands in one file
- `docker.toml` - docker and all subcommands in one file

**Always write descriptions**:
```toml
# Good - has description
[[command.flags]]
name = "--verbose"
short = "-v"
description = "Make the operation more talkative"

# Not good - missing description
[[command.flags]]
name = "--verbose"
short = "-v"
```

Descriptions help users **learn the tool** while using Wit. This is one of the greatest values of the completion system.

**Sort flags alphabetically** (by long name `--abc`):

```toml
# Good - alphabetical
[[command.flags]]
name = "--all"
# ...

[[command.flags]]
name = "--branch"
# ...

[[command.flags]]
name = "--cached"
# ...
```

**Cover common flags first**, no need to be exhaustive. A file covering 30 common flags well is better than 200 flags missing descriptions.

**Consistent formatting**:
- Use double quotes for strings: `name = "--verbose"`
- Blank line between `[[command.flags]]` blocks
- Comment at the top of the file noting source and author
- Indent with spaces, do not use tabs

---

## Quality Checklist

Before submitting a PR, check:

- [ ] **Follows schema format** - TOML follows the schema specified in [Completion Data Format](../06-reference/completion-data-format.md)
- [ ] **Includes descriptions** - Every flag, subcommand, and argument has a description
- [ ] **Tested locally** - Validated with the validation tool or a TOML linter
- [ ] **No duplicate entries** - No flag or subcommand is repeated
- [ ] **Common flags covered** - At least covers the most common flags
- [ ] **Special characters escaped** - TOML special characters in descriptions are properly escaped
- [ ] **Source documented** - Comment at the top of the file clearly states reference source (man page version, docs URL)
- [ ] **Alphabetically sorted** - Flags sorted in alphabetical order
- [ ] **Short flags included** - If a short form exists (e.g., `-v`), it must be included
- [ ] **Argument types correct** - Flags requiring arguments clearly specify the type (file, number, string, enum values)

---

## Commands Needing Completions

### Tier 1 - Ship with v0.1 (highest priority)

These are the most basic commands - nearly everyone using a terminal needs them.

| Command | Category | Complexity | Status |
|---------|----------|------------|--------|
| `git` | Version Control | High (many subcommands) | Needed |
| `docker` | Containers | High | Needed |
| `npm` | Package Manager | Medium | Needed |
| `yarn` | Package Manager | Medium | Needed |
| `cargo` | Package Manager (Rust) | Medium | Needed |
| `pip` | Package Manager (Python) | Medium | Needed |
| `python` | Runtime | Low | Needed |
| `node` | Runtime | Low | Needed |
| `make` | Build Tool | Low-Medium | Needed |
| `curl` | HTTP Client | Medium | Needed |
| `wget` | HTTP Client | Low | Needed |
| `ssh` | Remote Access | Medium | Needed |
| `scp` | File Transfer | Low | Needed |
| `rsync` | File Sync | Medium | Needed |
| `tar` | Archive | Low | Needed |
| `grep` | Search | Low | Needed |
| `find` | Search | Medium | Needed |
| `sed` | Text Processing | Low | Needed |
| `awk` | Text Processing | Low | Needed |
| `chmod` | Permissions | Low | Needed |
| `chown` | Permissions | Low | Needed |

**Suggestion for newcomers**: start with commands that have "Low" complexity like `wget`, `tar`, `grep`, `chmod`, `chown`. This is an excellent way to get familiar with the format before tackling large commands like `git` or `docker`.

### Tier 2 - DevOps and Cloud

| Command | Category | Complexity |
|---------|----------|------------|
| `kubectl` | Kubernetes | High |
| `terraform` | IaC | High |
| `aws` | Cloud CLI | Very high |
| `gcloud` | Cloud CLI | Very high |
| `az` | Cloud CLI | Very high |
| `helm` | Kubernetes | Medium |
| `systemctl` | System Management | Medium |
| `journalctl` | Logging | Medium |
| `tmux` | Terminal Multiplexer | Medium |
| `screen` | Terminal Multiplexer | Low |

### Tier 3 - Specialized Tools and Community Requests

| Command | Category | Complexity |
|---------|----------|------------|
| `ffmpeg` | Media Processing | Very high |
| `imagemagick` / `convert` | Image Processing | High |
| `pandoc` | Document Conversion | Medium |
| `jq` | JSON Processing | Medium |
| `yq` | YAML Processing | Medium |
| Community requests | Various | Various |

You can also suggest new commands by creating a GitHub Issue!

---

## Recognition

All contributions are acknowledged:

### In the completion file

Your name will be noted in the header comment:

```toml
# Completion rules for curl
# Source: man curl (curl 8.7)
# Author: your-github-username
# Contributors: other-contributor-1, other-contributor-2
# Date: 2026-03-23
```

### In CONTRIBUTORS.md

The `CONTRIBUTORS.md` file at the repo root lists all contributors:

```markdown
## Completion Rule Contributors

- **@your-username** - curl, wget, tar
- **@other-contributor** - git, docker
```

### In release notes

Each release will acknowledge new completion rules and their contributors.

---

## FAQ

### Do I need to set up the full dev environment?

**No!** To contribute completion rules, you only need:
- A text editor (VS Code, Vim, any editor)
- Git (to fork, clone, and submit PRs)
- Knowledge of TOML format
- Understanding of the command you want to write completions for

Setting up Rust/Tauri/Node.js is only needed if you want to test completions directly in Wit.

### I'm not sure which flags are "common"?

Some ways to determine:
- **Flags that appear in tutorials** - if every tutorial uses it, it's common
- **Flags in `tldr`** - `tldr curl` gives you the most common use cases
- **Flags you use daily** - personal experience is a good source
- **Flags in `--help` summary** - common flags are usually listed first

### Can I update an existing completion file?

**Yes!** You can:
- Add missing flags
- Improve descriptions
- Fix errors
- Add argument types

Just submit a PR and note what you changed.

### How big is a completion file?

There is no hard limit, but general guidelines:
- **Small** (20-30 flags): `wget`, `tar`, `grep` - suitable for newcomers
- **Medium** (50-80 flags): `curl`, `ssh`, `rsync`
- **Large** (100+ flags + subcommands): `git`, `docker`, `kubectl`

Start small, expand gradually.

### I'm not familiar with TOML syntax, is there documentation?

TOML is very simple. You just need to know:

```toml
# Comments start with #

# Key-value pairs
name = "curl"
version = 1

# Sections
[command]
name = "curl"

# Array of tables (multiple items of the same type)
[[command.flags]]
name = "--verbose"

[[command.flags]]
name = "--silent"

# Arrays
argument_values = ["GET", "POST", "PUT", "DELETE"]

# Boolean
repeatable = true
```

Reference: [TOML specification](https://toml.io/en/)

---

## Get Started Now!

1. See the [list of commands needing completions](#commands-needing-completions)
2. Choose a command with "Low" complexity if this is your first time
3. Read `man <command>` or `<command> --help`
4. Write a TOML file following the [curl example above](#complete-example-writing-completions-for-curl)
5. Submit a PR

Every contribution, even just a small completion file for a simple command, creates real value for the community. Thank you!
