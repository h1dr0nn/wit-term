# Completion Engine

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Overview

The Completion Engine is the module responsible for suggesting commands to the user as they type in the terminal. The engine operates entirely locally, without using AI or cloud services - everything is based on **static rules**, **command history**, **file system**, and **project context**.

The engine runs on the Rust core (background thread) and communicates with the frontend via Tauri IPC. The frontend only displays results; it does not participate in matching or ranking logic.

---

## Completion Lifecycle

Each completion request goes through the following pipeline:

```
User keystroke
    |
    v
┌─────────┐     ┌─────────┐     ┌──────────────┐     ┌─────────┐
│ Trigger │────►│  Parse  │────►│ Query Sources│────►│  Match  │
│  Check  │     │  Input  │     │  (parallel)  │     │ & Score │
└─────────┘     └─────────┘     └──────────────┘     └─────────┘
                                                          |
                                                          v
                                                     ┌─────────┐     ┌─────────┐
                                                     │  Rank   │────►│ Display │
                                                     │& Dedupe │     │  (UI)   │
                                                     └─────────┘     └─────────┘
                                                                          |
                                                                          v
                                                                     ┌─────────┐
                                                                     │ Accept  │
                                                                     │ / Cycle │
                                                                     └─────────┘
```

### Step 1: Trigger Check

Completion is triggered by one of the following conditions:

| Trigger | Type | Description |
|---|---|---|
| `Tab` | Explicit | User actively requests completion |
| `Ctrl+Space` | Explicit | Open completion popup (IDE-style) |
| Typing a character | Automatic | Triggered after typing a certain number of characters |
| `/` or `\` | Automatic | Path separator - triggers path completion |
| `-` or `--` | Automatic | Flag prefix - triggers flag completion |
| Space after command | Automatic | Triggers subcommand/argument completion |

**Automatic trigger rules:**

- The completion popup automatically appears when the user has typed at least **2 characters** after the command position (to avoid noise)
- Path completion triggers immediately after `/` or `\` (no need to wait for 2 characters)
- Flag completion triggers immediately after `--` or `-`
- Automatic triggers have a **debounce of 50ms** - if the user types more within 50ms, the previous request is cancelled

**Explicit trigger rules:**

- `Tab` when the completion popup is open - cycle through items
- `Tab` when the popup is closed - open the popup and select the first item
- `Ctrl+Space` always opens the popup, even when input is empty (show all available commands)

### Step 2: Parse Input

The engine parses the current command line to determine the **cursor position** within the command structure:

```rust
pub struct ParsedInput {
    /// Full raw input string
    raw: String,

    /// Parsed tokens
    tokens: Vec<Token>,

    /// Which token the cursor is currently at
    cursor_position: CursorPosition,

    /// Prefix text at cursor (the part the user has already typed for the current token)
    prefix: String,
}

pub enum Token {
    Command(String),          // git
    Subcommand(String),       // commit
    Flag(FlagToken),          // --message, -m
    FlagValue(String),        // value after flag
    Argument(String),         // positional argument
    Pipe,                     // |
    Redirect(String),         // >, >>, <
    Separator,                // ;, &&, ||
}

pub enum CursorPosition {
    /// Typing a command name (first position)
    Command,
    /// Typing a subcommand (after recognized command)
    Subcommand { command: String },
    /// Typing a flag (after - or --)
    Flag { command: String, subcommand: Option<String> },
    /// Typing a value for a flag
    FlagValue { command: String, flag: String },
    /// Typing a positional argument
    Argument { command: String, subcommand: Option<String>, arg_index: usize },
}
```

**Parsing rules:**

1. Tokenize by whitespace, but respect quotes (`"..."`, `'...'`)
2. The first token is always `Command` (unless there is an environment variable prefix)
3. Tokens starting with `-` are `Flag`
4. Pipe `|` resets context - the token after pipe is a new `Command`
5. `;`, `&&`, `||` also reset context
6. Handle escaped characters (`\ ` is a space within an argument, not a separator)

### Step 3: Query Sources

Based on `ParsedInput`, the engine queries all completion sources **in parallel**. Each source returns a list of `CompletionItem`:

```rust
pub struct CompletionItem {
    /// Text to insert into the command line
    insert_text: String,

    /// Text displayed in the popup (may differ from insert_text)
    display_text: String,

    /// Short description (shown on the right side of the popup)
    description: Option<String>,

    /// Completion type (used to display icon)
    kind: CompletionKind,

    /// Source that produced this item
    source: SourceId,

    /// Score from the source (before aggregate ranking)
    source_score: f64,

    /// Additional metadata
    metadata: CompletionMetadata,
}

pub enum CompletionKind {
    Command,
    Subcommand,
    Flag,
    FlagValue,
    FilePath,
    DirectoryPath,
    GitBranch,
    GitTag,
    NpmScript,
    DockerImage,
    EnvironmentVariable,
    Custom(String),
}

pub struct CompletionMetadata {
    /// Whether this flag is deprecated
    deprecated: bool,
    /// Whether this flag requires an argument
    requires_value: bool,
    /// Mutually exclusive with which flags
    conflicts_with: Vec<String>,
    /// How many times it has been used in history
    usage_count: Option<u64>,
    /// Last time used
    last_used: Option<SystemTime>,
}
```

For details on each source, see [Completion Sources](#completion-sources).

### Step 4: Match & Score

For each `CompletionItem` from all sources, the engine runs **fuzzy matching** between the `prefix` (what the user has typed) and `insert_text`:

```rust
pub struct MatchResult {
    /// Original item
    item: CompletionItem,
    /// Fuzzy match score (0.0 - 1.0)
    match_score: f64,
    /// Character positions that matched (for highlighting in UI)
    matched_positions: Vec<usize>,
}
```

For details on the fuzzy matching algorithm, see [Fuzzy Matching](#fuzzy-matching-algorithm).

### Step 5: Rank & Deduplicate

Combine scores from multiple sources into a **final score**:

```
final_score = match_score * 0.4
            + frecency_score * 0.3
            + context_score * 0.2
            + source_priority * 0.1
```

**Deduplication:**

- If multiple sources return the same `insert_text`, keep the item with the highest `final_score` but merge metadata (e.g., merge `description` from the static source with `usage_count` from the history source)

**Sort:** Descending by `final_score`. If equal, sort alphabetically by `insert_text`.

### Step 6: Display

The engine sends the top **N items** to the frontend via a Tauri event. The frontend displays completions in 2 modes:

1. **Inline hint**: The first item appears as ghost text after the cursor (dimmed color)
2. **Popup**: A list of items appears below the cursor

Both modes can operate simultaneously: the inline hint shows item #1, the popup shows the full list.

### Step 7: Accept / Cycle

| Action | Behavior |
|---|---|
| `Tab` (popup open) | Select the highlighted item |
| `Tab` (popup closed, inline hint visible) | Accept the inline hint |
| `Enter` | Accept the highlighted item and execute the command |
| Up/Down arrows | Navigate the popup list |
| `Escape` | Close popup, clear inline hint |
| Continue typing | Re-filter the popup, update inline hint |
| `Tab Tab` (double tab) | Show all completions (bash-style behavior) |

---

## Completion Sources

### StaticSource

**Purpose:** Provide completions for known commands, flags, and subcommands from data files shipped with the app or added by the user.

**Data format:** See spec [Completion Data Format](./completion-data-format.md).

**Loading:**

```rust
pub struct StaticSource {
    /// Map command name -> command definition
    commands: HashMap<String, CommandDef>,
}

impl StaticSource {
    /// Load all completion files from a directory
    pub fn load_from_dir(dir: &Path) -> Result<Self>;

    /// Reload a specific file (when the file changes)
    pub fn reload_file(&mut self, path: &Path) -> Result<()>;
}
```

- Load all `.toml` files from `~/.config/wit/completions/` and bundled completions at startup
- Watch file changes for hot-reload (no app restart needed)
- User completions override bundled completions with the same command name

**Query behavior:**

- `CursorPosition::Command` - return all known command names
- `CursorPosition::Subcommand` - return subcommands of the corresponding command
- `CursorPosition::Flag` - return available flags (excluding already-used flags, respecting mutually exclusive groups)
- `CursorPosition::FlagValue` - return enum values if the flag has defined values
- `CursorPosition::Argument` - return argument hints if defined

### HistorySource

**Purpose:** Suggest commands the user has typed before, prioritized by **frecency** (frequency x recency).

**Storage:**

```rust
pub struct HistoryEntry {
    /// Full command line
    command: String,
    /// Execution timestamp
    timestamp: SystemTime,
    /// CWD when the command was run
    cwd: PathBuf,
    /// Exit code (if known, requires shell integration)
    exit_code: Option<i32>,
    /// Duration (if known)
    duration: Option<Duration>,
}
```

- Stored in a SQLite database at `~/.local/share/wit/history.db`
- Limited to **50,000 entries** (oldest deleted when exceeded)
- Deduplicate: same command + same CWD - update timestamp, increment count

**Frecency algorithm:**

```
frecency(entry) = frequency_score * recency_weight

frequency_score = ln(1 + count)

recency_weight = {
    used within 1 hour:   1.0
    used within 1 day:    0.8
    used within 1 week:   0.6
    used within 1 month:  0.4
    older:                0.2
}
```

**Query behavior:**

- Match prefix against the beginning of each command
- Also supports fuzzy match (but exact prefix match gets a higher bonus)
- If the context engine indicates the current CWD - boost entries with the same CWD
- Entries with exit code != 0 are penalized (multiplied by 0.5) - commands that frequently fail are suggested less often

### PathSource

**Purpose:** Completion for file and directory paths.

**Triggered when:**

- The current token contains `/` or `\`
- The current token starts with `~`, `.`, or `..`
- Currently at an argument position where the command expects it (e.g., `cat <file>`, `cd <dir>`)

**Implementation:**

```rust
pub struct PathSource;

impl PathSource {
    /// List directory entries matching prefix
    pub async fn complete_path(
        base_dir: &Path,
        prefix: &str,
        kind: PathKind,  // FilesOnly, DirsOnly, Both
    ) -> Vec<CompletionItem>;
}
```

- Read directory listing async (tokio::fs)
- Respect `.gitignore` if inside a git repo (using the `ignore` crate)
- Hidden files (starting with `.`) are only shown when the prefix starts with `.`
- Directories have a trailing `/` in `insert_text`
- Sort: directories before files, then alphabetical
- Limited to **200 entries** per directory (to avoid listing too many items in node_modules)
- Skip symlink loops

### ContextSource

**Purpose:** Provide completions based on the current project context. This is the key differentiator of Wit compared to regular terminals.

**How it works:** ContextSource queries the Context Engine to determine what the current project uses, then provides appropriate completions.

| Context | Completions |
|---|---|
| Git repo | Branch names, tag names, remote names, changed files |
| Node.js project | `npm run <script>` scripts from package.json |
| Cargo project | `cargo <target>` targets, features |
| Docker project | Service names from docker-compose.yml, image names |
| Python venv | pip package names, pytest markers |
| Makefile present | Make targets |

**Implementation:**

```rust
pub struct ContextSource {
    context_engine: Arc<ContextEngine>,
}

impl ContextSource {
    /// Get completions based on the current context
    pub async fn complete(
        &self,
        parsed: &ParsedInput,
        session_context: &SessionContext,
    ) -> Vec<CompletionItem>;
}
```

**Specific example for git:**

```
User types: git checkout <Tab>
ParsedInput: Command("git"), Subcommand("checkout"), Argument(index=0)

ContextSource detects:
  - Context has a git repo
  - Command is "git checkout"
  - Argument position 0 = branch/file

ContextSource will:
  1. Call `git branch --list` (or read .git/refs/heads/)
  2. Call `git tag --list` (or read .git/refs/tags/)
  3. Return branches + tags as CompletionItems
  4. The current branch is excluded (already checked out)
  5. Recently used branches get a score boost
```

**Caching:**

- Context data (git branches, npm scripts, etc.) is cached for **5 seconds**
- Cache is invalidated when CWD changes
- Cache is invalidated when the file system watcher detects related changes (e.g., `.git/refs/` changes - invalidate git branch cache)

### DynamicSource

**Purpose:** Use the shell's built-in completion system as a fallback.

**How it works:**

- Send a completion request to the running shell process
- Parse the returned results into `CompletionItem`

**Supported shells:**

| Shell | Mechanism |
|---|---|
| Bash | `compgen -W "$(complete -p <cmd>)" -- <prefix>` |
| Zsh | `_main_complete` internal, or capture `compadd` output |
| Fish | `complete --do-complete '<input>'` |
| PowerShell | `TabExpansion2 -inputScript '<input>' -cursorColumn <pos>` |

**Priority:** DynamicSource has the lowest priority. Only used when:

- There is no static completion for the command
- Or the user configures `prefer_shell_completions = true`

**Performance:**

- Timeout **500ms** - if shell completion does not return in time, skip it
- Runs on a separate thread, does not block other sources
- Cache results for the same command + prefix for **2 seconds**

---

## Fuzzy Matching Algorithm

Wit uses a fuzzy matching algorithm based on the approaches of `fzf` and `VS Code`, optimized for command-line completion.

### Input

- `pattern`: The string the user has typed (prefix at cursor)
- `target`: The candidate string (`insert_text` of `CompletionItem`)

### Algorithm

```rust
pub fn fuzzy_match(pattern: &str, target: &str) -> Option<MatchResult> {
    // Returns None if no match (a character in pattern is not found
    // in target in order)

    let mut score: f64 = 0.0;
    let mut pattern_idx = 0;
    let mut prev_match_idx: Option<usize> = None;
    let mut consecutive_count = 0;
    let mut matched_positions = Vec::new();

    for (target_idx, target_char) in target.char_indices() {
        if pattern_idx >= pattern.len() {
            break;
        }

        let pattern_char = pattern.chars().nth(pattern_idx).unwrap();

        if chars_match(pattern_char, target_char) {
            // Base score for each match
            score += 1.0;

            // Consecutive bonus: consecutive matching characters get a bonus
            if let Some(prev) = prev_match_idx {
                if target_idx == prev + 1 {
                    consecutive_count += 1;
                    score += consecutive_count as f64 * 2.0;
                } else {
                    consecutive_count = 0;
                    // Gap penalty: distance between 2 matches
                    let gap = target_idx - prev - 1;
                    score -= gap as f64 * 0.5;
                }
            }

            // Prefix bonus: match at the beginning of target
            if target_idx == 0 {
                score += 10.0;
            }

            // Word boundary bonus: match after _, -, /, space
            if target_idx > 0 {
                let prev_char = target.chars().nth(target_idx - 1).unwrap();
                if is_word_boundary(prev_char) {
                    score += 5.0;
                }
            }

            // Case exact match bonus
            if pattern_char == target_char {
                score += 0.5;
            }

            matched_positions.push(target_idx);
            prev_match_idx = Some(target_idx);
            pattern_idx += 1;
        }
    }

    // All characters in the pattern must match
    if pattern_idx < pattern.len() {
        return None;
    }

    // Normalize score by target length (shorter = better)
    let length_penalty = (target.len() as f64 - pattern.len() as f64) * 0.1;
    score -= length_penalty;

    // Normalize to 0.0 - 1.0
    let max_possible = pattern.len() as f64 * 15.0; // rough max
    let normalized = (score / max_possible).clamp(0.0, 1.0);

    Some(MatchResult {
        match_score: normalized,
        matched_positions,
    })
}

fn chars_match(a: char, b: char) -> bool {
    // Case-insensitive comparison
    a.to_lowercase().eq(b.to_lowercase())
}

fn is_word_boundary(c: char) -> bool {
    matches!(c, '_' | '-' | '/' | '\\' | '.' | ' ')
}
```

### Scoring Breakdown

| Factor | Bonus/Penalty | Explanation |
|---|---|---|
| Base match | +1.0 per char | Each character that matches |
| Consecutive | +2.0 x streak | Consecutive matches (increasing) |
| Gap | -0.5 x distance | Distance between matches |
| Prefix | +10.0 | Match at the beginning of target |
| Word boundary | +5.0 | Match after `_`, `-`, `/`, `.` |
| Exact case | +0.5 | Exact case match (on top of case-insensitive match) |
| Length penalty | -0.1 x excess | Longer target than pattern gets a slight penalty |

### Case Sensitivity

- Default: **smart case** - case-insensitive unless the pattern contains uppercase
- If the pattern contains at least one uppercase letter - case-sensitive matching
- Examples:
  - `com` matches "commit", "COMMIT", "Compare" - case-insensitive
  - `Com` only matches "Compare", "Commit" - case-sensitive

---

## Ranking Algorithm

Final ranking combines scores from multiple sources:

### Score Components

```rust
pub struct RankingScore {
    /// Fuzzy match quality (0.0 - 1.0)
    match_score: f64,

    /// Frecency from history (0.0 - 1.0)
    frecency_score: f64,

    /// Context relevance (0.0 - 1.0)
    context_score: f64,

    /// Source priority (0.0 - 1.0)
    source_priority: f64,
}

impl RankingScore {
    pub fn final_score(&self) -> f64 {
        self.match_score * 0.4
            + self.frecency_score * 0.3
            + self.context_score * 0.2
            + self.source_priority * 0.1
    }
}
```

### Match Score (weight: 0.4)

Direct output from the fuzzy matching algorithm. This is the most important factor - the completion must match the input.

### Frecency Score (weight: 0.3)

From HistorySource. Commands the user frequently uses recently are prioritized.

- If the item is not in history - `frecency_score = 0.0`
- Normalize: `frecency / max_frecency_in_results`

### Context Score (weight: 0.2)

Based on the current context:

| Condition | Score |
|---|---|
| Item from ContextSource and context is active | 1.0 |
| Item matches current CWD | 0.8 |
| Item from StaticSource, command belongs to detected context | 0.5 |
| No context relevance | 0.0 |

### Source Priority (weight: 0.1)

| Source | Priority |
|---|---|
| ContextSource | 1.0 |
| StaticSource | 0.8 |
| HistorySource | 0.6 |
| PathSource | 0.5 |
| DynamicSource | 0.3 |

### Tie-breaking

When `final_score` is equal:

1. Exact prefix match before fuzzy match
2. Shorter before longer
3. Alphabetical order

---

## Performance

### Async Architecture

```
┌─────────┐     ┌──────────────────────────────────────┐
│  Main   │     │       Completion Thread Pool          │
│ Thread  │     │                                       │
│         │     │  ┌───────────┐  ┌───────────┐        │
│  Input ─┼────►│  │  Static   │  │  History  │        │
│  Event  │     │  │  Query    │  │  Query    │        │
│         │     │  └─────┬─────┘  └─────┬─────┘        │
│         │     │        │              │               │
│         │     │  ┌─────┴─────┐  ┌─────┴─────┐        │
│         │     │  │   Path    │  │  Context  │        │
│         │     │  │   Query   │  │  Query    │        │
│         │     │  └─────┬─────┘  └─────┬─────┘        │
│         │     │        │              │               │
│         │     │        v              v               │
│         │     │  ┌───────────────────────────┐        │
│  Result◄┼─────│  │    Merge + Rank + Limit   │        │
│  Event  │     │  └───────────────────────────┘        │
│         │     │                                       │
└─────────┘     └──────────────────────────────────────┘
```

**Rules:**

1. Completion runs on a **dedicated thread** (or tokio task), not blocking the main thread or PTY I/O
2. All sources are queried **in parallel** (join all futures)
3. When the user types another character while a query is in progress - **cancel the old request** (using `CancellationToken` or generation counter)
4. Debounce **50ms** for automatic triggers (explicit triggers are processed immediately)

### Cancellation

```rust
pub struct CompletionRequest {
    /// Unique ID, incrementing
    id: u64,
    /// Input at the time of request
    input: ParsedInput,
    /// Token for cancellation
    cancel: CancellationToken,
}

// When a new request arrives, cancel the old one
fn on_input_changed(&mut self, new_input: ParsedInput) {
    // Cancel pending request
    if let Some(prev) = self.pending_request.take() {
        prev.cancel.cancel();
    }

    // Create a new request
    let request = CompletionRequest {
        id: self.next_id(),
        input: new_input,
        cancel: CancellationToken::new(),
    };

    self.pending_request = Some(request.clone());
    self.spawn_completion(request);
}
```

### Performance Targets

| Metric | Target |
|---|---|
| Time from keystroke to first result | < 16ms (1 frame at 60fps) |
| Time to complete full ranking | < 50ms |
| StaticSource lookup | < 1ms |
| HistorySource query | < 5ms |
| PathSource listing (typical dir) | < 10ms |
| ContextSource query | < 20ms |
| DynamicSource query | < 500ms (timeout) |
| Memory per completion dataset | < 10MB for 1000 commands |

### Caching Strategy

| Data | Cache duration | Invalidation |
|---|---|---|
| Static completions | Permanent (until file change) | File watcher |
| History frecency | 30 seconds | New command executed |
| Path listings | 2 seconds | Timer expiry |
| Context data (branches, etc.) | 5 seconds | CWD change, file watcher |
| DynamicSource results | 2 seconds | Timer expiry |

---

## Display Configuration

### Maximum Items

- **Inline hint:** 1 item (top-ranked)
- **Popup:** Maximum **12 items** visible, scrollable
- **Total computed:** Maximum **50 items** (to avoid computing too many)
- If there are more than 50 results - show an indicator "and N more..." at the bottom of the popup

### Popup Layout

```
┌─────────────────────────────────────────┐
│ commit    Create a new commit           │ <- highlighted
│ checkout  Switch branches or restore    │
│ cherry-pick Apply changes from comm     │
│ clean     Remove untracked files        │
│                                         │
│ ---- History ──────────────────────── │
│ commit -m "fix: bug"    2 min ago       │
│ checkout feature/auth   yesterday       │
│                                         │
│                              8 more...  │
└─────────────────────────────────────────┘
```

**Grouping:** Items may be grouped by source (Static, History, Context) with separator headers. Users can disable grouping in settings.

### Inline Hint Style

```
$ git com|mit --message "fix: update readme"
         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
         Ghost text (dimmed color, italic)
```

- Ghost text displays the remainder of the top completion
- Inline hints are only shown when match score > 0.5 (to avoid incorrect suggestions)
- Users can disable inline hints in settings

### Keyboard Navigation

| Key | Popup closed | Popup open |
|---|---|---|
| `Tab` | Accept inline hint / Open popup | Select highlighted item |
| `Shift+Tab` | - | Select previous item |
| Up arrow | History (shell) | Previous item in popup |
| Down arrow | - | Next item in popup |
| `Enter` | Execute command | Accept item + execute |
| `Escape` | - | Close popup |
| `Ctrl+Space` | Open popup | Close popup |

---

## Configuration

Users can customize completion behavior in `~/.config/wit/config.toml`:

```toml
[completion]
# Enable/disable automatic completion
auto_complete = true

# Debounce time for automatic triggers (ms)
debounce_ms = 50

# Minimum characters before automatic trigger
min_chars = 2

# Maximum items in popup
max_items = 12

# Show inline hint
inline_hints = true

# Minimum match score for inline hint (0.0 - 1.0)
inline_hint_threshold = 0.5

# Show description in popup
show_descriptions = true

# Grouping by source
group_by_source = false

# Prefer shell completions over static
prefer_shell_completions = false

# Custom completion directories
extra_completion_dirs = [
    "~/my-completions",
]

[completion.ranking]
# Weights for ranking (must sum to 1.0)
match_weight = 0.4
frecency_weight = 0.3
context_weight = 0.2
source_weight = 0.1

[completion.history]
# Maximum entries in history database
max_entries = 50000

# Penalize commands with non-zero exit code
penalize_failures = true
```

---

## Error Handling

| Scenario | Behavior |
|---|---|
| Source query fails (I/O error) | Log warning, skip source, return results from other sources |
| All sources fail | Do not show completion, do not show error to user |
| Timeout (DynamicSource) | Cancel request, return results from other sources |
| Corrupt history DB | Recreate DB, log error, continue |
| Invalid completion file | Skip file, log warning with file path |
| Parse error on input | Fall back to basic prefix matching |

---

## Testing Strategy

### Unit Tests

- Fuzzy matching: test each scoring factor (prefix, consecutive, gap, boundary)
- Input parsing: test tokenizer with edge cases (quotes, escapes, pipes)
- Ranking: test with known inputs, verify ordering

### Integration Tests

- Full pipeline: keystroke - parsed input - query - match - rank - result
- Multi-source: verify deduplication and merge behavior
- Cancellation: verify stale requests are cancelled properly

### Benchmarks

- Fuzzy match 10,000 candidates: target < 5ms
- Full completion pipeline: target < 50ms
- Memory usage with 1000 commands loaded: target < 10MB

---

## Dependencies

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime for parallel source queries |
| `rusqlite` | SQLite for history storage |
| `toml` | Parse completion data files |
| `ignore` | Respect .gitignore in path completion |
| `nucleo` or custom | Fuzzy matching (can use a crate or write custom) |

---

## Known Limitations and Future Work

1. **v0.1:** Only supports StaticSource + PathSource + HistorySource
2. **v0.2:** Add ContextSource (git branches, npm scripts)
3. **v0.3:** Add DynamicSource (shell completions)
4. **Future:** Snippet completions (e.g., `for` - expand into a for loop)
5. **Future:** Local completion telemetry (track accept rate to tune ranking)
