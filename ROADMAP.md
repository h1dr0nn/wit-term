# Wit Terminal — Roadmap

> Context-aware terminal emulator. No AI, no cloud, no telemetry.

## Timeline Overview

| Phase | Period | Goal | Key Milestone |
|-------|--------|------|---------------|
| **Phase 1** | Months 1–3 | Foundation | Working terminal emulator |
| **Phase 2** | Months 4–6 | Context | Context detection + smart completions |
| **Phase 3** | Months 7–9 | Polish | Daily-driver quality |
| **Phase 4** | Months 10–12 | Ecosystem | v0.1.0 public release |

---

## Phase 1: Foundation

**Goal:** Build a working terminal that spawns a shell, renders ANSI output with colors, and handles all keyboard input across Windows, macOS, and Linux.

### Schedule

| Weeks | Focus | Deliverable |
|-------|-------|-------------|
| 1–2 | Project setup | Tauri v2 + React + Rust, CI/CD pipeline |
| 3–4 | PTY layer | Unix (`forkpty`) + Windows (ConPTY), shell spawn |
| 5–6 | ANSI parser | 14-state VT parser, CSI/SGR/OSC sequences |
| 7–8 | Grid rendering | Terminal grid, cursor, scrollback, React renderer |
| 9–10 | Input handling | Full keyboard mapping, clipboard, F1–F12 |
| 11–12 | Stabilization | Integration tests, cross-platform fixes, memory |

### Milestones

| # | Name | Week | Criteria |
|---|------|------|----------|
| M1 | First Light | 4 | Shell prompt appears, `echo`/`ls`/`pwd` work |
| M2 | Color TV | 8 | ANSI colors render, `vim`/`htop` usable |
| M3 | Keyboard Warrior | 10 | All keys work, Ctrl+C/D/Z, clipboard |
| M4 | Foundation Complete | 12 | Stable on 3 platforms, CI green, no crash in 1h |

### Success Criteria

- [x] Shell spawn → prompt on all 3 platforms
- [ ] `ls --color`, `git diff` show correct colors
- [ ] `vim`, `nano`, `htop` render and work
- [ ] Ctrl+C, arrow keys, Tab, function keys work
- [ ] Copy/paste works
- [ ] Window resize → content reflows
- [ ] 1 hour of use without crash
- [ ] CI passes on macOS, Linux, Windows

---

## Phase 2: Context

**Goal:** Transform from a regular terminal into a context-aware terminal. Detect project environments and provide intelligent tab completions.

### Schedule

| Weeks | Focus | Deliverable |
|-------|-------|-------------|
| 13–14 | Context engine | Provider trait, directory scanning, file watcher |
| 15–16 | Built-in providers | Git, Node.js, Python, Rust, Docker detection |
| 17–18 | Completion data | TOML format spec, 10+ command completion files |
| 19–20 | Completion engine | Fuzzy matching, ranking, <10ms for 10k candidates |
| 21–22 | Shell integration | CWD tracking (OSC 7), command history, bash/zsh/fish/PowerShell |
| 23–24 | Tab completion UI | Inline ghost hints, popup with descriptions, keyboard nav |

### Milestones

| # | Name | Week | Criteria |
|---|------|------|----------|
| M5 | Context Aware | 16 | `cd` into git/Node/Rust repo → detected, info shown |
| M6 | Smart Tab | 24 | `git checkout` + Tab → real branches; `npm run` + Tab → scripts |
| M7 | Phase 2 Complete | 24 | 10+ command groups, shell integration, <50ms completions |

### Completion Coverage

**P0 (Must have):** git, npm/yarn/pnpm, cargo, docker
**P1 (Should have):** kubectl, ssh, make, pip/pipenv
**P2 (Nice to have):** cd/ls/cat, systemctl, brew, apt

### Success Criteria

- [ ] Detects git, Node, Python, Rust, Docker projects
- [ ] Context updates within 200ms after `cd`
- [ ] Tab shows context-aware suggestions with descriptions
- [ ] Fuzzy matching: `git chk` → suggests `checkout`
- [ ] Dynamic completions: git branches, npm scripts, cargo targets
- [ ] Completion popup keyboard navigable (Up/Down, Enter, Esc)
- [ ] Shell integration scripts for 4 shells
- [ ] Completion response < 50ms

---

## Phase 3: Polish

**Goal:** Go from prototype to daily driver. Multi-session, themes, search, performance optimization, and settings UI.

### Schedule

| Weeks | Focus | Deliverable |
|-------|-------|-------------|
| 25–26 | Multi-session | Sidebar, tabs, split panes, session restore |
| 27–28 | Theming | 8+ themes, hot-reload, custom theme support |
| 29–30 | Selection & search | Text selection, clipboard, Ctrl+Shift+F, URL detection |
| 31–32 | Performance | Virtual scrolling, batch rendering, startup < 500ms |
| 33–34 | Settings | Settings UI, keybinding customization, config file |
| 35–36 | Final polish | Context sidebar, command palette, bug fixes |

### Milestones

| # | Name | Week | Criteria |
|---|------|------|----------|
| M8 | Daily Driver | 36 | 5+ sessions, 8+ themes, search, 8h stable, all perf targets met |

### Themes

Wit Dark, Wit Light, Catppuccin Mocha, Dracula, One Dark, Solarized Dark, Tokyo Night, Nord

### Performance Targets

| Metric | Target |
|--------|--------|
| Cold startup | < 500ms |
| Input latency | < 16ms (60fps) |
| Grid render (80×24) | < 2ms |
| Scroll 10k lines | 60fps |
| Memory per session | < 20MB |
| Memory (5 sessions) | < 100MB |
| Theme switch | < 100ms |
| Session switch | < 100ms |

### Success Criteria

- [ ] 5+ sessions open, smooth switching
- [ ] Split panes (horizontal + vertical)
- [ ] 8+ themes with hot-reload
- [ ] Text selection + copy/paste on all platforms
- [ ] Search in scrollback (Ctrl+Shift+F)
- [ ] URL detection + Ctrl+click → opens browser
- [ ] Command palette (Ctrl+Shift+P)
- [ ] Settings UI: font, theme, keybindings
- [ ] Config file at `~/.config/wit/config.toml`
- [ ] All performance targets met
- [ ] 8 hours continuous use without crash

---

## Phase 4: Ecosystem

**Goal:** Prepare for public release. Plugin system, packaging, documentation, community launch.

### Schedule

| Weeks | Focus | Deliverable |
|-------|-------|-------------|
| 37–38 | Plugin system | Plugin API, loader, manifest format, example plugin |
| 39–40 | Community | Contribution guides, validation tools, PR templates |
| 41–42 | Testing | Cross-platform test matrix (macOS 13+, Ubuntu 22.04, Windows 10/11) |
| 43–44 | Packaging | .dmg, .deb, .AppImage, .msi, auto-update, Homebrew/AUR/winget |
| 45–46 | Documentation | README with demo GIF, docs site, GitHub setup |
| 47–48 | Release | Beta testing, v0.1.0 release, community launch |

### Milestones

| # | Name | Week | Criteria |
|---|------|------|----------|
| M9 | v0.1.0 | 48 | Public release with packages, docs, auto-update |

### Packages

| Platform | Format |
|----------|--------|
| macOS | `.dmg` (signed + notarized), Homebrew |
| Linux | `.deb`, `.AppImage`, AUR |
| Windows | `.msi`, winget |

### Success Criteria

- [ ] Plugin API documented, 1+ example plugin works
- [ ] `CONTRIBUTING.md` with clear workflow
- [ ] `wit validate` CLI tool works
- [ ] Packages available for all 3 platforms
- [ ] Auto-update works (Tauri updater)
- [ ] README: demo GIF, install instructions, badges
- [ ] Documentation site live
- [ ] GitHub Release v0.1.0 published
- [ ] ≥ 3 beta testers with feedback
- [ ] ≥ 1 community post (Reddit/HN)
- [ ] No known critical bugs

---

## All Milestones

```
M1  Week 4  ── First Light          Shell prompt appears
M2  Week 8  ── Color TV             ANSI colors + vim works
M3  Week 10 ── Keyboard Warrior     All keyboard input works
M4  Week 12 ── Foundation Complete  Basic terminal stable
M5  Week 16 ── Context Aware        Project detection works
M6  Week 24 ── Smart Tab            Context completions work
M7  Week 24 ── Phase 2 Complete     10+ command groups + shell integration
M8  Week 36 ── Daily Driver         Polished, 8h stable
M9  Week 48 ── v0.1.0               First public release
```

---

## Risk Register

| Risk | Impact | Mitigation |
|------|--------|------------|
| Windows ConPTY edge cases | High | Study Windows Terminal source, extra buffer time |
| ANSI parser edge cases | Medium | Use vttest suite, implement incrementally |
| WebView rendering perf | High | Prototype early, Canvas/WebGL fallback ready |
| Scope creep | Medium | Strict phase boundaries, features → backlog |

---

## 12-Month Success Metrics

| Metric | Target |
|--------|--------|
| GitHub stars | ≥ 500 |
| Contributors | ≥ 10 |
| Completion rulesets | ≥ 20 ecosystems |
| Platform support | macOS + Linux + Windows |
| Terminal compatibility | 95% of common CLI tools |
| Startup time | < 500ms |
| Input latency | < 16ms |
