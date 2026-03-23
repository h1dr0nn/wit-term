# Glossary

> **Status:** living
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

Technical terms used in the Wit project, organized by category.

---

## Terminal Emulation

| Term | Definition |
|---|---|
| **Terminal emulator** | Software that emulates a hardware terminal, allowing interaction with a shell through a graphical interface |
| **PTY (Pseudo-Terminal)** | A pair of virtual devices (master/slave) that allow a terminal emulator to communicate with a shell process. The master side is the terminal app, the slave side is the shell |
| **VT100** | A hardware terminal by DEC (1978), which became the de-facto standard for terminal emulation. Most terminal emulators are VT100-compatible |
| **xterm** | A terminal emulator for the X Window System, extending VT100 with many escape sequences. The xterm-256color standard is the primary compatibility target |
| **Shell** | A program that receives and executes commands (bash, zsh, fish, PowerShell). Runs inside the terminal emulator |
| **TTY** | Abbreviation for "TeleTYpewriter". In Unix, `/dev/tty*` are terminal devices. A historical term, now used synonymously with terminal |
| **Cell** | A single position in the terminal grid, containing one character + attributes (color, bold, etc.) |
| **Grid** | A 2D matrix of cells, representing the displayed content on the terminal. Size = columns x rows |
| **Scrollback buffer** | Memory area storing output that has scrolled past the viewport. Allows the user to scroll up to view history |
| **Alternate screen** | A secondary buffer used by full-screen applications (vim, less, htop). When the app exits, the terminal returns to the main buffer |

## ANSI / Escape Sequences

| Term | Definition |
|---|---|
| **ANSI escape code** | Special character sequences starting with ESC (`\x1b`) used to control the terminal (change colors, move cursor, clear screen, etc.) |
| **CSI (Control Sequence Introducer)** | The prefix `ESC [` (`\x1b[`), the most common type of escape sequence. E.g.: `\x1b[31m` = red text |
| **OSC (Operating System Command)** | The prefix `ESC ]` (`\x1b]`), used for terminal-specific features (set title, hyperlinks, clipboard) |
| **SGR (Select Graphic Rendition)** | A CSI sequence ending with `m`, controlling text styling: color, bold, italic, underline. E.g.: `\x1b[1;32m` = bold green |
| **DCS (Device Control String)** | The prefix `ESC P`, used for device-specific commands (Sixel graphics, DECRQSS) |
| **C0 controls** | Control characters 0x00-0x1F: BEL (0x07), BS (0x08), TAB (0x09), LF (0x0A), CR (0x0D) |
| **C1 controls** | 8-bit control characters 0x80-0x9F, equivalent to ESC + the second byte |

## Context Engine

| Term | Definition |
|---|---|
| **Context** | Aggregated information about the current environment: project type, git status, running processes, environment variables |
| **Context detection** | The process by which Wit scans the file system and environment to determine the current context |
| **Context provider** | A module responsible for detecting a specific aspect of context (GitProvider, NodeProvider, etc.) |
| **Project type** | The type of project detected via marker files: Cargo.toml -> Rust, package.json -> Node, etc. |
| **Marker file** | A file or directory used to identify a project type. E.g.: `.git/` -> git repo, `Dockerfile` -> Docker project |
| **Context event** | An event emitted when context changes (directory change, file created/deleted, git branch change) |

## Completion Engine

| Term | Definition |
|---|---|
| **Completion** | A suggestion for the current input: command name, flag, argument, file path, etc. |
| **Completion rule** | A definition of when and what to suggest for a specific command. Stored as data files |
| **Completion set** | A group of completion rules for an ecosystem (git completions, docker completions, etc.) |
| **Completion source** | A source providing completions: static rules, file system, command history, context |
| **Fuzzy matching** | A search algorithm that allows inexact matching. E.g.: "gco" matches "git checkout" |
| **Frecency** | A combination of frequency (how often) and recency (how recently) for ranking completions |
| **Inline hint** | A faded suggestion displayed after the cursor (like fish shell autosuggestion) |
| **Completion popup** | A dropdown UI displaying the list of completions when there are multiple suggestions |

## Shell Integration

| Term | Definition |
|---|---|
| **Shell integration** | A feature allowing the terminal emulator to communicate more deeply with the shell via OSC sequences |
| **Prompt detection** | Identifying the position of the prompt in terminal output, distinguishing prompt from command output |
| **Command tracking** | Tracking which command is running, its exit code, and execution time |
| **CWD tracking** | Tracking the current working directory of the shell, updating the context engine |
| **Precmd / Preexec hooks** | Shell hooks that run before displaying the prompt (precmd) or before executing a command (preexec) |

## Architecture

| Term | Definition |
|---|---|
| **Tauri** | A framework for building desktop apps with a Rust backend + webview frontend. A lightweight alternative to Electron |
| **IPC (Inter-Process Communication)** | Communication between the Rust core and frontend via Tauri commands and events |
| **Tauri command** | A Rust function exposed to the frontend via `invoke()`. Synchronous request-response |
| **Tauri event** | A message system between Rust and frontend. Asynchronous, event-driven |
| **Webview** | A native browser component that renders the frontend. WKWebView (macOS), WebView2 (Windows), WebKitGTK (Linux) |
| **State machine** | A pattern for managing terminal emulator state: current state + input -> new state + output |

## UI

| Term | Definition |
|---|---|
| **Terminal view** | The main component that renders the terminal grid, cursor, selection |
| **Session** | A terminal instance with its own shell process, buffer, and context |
| **Tab / Pane** | UI representation of a session. A tab is full-width, a pane is a split view |
| **Design token** | A named design value (colors, spacing, fonts) used throughout the UI |
| **Theme** | A collection of design tokens forming a complete visual style |

## Build & Development

| Term | Definition |
|---|---|
| **Cargo** | Rust package manager and build system |
| **Crate** | A Rust package/library, similar to an npm package |
| **Vite** | Frontend build tool, used for React development |
| **Tauri CLI** | Command-line tool for managing Tauri projects: `tauri dev`, `tauri build` |
| **Cross-compilation** | Building a binary for a platform different from the development platform |
