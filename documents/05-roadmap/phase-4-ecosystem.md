# Phase 4: Ecosystem (Months 10-12)

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Goals and Success Criteria

### Goals
1. Plugin system that allows extending Wit functionality
2. Community workflow for contributing completions and themes
3. Cross-platform testing and platform-specific fixes
4. Packaging for 3 platforms with auto-update
5. Documentation site and README with demo GIFs
6. v0.1.0 public release

### Success Criteria
- [ ] Plugin API documented, at least 1 example plugin working
- [ ] Community contribution guide is clear, has validation tooling
- [ ] Installable on macOS (.dmg), Linux (.deb, .AppImage), Windows (.msi)
- [ ] Auto-update works
- [ ] README has demo GIF, badges, clear installation instructions
- [ ] v0.1.0 released on GitHub Releases
- [ ] At least 1 beta tester outside core team has tried it

---

## Week-by-Week Breakdown

### Week 37-38: Plugin System Architecture

**Objective:** Design and implement plugin loading system.

**Tasks:**
- [ ] **Plugin architecture decision:**
  - Option A: WASM-based plugins (sandboxed, safe, cross-platform)
  - Option B: Native Rust plugins (dynamic loading, full power, less safe)
  - Option C: Script-based plugins (Lua/JS, easy to write, limited)
  - Recommend: WASM for v1, script-based for quick iteration
- [ ] **Plugin API design:**
  ```rust
  trait WitPlugin {
      fn name(&self) -> &str;
      fn version(&self) -> &str;
      fn on_load(&mut self, api: &WitApi);
      fn on_unload(&mut self);
  }

  trait WitApi {
      fn register_context_provider(&self, provider: Box<dyn ContextProvider>);
      fn register_command(&self, name: &str, handler: CommandHandler);
      fn register_theme(&self, theme: Theme);
      fn on_event(&self, event_type: &str, handler: EventHandler);
      fn get_config(&self, key: &str) -> Option<String>;
  }
  ```
- [ ] Plugin manifest format (TOML):
  ```toml
  [plugin]
  name = "wit-kubernetes"
  version = "0.1.0"
  description = "Kubernetes completions and context"
  author = "community"
  wit_version = ">=0.1.0"

  [permissions]
  filesystem = ["read"]
  network = false
  shell = false
  ```
- [ ] Plugin loader: discover, validate, load, initialize
- [ ] Plugin directory: `~/.config/wit/plugins/`
- [ ] Plugin lifecycle: load -> init -> active -> unload
- [ ] Plugin isolation: error in plugin does not crash app
- [ ] Plugin settings: each plugin has its own config section

**Output:** Plugin architecture documented, loader implemented.

### Week 39-40: Community Contribution Workflow

**Objective:** Make it easy for community to contribute completions, themes, plugins.

**Tasks:**
- [ ] **Completion contribution workflow:**
  1. Fork repo
  2. Add/edit completion file in `completions/` directory
  3. Run `wit validate` -> check format
  4. Submit PR -> CI automatically runs validation
  5. Review -> merge -> available in next release
- [ ] **Validation tooling:**
  - `wit validate completions/git.yaml` - check format, required fields
  - `wit validate themes/my-theme.toml` - check colors, contrast
  - `wit test completions/git.yaml` - interactive test mode
  - CI action: auto-validate PRs
- [ ] **Completion contribution guide:**
  - Format specification reference
  - Step-by-step tutorial: "Add completions for X command"
  - Examples: simple command, complex command with subcommands
  - Testing guide: how to verify completions work
- [ ] **Theme contribution guide:**
  - Color specification reference
  - Template theme file
  - Contrast checker: ensure readability
  - Screenshot generator: auto-generate theme previews
- [ ] **Community infrastructure:**
  - GitHub issue templates (bug, feature, completion request)
  - PR template
  - Contributing guide (CONTRIBUTING.md)
  - Code of Conduct
- [ ] **Completion registry/index:**
  - Centralized index of available completions
  - Version compatibility tracking
  - Download stats

**Output:** Clear contribution workflow, validation tools, documentation.

### Week 41-42: Cross-Platform Testing

**Objective:** Wit works well on all 3 platforms.

**Tasks:**
- [ ] **Test matrix:**

  | Platform | Shell | Terminal features |
  | -------- | ----- | ----------------- |
  | macOS 13+ (Intel) | bash, zsh, fish | PTY, clipboard, themes |
  | macOS 13+ (Apple Silicon) | bash, zsh, fish | PTY, clipboard, themes |
  | Ubuntu 22.04 | bash, zsh, fish | PTY, clipboard, themes |
  | Fedora 38 | bash, zsh | PTY, clipboard, themes |
  | Windows 11 | PowerShell, cmd, WSL | ConPTY, clipboard, themes |
  | Windows 10 | PowerShell, cmd | ConPTY, clipboard, themes |

- [ ] **Platform-specific fixes:**
  - macOS: Cmd key handling, native menu bar, .app bundle
  - Linux: Wayland vs X11 clipboard, different DE behaviors
  - Windows: ConPTY edge cases, font rendering differences
- [ ] **Automated testing:**
  - CI builds on all platforms (GitHub Actions matrix)
  - Integration tests for PTY on each platform
  - Screenshot comparison tests (optional)
- [ ] **Manual testing checklist:**
  - [ ] Shell starts correctly
  - [ ] Colors render correctly
  - [ ] Keyboard input (special keys, Ctrl, function keys)
  - [ ] Clipboard (copy, paste)
  - [ ] Text selection
  - [ ] URL clicking
  - [ ] Window resize / reflow
  - [ ] Multiple sessions
  - [ ] Theme switching
  - [ ] Settings persistence
  - [ ] Completions work
  - [ ] Context detection
- [ ] Platform-specific features:
  - macOS: Touch Bar support (if applicable), system theme detection
  - Linux: Desktop file, tray icon
  - Windows: Jump list, taskbar integration

**Output:** Wit works consistently on 3 platforms.

### Week 43-44: Packaging and Distribution

**Objective:** Installable packages for 3 platforms, auto-update.

**Tasks:**
- [ ] **macOS:**
  - .dmg installer with drag-to-Applications
  - .app bundle signed (Apple Developer Certificate)
  - Notarization (Apple requirement)
  - Homebrew formula: `brew install wit`
  - Universal binary (Intel + Apple Silicon)
- [ ] **Linux:**
  - .deb package (Debian/Ubuntu)
  - .rpm package (Fedora/RHEL)
  - .AppImage (universal Linux)
  - Flatpak (optional)
  - AUR package (Arch Linux)
  - Snap package (optional)
- [ ] **Windows:**
  - .msi installer
  - Portable .zip (no install needed)
  - winget manifest: `winget install wit`
  - Scoop manifest (optional)
  - Code signing certificate
- [ ] **Auto-update:**
  - Tauri updater integration
  - Update check on startup (configurable)
  - Background download
  - User notification: "Update available v0.1.1"
  - One-click update
  - Rollback if update fails
- [ ] **Release automation:**
  - GitHub Actions: tag -> build -> sign -> publish
  - Changelog generation from commits
  - Asset upload to GitHub Releases
  - Package registry submissions

**Output:** One-click install on 3 platforms, auto-update working.

### Week 45-46: Documentation and GitHub Setup

**Objective:** Professional online presence, comprehensive documentation.

**Tasks:**
- [ ] **README.md:**
  - Hero banner / logo
  - Demo GIF (terminal in action: completions, themes, context)
  - Feature list with screenshots
  - Installation instructions (3 platforms)
  - Quick start guide
  - Badges: CI status, version, license, downloads
  - Links: docs, contributing, community
- [ ] **Demo GIFs (3-4 GIFs):**
  1. Basic usage: open terminal, run commands, colored output
  2. Context-aware completions: Tab in git repo, Node project
  3. Themes: switch between themes
  4. Multi-session: split panes, tabs, context sidebar
- [ ] **Documentation site** (mdBook or Docusaurus):
  - Getting Started guide
  - Configuration reference
  - Keybinding reference
  - Theming guide
  - Completion authoring guide
  - Plugin development guide
  - FAQ
  - Troubleshooting
- [ ] **GitHub repository setup:**
  - Issue templates: bug report, feature request, completion request
  - PR template
  - CONTRIBUTING.md
  - CODE_OF_CONDUCT.md
  - LICENSE (MIT or Apache 2.0)
  - CHANGELOG.md
  - GitHub Discussions enabled
  - Branch protection rules
  - CI badges in README

**Output:** Professional GitHub repo, documentation site, demo GIFs.

### Week 47-48: Beta Testing and v0.1.0 Release

**Objective:** Beta testing, fix critical issues, release v0.1.0.

**Tasks:**
- [ ] **Beta release (v0.1.0-beta.1):**
  - Build packages for 3 platforms
  - Distribute to beta testers (5-10 people)
  - Setup feedback channels (GitHub Issues, Discord)
- [ ] **Beta testing period (1 week):**
  - Testers use Wit for daily work
  - Collect bug reports
  - Collect UX feedback
  - Monitor crash reports
- [ ] **Bug fix sprint:**
  - Triage beta feedback
  - Fix critical bugs (crash, data loss)
  - Fix major UX issues
  - Defer non-critical issues to v0.1.1
- [ ] **v0.1.0 Release:**
  - Final build from release branch
  - Sign packages
  - Upload to GitHub Releases
  - Update Homebrew/AUR/winget manifests
  - Publish documentation site
  - Write release blog post / announcement
- [ ] **Community launch:**
  - Post on Reddit (r/rust, r/programming, r/commandline)
  - Post on Hacker News
  - Post on Twitter/X
  - Post on relevant Discord servers
  - Product Hunt submission (optional)

**Output:** v0.1.0 released, available on 3 platforms.

---

## Phase 4 Deliverables

| # | Deliverable | Description |
| - | ----------- | ----------- |
| 1 | Plugin system | Architecture, API, loader, example plugin |
| 2 | Contribution workflow | Guides, validation tools, PR templates |
| 3 | Cross-platform | Tested on macOS, Linux, Windows |
| 4 | Packages | .dmg, .deb, .AppImage, .msi, Homebrew, winget |
| 5 | Auto-update | Tauri updater, background check, one-click update |
| 6 | Documentation | README, docs site, demo GIFs |
| 7 | v0.1.0 | First public release |

---

## Community Engagement Strategy

### Pre-Launch (Week 37-44)
- Build in public: share progress on Twitter/X, blog posts
- Early access: invite 5-10 developers to try Wit
- Collect feedback early, iterate

### Launch (Week 47-48)
- Coordinated launch: blog post + social media + community posts
- Respond to all issues/comments within the first 24 hours
- Be transparent: acknowledge limitations, share roadmap

### Post-Launch (Ongoing)
- Regular releases: bugfix releases every 2 weeks
- Feature releases: monthly
- Community contributions: review PRs within 48h
- Discord/Discussions: active engagement
- Roadmap updates: share plans, gather input

---

## Release Checklist

### Pre-Release
- [ ] All critical bugs fixed
- [ ] Performance targets met
- [ ] Cross-platform testing complete
- [ ] Documentation up to date
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml, package.json, tauri.conf.json
- [ ] License file present
- [ ] No hardcoded paths or developer-specific config

### Build & Sign
- [ ] Release builds successful on 3 platforms
- [ ] macOS: signed + notarized
- [ ] Windows: code signed
- [ ] Linux: packages built correctly
- [ ] All packages tested on clean machines

### Publish
- [ ] GitHub Release created with changelog
- [ ] Packages uploaded as release assets
- [ ] Homebrew formula PR submitted
- [ ] AUR package updated
- [ ] winget manifest PR submitted
- [ ] Documentation site deployed
- [ ] Auto-update endpoint configured

### Announce
- [ ] Blog post published
- [ ] Social media posts scheduled
- [ ] Community posts (Reddit, HN) submitted
- [ ] Email beta testers: "v0.1.0 is live!"

---

## Definition of "Phase 4 Complete"

Phase 4 is considered complete when **all** of the following conditions are met:

1. **Plugin system:** Plugin API documented, at least 1 example plugin loads and works
2. **Contribution:** CONTRIBUTING.md exists, validation tool works, at least 1 external PR accepted
3. **Cross-platform:** Tested and working on macOS, Ubuntu, Windows 11
4. **Packages:** .dmg (macOS), .deb + .AppImage (Linux), .msi (Windows) available
5. **Auto-update:** Update check and install works
6. **README:** Has demo GIF, installation instructions, badges
7. **Docs:** Documentation site live, covers getting started + configuration
8. **v0.1.0:** Tag created, packages published on GitHub Releases
9. **Community:** At least 1 community post (Reddit/HN) published
10. **Feedback:** At least 3 beta testers have tried it and given feedback

**Acceptance scenario:**
```
1. A newcomer finds Wit on GitHub
2. README has GIF demo -> understands what Wit does
3. Clicks download -> installs on their platform
4. Opens Wit -> shell prompt appears
5. Types commands -> completions work
6. Customizes theme -> looks great
7. Files an issue on GitHub -> receives a response
8. Wants to contribute a completion -> follows guide, submits PR
```
