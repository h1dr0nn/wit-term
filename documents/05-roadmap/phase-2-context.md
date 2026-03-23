# Phase 2: Context (Months 4-6)

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Goals and Success Criteria

### Goals
1. Build context engine that detects project type from directory structure
2. Implement provider system with built-in providers for common toolchains
3. Design a standardized completion data format that is easy to contribute
4. Build completion engine with fuzzy matching and intelligent ranking
5. Integrate shell via OSC sequences (CWD tracking, prompt detection)
6. UI for tab completion: inline hints and completion popup

### Success Criteria
- [ ] `cd` into a git repo -> Wit detects it as a git project
- [ ] Tab after `git ` -> shows list of git subcommands
- [ ] Tab after `git checkout ` -> shows list of branches
- [ ] `cd` into a Node project -> Tab after `npm run ` -> shows list of scripts from package.json
- [ ] Fuzzy matching: type `git chk` -> suggests `checkout`
- [ ] Completion popup looks good, is keyboard navigable
- [ ] Context detection < 100ms for a project with 1000 files

---

## Week-by-Week Breakdown

### Week 13-14: Context Engine Foundation

**Objective:** Architecture for context detection, provider trait, directory scanning.

**Tasks:**
- [ ] Design `ContextProvider` trait in Rust:
  ```
  trait ContextProvider {
      fn name(&self) -> &str;
      fn detect(&self, dir: &Path) -> Option<ProjectContext>;
      fn completions(&self, ctx: &ProjectContext, partial: &str) -> Vec<Completion>;
  }
  ```
- [ ] Implement `ProjectContext` struct: project type, root dir, metadata
- [ ] Directory scanner: walk up from CWD to find project markers
- [ ] Marker detection: `.git`, `package.json`, `Cargo.toml`, `Dockerfile`, etc.
- [ ] Context cache: avoid re-scanning every time a command is typed
- [ ] Context change detection: watch filesystem events (notify crate)
- [ ] Tauri IPC: send context info from Rust -> React frontend
- [ ] Unit tests for context detection logic

**Output:** Wit detects project type when `cd`-ing into a directory.

### Week 15-16: Built-in Providers

**Objective:** Implement providers for 5+ common toolchains.

**Tasks:**
- [ ] **Git provider:**
  - Detect: `.git` directory
  - Context: current branch, remotes, status (clean/dirty)
  - Dynamic completions: branches, tags, remotes, changed files
- [ ] **Node/npm provider:**
  - Detect: `package.json`
  - Context: package name, version, dependencies
  - Dynamic completions: npm scripts, package names from node_modules
- [ ] **Python provider:**
  - Detect: `requirements.txt`, `pyproject.toml`, `setup.py`, `venv/`
  - Context: virtual env status, Python version
  - Dynamic completions: pip packages, pytest test files
- [ ] **Rust/Cargo provider:**
  - Detect: `Cargo.toml`
  - Context: crate name, workspace members
  - Dynamic completions: cargo subcommands, workspace targets
- [ ] **Docker provider:**
  - Detect: `Dockerfile`, `docker-compose.yml`
  - Context: services, images
  - Dynamic completions: docker-compose services, image names
- [ ] Provider registry: register, lookup, priority ordering
- [ ] Multiple providers active simultaneously (git + node for JS project)

**Output:** `cd` into a project -> Wit knows what type of project it is, shows metadata.

### Week 17-18: Completion Data Format and Completion Files

**Objective:** Design standardized format for completion data, write initial completion files.

**Tasks:**
- [ ] Design completion data format (YAML or TOML):
  ```yaml
  command: git
  subcommands:
    - name: commit
      description: Record changes to the repository
      flags:
        - name: --message
          short: -m
          description: Commit message
          takes_value: true
        - name: --all
          short: -a
          description: Stage all modified files
  ```
- [ ] Validation schema for completion files
- [ ] Parser: read completion files -> internal data structures
- [ ] **Completion files to write:**
  1. `git` - subcommands, flags, dynamic completions (branches, files)
  2. `npm` / `yarn` / `pnpm` - scripts, packages
  3. `cargo` - subcommands, targets
  4. `docker` - subcommands, common flags
  5. `kubectl` - resources, namespaces
  6. `ssh` - hosts from ~/.ssh/config
  7. `cd` / `ls` / `cat` - path completions
  8. `make` - targets from Makefile
  9. `pip` / `python` - packages, modules
  10. `systemctl` - units, subcommands
- [ ] Hot-reload: edit completion file -> completions update immediately
- [ ] Documentation for format so community can contribute

**Output:** 10+ command groups have completion data.

### Week 19-20: Completion Engine

**Objective:** Engine that handles Tab press, fuzzy matching, ranking.

**Tasks:**
- [ ] Parse current command line: command, subcommand, flags, arguments, cursor position
- [ ] Determine completion context: completing a command, subcommand, flag, or argument?
- [ ] **Fuzzy matching algorithm:**
  - Exact prefix match (highest priority)
  - Substring match
  - Fuzzy match (character-by-character, allow gaps)
  - Abbreviation match (`gc` -> `git commit`)
- [ ] **Ranking algorithm:**
  - Match quality score (exact > prefix > fuzzy)
  - Frequency score (frequently used commands rank higher)
  - Recency score (recently used commands rank higher)
  - Context score (git commands rank higher in a git repo)
- [ ] Completion result struct: text, description, icon, score, source
- [ ] Performance: < 10ms for 10000 completion candidates
- [ ] Usage tracking: record commands used to improve ranking
- [ ] Unit tests for fuzzy matching and ranking

**Output:** Type partial command -> receive list of completions sorted by relevance.

### Week 21-22: Shell Integration

**Objective:** Deeper integration with shell via OSC sequences.

**Tasks:**
- [ ] **CWD tracking:** Detect shell CWD changes
  - Option A: Parse OSC 7 (shell reports CWD)
  - Option B: Parse prompt (regex-based, less reliable)
  - Option C: Monitor /proc/{pid}/cwd (Linux only)
  - Recommend: OSC 7 primary, fallback to /proc
- [ ] Shell integration scripts:
  - bash: `.bashrc` additions (PROMPT_COMMAND, OSC 7 reporting)
  - zsh: `.zshrc` additions (precmd hook, OSC 7)
  - fish: `config.fish` additions
  - PowerShell: profile additions
- [ ] **Prompt detection:** Determine when shell is ready to receive input
  - OSC 133 (command start/end markers)
  - Heuristic: detect prompt pattern
- [ ] **Command history integration:**
  - Read shell history file
  - Track commands typed in Wit
  - Use history for completion ranking
- [ ] **Semantic prompt zones:**
  - Distinguish prompt, command, output
  - Click on command -> highlight related output
- [ ] Auto-detect shell type and suggest integration script

**Output:** Wit knows the current CWD, knows when the user is at a prompt.

### Week 23-24: Tab Completion UI

**Objective:** UI components for the completion experience.

**Tasks:**
- [ ] **Inline hint:** Ghost text displaying top completion
  - Shown in a faded color after cursor
  - Tab to accept, continue typing to refine
  - Esc to dismiss
- [ ] **Completion popup:**
  - Shown when there are multiple completions
  - Scrollable list with descriptions
  - Keyboard navigation: Up/Down, Enter to select, Esc to dismiss
  - Fuzzy filter: continue typing to filter list
  - Category grouping (subcommands, flags, arguments)
- [ ] **Completion popup styling:**
  - Width auto-adjusts to content
  - Position: above or below cursor (depending on space)
  - Highlight matched characters in completion text
  - Icon for each completion type (command, flag, file, branch)
- [ ] Tab behavior:
  - Single match -> auto-complete
  - Multiple matches -> show popup
  - No match -> do nothing (or terminal bell)
- [ ] Double-Tab: show all completions (traditional behavior)
- [ ] Integration: completion engine -> UI update cycle
- [ ] Animation: popup appear/disappear smoothly

**Output:** Tab press -> see completions. Experience similar to Fig/Warp.

---

## Phase 2 Deliverables

| # | Deliverable | Description |
| - | ----------- | ----------- |
| 1 | Context engine | Detect project type, cache context, watch changes |
| 2 | 5 built-in providers | Git, Node, Python, Rust, Docker |
| 3 | Completion data format | YAML/TOML spec, validation, documentation |
| 4 | 10+ completion files | git, npm, cargo, docker, kubectl, ssh, cd, make, pip, systemctl |
| 5 | Completion engine | Fuzzy matching, ranking, < 10ms response |
| 6 | Shell integration | OSC 7 CWD tracking, integration scripts for 4 shells |
| 7 | Tab completion UI | Inline hints, popup, keyboard navigation |

---

## Completion Coverage Targets

By the end of Phase 2, Wit must have completions for at least 10 command groups:

| Priority | Command Group | Scope |
| -------- | ------------- | ----- |
| P0 | `git` | Subcommands, flags, branches, tags, remotes, files |
| P0 | `npm` / `yarn` | Scripts, packages, subcommands |
| P0 | `cargo` | Subcommands, targets, features |
| P0 | `docker` | Subcommands, containers, images, volumes |
| P1 | `kubectl` | Resources, namespaces, pods |
| P1 | `ssh` / `scp` | Hosts from config, common flags |
| P1 | `cd` / `ls` / `cat` | Path completions (directories, files) |
| P1 | `make` | Targets from Makefile |
| P2 | `pip` / `python` | Packages, modules |
| P2 | `systemctl` | Units, subcommands |

---

## Integration Context <-> Completions

How the context engine influences completions:

```
User CWD: ~/projects/my-app/
  |
  ├── Context: Git repo (branch: feature/login)
  |   └── git checkout -> suggest: feature/login, main, develop, ...
  |
  ├── Context: Node project (package.json found)
  |   └── npm run -> suggest: dev, build, test, lint (from scripts)
  |
  └── Context: Docker (docker-compose.yml found)
      └── docker-compose up -> suggest: web, db, redis (from services)
```

**Completion priority based on context:**
- In a git repo: git completions rank higher
- In a Node project: npm/yarn completions rank higher
- In a Rust project: cargo completions rank higher
- Outside any project: generic Unix completions

---

## Definition of "Phase 2 Complete"

Phase 2 is considered complete when **all** of the following conditions are met:

1. **Context detection:** `cd` into a git repo -> Wit detects git project + branch name
2. **Context detection:** `cd` into a Node project -> Wit detects Node project + package name
3. **Git completions:** Tab after `git ` -> list of subcommands (commit, push, pull, ...)
4. **Git dynamic:** Tab after `git checkout ` -> list of actual branches in the repo
5. **npm completions:** Tab after `npm run ` in a Node project -> list of scripts from package.json
6. **Fuzzy matching:** Type `git chk` + Tab -> suggest `checkout`
7. **Completion popup:** Shows popup when there are multiple options, keyboard navigable
8. **Inline hint:** Ghost text shown for top suggestion
9. **CWD tracking:** `cd` into a different directory -> context updates automatically
10. **Performance:** Completion response < 50ms in all cases

**Specific acceptance test:**
```
1. Open Wit
2. cd into a git + Node project
3. Type "git " + Tab -> see popup with git subcommands
4. Type "che" -> popup filters to "checkout"
5. Enter -> completes to "git checkout "
6. Tab -> see list of branches
7. Type "npm run " + Tab -> see npm scripts
8. Everything must be fast, smooth, no lag
```
