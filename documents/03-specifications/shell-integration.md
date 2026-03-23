# Shell Integration

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

Shell integration is the mechanism that allows Wit to **understand** what the shell is doing - knowing the current CWD, knowing when a prompt appears, knowing which command just finished and what the exit code was. This transforms the terminal from a "black box that only displays text" into a "context-aware tool".

Shell integration works by injecting a small script into the shell startup. This script uses **OSC escape sequences** (Operating System Commands) to send metadata from the shell to the terminal emulator through the same PTY stream - no separate IPC needed, no socket, no file.

**Important principle:** Shell integration is **optional**. Wit works normally without it (Level 0). Integration only **adds features**; it is not a requirement.

---

## Integration Levels

### Level 0: None (Baseline)

**No setup required.** This is the default state when the user has not set up shell integration.

| Feature | Available |
|---|---|
| Terminal I/O | Yes |
| ANSI rendering | Yes |
| Completion (static, history, path) | Yes |
| CWD tracking | No - only knows CWD at session start |
| Context detection | Limited - only based on initial CWD |
| Prompt boundaries | No |
| Command tracking | No |

### Level 1: CWD Tracking

**Shell sends the new CWD whenever CWD changes** (after each `cd`, `pushd`, `popd`, or any CWD change).

| Additional Feature | Description |
|---|---|
| Real-time CWD | Knows the current CWD at any time |
| Context detection | Context engine detects project type based on new CWD |
| Context completions | Suggests git branches, npm scripts, etc. based on current project |
| Tab title update | Tab title automatically updates based on CWD |

**Implementation:** OSC 7 (standard, supported by many terminals).

### Level 2: Prompt Detection

**Shell marks prompt boundaries** - letting Wit know where the prompt is and where command output is.

| Additional Feature | Description |
|---|---|
| Prompt markers | Visual indicators at the beginning of each prompt line |
| Click-to-select | Click on a command block to select all its output |
| Scroll-to-prompt | Navigate between prompts (Ctrl+Up/Down) |
| Command output isolation | Knows which output belongs to which command |

**Implementation:** OSC 133 (FinalTerm/iTerm2 protocol).

### Level 3: Command Tracking

**Shell notifies when a command starts and finishes**, including exit code.

| Additional Feature | Description |
|---|---|
| Command duration | Shows execution time for each command |
| Exit code badges | Checkmark/X icon with exit code for each command |
| Re-run last command | Button to re-run the last command |
| Failed command highlighting | Highlight output of failed commands |
| History enrichment | History entries include exit code and duration |

**Implementation:** OSC 133 (extended) + custom precmd/preexec hooks.

### Feature Matrix

```
                          Level 0   Level 1   Level 2   Level 3
Terminal I/O               Y         Y         Y         Y
ANSI rendering             Y         Y         Y         Y
Static completion          Y         Y         Y         Y
History completion         Y         Y         Y         Y
Path completion            Y         Y         Y         Y
Real-time CWD              X         Y         Y         Y
Context detection          limited   Y         Y         Y
Context completion         X         Y         Y         Y
Tab title (CWD)            X         Y         Y         Y
Prompt markers             X         X         Y         Y
Click-to-select output     X         X         Y         Y
Scroll-to-prompt           X         X         Y         Y
Command duration           X         X         X         Y
Exit code badges           X         X         X         Y
Re-run command             X         X         X         Y
History with exit codes    X         X         X         Y
```

---

## OSC Escape Sequences

### OSC Basics

OSC (Operating System Command) is an escape sequence with the format:

```
ESC ] <code> ; <data> ST
```

Where:

- `ESC ]` = `\x1b]` or `\033]` - OSC introducer
- `<code>` = integer identifying the command type
- `<data>` = payload (depends on type)
- `ST` = String Terminator = `ESC \` (`\x1b\x5c`) or `BEL` (`\x07`)

### OSC 7: CWD Reporting (Level 1)

**Standard:** Used by macOS Terminal, iTerm2, VTE-based terminals.

**Format:**

```
ESC ] 7 ; file://HOSTNAME/PATH ST
```

**Example:**

```
\x1b]7;file://laptop/home/user/Projects/wit-term\x07
```

**When received:** The terminal emulator parses the URL, extracts the path, and updates the session CWD.

```rust
fn handle_osc_7(&mut self, uri: &str) {
    if let Ok(url) = url::Url::parse(uri) {
        if url.scheme() == "file" {
            if let Some(path) = url.to_file_path().ok() {
                self.session.update_cwd(path);
                self.emit(SessionEvent::CwdChanged {
                    id: self.session.id,
                    cwd: path,
                });
            }
        }
    }
}
```

### OSC 133: Prompt/Command Marking (Level 2 + 3)

**Standard:** FinalTerm semantic prompts, adopted by iTerm2, VS Code terminal, WezTerm.

**Sequences:**

| Sequence | Meaning | Level |
|---|---|---|
| `OSC 133 ; A ST` | **Prompt start** - beginning of prompt area | 2 |
| `OSC 133 ; B ST` | **Command start** - user pressed Enter, execution begins | 2 |
| `OSC 133 ; C ST` | **Command output start** - command begins producing output | 3 |
| `OSC 133 ; D ; <exitcode> ST` | **Command finished** - command completed, with exit code | 3 |

**Timeline:**

```
Shell shows prompt:           OSC 133;A  ->  $ git status  ->  OSC 133;B
                             ^ prompt       ^ user input     ^ user pressed Enter
                             start                           command start

Command runs and outputs:    OSC 133;C  ->  [output lines]  ->  OSC 133;D;0
                             ^ output       ^ command output    ^ command done
                             start                              exit code = 0
```

**Concrete example in the terminal stream:**

```
\x1b]133;A\x07$ git status\x1b]133;B\x07\x1b]133;C\x07On branch main
Changes not staged for commit:
  modified:   src/main.rs
\x1b]133;D;0\x07\x1b]133;A\x07$ _
```

### Custom OSC: Wit-specific (Future)

> **Status:** planned, not implemented in v1

Beyond OSC 7 and OSC 133, Wit may define custom OSC for its own features:

| Sequence | Description |
|---|---|
| `OSC 7777 ; witcmd=<json> ST` | Wit custom command (extensible) |

**Example usage for git context:**

```
\x1b]7777;witcmd={"type":"git_info","branch":"main","dirty":true}\x07
```

However, v1 will **not use custom OSC**. All features are based on standard OSC 7 and OSC 133.

---

## Shell Integration Scripts

### Implementation Overview

Each shell needs a separate script to inject OSC sequences. The script must be:

1. **Minimal:** As little code as possible - easy to audit, low risk of breaking shell config
2. **Non-invasive:** Does not override the user's existing hooks, appends instead
3. **Safe:** Handles edge cases (subshells, non-interactive mode, etc.)
4. **Portable:** Works on all common versions of the shell

### Bash Integration

```bash
# wit-integration.bash
# Shell integration for Wit terminal emulator
# Source: https://github.com/user/wit-term

# Guard: only run inside Wit terminal
if [ -z "$WIT_TERMINAL" ]; then
    return 0
fi

# Guard: only run in interactive mode
if [[ $- != *i* ]]; then
    return 0
fi

# Guard: do not run again if already injected
if [ -n "$__WIT_INTEGRATION_ACTIVE" ]; then
    return 0
fi
__WIT_INTEGRATION_ACTIVE=1

# -- Level 1: CWD reporting --------------------------------------------------

__wit_report_cwd() {
    printf '\033]7;file://%s%s\033\\' "$(hostname)" "$(pwd)"
}

# -- Level 2: Prompt marking -------------------------------------------------

__wit_prompt_start() {
    printf '\033]133;A\033\\'
}

__wit_command_start() {
    printf '\033]133;B\033\\'
}

# -- Level 3: Command tracking -----------------------------------------------

__wit_preexec() {
    printf '\033]133;C\033\\'
    __wit_command_start_time=$SECONDS
}

__wit_precmd() {
    local exit_code=$?
    printf '\033]133;D;%d\033\\' "$exit_code"
    __wit_report_cwd
    __wit_prompt_start
}

# -- Hook installation -------------------------------------------------------

# Use PROMPT_COMMAND for precmd
if [[ -z "$PROMPT_COMMAND" ]]; then
    PROMPT_COMMAND="__wit_precmd"
elif [[ "$PROMPT_COMMAND" != *"__wit_precmd"* ]]; then
    PROMPT_COMMAND="__wit_precmd;${PROMPT_COMMAND}"
fi

# Use DEBUG trap for preexec
# Caution: DEBUG trap runs before EVERY command, including those in PROMPT_COMMAND
__wit_debug_trap() {
    # Do not run preexec if inside PROMPT_COMMAND
    if [[ -n "$COMP_LINE" ]]; then
        return
    fi
    # Bash 4.4+: check BASH_COMMAND
    if [[ "$BASH_COMMAND" == "$PROMPT_COMMAND" ]] ||
       [[ "$BASH_COMMAND" == "__wit_precmd"* ]]; then
        return
    fi
    __wit_preexec
}

trap '__wit_debug_trap' DEBUG

# PS1 wrapping: add command_start marker after prompt
# Preserve user's PS1, add OSC 133;B at end
if [[ "$PS1" != *'133;B'* ]]; then
    PS1="${PS1}\[\033]133;B\033\\\\\]"
fi

# Initial CWD report
__wit_report_cwd
```

### Zsh Integration

```zsh
# wit-integration.zsh
# Shell integration for Wit terminal emulator

# Guard
[[ -z "$WIT_TERMINAL" ]] && return 0
[[ -o interactive ]] || return 0
[[ -n "$__WIT_INTEGRATION_ACTIVE" ]] && return 0
__WIT_INTEGRATION_ACTIVE=1

# -- Functions ----------------------------------------------------------------

__wit_report_cwd() {
    printf '\033]7;file://%s%s\033\\' "$(hostname)" "$(pwd)"
}

# -- Hooks --------------------------------------------------------------------

# precmd: runs before each prompt
__wit_precmd() {
    local exit_code=$?
    # Command finished (Level 3)
    printf '\033]133;D;%d\033\\' "$exit_code"
    # Report CWD (Level 1)
    __wit_report_cwd
    # Prompt start (Level 2)
    printf '\033]133;A\033\\'
}

# preexec: runs before each command execution
__wit_preexec() {
    # Command output start (Level 3)
    printf '\033]133;C\033\\'
}

# Zsh has a built-in hook system - much simpler than bash
autoload -Uz add-zsh-hook
add-zsh-hook precmd __wit_precmd
add-zsh-hook preexec __wit_preexec

# PS1 wrapping: add command_start marker
# %{...%} = zsh zero-width escape sequence wrapper
if [[ "$PS1" != *'133;B'* ]]; then
    PS1="${PS1}%{\033]133;B\033\\%}"
fi

# Initial CWD report
__wit_report_cwd
```

### Fish Integration

```fish
# wit-integration.fish
# Shell integration for Wit terminal emulator

# Guard
if not set -q WIT_TERMINAL
    exit 0
end

if set -q __WIT_INTEGRATION_ACTIVE
    exit 0
end
set -g __WIT_INTEGRATION_ACTIVE 1

# -- Functions ----------------------------------------------------------------

function __wit_report_cwd
    printf '\033]7;file://%s%s\033\\' (hostname) (pwd)
end

# -- Event handlers -----------------------------------------------------------

# fish_prompt: wrap existing prompt function
# Fish calls fish_prompt to render the prompt
functions --copy fish_prompt __wit_original_fish_prompt 2>/dev/null

function fish_prompt
    set -l exit_code $status

    # Command finished (Level 3)
    printf '\033]133;D;%d\033\\' $exit_code

    # Report CWD (Level 1)
    __wit_report_cwd

    # Prompt start (Level 2)
    printf '\033]133;A\033\\'

    # Original prompt
    __wit_original_fish_prompt

    # Command start marker (Level 2) - after prompt content
    printf '\033]133;B\033\\'
end

# preexec: runs before command execution
function __wit_preexec --on-event fish_preexec
    # Command output start (Level 3)
    printf '\033]133;C\033\\'
end

# Initial CWD report
__wit_report_cwd
```

### PowerShell Integration

```powershell
# wit-integration.ps1
# Shell integration for Wit terminal emulator

# Guard
if (-not $env:WIT_TERMINAL) { return }
if ($__WIT_INTEGRATION_ACTIVE) { return }
$global:__WIT_INTEGRATION_ACTIVE = $true

# -- Functions ----------------------------------------------------------------

function __wit_report_cwd {
    $cwd = (Get-Location).Path.Replace('\', '/')
    # Windows: convert C:\Users to /C:/Users for file:// URL
    if ($cwd -match '^([A-Za-z]):(.*)$') {
        $cwd = "/$($Matches[1]):$($Matches[2])"
    }
    [Console]::Write("`e]7;file://$([System.Net.Dns]::GetHostName())$cwd`e\")
}

# -- Prompt override ---------------------------------------------------------

# Save original prompt
if (Test-Path Function:\prompt) {
    $__wit_original_prompt = Get-Content Function:\prompt
    # Rename original prompt
    Rename-Item Function:\prompt __wit_original_prompt -ErrorAction SilentlyContinue
}

function global:prompt {
    $exitCode = if ($?) { 0 } else { $LASTEXITCODE }
    if ($null -eq $exitCode) { $exitCode = 0 }

    # Command finished (Level 3)
    [Console]::Write("`e]133;D;$exitCode`e\")

    # Report CWD (Level 1)
    __wit_report_cwd

    # Prompt start (Level 2)
    [Console]::Write("`e]133;A`e\")

    # Original prompt
    if (Test-Path Function:\__wit_original_prompt) {
        $result = __wit_original_prompt
    } else {
        $result = "PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
    }

    # Command start marker (Level 2)
    [Console]::Write("`e]133;B`e\")

    return $result
}

# PowerShell does not have a native preexec hook
# Workaround: use PSReadLine CommandNotFoundHandler or skip Level 3 preexec
# Level 3 still works via precmd (has exit code), only missing exact command start timing

# Initial CWD report
__wit_report_cwd
```

---

## Terminal Emulator: Processing OSC Sequences

### Parser Integration

Wit's ANSI parser must recognize and route OSC sequences:

```rust
pub enum OscAction {
    /// OSC 7: CWD changed
    SetCwd(PathBuf),

    /// OSC 133;A: Prompt started
    PromptStart,

    /// OSC 133;B: Command started (user pressed Enter)
    CommandStart,

    /// OSC 133;C: Command output started
    CommandOutputStart,

    /// OSC 133;D;N: Command finished with exit code N
    CommandFinished { exit_code: i32 },

    /// OSC 0/2: Set window/tab title
    SetTitle(String),

    /// Other OSC (pass through or ignore)
    Unknown { code: u16, data: String },
}

impl TerminalEmulator {
    fn handle_osc(&mut self, code: u16, data: &str) {
        let action = match code {
            7 => {
                // Parse file:// URL
                if let Some(path) = parse_file_url(data) {
                    OscAction::SetCwd(path)
                } else {
                    return;
                }
            }
            133 => {
                // Parse FinalTerm semantic prompt
                match data.chars().next() {
                    Some('A') => OscAction::PromptStart,
                    Some('B') => OscAction::CommandStart,
                    Some('C') => OscAction::CommandOutputStart,
                    Some('D') => {
                        let exit_code = data.get(2..)
                            .and_then(|s| s.parse::<i32>().ok())
                            .unwrap_or(0);
                        OscAction::CommandFinished { exit_code }
                    }
                    _ => return,
                }
            }
            0 | 2 => OscAction::SetTitle(data.to_string()),
            _ => OscAction::Unknown {
                code,
                data: data.to_string(),
            },
        };

        self.process_osc_action(action);
    }
}
```

### Prompt Region Tracking

The terminal emulator tracks prompt regions to support UI features:

```rust
pub struct PromptRegion {
    /// Line where the prompt started (in scrollback buffer)
    prompt_start_line: usize,

    /// Line where command output started
    output_start_line: Option<usize>,

    /// Line where command output ended
    output_end_line: Option<usize>,

    /// Raw command text (between prompt end and command start)
    command_text: Option<String>,

    /// Exit code (when command finishes)
    exit_code: Option<i32>,

    /// Command duration (if tracking)
    duration: Option<Duration>,
}

pub struct PromptTracker {
    /// All known prompt regions (in scrollback)
    regions: Vec<PromptRegion>,

    /// Current prompt (not yet complete)
    current: Option<PromptRegion>,

    /// Timestamp when command started (to calculate duration)
    command_start_time: Option<Instant>,
}

impl PromptTracker {
    pub fn on_prompt_start(&mut self, line: usize) {
        // Finalize previous region if any
        if let Some(mut current) = self.current.take() {
            self.regions.push(current);
        }
        self.current = Some(PromptRegion {
            prompt_start_line: line,
            output_start_line: None,
            output_end_line: None,
            command_text: None,
            exit_code: None,
            duration: None,
        });
    }

    pub fn on_command_start(&mut self) {
        self.command_start_time = Some(Instant::now());
    }

    pub fn on_command_output_start(&mut self, line: usize) {
        if let Some(ref mut current) = self.current {
            current.output_start_line = Some(line);
        }
    }

    pub fn on_command_finished(&mut self, line: usize, exit_code: i32) {
        if let Some(ref mut current) = self.current {
            current.output_end_line = Some(line);
            current.exit_code = Some(exit_code);
            if let Some(start) = self.command_start_time.take() {
                current.duration = Some(start.elapsed());
            }
        }
    }

    /// Get the prompt region containing this line
    pub fn region_at_line(&self, line: usize) -> Option<&PromptRegion> {
        self.regions.iter().find(|r| {
            line >= r.prompt_start_line
                && r.output_end_line
                    .map_or(true, |end| line <= end)
        })
    }
}
```

---

## Features Enabled by Shell Integration

### CWD Tracking (Level 1) - Context Engine

When OSC 7 is received:

1. Update session CWD
2. Context engine re-scans project type at the new CWD
3. Completion engine updates context-aware completions
4. Tab title updates
5. Emit `CwdChanged` event

```
User: cd ~/Projects/api-server
Shell: OSC 7 file://host/home/user/Projects/api-server
Wit:   CWD = /home/user/Projects/api-server
       Context: Node.js project (found package.json)
       Completions: npm scripts loaded
       Tab title: "api-server"
```

### Prompt Detection (Level 2) - UI Features

**Visual prompt markers:**

```
┌──────────────────────────────────────────────┐
│ > $ git status                               │  <- prompt marker (>)
│   On branch main                             │
│   Changes not staged:                        │
│     modified:   src/main.rs                  │
│                                              │
│ > $ cargo build                              │  <- prompt marker
│   Compiling wit-term v0.1.0                  │
│   Finished dev target in 2.3s               │
│                                              │
│ > $ _                                        │  <- current prompt
└──────────────────────────────────────────────┘
```

**Click-to-select output:**

- Click on an output region - select the entire output of that command
- Ctrl+Click - copy output to clipboard
- Right-click - context menu: "Copy output", "Re-run command", "Copy command"

**Scroll-to-prompt navigation:**

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+Up` | Scroll up to previous prompt |
| `Ctrl+Shift+Down` | Scroll down to next prompt |

### Command Tracking (Level 3) - Rich Command UI

**Command duration display:**

```
> $ cargo build                                    12.3s
  Compiling wit-term v0.1.0
  Finished dev target in 12.3s

> $ cargo test                               Y    3.1s
  running 42 tests
  test result: ok. 42 passed

> $ cargo publish                            X 1   0.5s
  error: failed to verify package
```

| Element | Description |
|---|---|
| Duration badge | Shown at the right side of the prompt line, only when > 1s |
| Exit code Y | Exit code 0 - success |
| Exit code X N | Exit code != 0 - error, shows code |

**Re-run last command:**

- Button or shortcut `Ctrl+Shift+R`: re-type the last command
- Hover over prompt - show "Re-run" button

**History enrichment:**

HistorySource receives additional data:

```rust
pub struct EnrichedHistoryEntry {
    command: String,
    timestamp: SystemTime,
    cwd: PathBuf,
    exit_code: i32,       // From Level 3
    duration: Duration,   // From Level 3
}
```

Commands with exit_code != 0 are penalized in frecency ranking.

---

## Installation

### Auto-inject (Recommended)

Wit automatically injects the integration script when creating a new session:

1. Wit sets environment variable `WIT_TERMINAL=1` before spawning the shell
2. Wit places integration scripts at known locations:
   - `~/.config/wit/shell/wit-integration.bash`
   - `~/.config/wit/shell/wit-integration.zsh`
   - `~/.config/wit/shell/wit-integration.fish`
   - `~/.config/wit/shell/wit-integration.ps1`
3. Wit adds arguments when spawning the shell to source the script:

| Shell | Method |
|---|---|
| Bash | `--rcfile` or inject via `$BASH_ENV` |
| Zsh | Prepend to `$ZDOTDIR/.zshrc` via temp file, or `$ZDOTDIR` override |
| Fish | Set `$XDG_CONFIG_HOME/fish/conf.d/wit-integration.fish` |
| PowerShell | `-NoExit -File wit-integration.ps1` |

**Safer approach for bash/zsh:**

Instead of overriding rcfile, Wit injects by setting environment variable `WIT_SHELL_INTEGRATION` pointing to the script path, then adds to the end of the user's rcfile (only once if not already present):

```bash
# Add to end of ~/.bashrc (one time only)
[ -n "$WIT_SHELL_INTEGRATION" ] && source "$WIT_SHELL_INTEGRATION"
```

### Manual Installation

User manually adds to shell config:

**Bash (~/.bashrc):**

```bash
if [ -n "$WIT_TERMINAL" ]; then
    source ~/.config/wit/shell/wit-integration.bash
fi
```

**Zsh (~/.zshrc):**

```zsh
if [[ -n "$WIT_TERMINAL" ]]; then
    source ~/.config/wit/shell/wit-integration.zsh
fi
```

**Fish (~/.config/fish/config.fish):**

```fish
if set -q WIT_TERMINAL
    source ~/.config/wit/shell/wit-integration.fish
end
```

**PowerShell ($PROFILE):**

```powershell
if ($env:WIT_TERMINAL) {
    . "$HOME/.config/wit/shell/wit-integration.ps1"
}
```

### Installation Detection

When creating a new session, Wit checks integration status:

1. Wait for the first prompt (timeout 3 seconds)
2. If OSC 133;A is received - integration is working (Level 2+)
3. If OSC 7 is received but not OSC 133 - Level 1
4. If nothing is received - Level 0

If Level 0 and auto-inject is enabled, Wit may show a small notification:

> "Shell integration not detected. [Setup] [Don't show again]"

---

## Graceful Degradation

### Principles

Wit **never breaks** when shell integration is missing or malfunctioning. Every feature that depends on shell integration has a fallback:

| Feature | With integration | Without integration |
|---|---|---|
| CWD | Real-time from OSC 7 | CWD at session start (static) |
| Tab title | Dynamic from CWD | Shell name |
| Context | Continuously detects project | Detects once at start |
| Prompt markers | Visual markers | Not shown |
| Command duration | Accurate | Not shown |
| Exit code | Per-command | Not shown |
| History | Enriched (exit code, duration) | Basic (command, timestamp) |

### Error Handling

| Scenario | Behavior |
|---|---|
| OSC sequence malformed | Ignore, log debug warning |
| Unexpected OSC order (B before A) | Reset state, start fresh |
| Multiple A without B/D | Treat as prompt re-draw |
| Very large OSC payload (> 4KB) | Truncate, log warning |
| Integration script fails to source | Shell starts normally, Level 0 |
| User's .bashrc overrides PROMPT_COMMAND | Integration hooks lost, degrade to Level 0 |

### Re-detection

If integration is lost mid-session (e.g., user re-sources .bashrc):

- Wit does not crash, just stops receiving OSC events
- If OSC events resume - Wit automatically re-establishes tracking
- No session restart needed

---

## Security Considerations

### Injected Code

1. **Minimal footprint:** Integration scripts contain only necessary functions (< 100 lines per shell)
2. **No external calls:** Scripts do not call curl, wget, or any network calls
3. **No file writes:** Scripts do not write files (only output OSC sequences)
4. **Read-only env:** Scripts only read `$PWD`, `$?`, `hostname` - do not modify user environment (aside from hook registration)
5. **Guard clauses:** Check `$WIT_TERMINAL` before doing anything - do not affect shell when running outside Wit
6. **Idempotent:** Sourcing multiple times does not cause errors (guard `__WIT_INTEGRATION_ACTIVE`)

### Auditing

- Integration scripts are shipped as plain text, easy to read
- Each script is < 100 lines, easy to audit
- Source code is open; the community can review
- Hash verification: Wit verifies SHA256 hash of scripts before injecting

### OSC Security

- Wit does not execute arbitrary commands from OSC sequences
- OSC data is only used to update internal state (CWD, prompt tracking)
- Malicious OSC sequences (e.g., from `cat malicious-file`) can only affect display, cannot execute code

### Sandbox Escapes

- Integration scripts run inside the shell process - same security context as the shell
- No privilege escalation: scripts run with user permissions
- Wit terminal emulator does not expose any additional permissions beyond the PTY

---

## Configuration

```toml
[shell_integration]
# Enable/disable auto-injection
auto_inject = true

# Integration level to use
# "auto" = detect highest supported level
# "cwd" = Level 1 only
# "prompt" = Level 1 + 2
# "full" = Level 1 + 2 + 3
# "none" = Level 0 (disabled)
level = "auto"

# Show setup notification when integration not detected
show_setup_notification = true

# Path to custom integration scripts (override bundled)
# custom_scripts_dir = "~/.config/wit/shell"
```

---

## Testing Strategy

### Unit Tests

- OSC parser: test each sequence type with valid/invalid data
- PromptTracker: test state transitions (A - B - C - D cycle)
- CWD parsing: test file:// URL parsing with edge cases (spaces, unicode, Windows paths)

### Integration Tests

- Bash integration: spawn bash, source script, type commands, verify OSC output
- Zsh integration: same
- Fish integration: same
- PowerShell integration: same (Windows)
- Graceful degradation: remove integration mid-session, verify no crash

### Manual Testing Checklist

- [ ] Bash: source script, `cd` multiple times, verify CWD tracking
- [ ] Zsh: source script, run commands, verify prompt markers
- [ ] Fish: source script, verify all 3 levels
- [ ] PowerShell: source script, verify on Windows
- [ ] Nested shells: bash - zsh - exit - verify tracking still works
- [ ] tmux/screen: integration works through multiplexer
- [ ] SSH: integration on remote session (requires manual install on remote)
- [ ] Non-interactive: `bash -c "command"` - verify integration does not inject

---

## Known Limitations and Future Work

1. **v1.0:** Support Bash, Zsh, Fish, PowerShell. Auto-inject for Bash and Zsh.
2. **v1.1:** Auto-inject for Fish and PowerShell. Tab completion leveraging prompt regions.
3. **v1.2:** Custom OSC for Wit-specific features (inline git status, etc.)
4. **Future:** Nushell, Elvish support
5. **Future:** Remote shell integration (SSH sessions)
6. **Future:** Integration with tmux/screen (passthrough OSC sequences)
7. **Future:** Command palette populated from tracked commands
