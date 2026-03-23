# Changelog

All notable changes to Wit Terminal are documented in this file.

## [0.1.0] - Unreleased

### Added

#### Terminal Core (Phase 1)
- VT100/xterm terminal emulator with 14-state ANSI parser
- Unix PTY support via `forkpty`
- Windows ConPTY support
- Terminal grid rendering with DOM-per-row architecture
- Full keyboard input mapping (Ctrl, Alt, function keys, clipboard)
- OSC 7 CWD tracking
- Session management with event broadcasting

#### Context & Completions (Phase 2)
- Context engine with 5 providers: git, Node.js, Rust, Python, Docker
- Completion engine with fuzzy matching and multi-source architecture
- 14 completion TOML files: git, npm, yarn, pnpm, cargo, docker, kubectl, pip, ssh, make, systemctl, brew, apt, general
- Dynamic completions: git branches/tags, npm scripts, cargo targets, docker containers/images, make targets
- Path completion source with filesystem scanning
- Shell integration scripts for bash, zsh, fish, PowerShell
- Tab key triggers completions with input buffer tracking
- Smart completion insertion (replaces current word)
- Configuration system with AppConfig and theme loading

#### Polish (Phase 3)
- Multi-session management with tab bar and sidebar
- Session switching: Ctrl+T/W/Tab, Ctrl+1-9
- 8 built-in themes: Wit Dark/Light, Catppuccin Mocha, Dracula, One Dark, Solarized Dark, Tokyo Night, Nord
- CSS custom properties for dynamic theming
- Settings UI with General, Appearance, Terminal tabs
- Command palette (Ctrl+Shift+P) with fuzzy search
- Context sidebar showing project info
- Search overlay (Ctrl+Shift+F) with match navigation
- Virtual scrolling for terminal grid performance
- Theme-aware cursor rendering

#### Ecosystem (Phase 4)
- Plugin system with trait-based API, manifest format, and loader
- Example Terraform plugin
- CONTRIBUTING.md with completion and theme contribution guides
- PR and issue templates
- GitHub Actions CI (lint, build, test on 3 platforms)
- Release workflow with tauri-action
- Bundled resources (completions, themes, shell-integration)
