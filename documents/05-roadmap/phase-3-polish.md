# Phase 3: Polish (Months 7-9)

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Goals and Success Criteria

### Goals
1. Multi-session management with sidebar UI, tab switching
2. Complete theming system with hot-reload
3. Advanced text selection, clipboard, search, URL detection
4. Performance optimization reaching production-grade targets
5. Settings UI and keybinding customization
6. Context sidebar and command palette

### Success Criteria
- [ ] Open multiple sessions, switch between them smoothly
- [ ] Apply a new theme -> terminal updates immediately, no restart needed
- [ ] Select text -> copy -> paste works on all 3 platforms
- [ ] Ctrl+Shift+F -> search in terminal output
- [ ] Startup time < 500ms (cold start)
- [ ] Input latency < 16ms (1 frame at 60fps)
- [ ] Memory < 100MB for 5 simultaneous sessions
- [ ] Use Wit as a replacement for the default terminal for daily work

---

## Week-by-Week Breakdown

### Week 25-26: Session Sidebar UI and Multi-Session Management

**Objective:** Manage multiple terminal sessions with an intuitive UI.

**Tasks:**
- [ ] Session data model: id, title, shell type, CWD, status, created_at
- [ ] Session manager in Rust: create, destroy, list, switch
- [ ] **Sidebar component:**
  - List of sessions with title, status indicator
  - Right-click context menu: rename, duplicate, close
  - Drag-and-drop reorder
  - Session grouping (optional)
- [ ] **Tab bar** (alternative/supplement to sidebar):
  - Tabs for each session
  - Ctrl+T: new session
  - Ctrl+W: close current session
  - Ctrl+Tab / Ctrl+Shift+Tab: switch sessions
  - Ctrl+1-9: switch to session by number
- [ ] Split panes (horizontal/vertical):
  - Ctrl+Shift+D: split vertical
  - Ctrl+Shift+E: split horizontal
  - Ctrl+Shift+Arrow: resize panes
  - Ctrl+Shift+W: close pane
- [ ] Session restore: save session layout on quit, restore on reopen
- [ ] Session title auto-update: show running command or CWD

**Output:** Open 3+ sessions, split panes, switch smoothly.

### Week 27-28: Theming System

**Objective:** Flexible theming system, beautiful default themes.

**Tasks:**
- [ ] Theme data format (TOML or JSON):
  ```toml
  [metadata]
  name = "Wit Dark"
  author = "Wit Team"

  [colors]
  background = "#1e1e2e"
  foreground = "#cdd6f4"
  cursor = "#f5e0dc"
  selection = "#45475a"

  [colors.ansi]
  black = "#45475a"
  red = "#f38ba8"
  green = "#a6e3a1"
  # ... 16 ANSI colors

  [ui]
  sidebar_bg = "#181825"
  tab_active = "#313244"
  ```
- [ ] Theme loader: read theme files, validate, apply
- [ ] **Hot-reload:** Change theme file -> UI updates in real-time
- [ ] CSS custom properties: map theme colors -> CSS variables
- [ ] **Default themes:**
  1. Wit Dark (default) - custom dark theme
  2. Wit Light - clean light theme
  3. Catppuccin Mocha - popular dark theme
  4. Dracula - classic dark theme
  5. One Dark - Atom-inspired
  6. Solarized Dark/Light - accessible
  7. Tokyo Night - modern aesthetic
  8. Nord - cool blue tones
- [ ] Theme picker UI in settings
- [ ] Theme preview: hover -> preview, click -> apply
- [ ] Custom theme support: user creates theme file in config dir
- [ ] Export/import themes

**Output:** 8+ themes, hot-reload, theme picker.

### Week 29-30: Text Selection, Clipboard, Search, URL Detection

**Objective:** Text interaction on par with modern terminals.

**Tasks:**
- [ ] **Text selection:**
  - Click + drag to select
  - Double-click: select word
  - Triple-click: select line
  - Shift+click: extend selection
  - Ctrl+A: select all (in scrollback)
  - Selection highlight styling (from theme)
- [ ] **Advanced clipboard:**
  - Auto-copy selection (configurable)
  - Ctrl+Shift+C: copy
  - Ctrl+Shift+V: paste
  - Right-click: copy/paste context menu
  - Paste formatting: strip ANSI, handle newlines
- [ ] **Search in output:**
  - Ctrl+Shift+F: open search bar
  - Incremental search: highlight matches as you type
  - Next/Previous match: Enter / Shift+Enter
  - Case sensitive toggle
  - Regex search toggle
  - Match count display
  - Search in scrollback buffer
- [ ] **URL detection:**
  - Regex detect URLs in terminal output
  - Ctrl+click URL -> open in browser
  - URL underline on hover
  - File path detection: Ctrl+click -> open file in editor (configurable)

**Output:** Select, copy, search, click URLs - everything works smoothly.

### Week 31-32: Scrollback Optimization and Performance

**Objective:** Meet performance targets, optimize memory.

**Tasks:**
- [ ] **Scrollback optimization:**
  - Virtual scrolling: only render visible rows
  - Scrollback limit configurable (default: 10,000 lines)
  - Compress old scrollback data (store as text, not styled cells)
  - Memory budget per session
- [ ] **Rendering performance:**
  - Profile rendering pipeline
  - Batch DOM updates (requestAnimationFrame)
  - Minimize re-renders (React.memo, useMemo)
  - Consider Canvas rendering for hot path
  - Lazy style computation
- [ ] **Startup optimization:**
  - Measure cold start time
  - Lazy load non-critical modules
  - Pre-spawn shell process
  - Optimize Tauri init sequence
- [ ] **Input latency:**
  - Measure keypress -> screen update latency
  - Optimize IPC hot path (keypress -> PTY write -> read -> render)
  - Consider direct WebSocket for PTY data instead of Tauri events
- [ ] **Performance monitoring:**
  - Built-in performance overlay (toggle with hotkey)
  - Show FPS, memory, latency metrics
  - Performance regression tests in CI
- [ ] **Memory profiling:**
  - Track memory usage per session
  - Identify and fix memory leaks
  - Limit total memory usage

**Performance targets to meet:**

| Metric | Target | Measurement method |
| ------ | ------ | ------------------ |
| Cold startup | < 500ms | Time from click -> shell prompt |
| Input latency | < 16ms | Keypress -> pixel update |
| Render 80x24 | < 2ms | Performance.now() around render |
| Scroll 10k lines | Smooth 60fps | FPS counter during scroll |
| Memory per session | < 20MB | Process memory monitor |
| Memory 5 sessions | < 100MB | Process memory monitor |
| Large output (cat 1MB file) | No freeze | UI remains responsive |

**Output:** All performance targets met or exceeded.

### Week 33-34: Settings UI and Keybinding Customization

**Objective:** Users can customize Wit to their liking.

**Tasks:**
- [ ] **Configuration file** (TOML):
  ```toml
  [general]
  default_shell = "/bin/zsh"
  startup_directory = "~"
  confirm_close = true

  [appearance]
  theme = "wit-dark"
  font_family = "JetBrains Mono"
  font_size = 14
  line_height = 1.2
  cursor_style = "block"  # block, beam, underline
  cursor_blink = true

  [terminal]
  scrollback_lines = 10000
  copy_on_select = true

  [keybindings]
  new_tab = "ctrl+t"
  close_tab = "ctrl+w"
  split_vertical = "ctrl+shift+d"
  ```
- [ ] Config file location: `~/.config/wit/config.toml` (Unix), `%APPDATA%/wit/config.toml` (Windows)
- [ ] Config hot-reload: edit file -> settings update immediately
- [ ] **Settings UI:**
  - General tab: shell, startup dir, behavior
  - Appearance tab: theme picker, font, cursor
  - Terminal tab: scrollback, clipboard behavior
  - Keybindings tab: searchable keybinding list, rebind UI
- [ ] **Keybinding system:**
  - Default keybindings (sensible defaults)
  - User overrides in config
  - Keybinding conflict detection
  - "Record shortcut" UI: press keys -> capture binding
- [ ] Font selection:
  - List available monospace fonts
  - Font preview
  - Font size adjustment (Ctrl+Plus, Ctrl+Minus, Ctrl+0)
- [ ] Settings search: filter settings by keyword

**Output:** Full Settings UI, config file working, keybindings customizable.

### Week 35-36: Context Sidebar, Command Palette, and Final Polish

**Objective:** Power-user features, polish entire UI.

**Tasks:**
- [ ] **Context sidebar:**
  - Show context info for current directory
  - Git: branch, status, recent commits
  - Node: package name, scripts, dependencies count
  - Cargo: crate name, targets
  - Docker: running containers, compose services
  - Collapsible sections
  - Toggle sidebar: Ctrl+B
- [ ] **Command palette:**
  - Ctrl+Shift+P: open command palette
  - Search all available commands
  - Fuzzy search
  - Recent commands
  - Category grouping
  - Keyboard shortcut displayed next to command
- [ ] **UI Polish:**
  - Smooth animations (sidebar toggle, tab switch, popup)
  - Loading states (spinner when shell starting)
  - Error states (friendly error messages)
  - Empty states (no sessions, first launch)
  - Tooltips for icons and buttons
  - Responsive layout (narrow window still usable)
- [ ] **Bug fix sprint:**
  - Triage all known bugs
  - Fix critical and major bugs
  - Edge case handling
  - Cross-platform consistency check

**Output:** Wit feels polished, professional, ready for daily use.

---

## Phase 3 Deliverables

| # | Deliverable | Description |
| - | ----------- | ----------- |
| 1 | Multi-session UI | Sidebar, tabs, split panes, session restore |
| 2 | Theming system | 8+ themes, hot-reload, custom themes, theme picker |
| 3 | Text interaction | Selection, clipboard, search, URL detection |
| 4 | Performance | All performance targets met |
| 5 | Settings | Config file, Settings UI, keybinding customization |
| 6 | Context sidebar | Project info display, collapsible sections |
| 7 | Command palette | Ctrl+Shift+P, fuzzy search, all commands |

---

## Consolidated Performance Targets

```
Metric              Target        Status
---------------------------------------------
Cold startup        < 500ms       [ ]
Input latency       < 16ms        [ ]
Render frame        < 2ms         [ ]
Scroll 10k lines    60fps         [ ]
Memory/session      < 20MB        [ ]
Memory 5 sessions   < 100MB       [ ]
Large output        No freeze     [ ]
Theme switch        < 100ms       [ ]
Session switch      < 50ms        [ ]
Search 10k lines    < 200ms       [ ]
```

---

## UX Audit Checklist

Before concluding Phase 3, audit the entire UX:

### First Launch Experience
- [ ] App opens quickly, no prolonged blank screen
- [ ] Default theme looks beautiful, professional
- [ ] Shell prompt appears immediately
- [ ] No complex setup wizard

### Daily Use
- [ ] Create new tab quickly (< 200ms)
- [ ] Switch tabs without lag
- [ ] Copy/paste works as expected
- [ ] Search finds text in output
- [ ] URLs are clickable
- [ ] Resize window is smooth

### Keyboard-First
- [ ] All actions have keyboard shortcuts
- [ ] Shortcuts are consistent with conventions (Ctrl+T, Ctrl+W, ...)
- [ ] Shortcut hints shown in UI
- [ ] Command palette accessible

### Error Handling
- [ ] Shell crash -> friendly message, option to restart
- [ ] Network error -> clear message
- [ ] Invalid config -> fallback to defaults, show warning
- [ ] No unhandled exceptions visible to user

### Accessibility
- [ ] Sufficient color contrast (WCAG AA)
- [ ] Font size adjustable
- [ ] Screen reader basics (ARIA labels for UI elements)

---

## Definition of "Phase 3 Complete"

Phase 3 is considered complete when **all** of the following conditions are met:

1. **Multi-session:** Open 5+ sessions, switch between them, split panes work
2. **Themes:** At least 8 themes, hot-reload, user can create custom theme
3. **Text selection:** Select, copy, paste works on all 3 platforms
4. **Search:** Ctrl+Shift+F searches in output, highlights matches
5. **URL click:** Ctrl+click URL -> opens browser
6. **Performance:** All performance targets met (see table above)
7. **Settings:** Config file and Settings UI work, keybindings customizable
8. **Command palette:** Ctrl+Shift+P works, fuzzy search
9. **Stability:** 8 hours of continuous use without crash
10. **Daily driver:** Developer (author) is ready to use Wit as default terminal

**Litmus test:** Use Wit for daily development work for 1 week. If you have to go back to the old terminal due to a missing feature or bug -> Phase 3 is not complete.
