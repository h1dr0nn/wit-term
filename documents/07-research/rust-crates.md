# Rust Crates - Project Dependencies Research

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. PTY - Pseudo-Terminal

### portable-pty

| Field | Value |
|-------|-------|
| **Crate** | `portable-pty` |
| **Version** | ~0.8 |
| **Repository** | https://github.com/nickelpack/portable-pty |
| **License** | MIT |
| **Recommendation** | **Use** |

**Description:** Cross-platform PTY library, abstracting Unix PTY and Windows ConPTY. Originally extracted from WezTerm.

**Why use for Wit:**
- Unified API for both Unix and Windows
- Handles `openpty()`, `forkpty()`, ConPTY setup
- Proven in production (WezTerm)
- Manages child process lifecycle

**Trade-offs:**
- (+) No need to write platform-specific PTY code
- (+) Battle-tested through WezTerm
- (-) May need to fork/patch for custom behaviors
- (-) Some ConPTY edge cases may need workarounds

### nix

| Field | Value |
|-------|-------|
| **Crate** | `nix` |
| **Version** | ~0.29 |
| **Repository** | https://github.com/nix-rust/nix |
| **License** | MIT |
| **Recommendation** | **Consider** |

**Description:** Rust-friendly bindings for Unix/POSIX APIs (ioctl, termios, signals, etc.).

**Why needed:**
- Low-level PTY operations if bypassing `portable-pty` is necessary
- `ioctl(TIOCSWINSZ)` for window resize
- Signal handling (SIGCHLD, SIGWINCH)
- termios manipulation

**Trade-offs:**
- (+) Type-safe Unix API access
- (+) Comprehensive POSIX coverage
- (-) Unix only - needs conditional compilation
- (-) May not be needed if `portable-pty` is sufficient

### windows-rs

| Field | Value |
|-------|-------|
| **Crate** | `windows` |
| **Version** | ~0.58 |
| **Repository** | https://github.com/microsoft/windows-rs |
| **License** | MIT/Apache-2.0 |
| **Recommendation** | **Consider** |

**Description:** Official Microsoft Rust bindings for Windows APIs.

**Why needed:**
- Direct ConPTY API access if needed
- Windows-specific features (registry, shell integration)
- `CreatePseudoConsole`, `ResizePseudoConsole` bindings

**Trade-offs:**
- (+) Official, generated from Windows metadata
- (+) Complete API coverage
- (-) Large dependency (feature-gate carefully)
- (-) May not be needed if `portable-pty` handles everything

---

## 2. Terminal Parsing

### vte

| Field | Value |
|-------|-------|
| **Crate** | `vte` |
| **Version** | ~0.13 |
| **Repository** | https://github.com/alacritty/vte |
| **License** | Apache-2.0 / MIT |
| **Recommendation** | **Use** |

**Description:** VT parser based on Paul Williams' state machine model. Extracted from Alacritty.

**Why use:**
- Proven, high-performance VT parser
- Handles all standard ANSI/DEC escape sequences
- No-alloc design - zero-copy parsing
- Well-tested through Alacritty

**Trade-offs:**
- (+) Battle-tested, performant
- (+) Clean state machine design
- (+) Handles partial sequences across read boundaries
- (-) Callback-based API (trait `Perform`) - can be awkward
- (-) Does not parse semantics - only tokenizes, Wit must interpret

### Should we use vte or build a custom parser?

| Criteria | vte | Custom |
|----------|-----|--------|
| **Development time** | Immediate | Weeks/months |
| **Correctness** | Proven | Need extensive testing |
| **Performance** | Optimized | Need to optimize |
| **Flexibility** | Limited by API | Full control |
| **Maintenance** | Maintained by Alacritty team | Self-maintained |
| **Custom extensions** | Need to fork/extend | Easy to add |

> **Recommendation:** Start with `vte`. If custom extensions are needed later (e.g., Kitty keyboard protocol parsing), consider forking or wrapping.

### strip-ansi-escapes

| Field | Value |
|-------|-------|
| **Crate** | `strip-ansi-escapes` |
| **Version** | ~0.2 |
| **License** | Apache-2.0 / MIT |
| **Recommendation** | **Skip** |

**Description:** Strip ANSI escape sequences from text.

**Why skip:** Wit needs to INTERPRET escape sequences, not strip them. This crate is only useful for logging/testing.

---

## 3. Async Runtime

### tokio vs async-std vs smol

| Criteria | tokio | async-std | smol |
|----------|-------|-----------|------|
| **Ecosystem** | Largest | Medium | Small |
| **Maturity** | Most mature | Mature | Newer |
| **Performance** | Excellent | Good | Good |
| **Binary size** | Larger | Medium | Smallest |
| **Tauri compat** | Yes (Tauri uses Tokio) | Partial | Partial |
| **Features** | Comprehensive | std-like API | Minimal |

**Recommendation: `tokio`**

**Reasoning:**
- Tauri v2 already uses Tokio internally - adding another runtime creates conflicts
- Largest ecosystem - most async crates support Tokio
- Best I/O performance for PTY read/write
- Feature-gated - only include what Wit needs (`rt`, `io-util`, `sync`, `macros`)

```toml
[dependencies]
tokio = { version = "1", features = ["rt", "io-util", "sync", "macros", "process"] }
```

---

## 4. Serialization

### serde

| Field | Value |
|-------|-------|
| **Crate** | `serde` |
| **Version** | ~1.0 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Use** |

**Description:** De facto standard serialization framework for Rust.

**Wit uses:** Config files, IPC messages, state persistence, plugin communication.

### toml

| Field | Value |
|-------|-------|
| **Crate** | `toml` |
| **Version** | ~0.8 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Use** |

**Description:** TOML parser/serializer.

**Wit uses:** Configuration files (`.wit/config.toml`, user settings).

**Why TOML instead of YAML/JSON:**
- Human-readable and writable (better than JSON)
- Less error-prone than YAML (no significant whitespace)
- Rust ecosystem standard (Cargo.toml)
- Good comment support

### serde_json

| Field | Value |
|-------|-------|
| **Crate** | `serde_json` |
| **Version** | ~1.0 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Use** |

**Wit uses:** Tauri IPC (commands/events use JSON), completion specs, plugin APIs.

---

## 5. File Watching

### notify

| Field | Value |
|-------|-------|
| **Crate** | `notify` |
| **Version** | ~7.0 |
| **Repository** | https://github.com/notify-rs/notify |
| **License** | CC0-1.0 / Artistic-2.0 |
| **Recommendation** | **Use** |

**Description:** Cross-platform filesystem notification library.

**Wit uses:**
- Context engine: watch `.git`, `package.json`, `Cargo.toml`, etc. for project detection
- Config hot-reload: watch config files for live updates
- File change detection for context-aware features

**Trade-offs:**
- (+) Cross-platform (inotify, FSEvents, ReadDirectoryChanges)
- (+) Debouncing support
- (+) Active maintenance
- (-) Some platforms have event coalescing quirks
- (-) Recursive watching can be expensive on large directories

**Recommendation:** Use `notify` with careful filtering - don't watch the entire filesystem, only relevant paths.

---

## 6. String Handling

### compact_str

| Field | Value |
|-------|-------|
| **Crate** | `compact_str` |
| **Version** | ~0.8 |
| **License** | MIT |
| **Recommendation** | **Consider** |

**Description:** Inline string storage - strings <= 24 bytes stored inline (no heap allocation).

**Wit use case:** Terminal cells contain mostly 1-4 byte characters. Inline storage eliminates heap alloc for the vast majority of cells.

**Trade-offs:**
- (+) Significant memory reduction for terminal grid (millions of cells)
- (+) Better cache locality
- (-) 24 bytes per string (vs 24 bytes for String, but no heap pointer indirection)
- (-) API differences from `String`

### smol_str

| Field | Value |
|-------|-------|
| **Crate** | `smol_str` |
| **Version** | ~0.3 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Consider** |

**Description:** Immutable small string - inline for <= 23 bytes, Arc-based sharing for larger.

**Trade-offs:**
- (+) Immutable - good for cells that rarely change
- (+) Clone is cheap (memcpy or Arc::clone)
- (-) Immutable - can't modify in place
- (-) 24 bytes overhead per string

> **Recommendation:** Benchmark both. For terminal cells, `compact_str` is likely better since cells DO change (when overwritten). Alternatively, consider just storing `char` + continuation flag if full grapheme cluster string support is not needed.

---

## 7. Error Handling

### thiserror

| Field | Value |
|-------|-------|
| **Crate** | `thiserror` |
| **Version** | ~2.0 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Use** |

**Description:** Derive macro for `std::error::Error` - define custom error types ergonomically.

**Wit uses:** Library-style error types for core modules (PTY errors, parser errors, config errors).

```rust
#[derive(Debug, thiserror::Error)]
pub enum PtyError {
    #[error("Failed to create PTY: {0}")]
    Creation(#[from] std::io::Error),
    #[error("Shell not found: {path}")]
    ShellNotFound { path: String },
}
```

### anyhow

| Field | Value |
|-------|-------|
| **Crate** | `anyhow` |
| **Version** | ~1.0 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Use** |

**Description:** Flexible error handling for applications - `anyhow::Result`, `context()`.

**Wit uses:** Application-level code, Tauri commands, scripts. Where typed errors are not needed.

**Strategy:** `thiserror` for library/core code, `anyhow` for application/glue code.

---

## 8. Logging

### tracing vs log + env_logger

| Criteria | tracing | log + env_logger |
|----------|---------|------------------|
| **Structured logging** | Native | Text only |
| **Spans** | Yes (trace function execution) | No |
| **Async support** | Yes (instrument attribute) | Partial |
| **Performance** | Excellent (compile-time filtering) | Good |
| **Ecosystem** | Growing (tracing-subscriber, tracing-appender) | Mature |
| **Complexity** | Medium | Simple |
| **Tauri compat** | Yes (Tauri uses tracing) | Partial |

**Recommendation: `tracing`**

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**Reasoning:**
- Tauri v2 uses tracing internally
- Structured logging essential for debugging terminal emulator
- Spans help trace PTY read -> parse -> render pipeline
- `#[instrument]` attribute very useful

---

## 9. CLI

### clap

| Field | Value |
|-------|-------|
| **Crate** | `clap` |
| **Version** | ~4.5 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Use** |

**Description:** Command-line argument parser.

**Wit uses:** `wit` binary CLI arguments:
```
wit                     # Launch GUI
wit --config <path>     # Custom config
wit --working-dir <dir> # Start in directory
wit -e <command>        # Execute command
wit --version           # Version info
```

**Feature flags:** Use `derive` feature for ergonomic API.

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

---

## 10. Testing

### criterion

| Field | Value |
|-------|-------|
| **Crate** | `criterion` |
| **Version** | ~0.5 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Use** |

**Description:** Statistical benchmarking framework.

**Wit uses:** Benchmark parser throughput, grid operations, rendering pipeline.

### proptest

| Field | Value |
|-------|-------|
| **Crate** | `proptest` |
| **Version** | ~1.5 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Use** |

**Description:** Property-based testing (like QuickCheck/Hypothesis).

**Wit uses:**
- Fuzz VT parser with random byte sequences
- Test terminal grid operations with random sequences of commands
- Verify Unicode handling with arbitrary strings

### insta

| Field | Value |
|-------|-------|
| **Crate** | `insta` |
| **Version** | ~1.40 |
| **License** | Apache-2.0 |
| **Recommendation** | **Use** |

**Description:** Snapshot testing - compare output against saved snapshots.

**Wit uses:**
- Snapshot terminal grid state after processing escape sequences
- Snapshot rendered output
- Config parsing results

### tempfile

| Field | Value |
|-------|-------|
| **Crate** | `tempfile` |
| **Version** | ~3.14 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Use** |

**Description:** Temporary files and directories that are automatically cleaned up.

**Wit uses:** Testing file operations, config file tests, PTY tests.

---

## 11. Concurrency

### crossbeam

| Field | Value |
|-------|-------|
| **Crate** | `crossbeam` |
| **Version** | ~0.8 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Consider** |

**Description:** Tools for concurrent programming - channels, deques, epoch-based reclamation.

**Wit use case:**
- `crossbeam-channel` - bounded/unbounded channels (faster than `std::sync::mpsc`)
- `crossbeam-utils` - scoped threads, CachePadded

**Trade-offs:**
- (+) Faster channels than stdlib
- (+) Better API (select!, bounded channels)
- (-) Tokio channels (`tokio::sync::mpsc`) may be sufficient for async code
- (-) Additional dependency

**Recommendation:** Use Tokio channels for async code. Use crossbeam if sync channels are needed (e.g., between parser thread and main thread).

### parking_lot

| Field | Value |
|-------|-------|
| **Crate** | `parking_lot` |
| **Version** | ~0.12 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Consider** |

**Description:** Faster Mutex, RwLock replacements.

**Trade-offs:**
- (+) Faster than `std::sync::Mutex` in contended scenarios
- (+) Smaller memory footprint
- (+) No poisoning (simpler API)
- (-) Std mutexes have improved significantly
- (-) Additional dependency

**Recommendation:** Start with `std::sync::Mutex`. Switch to `parking_lot` if profiling shows lock contention.

### dashmap

| Field | Value |
|-------|-------|
| **Crate** | `dashmap` |
| **Version** | ~6.1 |
| **License** | MIT |
| **Recommendation** | **Consider** |

**Description:** Concurrent HashMap (sharded locking).

**Wit use case:** Shared state between Tauri commands (e.g., active terminal sessions map).

**Trade-offs:**
- (+) Good concurrent read/write performance
- (+) Simple API (similar to HashMap)
- (-) Can cause deadlocks if references are held across operations
- (-) `RwLock<HashMap>` may be simpler and sufficient

---

## 12. Text/Unicode

### unicode-width

| Field | Value |
|-------|-------|
| **Crate** | `unicode-width` |
| **Version** | ~0.2 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Use** |

**Description:** Determine display width of Unicode characters (1 vs 2 cells).

**Wit uses:** Essential for terminal emulation - CJK characters, emoji width calculation.

### unicode-segmentation

| Field | Value |
|-------|-------|
| **Crate** | `unicode-segmentation` |
| **Version** | ~1.12 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Use** |

**Description:** Unicode grapheme cluster boundaries (UAX #29).

**Wit uses:** Splitting text into grapheme clusters for cell storage. Essential for combining characters, emoji sequences.

---

## 13. Path Handling

### dirs

| Field | Value |
|-------|-------|
| **Crate** | `dirs` |
| **Version** | ~5.0 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Use** |

**Description:** Platform-specific standard directories (home, config, data, cache).

**Wit uses:**
- `dirs::config_dir()` -> Wit config location
- `dirs::data_dir()` -> Wit data (history, sessions)
- `dirs::home_dir()` -> Home directory detection

### directories

| Field | Value |
|-------|-------|
| **Crate** | `directories` |
| **Version** | ~5.0 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Consider** |

**Description:** Like `dirs` but with `ProjectDirs` - application-specific directories.

```rust
let proj = ProjectDirs::from("dev", "wit", "wit-term").unwrap();
proj.config_dir()  // ~/.config/wit-term/ (Linux)
proj.data_dir()    // ~/.local/share/wit-term/ (Linux)
proj.cache_dir()   // ~/.cache/wit-term/ (Linux)
```

**Recommendation:** `directories` if app-specific paths are needed (likely). `dirs` if only standard dirs are needed.

---

## 14. Fuzzy Matching

### fuzzy-matcher

| Field | Value |
|-------|-------|
| **Crate** | `fuzzy-matcher` |
| **Version** | ~0.3 |
| **License** | MIT |
| **Recommendation** | **Consider** |

**Description:** Fuzzy string matching with scoring (Skim algorithm).

**Wit uses:** Command history search, file completion, command completion.

**Trade-offs:**
- (+) Simple API, proven (used in skim fuzzy finder)
- (+) Supports multiple matching algorithms
- (-) Not as fast as nucleo for large datasets

### nucleo

| Field | Value |
|-------|-------|
| **Crate** | `nucleo` |
| **Version** | ~0.5 |
| **Repository** | https://github.com/helix-editor/nucleo |
| **License** | MPL-2.0 |
| **Recommendation** | **Use** |

**Description:** High-performance fuzzy matcher from Helix editor. Multi-threaded, supports large datasets.

**Wit uses:** Primary fuzzy matching engine for completions, search, navigation.

**Trade-offs:**
- (+) Very fast - designed for interactive use
- (+) Multi-threaded matching
- (+) Good scoring algorithm
- (+) Used in production (Helix editor)
- (-) MPL-2.0 license (copyleft for file, not project)
- (-) Newer, smaller community

> **Recommendation:** `nucleo` for performance-critical fuzzy matching (completions). `fuzzy-matcher` as simpler fallback.

---

## 15. Git

### git2 vs Git CLI

| Criteria | git2 (libgit2) | Git CLI (`Command::new("git")`) |
|----------|---------------|--------------------------------|
| **Performance** | Faster for repeated ops | Process spawn overhead |
| **Features** | Subset of git | Full git features |
| **Binary size** | +3-5MB (libgit2) | No added size |
| **Dependencies** | C library (libgit2), OpenSSL | Requires git installed |
| **Compatibility** | May differ from git | IS git |
| **Complexity** | Complex API | Simple (parse stdout) |
| **Error handling** | Typed errors | Parse stderr |

### git2

| Field | Value |
|-------|-------|
| **Crate** | `git2` |
| **Version** | ~0.19 |
| **License** | MIT / Apache-2.0 |
| **Recommendation** | **Consider** |

**Wit use cases:**
- Detect git repository (context engine)
- Get current branch name
- Get file status (modified/staged/untracked)
- Get recent commits

**Trade-offs:**
- (+) No process spawning - faster for frequent queries
- (+) Structured data - no stdout parsing
- (-) Large dependency (libgit2 C library)
- (-) Build complexity (C compilation)
- (-) Feature gaps vs real git
- (-) Cross-compilation harder

### Git CLI approach

```rust
use std::process::Command;

fn git_branch() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
}
```

**Trade-offs:**
- (+) Always matches user's git behavior
- (+) No binary size increase
- (+) Simple implementation
- (-) Requires git installed
- (-) Process spawn overhead (~5ms per call)
- (-) Output parsing fragile

> **Recommendation:** Start with Git CLI - simpler, always correct. Cache results. Move to `git2` if profiling shows Git CLI is a bottleneck. Context engine queries git infrequently enough that CLI overhead is acceptable.

---

## 16. Summary Table

| Category | Crate | Recommendation | Priority |
|----------|-------|---------------|----------|
| **PTY** | `portable-pty` | Use | P0 (core) |
| **PTY (Unix)** | `nix` | Consider | P1 |
| **PTY (Windows)** | `windows` | Consider | P1 |
| **Parsing** | `vte` | Use | P0 (core) |
| **Async** | `tokio` | Use | P0 (core) |
| **Serialization** | `serde` | Use | P0 (core) |
| **Config format** | `toml` | Use | P0 (core) |
| **IPC format** | `serde_json` | Use | P0 (core) |
| **File watching** | `notify` | Use | P1 (context) |
| **Strings** | `compact_str` | Consider | P2 (optimize) |
| **Errors (lib)** | `thiserror` | Use | P0 (core) |
| **Errors (app)** | `anyhow` | Use | P0 (core) |
| **Logging** | `tracing` | Use | P0 (core) |
| **CLI** | `clap` | Use | P1 |
| **Benchmarks** | `criterion` | Use | P1 (testing) |
| **Property test** | `proptest` | Use | P1 (testing) |
| **Snapshot test** | `insta` | Use | P1 (testing) |
| **Temp files** | `tempfile` | Use | P1 (testing) |
| **Channels** | `crossbeam` | Consider | P2 |
| **Locks** | `parking_lot` | Consider | P2 (optimize) |
| **Concurrent map** | `dashmap` | Consider | P2 |
| **Unicode width** | `unicode-width` | Use | P0 (core) |
| **Unicode segm.** | `unicode-segmentation` | Use | P0 (core) |
| **Directories** | `directories` | Use | P0 (core) |
| **Fuzzy match** | `nucleo` | Use | P1 (completions) |
| **Fuzzy match (alt)** | `fuzzy-matcher` | Consider | P2 |
| **Git** | Git CLI | Use (start) | P1 (context) |
| **Git (alt)** | `git2` | Consider (later) | P2 |

**Priority Legend:**
- **P0:** Required for MVP - add to Cargo.toml immediately
- **P1:** Needed soon - add when implementing relevant feature
- **P2:** Optimization/enhancement - evaluate later based on profiling

---

## 17. Cargo.toml Starter

```toml
[dependencies]
# Core
tokio = { version = "1", features = ["rt", "io-util", "sync", "macros", "process"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Terminal
portable-pty = "0.8"
vte = "0.13"

# Unicode
unicode-width = "0.2"
unicode-segmentation = "1.12"

# Error handling
thiserror = "2"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Paths
directories = "5"

# CLI
clap = { version = "4", features = ["derive"] }

# File watching (for context engine)
notify = { version = "7", features = ["serde"] }

# Fuzzy matching (for completions)
nucleo = "0.5"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.5"
insta = "1.40"
tempfile = "3.14"
```

> **Note:** Version numbers are approximate - check crates.io for latest at time of implementation.

---

## References

1. crates.io: https://crates.io/
2. lib.rs (crate search): https://lib.rs/
3. Blessed.rs (community-recommended crates): https://blessed.rs/
4. Alacritty dependencies: https://github.com/alacritty/alacritty/blob/master/Cargo.toml
5. WezTerm dependencies: https://github.com/wez/wezterm/blob/main/Cargo.toml
