# Keyboard Shortcuts

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Overview

This document defines all keyboard shortcuts for Wit, design rules, and how users can customize keybindings.

---

## 2. Design Philosophy

### 2.1. Core Principles

1. **No conflict with shell shortcuts**: Ctrl+C (SIGINT), Ctrl+Z (SIGTSTP), Ctrl+D (EOF), Ctrl+L (clear) must be forwarded directly to the shell. Wit must not intercept them.

2. **Ctrl+Shift as prefix for app shortcuts**: most Wit shortcuts use Ctrl+Shift+{key} to avoid conflicts with shell and CLI tools.

3. **Consistent with conventions**: follows common terminal emulator conventions (Ctrl+Shift+C for copy instead of Ctrl+C).

4. **Discoverable**: shortcuts are displayed in menus, tooltips, and the command palette.

5. **Customizable**: every shortcut can be rebound by the user.

### 2.2. Modifier Key Hierarchy

| Modifier         | Used for                          |
|------------------|-----------------------------------|
| (none)           | Forwarded to shell/CLI (typing)   |
| Ctrl             | Shell shortcuts (Ctrl+C, Ctrl+Z, etc.) and some safe app shortcuts (Ctrl+B, Ctrl+1-9) |
| Ctrl+Shift       | App-level shortcuts (copy, paste, search, new tab) |
| Alt              | Shell-specific (Alt+B, Alt+F word navigation) and some app shortcuts |
| F-keys           | App functions (F2 rename, F11 fullscreen) |

---

## 3. Default Keybindings

### 3.1. Terminal Operations

| Shortcut           | Action                       | Description                    |
|--------------------|------------------------------|--------------------------------|
| Ctrl+Shift+C       | Copy                         | Copy selected text to clipboard |
| Ctrl+Shift+V       | Paste                        | Paste from clipboard into terminal |
| Ctrl+Shift+F       | Find                         | Open search bar in terminal    |
| Ctrl+Shift+A       | Select All                   | Select all text in scrollback  |
| Ctrl+Shift+K       | Clear Terminal               | Clear all terminal output      |
| Ctrl+Shift+Enter   | New Line (paste mode)        | Insert newline when in paste mode |

**Note:** Ctrl+C and Ctrl+V are not used because:
- Ctrl+C = SIGINT (cancel running process)
- Ctrl+V = literal next character (in bash)

### 3.2. Session Management

| Shortcut           | Action                       | Description                    |
|--------------------|------------------------------|--------------------------------|
| Ctrl+Shift+T       | New Session                  | Create a new terminal session  |
| Ctrl+Shift+W       | Close Session                | Close the current session      |
| Ctrl+1             | Switch to Session 1          | Switch to session 1            |
| Ctrl+2             | Switch to Session 2          | Switch to session 2            |
| Ctrl+3             | Switch to Session 3          | Switch to session 3            |
| Ctrl+4             | Switch to Session 4          | Switch to session 4            |
| Ctrl+5             | Switch to Session 5          | Switch to session 5            |
| Ctrl+6             | Switch to Session 6          | Switch to session 6            |
| Ctrl+7             | Switch to Session 7          | Switch to session 7            |
| Ctrl+8             | Switch to Session 8          | Switch to session 8            |
| Ctrl+9             | Switch to Session 9          | Switch to the last session     |
| Ctrl+Tab           | Next Session                 | Switch to the next session     |
| Ctrl+Shift+Tab     | Previous Session             | Switch to the previous session |
| F2                 | Rename Session               | Rename the current session     |
| Alt+Up             | Move Session Up              | Move session up in the list    |
| Alt+Down           | Move Session Down            | Move session down in the list  |

### 3.3. Completion

| Shortcut           | Action                       | Description                    |
|--------------------|------------------------------|--------------------------------|
| Tab                | Trigger / Accept             | Open completion popup or accept completion |
| Escape             | Dismiss                      | Close completion popup         |
| Arrow Down         | Next Item                    | Select next item               |
| Arrow Up           | Previous Item                | Select previous item           |
| Enter              | Accept                       | Accept selected completion     |
| Ctrl+Space         | Force Trigger                | Force open completion popup    |
| Page Down          | Scroll Down                  | Scroll down 10 items           |
| Page Up            | Scroll Up                    | Scroll up 10 items             |

### 3.4. Sidebar

| Shortcut           | Action                       | Description                    |
|--------------------|------------------------------|--------------------------------|
| Ctrl+B             | Toggle Left Sidebar          | Show/hide session sidebar      |
| Ctrl+Shift+B       | Toggle Right Sidebar         | Show/hide context sidebar      |

### 3.5. Application

| Shortcut           | Action                       | Description                    |
|--------------------|------------------------------|--------------------------------|
| Ctrl+,             | Open Settings                | Open Settings page             |
| Ctrl+Shift+P       | Command Palette              | Open command palette           |
| F11                | Toggle Fullscreen            | Toggle fullscreen mode         |
| Ctrl+Q             | Quit Application             | Quit Wit (with confirmation)   |

### 3.6. Zoom / Font Size

| Shortcut           | Action                       | Description                    |
|--------------------|------------------------------|--------------------------------|
| Ctrl+=             | Zoom In                      | Increase font size             |
| Ctrl+-             | Zoom Out                     | Decrease font size             |
| Ctrl+0             | Reset Zoom                   | Reset to default font size     |
| Ctrl+Scroll Up     | Zoom In (mouse)              | Increase font size with mouse  |
| Ctrl+Scroll Down   | Zoom Out (mouse)             | Decrease font size with mouse  |

### 3.7. Scroll / Navigation

| Shortcut           | Action                       | Description                    |
|--------------------|------------------------------|--------------------------------|
| Scroll Wheel       | Scroll (3 lines)             | Scroll terminal output         |
| Shift+Scroll       | Scroll (1 page)              | Scroll faster                  |
| Page Up            | Scroll Up (page)             | Scroll up 1 page               |
| Page Down          | Scroll Down (page)           | Scroll down 1 page             |
| Shift+Home         | Scroll to Top                | Scroll to top of scrollback    |
| Shift+End          | Scroll to Bottom             | Scroll to bottom of output     |

### 3.8. Search (when search bar is open)

| Shortcut           | Action                       | Description                    |
|--------------------|------------------------------|--------------------------------|
| Enter              | Next Match                   | Jump to next match             |
| Shift+Enter        | Previous Match               | Jump to previous match         |
| Escape             | Close Search                 | Close search bar               |
| Alt+C              | Toggle Case Sensitive        | Toggle case sensitivity        |
| Alt+R              | Toggle Regex                 | Toggle regex mode              |
| Alt+W              | Toggle Whole Word            | Toggle whole word match        |

---

## 4. Customization

### 4.1. Keybindings File

Users can customize keybindings with the `keybindings.toml` file:

| Platform  | File location                          |
|-----------|----------------------------------------|
| Linux     | `~/.config/wit/keybindings.toml`       |
| macOS     | `~/Library/Application Support/wit/keybindings.toml` |
| Windows   | `%APPDATA%\wit\keybindings.toml`       |

### 4.2. File Format

```toml
# Wit Keybindings Configuration
# Only define keybindings you want to override.
# Keybindings not defined here will use default values.

# Format: action = "modifier+key"
# Modifiers: ctrl, shift, alt, super (cmd on macOS)
# Separator: "+" between modifier and key
# Key names: a-z, 0-9, f1-f12, tab, enter, escape, space,
#            up, down, left, right, home, end, pageup, pagedown,
#            backspace, delete, insert

[terminal]
copy = "ctrl+shift+c"
paste = "ctrl+shift+v"
find = "ctrl+shift+f"
select_all = "ctrl+shift+a"
clear = "ctrl+shift+k"

[session]
new = "ctrl+shift+t"
close = "ctrl+shift+w"
next = "ctrl+tab"
previous = "ctrl+shift+tab"
rename = "f2"
# switch_1 ... switch_9 use format: switch_N = "ctrl+N"
switch_1 = "ctrl+1"
switch_2 = "ctrl+2"
switch_3 = "ctrl+3"
# ... etc

[sidebar]
toggle_left = "ctrl+b"
toggle_right = "ctrl+shift+b"

[app]
settings = "ctrl+,"
command_palette = "ctrl+shift+p"
fullscreen = "f11"
quit = "ctrl+q"

[zoom]
zoom_in = "ctrl+="
zoom_out = "ctrl+-"
reset = "ctrl+0"

# Example: change copy to Ctrl+Shift+X
# [terminal]
# copy = "ctrl+shift+x"

# Example: add a new shortcut for a custom action
# [custom]
# run_build = "ctrl+shift+r"
```

### 4.3. Override Rules

- The `keybindings.toml` file only overrides defined entries
- Shortcuts not in the file keep their default values
- Set the value to `""` (empty string) to completely unbind a shortcut
- Set the value to `"disabled"` to disable a shortcut

```toml
# Example: disable the quit shortcut (prevent Ctrl+Q from quitting the app)
[app]
quit = "disabled"
```

---

## 5. Chord Shortcuts

### 5.1. Concept

Chord shortcuts are a sequence of 2 key presses in succession, similar to VS Code.

### 5.2. Format

```toml
# Chord shortcuts use a space to separate the 2 parts
[chords]
open_keybindings = "ctrl+k ctrl+s"      # Ctrl+K then Ctrl+S
open_theme_picker = "ctrl+k ctrl+t"      # Ctrl+K then Ctrl+T
toggle_word_wrap = "ctrl+k ctrl+w"       # Ctrl+K then Ctrl+W
```

### 5.3. Behavior

1. User presses Ctrl+K -> Wit enters "chord mode"
2. Display indicator in status bar: "Ctrl+K pressed, waiting for next key..."
3. User presses Ctrl+S -> executes the "open keybindings" action
4. If user presses a key that is not part of a chord -> cancel chord mode, forward key to shell
5. Timeout: 2 seconds, after which chord mode is cancelled

### 5.4. Built-in Chords (Default)

| Chord               | Action                       |
|----------------------|------------------------------|
| Ctrl+K Ctrl+S        | Open keybindings settings   |
| Ctrl+K Ctrl+T        | Open theme picker           |

---

## 6. Platform Differences

### 6.1. macOS

On macOS, `Ctrl` is replaced with `Cmd` for app-level shortcuts:

| Windows/Linux        | macOS                        | Action                 |
|----------------------|------------------------------|------------------------|
| Ctrl+Shift+C         | Cmd+C                        | Copy                   |
| Ctrl+Shift+V         | Cmd+V                        | Paste                  |
| Ctrl+Shift+F         | Cmd+F                        | Find                   |
| Ctrl+Shift+T         | Cmd+T                        | New Session            |
| Ctrl+Shift+W         | Cmd+W                        | Close Session          |
| Ctrl+Shift+P         | Cmd+Shift+P                  | Command Palette        |
| Ctrl+,               | Cmd+,                        | Settings               |
| Ctrl+Q               | Cmd+Q                        | Quit                   |
| Ctrl+=               | Cmd+=                        | Zoom In                |
| Ctrl+-               | Cmd+-                        | Zoom Out               |
| Ctrl+0               | Cmd+0                        | Reset Zoom             |
| Ctrl+B               | Cmd+B                        | Toggle Left Sidebar    |
| Ctrl+1-9             | Cmd+1-9                      | Switch Session         |
| Ctrl+Tab             | Cmd+Tab (override)           | Next Session           |

**Special macOS notes:**
- Cmd+C/V are used directly (no Shift needed) because macOS clearly distinguishes Cmd vs Ctrl
- Ctrl+C still sends SIGINT normally (because macOS uses Ctrl for terminal signals)
- Cmd+Tab on macOS is typically the OS app switcher; Wit can use Ctrl+Tab or allow the user to rebind

### 6.2. Linux

- Uses all shortcuts as listed in the default table (Ctrl+Shift+... prefix)
- Super key is not used (to avoid conflicts with DE shortcuts)

### 6.3. Windows

- Same as Linux, uses Ctrl+Shift prefix
- F11 for fullscreen (same as other Windows apps)
- Alt+F4 still works to close the window (OS-level, no need to define)

---

## 7. Conflict Detection

### 7.1. Automatic Detection

When the user edits `keybindings.toml`, Wit will:
1. Parse the file
2. Check each new binding for conflicts with existing bindings
3. If there is a conflict: show a warning notification

### 7.2. Conflict Types

| Type                  | Description                    | Handling                 |
|-----------------------|--------------------------------|--------------------------|
| Duplicate binding     | 2 actions use the same shortcut | Warning, later binding overrides the earlier one |
| Shell conflict        | Binding conflicts with a shell shortcut (Ctrl+C, etc.) | Error, binding is rejected |
| OS conflict           | Binding conflicts with an OS shortcut (Alt+F4, etc.) | Warning, allowed but with a warning |
| Chord conflict        | Chord prefix conflicts with a single shortcut | Error, rejected |

### 7.3. Reserved Keys

The following keys **cannot be rebound** because they are shell-critical:

| Key         | Reason                            |
|-------------|-----------------------------------|
| Ctrl+C      | SIGINT - cancel process           |
| Ctrl+Z      | SIGTSTP - suspend process         |
| Ctrl+D      | EOF - close stdin/shell           |
| Ctrl+\      | SIGQUIT                           |
| Ctrl+S      | XOFF (flow control, can be disabled) |
| Ctrl+Q      | XON (flow control) - **except** when used as app quit |

**Exception:** Ctrl+Q defaults to "Quit Application". If the user wants Ctrl+Q forwarded to the shell (for XON), they can unbind it:
```toml
[app]
quit = "disabled"
```

---

## 8. Command Palette

### 8.1. Trigger

- **Shortcut**: Ctrl+Shift+P
- **UI**: popup in the center of the screen (similar to VS Code)

### 8.2. Features

- Search bar: type to filter commands
- Displays all available commands with their corresponding shortcuts
- Fuzzy search on command names
- Execute a command with Enter
- Dismiss with Escape

### 8.3. Command List

The command palette contains all executable actions, including:
- All keybinding actions
- Theme switching
- Settings sections
- Debug commands (developer mode)

---

## 9. Keyboard Shortcuts Reference UI

### 9.1. Shortcut Hints

- Tooltips on buttons display the shortcut (e.g., "New Session (Ctrl+Shift+T)")
- Menu items display the shortcut on the right
- Command palette displays the shortcut for each command

### 9.2. Cheat Sheet

- Accessible from Help > Keyboard Shortcuts or Ctrl+K Ctrl+S
- Displays all shortcuts, grouped by category
- Search/filter within the cheat sheet
- Link to keybindings.toml for editing

### 9.3. Visual Format

Keyboard shortcuts are displayed as key badges:

```
 Ctrl  +  Shift  +  T
```

- Each key is a separate badge
- Background: `--color-surface-active`
- Border: 1px solid `--color-border`
- Border-radius: `--radius-xs`
- Font: `--text-xs`, monospace
- Padding: 2px 6px
- Gap: 2px between key badges, "+" has no badge
