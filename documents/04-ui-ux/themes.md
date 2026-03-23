# Theming System

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Overview

Wit supports a flexible theming system that allows users to customize the colors of the terminal and UI. Themes are defined using TOML files, support custom themes, and support hot-reloading.

---

## 2. Theme File Format

### 2.1. Format

Themes use the **TOML** format because:
- Easy to read and write for users
- Supports comments (unlike JSON)
- Popular in the Rust ecosystem (consistent with other app configs)
- No brackets required like JSON/YAML

### 2.2. File Location

| Location                        | Description                    |
|---------------------------------|--------------------------------|
| Built-in themes                 | Embedded in app binary         |
| User themes (Linux)             | `~/.config/wit/themes/`        |
| User themes (macOS)             | `~/Library/Application Support/wit/themes/` |
| User themes (Windows)           | `%APPDATA%\wit\themes\`        |

### 2.3. File Extension

- `.toml` (e.g., `dracula.toml`, `my-custom-theme.toml`)

---

## 3. Theme Structure

### 3.1. Schema

```toml
[metadata]
name = "Theme Name"
author = "Author Name"
version = "1.0.0"
description = "A brief description of the theme."
variant = "dark"  # "dark" or "light"

[terminal]
foreground = "#F0F6FC"
background = "#0D1117"
cursor = "#D4A857"
selection_foreground = "#F0F6FC"
selection_background = "#58E6D94D"

# ANSI standard colors (0-7)
black   = "#484F58"
red     = "#F85149"
green   = "#3FB950"
yellow  = "#D29922"
blue    = "#58A6FF"
magenta = "#BC8CFF"
cyan    = "#58E6D9"
white   = "#F0F6FC"

# ANSI bright colors (8-15)
bright_black   = "#6E7681"
bright_red     = "#FF7B72"
bright_green   = "#56D364"
bright_yellow  = "#E3B341"
bright_blue    = "#79C0FF"
bright_magenta = "#D2A8FF"
bright_cyan    = "#76EDE4"
bright_white   = "#FFFFFF"

# Extended 256-color support (optional, uncomment to override)
# [terminal.extended_colors]
# 16 = "#000000"
# 17 = "#00005F"
# ... (up to 255)

[ui]
background = "#0D1117"
surface = "#161B22"
surface_hover = "#1C2129"
surface_active = "#22272E"
border = "#30363D"
border_muted = "#21262D"

text = "#F0F6FC"
text_secondary = "#8B949E"
text_muted = "#6E7681"

primary = "#58E6D9"
primary_hover = "#6AEEE2"
accent = "#D4A857"
accent_hover = "#E0B96A"

success = "#3FB950"
warning = "#D29922"
error = "#F85149"
info = "#58A6FF"

[ui.scrollbar]
thumb = "#30363D"
thumb_hover = "#6E7681"
track = "transparent"
```

### 3.2. Required vs Optional Fields

- **Required**: `[metadata]` (name, variant), `[terminal]` (foreground, background, 16 ANSI colors)
- **Optional**: `[ui]` section (if missing, uses default dark/light based on variant), cursor, selection, extended_colors, scrollbar

---

## 4. Built-in Themes

### 4.1. Wit Dark (Default)

The default Wit theme, designed for this project.

```toml
[metadata]
name = "Wit Dark"
author = "Wit Team"
version = "1.0.0"
description = "The default dark theme for Wit terminal."
variant = "dark"

[terminal]
foreground = "#F0F6FC"
background = "#0D1117"
cursor = "#D4A857"
selection_foreground = "#F0F6FC"
selection_background = "#58E6D94D"

black   = "#484F58"
red     = "#F85149"
green   = "#3FB950"
yellow  = "#D29922"
blue    = "#58A6FF"
magenta = "#BC8CFF"
cyan    = "#58E6D9"
white   = "#F0F6FC"

bright_black   = "#6E7681"
bright_red     = "#FF7B72"
bright_green   = "#56D364"
bright_yellow  = "#E3B341"
bright_blue    = "#79C0FF"
bright_magenta = "#D2A8FF"
bright_cyan    = "#76EDE4"
bright_white   = "#FFFFFF"

[ui]
background = "#0D1117"
surface = "#161B22"
surface_hover = "#1C2129"
surface_active = "#22272E"
border = "#30363D"
border_muted = "#21262D"

text = "#F0F6FC"
text_secondary = "#8B949E"
text_muted = "#6E7681"

primary = "#58E6D9"
primary_hover = "#6AEEE2"
accent = "#D4A857"
accent_hover = "#E0B96A"

success = "#3FB950"
warning = "#D29922"
error = "#F85149"
info = "#58A6FF"
```

### 4.2. Wit Light

```toml
[metadata]
name = "Wit Light"
author = "Wit Team"
version = "1.0.0"
description = "Light variant of the Wit theme."
variant = "light"

[terminal]
foreground = "#1F2328"
background = "#FFFFFF"
cursor = "#D4A857"
selection_foreground = "#1F2328"
selection_background = "#58E6D940"

black   = "#1F2328"
red     = "#CF222E"
green   = "#1A7F37"
yellow  = "#9A6700"
blue    = "#0969DA"
magenta = "#8250DF"
cyan    = "#1B7C83"
white   = "#6E7781"

bright_black   = "#57606A"
bright_red     = "#E16F76"
bright_green   = "#4AC26B"
bright_yellow  = "#BF8700"
bright_blue    = "#218BFF"
bright_magenta = "#A475F9"
bright_cyan    = "#3192AA"
bright_white   = "#8C959F"

[ui]
background = "#FFFFFF"
surface = "#F6F8FA"
surface_hover = "#EAEEF2"
surface_active = "#D0D7DE"
border = "#D0D7DE"
border_muted = "#E6E8EB"

text = "#1F2328"
text_secondary = "#656D76"
text_muted = "#8B949E"

primary = "#1B7C83"
primary_hover = "#15666D"
accent = "#D4A857"
accent_hover = "#C49A4E"

success = "#1A7F37"
warning = "#9A6700"
error = "#CF222E"
info = "#0969DA"
```

### 4.3. Dracula

```toml
[metadata]
name = "Dracula"
author = "Zeno Rocha (adapted)"
version = "1.0.0"
description = "The famous Dracula color scheme."
variant = "dark"

[terminal]
foreground = "#F8F8F2"
background = "#282A36"
cursor = "#F8F8F2"
selection_foreground = "#F8F8F2"
selection_background = "#44475A"

black   = "#21222C"
red     = "#FF5555"
green   = "#50FA7B"
yellow  = "#F1FA8C"
blue    = "#BD93F9"
magenta = "#FF79C6"
cyan    = "#8BE9FD"
white   = "#F8F8F2"

bright_black   = "#6272A4"
bright_red     = "#FF6E6E"
bright_green   = "#69FF94"
bright_yellow  = "#FFFFA5"
bright_blue    = "#D6ACFF"
bright_magenta = "#FF92DF"
bright_cyan    = "#A4FFFF"
bright_white   = "#FFFFFF"
```

### 4.4. Solarized Dark

```toml
[metadata]
name = "Solarized Dark"
author = "Ethan Schoonover (adapted)"
version = "1.0.0"
description = "Solarized Dark color scheme."
variant = "dark"

[terminal]
foreground = "#839496"
background = "#002B36"
cursor = "#839496"
selection_foreground = "#93A1A1"
selection_background = "#073642"

black   = "#073642"
red     = "#DC322F"
green   = "#859900"
yellow  = "#B58900"
blue    = "#268BD2"
magenta = "#D33682"
cyan    = "#2AA198"
white   = "#EEE8D5"

bright_black   = "#002B36"
bright_red     = "#CB4B16"
bright_green   = "#586E75"
bright_yellow  = "#657B83"
bright_blue    = "#839496"
bright_magenta = "#6C71C4"
bright_cyan    = "#93A1A1"
bright_white   = "#FDF6E3"
```

### 4.5. Solarized Light

```toml
[metadata]
name = "Solarized Light"
author = "Ethan Schoonover (adapted)"
version = "1.0.0"
description = "Solarized Light color scheme."
variant = "light"

[terminal]
foreground = "#657B83"
background = "#FDF6E3"
cursor = "#657B83"
selection_foreground = "#586E75"
selection_background = "#EEE8D5"

black   = "#073642"
red     = "#DC322F"
green   = "#859900"
yellow  = "#B58900"
blue    = "#268BD2"
magenta = "#D33682"
cyan    = "#2AA198"
white   = "#EEE8D5"

bright_black   = "#002B36"
bright_red     = "#CB4B16"
bright_green   = "#586E75"
bright_yellow  = "#657B83"
bright_blue    = "#839496"
bright_magenta = "#6C71C4"
bright_cyan    = "#93A1A1"
bright_white   = "#FDF6E3"
```

### 4.6. One Dark

```toml
[metadata]
name = "One Dark"
author = "Atom (adapted)"
version = "1.0.0"
description = "Atom One Dark inspired theme."
variant = "dark"

[terminal]
foreground = "#ABB2BF"
background = "#282C34"
cursor = "#528BFF"
selection_foreground = "#ABB2BF"
selection_background = "#3E4451"

black   = "#282C34"
red     = "#E06C75"
green   = "#98C379"
yellow  = "#E5C07B"
blue    = "#61AFEF"
magenta = "#C678DD"
cyan    = "#56B6C2"
white   = "#ABB2BF"

bright_black   = "#5C6370"
bright_red     = "#E06C75"
bright_green   = "#98C379"
bright_yellow  = "#E5C07B"
bright_blue    = "#61AFEF"
bright_magenta = "#C678DD"
bright_cyan    = "#56B6C2"
bright_white   = "#FFFFFF"
```

---

## 5. Hot-Reloading

### 5.1. Mechanism

- **File watcher**: watches theme directory for file changes
- **Trigger**: when a `.toml` file in the theme directory changes (on save)
- **Process**:
  1. Detect file change
  2. Parse TOML file
  3. Validate theme structure
  4. Apply new colors (update CSS custom properties)
  5. Re-render terminal with new colors
- **Debounce**: 200ms to avoid multiple reloads when editor saves in multiple steps

### 5.2. Error Handling

- If theme file is invalid: keep current theme, show error notification
- Error notification: toast message "Theme reload failed: {error details}"
- Log detailed error to console/debug log

### 5.3. Performance

- Theme switch/reload: < 50ms
- Does not re-render entire terminal, only updates color values
- CSS custom properties ensure instant propagation

---

## 6. Custom Theme Creation

### 6.1. How to Create a New Theme

1. **Copy template**: copy a built-in theme file as a base
2. **Name it**: save file to the theme directory with the name `my-theme.toml`
3. **Edit**: modify the color values in the TOML file
4. **Apply**: select the theme in Settings > Appearance > Theme
5. **Iterate**: edit and save the file, the theme auto-reloads

### 6.2. Theme Template

```toml
# My Custom Theme
# Copy this file to ~/.config/wit/themes/my-theme.toml
# Edit the color values to your liking

[metadata]
name = "My Custom Theme"
author = "Your Name"
version = "1.0.0"
description = "Describe your theme here."
variant = "dark"  # "dark" or "light"

[terminal]
# Main terminal colors
foreground = "#FFFFFF"       # Default text color
background = "#1A1B26"       # Terminal background color
cursor = "#C0CAF5"           # Cursor color
selection_foreground = "#FFFFFF"
selection_background = "#33467C"

# 16 ANSI colors - all must be defined
# Normal colors (0-7)
black   = "#15161E"
red     = "#F7768E"
green   = "#9ECE6A"
yellow  = "#E0AF68"
blue    = "#7AA2F7"
magenta = "#BB9AF7"
cyan    = "#7DCFFF"
white   = "#A9B1D6"

# Bright colors (8-15)
bright_black   = "#414868"
bright_red     = "#F7768E"
bright_green   = "#9ECE6A"
bright_yellow  = "#E0AF68"
bright_blue    = "#7AA2F7"
bright_magenta = "#BB9AF7"
bright_cyan    = "#7DCFFF"
bright_white   = "#C0CAF5"

# [ui] section is optional
# If omitted, Wit will use default UI colors based on variant
# [ui]
# background = "#1A1B26"
# surface = "#24283B"
# ...
```

### 6.3. Tips for Creating Themes

- **Start from an existing theme**: no need to start from scratch, copy and modify
- **Check contrast**: ensure text is readable on the background (ratio >= 4.5:1)
- **Test with `colortest`**: run a colortest script in the terminal to see all 16 colors
- **Test bold/italic**: some themes need bright color adjustments for bold text to look good
- **Test with apps**: try running vim, htop, ls --color to ensure the theme works well

---

## 7. CSS Custom Properties Mapping

### 7.1. Mapping Table

Theme TOML values are mapped to CSS custom properties for frontend use:

| TOML Key                    | CSS Variable                 |
|-----------------------------|------------------------------|
| `terminal.foreground`       | `--term-fg`                  |
| `terminal.background`       | `--term-bg`                  |
| `terminal.cursor`           | `--term-cursor`              |
| `terminal.selection_foreground` | `--term-selection-fg`    |
| `terminal.selection_background` | `--term-selection-bg`    |
| `terminal.black`            | `--term-ansi-0`              |
| `terminal.red`              | `--term-ansi-1`              |
| `terminal.green`            | `--term-ansi-2`              |
| `terminal.yellow`           | `--term-ansi-3`              |
| `terminal.blue`             | `--term-ansi-4`              |
| `terminal.magenta`          | `--term-ansi-5`              |
| `terminal.cyan`             | `--term-ansi-6`              |
| `terminal.white`            | `--term-ansi-7`              |
| `terminal.bright_black`     | `--term-ansi-8`              |
| `terminal.bright_red`       | `--term-ansi-9`              |
| `terminal.bright_green`     | `--term-ansi-10`             |
| `terminal.bright_yellow`    | `--term-ansi-11`             |
| `terminal.bright_blue`      | `--term-ansi-12`             |
| `terminal.bright_magenta`   | `--term-ansi-13`             |
| `terminal.bright_cyan`      | `--term-ansi-14`             |
| `terminal.bright_white`     | `--term-ansi-15`             |
| `ui.background`             | `--color-bg`                 |
| `ui.surface`                | `--color-surface`            |
| `ui.text`                   | `--color-text`               |
| `ui.primary`                | `--color-primary`            |
| `ui.accent`                 | `--color-accent`             |
| (etc.)                      | (similarly for other design tokens) |

### 7.2. Runtime Application

```typescript
// Pseudocode: apply theme to CSS custom properties
function applyTheme(theme: Theme) {
  const root = document.documentElement;

  // Terminal colors
  root.style.setProperty('--term-fg', theme.terminal.foreground);
  root.style.setProperty('--term-bg', theme.terminal.background);
  root.style.setProperty('--term-cursor', theme.terminal.cursor);

  // ANSI colors
  const ansiNames = ['black', 'red', 'green', 'yellow', 'blue', 'magenta', 'cyan', 'white'];
  ansiNames.forEach((name, i) => {
    root.style.setProperty(`--term-ansi-${i}`, theme.terminal[name]);
    root.style.setProperty(`--term-ansi-${i + 8}`, theme.terminal[`bright_${name}`]);
  });

  // UI colors (if provided)
  if (theme.ui) {
    root.style.setProperty('--color-bg', theme.ui.background);
    root.style.setProperty('--color-surface', theme.ui.surface);
    // ... etc
  }
}
```

---

## 8. Terminal Color Compliance

### 8.1. Test Scripts

To ensure the theme displays correctly, run the following test scripts:

#### Basic 16-Color Test
```bash
# Display 16 ANSI colors
for i in {0..15}; do
  printf "\e[48;5;${i}m  %3d  \e[0m" "$i"
  [ $((($i + 1) % 8)) -eq 0 ] && echo
done
```

#### 256-Color Test
```bash
# Display 256 colors
for i in {0..255}; do
  printf "\e[48;5;${i}m %3d \e[0m" "$i"
  [ $((($i + 1) % 16)) -eq 0 ] && echo
done
```

#### True Color Test
```bash
# Display true color gradient
awk 'BEGIN{
  for (i = 0; i <= 255; i++) {
    printf "\033[48;2;%d;0;0m \033[0m", i;
  }
  print "";
}'
```

### 8.2. Contrast Checking

- Use an online tool (e.g., webaim.org/resources/contrastchecker/) to check
- Foreground vs background: >= 4.5:1 (WCAG AA)
- Each ANSI color vs background: >= 3:1 (minimum for readability)
- Cursor vs background: >= 3:1

### 8.3. Known Issues

- Bright colors too bright on light themes: need adjustment
- Red on dark background: ensure it is not overly muted
- Yellow on light background: typically hard to read, need a darker shade
- Blue on dark background: ensure it is not too dark

---

## 9. Theme Selection UI

### 9.1. Settings Page

- Location: Settings > Appearance > Theme
- Dropdown or grid showing available themes
- Preview: shows a small terminal preview with the selected theme
- "Open Theme Directory" button: opens file manager at the theme directory
- "Create Custom Theme" button: creates a new template file

### 9.2. Quick Switch

- Command Palette (Ctrl+Shift+P): type "theme" to quickly change
- Preview on hover: when moving over a theme in the list, temporarily apply it for preview
- Confirm with Enter, cancel with Escape (returns to previous theme)
