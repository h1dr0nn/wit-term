# Window Decoration

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Overview

Wit uses custom window chrome (window decoration) instead of the native OS title bar. This approach gives full control over the look, feel, and behavior of the window frame - including transparent backgrounds, custom window controls, and a unified menu bar that works consistently across macOS, Windows, and Linux.

The native title bar is disabled via Tauri's `decorations: false` configuration. All window management functionality (dragging, resizing, minimize, maximize, close) is re-implemented using Tauri v2 APIs and custom React components.

**Key decisions:**
- Custom decoration enables the transparent/blur header effect that defines Wit's visual identity
- Window controls adapt to each platform's conventions (users should feel at home)
- Menu bar is rendered inside the header, not in the OS system menu bar (with an opt-in macOS exception)
- All interactive elements are keyboard accessible

---

## 2. Window Controls

### 2.1. macOS - Traffic Lights

On macOS, window controls follow the standard traffic light pattern: three circular dots aligned to the left side of the header.

```
+--[ o  o  o ]--+------------------------------------------+
   red yel grn
```

#### Button Layout

| Button   | Position | Color (idle)  | Color (hover) | Action         |
|----------|----------|---------------|---------------|----------------|
| Close    | Left     | #FF5F57       | #FF5F57       | Close window   |
| Minimize | Center   | #FEBC2E       | #FEBC2E       | Minimize to dock |
| Maximize | Right    | #28C840       | #28C840       | Enter fullscreen or zoom |

#### Dimensions

| Property           | Value                         |
|--------------------|-------------------------------|
| Dot diameter       | 12px                          |
| Gap between dots   | 8px                           |
| Inset from left    | 12px                          |
| Vertical alignment | Centered in header            |

#### States

| State            | Appearance                     |
|------------------|--------------------------------|
| Idle             | Filled circles with platform colors |
| Hover (group)    | Icons appear inside dots (X, -, arrows) |
| Window unfocused | All dots become same muted gray (#6E7681) |
| Disabled         | Dot appears dimmed, no hover effect |

#### Behavior

| Interaction         | Result                        |
|---------------------|-------------------------------|
| Click close         | Closes the window (or hides to tray if configured) |
| Click minimize      | Minimizes window to dock      |
| Click maximize      | Toggles fullscreen            |
| Option+click maximize | Zoom (resize to fit content)|
| Hover over any dot  | All three dots show their icons |

#### Implementation

Tauri provides built-in traffic light positioning on macOS. Use the `setDecorations(false)` and render custom controls, or use Tauri's `titleBarStyle: 'overlay'` to let macOS draw the traffic lights natively while overlaying custom content.

Recommended approach: use `titleBarStyle: 'overlay'` on macOS so the traffic lights are native (users expect exact macOS behavior including force-click, Option-click, and right-click menus).

```json
{
  "windows": [{
    "titleBarStyle": "overlay",
    "hiddenTitle": true
  }]
}
```

With this setting, Tauri renders native traffic lights and hides the native title. Wit's custom header renders behind/around them.

### 2.2. Windows - Caption Buttons

On Windows, window controls follow the standard rectangular button pattern: three buttons aligned to the right side of the header.

```
+------------------------------------------+--[ _  []  X ]--+
                                              min max close
```

#### Button Layout

| Button   | Position | Icon            | Action           |
|----------|----------|-----------------|------------------|
| Minimize | Left     | Horizontal line | Minimize to taskbar |
| Maximize | Center   | Square outline  | Toggle maximize/restore |
| Close    | Right    | X mark          | Close window     |

When maximized, the maximize button icon changes to a "restore" icon (two overlapping squares).

#### Dimensions

| Property           | Value                         |
|--------------------|-------------------------------|
| Button width       | 46px                          |
| Button height      | Full header height (38px)     |
| Inset from right   | 0px (flush with window edge)  |
| Icon size          | 10px                          |

#### Styling

| Property              | Value                      |
|------------------------|----------------------------|
| Background (idle)      | Transparent                |
| Background (hover)     | `--color-surface-hover`    |
| Close hover background | #C42B1C (Windows red)      |
| Close hover icon color | #FFFFFF                    |
| Icon color (idle)      | `--color-text-secondary`   |
| Icon color (hover)     | `--color-text`             |
| Border radius          | 0 (sharp corners, per Windows convention) |

#### Behavior

| Interaction            | Result                     |
|------------------------|----------------------------|
| Click minimize         | Minimize to taskbar        |
| Click maximize         | Toggle maximize/restore    |
| Click close            | Close window               |
| Hover close            | Red background highlight   |
| Right-click title bar  | System menu (move, size, minimize, maximize, close) |

#### Implementation

Windows requires fully custom controls since `decorations: false` removes them entirely. Use Tauri v2 window APIs:

```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

// Button handlers
const minimize = () => appWindow.minimize();
const toggleMaximize = () => appWindow.toggleMaximize();
const close = () => appWindow.close();
```

### 2.3. Linux - Configurable Controls

On Linux, window control position varies by desktop environment. GNOME places controls on the right, some other DEs place them on the left.

#### Configuration

| Setting          | Options                       | Default |
|------------------|-------------------------------|---------|
| Control position | `left`, `right`               | `right` |
| Control style    | `circular`, `rectangular`     | `rectangular` |

Wit detects the desktop environment at startup and sets the default position accordingly:

| Desktop Environment | Default Position |
|---------------------|------------------|
| GNOME               | Right            |
| KDE Plasma          | Right            |
| XFCE                | Right            |
| Elementary OS       | Left             |
| Unity (legacy)      | Left             |

Users can override this in Settings > Appearance > Window Controls Position.

#### Styling

Linux controls follow the same styling as Windows rectangular buttons by default, but adopt macOS circular dots if the user selects `circular` style.

---

## 3. Transparent / Translucent Background

### 3.1. Design Intent

The window decoration (header area) uses a semi-transparent background with a blur effect. This means the user's desktop wallpaper or windows behind Wit are faintly visible through the header, creating a layered, modern appearance. This visual treatment is inspired by applications like Arc Browser, Linear, and Figma.

### 3.2. CSS Implementation

The base transparent effect is achieved with CSS:

```css
.window-decoration {
  background: rgba(13, 17, 23, 0.88);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}
```

| Property              | Value                      |
|------------------------|----------------------------|
| Base color             | #0D1117 (app background)   |
| Default opacity        | 0.88 (88%)                 |
| Blur radius            | 20px                       |
| Saturation             | 1.0 (no saturation boost)  |

### 3.3. Opacity Configuration

Users can adjust the header transparency level in Settings > Appearance.

| Setting          | Value Range | Default | Description                |
|------------------|-------------|---------|----------------------------|
| Header opacity   | 50% - 100% | 88%     | Controls header background transparency |
| Slider step      | 5%          | -       | Granularity of adjustment  |

At 100% the header is fully opaque (solid). At 50% it is strongly transparent and the desktop behind is clearly visible.

### 3.4. Native Platform Effects

For the best transparency quality, Wit leverages native platform compositing APIs through Tauri v2's window effects system.

#### macOS - Vibrancy

macOS provides `NSVisualEffectView` for native blur and vibrancy effects. Tauri v2 exposes these through the window effects API.

```rust
use tauri::window::{Effect, EffectState, EffectsBuilder};

let effects = EffectsBuilder::new()
    .effect(Effect::UnderWindowBackground)
    .state(EffectState::Active)
    .build();

window.set_effects(effects)?;
```

Available macOS effects (in order of preference for Wit):

| Effect                    | Description                  |
|---------------------------|------------------------------|
| `UnderWindowBackground`   | Blurs content behind the window, subtle |
| `HudWindow`              | Dark blur, suitable for dark themes |
| `Sidebar`                | Standard sidebar vibrancy    |

#### Windows - Mica / Acrylic

Windows 10 and 11 provide Acrylic and Mica material effects.

```rust
use tauri::window::{Effect, EffectsBuilder};

// Windows 11 - Mica (preferred, performance-friendly)
let effects = EffectsBuilder::new()
    .effect(Effect::Mica)
    .build();

// Windows 10/11 - Acrylic (heavier but available on Win10)
let effects = EffectsBuilder::new()
    .effect(Effect::Acrylic)
    .build();

window.set_effects(effects)?;
```

| Effect   | Windows Version | Performance | Notes                  |
|----------|-----------------|-------------|------------------------|
| Mica     | 11 only         | Excellent   | Preferred. Samples wallpaper, not windows behind. |
| Acrylic  | 10 and 11       | Moderate    | True blur of content behind window. |
| Tabbed   | 11 only         | Excellent   | Mica variant with tabbed appearance. |

#### Linux - CSS Only

Linux desktop compositors have varying support for transparency. Wit does not use a native transparency API on Linux and relies entirely on CSS `backdrop-filter`. This works on most modern compositors (Mutter, KWin with compositing) but may not work on older or lightweight window managers.

If `backdrop-filter` is not supported, Wit falls back to a solid background automatically.

### 3.5. Fallback Behavior

| Condition                           | Fallback                    |
|-------------------------------------|-----------------------------|
| Native effect unavailable           | CSS `backdrop-filter` only  |
| CSS `backdrop-filter` unsupported   | Solid background #0D1117    |
| OS "Reduce transparency" enabled    | Solid background #0D1117    |
| User sets opacity to 100%           | Solid background #0D1117    |
| Performance issue detected          | Disable blur, use solid     |

Wit checks the `prefers-reduced-transparency` CSS media query and the equivalent OS setting on startup.

---

## 4. Menu Bar

### 4.1. Overview

The menu bar is rendered inside the header, between the window controls (or left edge) and the title area. It contains five top-level menu categories: File, Edit, View, Session, and Help.

Menu bar styling is designed to be visible but not prominent. The terminal content area is the star - the menu bar is a supporting element that experienced users rarely interact with (they use keyboard shortcuts instead).

### 4.2. Menu Bar Styling

| Property         | Value                         |
|------------------|-------------------------------|
| Font             | Inter, 13px, regular weight   |
| Text color       | `--color-text-secondary` (#8B949E) |
| Hover text color | `--color-text` (#F0F6FC)      |
| Item padding     | 6px 12px                      |
| Hover background | `--color-surface-hover` (#1C2129) |
| Active/open bg   | `--color-primary-muted` (#58E6D920) |
| Border radius    | 4px                           |
| Gap between items| 2px                           |
| Transition       | background 100ms ease         |

### 4.3. Dropdown Panel Styling

When a menu category is clicked, a dropdown panel appears below it.

| Property         | Value                         |
|------------------|-------------------------------|
| Background       | `--color-surface` (#161B22)   |
| Border           | 1px solid `--color-border` (#30363D) |
| Border radius    | 8px                           |
| Box shadow       | `0 8px 24px rgba(0, 0, 0, 0.4)` |
| Min width        | 220px                         |
| Padding          | 4px 0 (top and bottom)        |
| Offset           | 4px below the menu bar item   |
| Animation        | Fade in + slight slide down, 120ms ease-out |

### 4.4. Menu Item Styling

| Property                | Value                      |
|-------------------------|----------------------------|
| Font                    | Inter, 13px, regular       |
| Padding                 | 6px 16px                   |
| Text color              | `--color-text` (#F0F6FC)   |
| Hover background        | `--color-surface-hover` (#1C2129) |
| Active background       | `--color-surface-active` (#22272E) |
| Disabled text color     | `--color-text-muted` (#6E7681) |
| Disabled hover          | None (no hover effect)     |
| Border radius           | 4px (with 4px horizontal margin) |
| Height                  | 32px                       |

### 4.5. Keyboard Accelerators in Menus

Keyboard shortcuts are displayed right-aligned within each menu item.

```
+-------------------------------+
| New Session    Ctrl+Shift+T   |
| New Window     Ctrl+Shift+N   |
|-------------------------------|
| Close Session  Ctrl+Shift+W   |
| Close Window   Ctrl+Q         |
+-------------------------------+
```

| Property              | Value                      |
|------------------------|----------------------------|
| Accelerator font       | Inter, 12px, regular      |
| Accelerator color      | `--color-text-secondary` (#8B949E) |
| Gap from label         | Minimum 24px              |
| Alignment              | Right-aligned within item |

On macOS, shortcuts display platform symbols: Cmd instead of Ctrl, Opt instead of Alt.

### 4.6. Menu Separators

| Property         | Value                         |
|------------------|-------------------------------|
| Height           | 1px                           |
| Color            | `--color-border-muted` (#21262D) |
| Margin           | 4px 8px (vertical, horizontal)|

### 4.7. Menu Interaction Behavior

| Interaction            | Result                     |
|------------------------|----------------------------|
| Click menu category    | Opens dropdown             |
| Click while open       | Closes dropdown            |
| Hover adjacent category| Switches to that dropdown  |
| Click menu item        | Executes action, closes menu |
| Click outside menu     | Closes dropdown            |
| Escape                 | Closes dropdown            |
| Arrow Down             | Moves focus to next item   |
| Arrow Up               | Moves focus to previous item |
| Arrow Right            | Opens next menu category   |
| Arrow Left             | Opens previous menu category |
| Enter                  | Activates focused item     |
| Alt+letter             | Opens menu by mnemonic (e.g., Alt+F for File) |

---

## 5. Menu Contents

### 5.1. File Menu

| Item             | Shortcut         | Description                  |
|------------------|------------------|------------------------------|
| New Session      | Ctrl+Shift+T     | Creates a new terminal session in the current window |
| New Window       | Ctrl+Shift+N     | Opens a new Wit window       |
| ---              | -                | Separator                    |
| Close Session    | Ctrl+Shift+W     | Closes the active terminal session |
| Close Window     | Ctrl+Q           | Closes the current window    |
| ---              | -                | Separator                    |
| Settings         | Ctrl+,           | Opens the settings panel     |
| ---              | -                | Separator                    |
| Exit             | Alt+F4           | Exits the application entirely (closes all windows) |

Notes:
- "Close Session" prompts if there is a running process in the session
- "Close Window" prompts if there are multiple sessions open
- "Exit" prompts if any sessions have running processes
- On macOS, "Exit" is replaced by "Quit Wit" (Cmd+Q) and appears under the application menu

### 5.2. Edit Menu

| Item             | Shortcut         | Description                  |
|------------------|------------------|------------------------------|
| Copy             | Ctrl+Shift+C     | Copies selected text from terminal |
| Paste            | Ctrl+Shift+V     | Pastes clipboard content into terminal |
| Select All       | Ctrl+Shift+A     | Selects all text in the terminal scrollback |
| ---              | -                | Separator                    |
| Find             | Ctrl+Shift+F     | Opens the search/find bar within the terminal |
| ---              | -                | Separator                    |
| Clear Terminal   | Ctrl+L           | Sends the `clear` command to the shell |
| Clear Scrollback | Ctrl+Shift+K     | Clears the terminal scrollback buffer entirely |

Notes:
- Terminal shortcuts use Ctrl+Shift instead of plain Ctrl to avoid conflicts with shell shortcuts (Ctrl+C for SIGINT, Ctrl+V for literal input in some shells)
- On macOS, Cmd replaces Ctrl+Shift for Copy/Paste (Cmd+C, Cmd+V), matching platform convention
- "Clear Terminal" sends the literal clear command; "Clear Scrollback" programmatically empties the buffer

### 5.3. View Menu

| Item                 | Shortcut       | Description                  |
|----------------------|----------------|------------------------------|
| Toggle Left Sidebar  | Ctrl+B         | Shows or hides the session sidebar |
| Toggle Right Sidebar | Ctrl+Shift+B   | Shows or hides the AI context sidebar |
| ---                  | -              | Separator                    |
| Zoom In              | Ctrl+=         | Increases terminal font size |
| Zoom Out             | Ctrl+-         | Decreases terminal font size |
| Reset Zoom           | Ctrl+0         | Resets terminal font size to default |
| ---                  | -              | Separator                    |
| Toggle Fullscreen    | F11            | Enters or exits fullscreen mode |
| ---                  | -              | Separator                    |
| Command Palette      | Ctrl+Shift+P   | Opens the command palette    |

Notes:
- Zoom affects only the terminal font size, not the UI elements
- On macOS, fullscreen uses the native fullscreen (green traffic light) behavior
- Ctrl+= also responds to Ctrl+Shift+= (since = and + share a key)

### 5.4. Session Menu

| Item              | Shortcut        | Description                  |
|-------------------|-----------------|------------------------------|
| Next Session      | Ctrl+Tab        | Switches to the next session in the list |
| Previous Session  | Ctrl+Shift+Tab  | Switches to the previous session |
| ---               | -               | Separator                    |
| Session 1         | Ctrl+1          | Switches directly to session 1 |
| Session 2         | Ctrl+2          | Switches directly to session 2 |
| Session 3         | Ctrl+3          | Switches directly to session 3 |
| Session 4         | Ctrl+4          | Switches directly to session 4 |
| Session 5         | Ctrl+5          | Switches directly to session 5 |
| Session 6         | Ctrl+6          | Switches directly to session 6 |
| Session 7         | Ctrl+7          | Switches directly to session 7 |
| Session 8         | Ctrl+8          | Switches directly to session 8 |
| Session 9         | Ctrl+9          | Switches directly to session 9 |
| ---               | -               | Separator                    |
| Rename Session    | -               | Opens inline rename for active session |
| Duplicate Session | -               | Creates a copy of the active session |
| ---               | -               | Separator                    |
| Split Horizontal  | -               | (Future) Splits pane horizontally |
| Split Vertical    | -               | (Future) Splits pane vertically |

Notes:
- Session 1-9 items are dynamic: they show the actual session names and are disabled/hidden if fewer than N sessions exist
- "Duplicate Session" creates a new session with the same shell, working directory, and environment variables
- Split items are shown as disabled with "(coming soon)" label until the split pane feature is implemented

### 5.5. Help Menu

| Item                         | Shortcut | Description                 |
|------------------------------|----------|-----------------------------|
| Documentation                | -        | Opens Wit documentation in default browser |
| Keyboard Shortcuts Reference | -        | Opens a modal overlay listing all keyboard shortcuts |
| ---                          | -        | Separator                   |
| Release Notes                | -        | Opens release notes for current version |
| Check for Updates            | -        | Checks for new versions of Wit |
| ---                          | -        | Separator                   |
| Report Issue                 | -        | Opens GitHub Issues page in default browser |
| ---                          | -        | Separator                   |
| About Wit                    | -        | Shows version, build, and license information |

Notes:
- "Check for Updates" uses Tauri's built-in updater if configured
- "About Wit" shows a small modal with: version number, build date, Tauri version, Rust version, license (MIT or similar), and a link to the repository
- On macOS, "About Wit" is conventionally placed in the application menu (before Settings)

---

## 6. Center Area / Title Display

### 6.1. Content

The center area of the header displays the active session's title. It fills the space between the menu bar and the action buttons.

| State              | Display                        |
|--------------------|--------------------------------|
| No sessions        | "Wit"                          |
| Session active     | Session title (user-set or default) |
| With CWD enabled   | Session title + working directory below |

### 6.2. Default Session Titles

When a new session is created, it receives a default title:

| Rule               | Example                        |
|--------------------|--------------------------------|
| Shell name         | "bash", "zsh", "PowerShell"    |
| Numbered           | "Session 1", "Session 2"      |

The default naming scheme is configurable in Settings.

### 6.3. Title Interaction

Clicking the title enters inline rename mode. See `header.md` section 7.3 and 7.4 for full interaction details.

### 6.4. Truncation

| Property              | Value                      |
|------------------------|----------------------------|
| Max display width      | Available space between menu bar and actions |
| Overflow behavior      | `text-overflow: ellipsis`  |
| Truncation direction   | End (right side)           |
| Minimum visible chars  | 8 characters before truncation |

---

## 7. Right Action Buttons

### 7.1. Buttons

Optional icon-only buttons in the right area of the header, before the window controls (on Windows) or at the right edge (on macOS).

| Button           | Icon         | Tooltip                    | Shortcut       |
|------------------|--------------|----------------------------|----------------|
| Command Palette  | Magnifying glass | "Command Palette (Ctrl+Shift+P)" | Ctrl+Shift+P |
| Settings         | Gear         | "Settings (Ctrl+,)"       | Ctrl+,         |

### 7.2. Styling

| Property         | Value                         |
|------------------|-------------------------------|
| Button size      | 28px x 28px                   |
| Icon size        | 16px                          |
| Icon color       | `--color-text-muted` (#6E7681) |
| Hover icon color | `--color-text` (#F0F6FC)      |
| Hover background | `--color-surface-hover` (#1C2129) |
| Active background| `--color-surface-active` (#22272E) |
| Border radius    | 6px                           |
| Gap between      | 4px                           |
| Focus outline    | 2px solid `--color-primary`   |

### 7.3. Visibility

These buttons are hidden when the window width drops below 600px (see responsive behavior in `header.md` section 11). Their functionality is still accessible via keyboard shortcuts or the command palette.

---

## 8. Platform-Specific Behavior

### 8.1. macOS

| Aspect             | Behavior                      |
|--------------------|-------------------------------|
| Title bar style    | `titleBarStyle: 'overlay'` (native traffic lights, custom content) |
| Traffic lights     | Native, positioned by macOS   |
| Menu bar           | Custom in-header (default) or native system menu bar (opt-in) |
| Fullscreen         | Native macOS fullscreen with slide-up animation |
| Vibrancy           | `NSVisualEffectView` via Tauri effects API |
| Cmd key            | Replaces Ctrl in all shortcuts |
| About menu         | Under application menu (macOS convention) |
| Quit               | Under application menu as "Quit Wit" (Cmd+Q) |

When the user opts in to the native macOS menu bar:
- The custom menu bar zone in the header is hidden
- Menus appear in the system menu bar at the top of the screen
- The header title area expands to fill the freed space
- This setting is stored in user preferences

### 8.2. Windows

| Aspect             | Behavior                      |
|--------------------|-------------------------------|
| Title bar          | `decorations: false` (fully custom) |
| Window controls    | Custom rectangular buttons on right |
| Menu bar           | Custom in-header only         |
| Fullscreen         | Borderless fullscreen (F11)   |
| Transparency       | Mica (Win11) or Acrylic (Win10) via Tauri effects |
| Snap layouts       | Hover maximize button shows Win11 snap layout picker |
| System menu        | Right-click header shows system menu (Move, Size, Minimize, Maximize, Close) |

#### Windows Snap Layout Support

On Windows 11, hovering over the maximize button should trigger the snap layout flyout. This requires the window to respond to `WM_NCHITTEST` with `HTMAXBUTTON` when the cursor is over the maximize button area. Tauri v2 may handle this automatically; otherwise, a custom plugin is needed.

### 8.3. Linux

| Aspect             | Behavior                      |
|--------------------|-------------------------------|
| Title bar          | `decorations: false` (fully custom) |
| Window controls    | Custom, position configurable (left or right) |
| Menu bar           | Custom in-header only         |
| Fullscreen         | _NET_WM_STATE_FULLSCREEN      |
| Transparency       | CSS `backdrop-filter` only (no native API) |
| DE detection       | Read `XDG_CURRENT_DESKTOP` or `DESKTOP_SESSION` env var |

#### Wayland Considerations

On Wayland, window dragging uses `xdg_toplevel.move` via Tauri, and there are no global window coordinates. Custom window decoration works well on Wayland since client-side decoration is the standard approach.

On X11, window dragging uses `_NET_WM_MOVERESIZE`. Both protocols are handled by Tauri's `data-tauri-drag-region`.

---

## 9. Window Frame and Borders

### 9.1. Window Border

When the window is not maximized, a subtle border is rendered around the entire window.

| Property         | Value                         |
|------------------|-------------------------------|
| Border width     | 1px                           |
| Border color     | `--color-border-muted` (#21262D) |
| Border radius    | 10px (rounded corners)        |
| Maximized radius | 0px (sharp corners when maximized) |

### 9.2. Window Shadow

The window has a drop shadow for visual separation from the desktop.

| Property         | Value                         |
|------------------|-------------------------------|
| Shadow           | `0 16px 48px rgba(0, 0, 0, 0.35)` |
| Maximized shadow | None                          |

Note: Window shadow is provided by the OS compositor on most platforms. Tauri windows inherit the platform's default shadow behavior. The CSS shadow above is a reference for the intended appearance.

### 9.3. Resize Handles

When the window is not maximized, the edges and corners of the window are resize handles.

| Zone             | Cursor          | Hit area        |
|------------------|-----------------|-----------------|
| Top edge         | `ns-resize`     | 4px             |
| Bottom edge      | `ns-resize`     | 4px             |
| Left edge        | `ew-resize`     | 4px             |
| Right edge       | `ew-resize`     | 4px             |
| Corners          | `nwse-resize` / `nesw-resize` | 8px x 8px |

Tauri handles window resizing natively once `decorations: false` is set and the window has a resizable configuration.

---

## 10. Design Philosophy

### 10.1. Guiding Principles

| Principle          | Application                   |
|--------------------|-------------------------------|
| Terminal is the star | The header and window chrome are minimal so the terminal content dominates the visual hierarchy |
| Clean, not cluttered | Menu bar text is muted, action buttons are small, no unnecessary decoration |
| Premium feel        | Transparent background with blur creates a modern, high-quality impression |
| Platform respect    | Window controls and behavior match what users expect on their OS |
| Keyboard first      | Menus exist for discoverability, but users primarily use keyboard shortcuts |
| Functional          | Every element has a purpose; no purely decorative elements in the header |

### 10.2. Visual Hierarchy (Header)

1. **Session title** - most prominent text in the center
2. **Window controls** - visible but conventional, users know where they are
3. **Menu bar items** - muted text, visible on hover
4. **Action buttons** - subtle icon-only buttons

---

## 11. Accessibility

| Feature              | Implementation                |
|----------------------|-------------------------------|
| ARIA roles           | `role="menubar"` on menu container, `role="menu"` on dropdowns, `role="menuitem"` on items |
| Keyboard navigation  | Full menu traversal with arrow keys, Enter, Escape |
| Focus management     | Focus trapped within open dropdown, returned to menu bar on close |
| Screen reader        | Menu items announce label + shortcut + disabled state |
| Reduced motion       | Dropdown appears instantly (no animation) |
| Reduced transparency | Solid background, no blur     |
| High contrast        | Increased border visibility, stronger text contrast |
| Focus indicators     | 2px solid `--color-primary` ring on all interactive elements |

### 11.1. Tab Order

Within the header, the tab order is:

1. Window controls (if focusable - macOS native controls handle their own focus)
2. Menu bar categories (File, Edit, View, Session, Help)
3. Title area (clickable for rename)
4. Action buttons (Command Palette, Settings)

---

## 12. Implementation Notes

### 12.1. Tauri Configuration

```json
{
  "app": {
    "windows": [
      {
        "title": "Wit",
        "width": 1200,
        "height": 800,
        "minWidth": 480,
        "minHeight": 320,
        "decorations": false,
        "transparent": true,
        "titleBarStyle": "overlay"
      }
    ]
  }
}
```

Note: `titleBarStyle: "overlay"` is macOS-specific. On Windows and Linux, `decorations: false` is the relevant setting. Tauri handles the platform branching.

### 12.2. React Component Structure

```
WindowDecoration
  +-- TrafficLights (macOS only)
  +-- MenuBar
  |     +-- MenuCategory ("File")
  |     |     +-- MenuDropdown
  |     |           +-- MenuItem ("New Session")
  |     |           +-- MenuItem ("New Window")
  |     |           +-- MenuSeparator
  |     |           +-- MenuItem ("Close Session")
  |     |           +-- ...
  |     +-- MenuCategory ("Edit")
  |     +-- MenuCategory ("View")
  |     +-- MenuCategory ("Session")
  |     +-- MenuCategory ("Help")
  +-- TitleArea
  |     +-- SessionTitle
  |     +-- WorkingDirectoryLabel (optional)
  +-- ActionButtons
  |     +-- IconButton (Command Palette)
  |     +-- IconButton (Settings)
  +-- WindowControls (Windows/Linux only)
```

### 12.3. Key Tauri v2 APIs

| API                           | Purpose                      |
|-------------------------------|------------------------------|
| `getCurrentWindow().minimize()` | Minimize window            |
| `getCurrentWindow().toggleMaximize()` | Toggle maximize/restore |
| `getCurrentWindow().close()`   | Close window                |
| `getCurrentWindow().setEffects()` | Apply vibrancy/mica/acrylic |
| `getCurrentWindow().isMaximized()` | Check maximize state (for icon toggle) |
| `getCurrentWindow().onResized()` | Listen for resize events   |
| `data-tauri-drag-region`       | Make element draggable      |

### 12.4. Performance Considerations

| Concern            | Mitigation                    |
|--------------------|-------------------------------|
| Backdrop blur cost | Limit blur to header area only (38px height), not full window |
| Native effects     | Prefer Mica over Acrylic on Windows (lower GPU cost) |
| Menu rendering     | Lazy render dropdowns (only mount when opened) |
| Resize events      | Debounce resize handlers (100ms) |
| Animation          | Use CSS transforms and opacity only (GPU-composited properties) |

### 12.5. Testing Checklist

| Test Case                          | Platforms        |
|------------------------------------|------------------|
| Window drag by header              | All              |
| Double-click to maximize/restore   | All              |
| Window controls (minimize/maximize/close) | All       |
| Menu open/close/navigate           | All              |
| Keyboard menu traversal            | All              |
| Transparency/blur effect visible   | macOS, Windows   |
| Fallback solid background          | Linux (no compositor) |
| Snap layouts on hover maximize     | Windows 11       |
| Native traffic lights              | macOS            |
| Session rename via title click     | All              |
| Responsive menu collapse           | All              |
| Screen reader announcement         | All              |
| Unfocused window state             | All              |
| Fullscreen header behavior         | All              |
