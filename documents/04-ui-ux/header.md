# Header / Title Bar

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Overview

The header is the topmost element of the Wit window. It sits above the terminal content area and both sidebars, spanning the full width of the application window. Because Wit uses a custom title bar (Tauri `decorations: false`), the header replaces the native OS title bar entirely and is responsible for window controls, menu access, title display, and window dragging.

The header is designed to be compact and unobtrusive. The terminal content is the primary focus of the application - the header exists to support it, not to compete with it. A semi-transparent background with a blur effect gives the header a modern, premium feel while keeping it visually lightweight.

**Design principles:**
- **Compact**: 36-40px height, minimal vertical footprint
- **Transparent**: semi-transparent background with backdrop blur, letting the desktop show through subtly
- **Supportive**: menus and actions are available but never visually dominant
- **Draggable**: the header doubles as the drag region for moving the window
- **Platform-aware**: window controls and behavior adapt to macOS, Windows, and Linux conventions

---

## 2. Layout

### 2.1. Structure

The header uses a horizontal flexbox layout with four distinct zones arranged left to right.

```
+--------------------------------------------------------------------------+
| [Window Controls] | [Menu Bar]  |   [Title / Center Area]   | [Actions] |
+--------------------------------------------------------------------------+
```

On macOS the window controls (traffic lights) sit on the left. On Windows the window controls (minimize, maximize, close) sit on the right. The layout adjusts accordingly:

**macOS layout:**
```
+--------------------------------------------------------------------------+
| [o o o] | [File Edit View Session Help] |  session-name  | [Q] [S]      |
+--------------------------------------------------------------------------+
```

**Windows layout:**
```
+--------------------------------------------------------------------------+
| [File Edit View Session Help] |    session-name    | [Q] [S] [_ [] X]   |
+--------------------------------------------------------------------------+
```

**Linux layout:**
```
Follows user DE preference - controls on left or right.
```

### 2.2. Dimensions

| Property        | Value                          |
|-----------------|--------------------------------|
| Height          | 38px                           |
| Min height      | 36px                           |
| Max height      | 40px                           |
| Padding left    | 12px (16px on macOS to accommodate traffic lights) |
| Padding right   | 12px                           |
| Full width      | 100% of window                 |
| Z-index         | 100 (above all content)        |

### 2.3. Positioning

| Property         | Value                         |
|------------------|-------------------------------|
| Position         | Fixed, top of window          |
| Stacking         | Above sidebars and terminal   |
| Relationship     | Sidebars start below the header |
| Terminal content | Starts below the header (top offset = header height) |

---

## 3. Background and Transparency

### 3.1. Default Appearance

The header uses a semi-transparent background combined with a CSS backdrop blur filter. This creates a frosted glass effect where the desktop behind the window is faintly visible through the header, following the modern design trend seen in applications like Arc Browser, Linear, and Figma.

```css
.header {
  background: rgba(13, 17, 23, 0.88);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}
```

### 3.2. Transparency Levels

| Level       | Opacity | Use case                       |
|-------------|---------|--------------------------------|
| Solid       | 100%    | Fallback, reduced transparency preference |
| Default     | 88%     | Standard look                  |
| Transparent | 70%     | User preference for more see-through |
| Minimum     | 50%     | Most transparent allowed       |

### 3.3. Platform-Native Effects

| Platform | Effect                         | Tauri API                      |
|----------|--------------------------------|--------------------------------|
| macOS    | NSVisualEffectView vibrancy    | `window.setEffects()` with `WindowEffect::UnderWindowBackground` |
| Windows  | Mica or Acrylic material       | `window.setEffects()` with `WindowEffect::Mica` or `WindowEffect::Acrylic` |
| Linux    | CSS backdrop-filter only       | No native transparency API     |

### 3.4. Fallback

When native transparency is unavailable or the user has enabled "reduce transparency" in their OS settings, the header falls back to a solid background:

```css
.header--solid {
  background: #0D1117;
  backdrop-filter: none;
}
```

### 3.5. Bottom Border

A subtle bottom border separates the header from the content below:

```css
.header {
  border-bottom: 1px solid rgba(48, 54, 61, 0.6);
}
```

Token: `--color-border` at reduced opacity.

---

## 4. Drag Region

### 4.1. Window Dragging

The header serves as the primary drag region for moving the Wit window. All empty space within the header (areas not occupied by interactive controls) is draggable.

Implementation uses Tauri's `data-tauri-drag-region` attribute:

```html
<header data-tauri-drag-region class="header">
  <!-- child elements do NOT have the drag attribute -->
  <div class="menu-bar">...</div>
  <div class="title-area" data-tauri-drag-region>...</div>
  <div class="action-buttons">...</div>
</header>
```

### 4.2. Double-Click Behavior

| Action          | Result                         |
|-----------------|--------------------------------|
| Double-click    | Toggle maximize/restore        |
| Single click    | No action (drag only)          |

On macOS, double-click behavior respects the system preference: it may zoom (maximize) or minimize, depending on the user's "Double-click a window's title bar to" setting in System Preferences.

### 4.3. Non-Draggable Zones

The following elements within the header are interactive and must not trigger window drag:

- Window control buttons (close, minimize, maximize)
- Menu bar items
- Title text (clickable for rename)
- Action buttons (command palette, settings)

These elements should not carry the `data-tauri-drag-region` attribute.

---

## 5. Window Controls Zone

The window controls zone contains the close, minimize, and maximize buttons. Detailed specification of these controls (styling, behavior, platform differences) is in `window-decoration.md`.

Summary:

| Platform | Position | Style                          |
|----------|----------|--------------------------------|
| macOS    | Left     | Circular traffic light dots    |
| Windows  | Right    | Rectangular icon buttons       |
| Linux    | Configurable | Follows DE preference      |

### 5.1. Spacing

| Property                     | Value       |
|------------------------------|-------------|
| Gap from header edge         | 8px         |
| macOS traffic light inset    | 12px from left edge |
| Windows controls inset       | 0px from right edge |

---

## 6. Menu Bar Zone

The menu bar zone contains the application menus: File, Edit, View, Session, Help. Full menu contents and behavior are specified in `window-decoration.md`.

### 6.1. Menu Bar Styling

| Property         | Value                         |
|------------------|-------------------------------|
| Font             | Inter, 13px, regular weight   |
| Text color       | `--color-text-secondary` (#8B949E) |
| Hover text color | `--color-text` (#F0F6FC)      |
| Item padding     | 6px 12px                      |
| Hover background | `--color-surface-hover` (#1C2129) |
| Active background| `--color-primary-muted` (#58E6D920) |
| Border radius    | 4px                           |
| Gap between items| 2px                           |

### 6.2. Dropdown Menus

| Property         | Value                         |
|------------------|-------------------------------|
| Background       | `--color-surface` (#161B22)   |
| Border           | 1px solid `--color-border`    |
| Border radius    | 8px                           |
| Shadow           | `0 8px 24px rgba(0, 0, 0, 0.4)` |
| Min width        | 220px                         |
| Padding          | 4px 0                         |
| Offset from bar  | 4px below menu item           |

### 6.3. Menu Item Styling

| Property             | Value                         |
|----------------------|-------------------------------|
| Font                 | Inter, 13px                   |
| Padding              | 6px 16px                      |
| Text color           | `--color-text`                |
| Hover background     | `--color-surface-hover`       |
| Disabled text color  | `--color-text-muted`          |
| Accelerator text     | `--color-text-secondary`, right-aligned |
| Separator            | 1px `--color-border-muted`, margin 4px 8px |

### 6.4. macOS Native Menu Option

On macOS, Wit can optionally use the native system menu bar instead of the custom in-window menu bar. This is a user preference:

| Option          | Description                    |
|-----------------|--------------------------------|
| Custom menu bar | Default. Rendered inside the header, consistent across platforms. |
| Native menu bar | macOS only. Uses the system menu bar at the top of the screen. Frees up header space. |

When native menu bar is active, the menu bar zone in the header is hidden, and the title area expands to fill the space.

---

## 7. Title / Center Area

### 7.1. Content

The center area displays contextual information about the active session.

| State              | Display                        |
|--------------------|--------------------------------|
| No sessions open   | "Wit" (application name)       |
| Session active     | Session title (e.g., "dev-server") |
| Session with path  | Optional: current working directory |

### 7.2. Styling

| Property         | Value                         |
|------------------|-------------------------------|
| Font             | Inter, 13px, medium weight    |
| Text color       | `--color-text-secondary`      |
| Alignment        | Centered in available space   |
| Max width        | Fills remaining space between menu bar and action buttons |
| Overflow         | Truncate with ellipsis (`text-overflow: ellipsis`) |
| White space      | `nowrap`                      |

### 7.3. Interaction

| Action              | Result                       |
|---------------------|------------------------------|
| Click on title      | Enters inline rename mode    |
| Escape during rename| Cancels rename               |
| Enter during rename | Confirms new name            |
| Blur during rename  | Confirms new name            |

### 7.4. Rename Mode

When the user clicks the session title, it becomes an editable text input:

```
Before: [ dev-server ]
After:  [ |my-project-server| ]  (editable input, selected text)
```

| Property         | Value                         |
|------------------|-------------------------------|
| Input background | `--color-surface`             |
| Input border     | 1px solid `--color-primary`   |
| Border radius    | 4px                           |
| Padding          | 2px 8px                       |
| Max width        | 300px                         |
| Selection color  | `--color-primary-muted`       |

### 7.5. Working Directory Display (Optional)

If enabled in settings, the current working directory is shown below or beside the session title in a smaller, muted font:

```
+-----------------------------------+
|      dev-server                   |
|      ~/projects/wit-term          |
+-----------------------------------+
```

| Property         | Value                         |
|------------------|-------------------------------|
| Font             | Inter, 11px, regular          |
| Text color       | `--color-text-muted`          |
| Path truncation  | Middle truncation for long paths (e.g., ~/proj.../src) |

This feature is off by default and can be enabled in Settings > Appearance.

---

## 8. Action Buttons Zone

### 8.1. Buttons

The right side of the header contains optional quick-access action buttons. These are icon-only buttons with tooltips.

| Button           | Icon       | Tooltip          | Action                     |
|------------------|------------|------------------|----------------------------|
| Command Palette  | Search     | "Command Palette (Ctrl+Shift+P)" | Opens command palette |
| Settings         | Gear       | "Settings (Ctrl+,)" | Opens settings panel   |

### 8.2. Button Styling

| Property         | Value                         |
|------------------|-------------------------------|
| Size             | 28px x 28px                   |
| Icon size        | 16px                          |
| Icon color       | `--color-text-muted`          |
| Hover icon color | `--color-text`                |
| Hover background | `--color-surface-hover`       |
| Border radius    | 6px                           |
| Gap between      | 4px                           |
| Tooltip delay    | 500ms                         |

### 8.3. Tooltip Styling

| Property         | Value                         |
|------------------|-------------------------------|
| Background       | `--color-surface`             |
| Text color       | `--color-text`                |
| Font             | Inter, 12px                   |
| Padding          | 4px 8px                       |
| Border radius    | 4px                           |
| Shadow           | `0 2px 8px rgba(0, 0, 0, 0.3)` |
| Position         | Below button, centered        |

---

## 9. States

### 9.1. Window Focus States

| State           | Appearance                     |
|-----------------|--------------------------------|
| Focused         | Normal appearance, full opacity |
| Unfocused       | Reduced opacity (0.6) on text and icons, background unchanged |

### 9.2. Fullscreen

When the window enters fullscreen mode (F11 or maximize on macOS):

| Platform | Behavior                        |
|----------|---------------------------------|
| macOS    | Header slides up and hides, revealed on mouse hover at top edge |
| Windows  | Header remains visible in fullscreen |
| Linux    | Header remains visible in fullscreen |

Auto-hide on macOS:
- Mouse approaches top 4px of screen - header slides down (200ms ease-out)
- Mouse leaves header area - header slides up after 500ms delay (200ms ease-in)

### 9.3. Maximized

When the window is maximized:
- No visual change to the header (it already spans full width)
- Border radius of the window is removed (set to 0)
- Window controls remain functional

---

## 10. Accessibility

| Feature              | Implementation                |
|----------------------|-------------------------------|
| Keyboard navigation  | Tab through interactive elements in header |
| Tab order            | Menu bar items, title, action buttons, window controls |
| ARIA roles           | `role="menubar"` on menu bar, `role="banner"` on header |
| Screen reader        | Menu items announced with name and shortcut |
| Focus indicators     | 2px solid `--color-primary` outline on focus |
| Reduced motion       | No slide animations, instant transitions |
| High contrast        | Solid background, higher contrast text colors |

---

## 11. Responsive Behavior

| Window Width   | Adaptation                     |
|----------------|--------------------------------|
| >= 900px       | Full layout: controls, menus, title, actions |
| 600-899px      | Menu bar collapses to hamburger icon, title truncated |
| < 600px        | Menus behind hamburger, action buttons hidden, title minimal |

### 11.1. Collapsed Menu Bar

When the window is too narrow for the full menu bar, it collapses into a single hamburger menu icon. Clicking the hamburger opens a vertical dropdown containing all menu categories.

```
Narrow window:
+------------------------------------------+
| [o o o] [=] |   session-name   | [Q]     |
+------------------------------------------+
```

---

## 12. Component Structure

```
WindowHeader
  +-- WindowControls (platform-specific, see window-decoration.md)
  +-- MenuBar
  |     +-- MenuItem ("File")
  |     +-- MenuItem ("Edit")
  |     +-- MenuItem ("View")
  |     +-- MenuItem ("Session")
  |     +-- MenuItem ("Help")
  +-- TitleArea
  |     +-- SessionTitle (text or editable input)
  |     +-- WorkingDirectory (optional)
  +-- ActionButtons
        +-- CommandPaletteButton
        +-- SettingsButton
```

---

## 13. CSS Variables

The header introduces the following CSS custom properties:

| Variable                   | Default Value                  |
|----------------------------|--------------------------------|
| `--header-height`          | 38px                           |
| `--header-bg`              | rgba(13, 17, 23, 0.88)        |
| `--header-blur`            | 20px                           |
| `--header-border`          | rgba(48, 54, 61, 0.6)         |
| `--header-padding-x`       | 12px                          |
| `--header-unfocused-opacity` | 0.6                          |

---

## 14. Implementation Notes

### 14.1. Tauri Configuration

```json
{
  "windows": [
    {
      "decorations": false,
      "transparent": true,
      "width": 1200,
      "height": 800
    }
  ]
}
```

### 14.2. React Component Skeleton

```tsx
const WindowHeader: React.FC = () => {
  const platform = usePlatform(); // 'macos' | 'windows' | 'linux'

  return (
    <header className="window-header" data-tauri-drag-region>
      {platform === 'macos' && <TrafficLights />}
      <MenuBar />
      <TitleArea />
      <ActionButtons />
      {platform === 'windows' && <WindowControls />}
    </header>
  );
};
```

### 14.3. Drag Region Handling

Interactive child elements must call `e.stopPropagation()` on `mousedown` to prevent Tauri from interpreting clicks on buttons and menus as drag events. Alternatively, only apply `data-tauri-drag-region` to non-interactive regions.

### 14.4. Vibrancy Setup

Use Tauri v2's `WebviewWindow.setEffects()` API to apply native transparency:

```rust
// In Rust backend setup
window.set_effects(WindowEffectsConfig {
    effects: vec![WindowEffect::UnderWindowBackground],
    state: None,
    radius: None,
    color: None,
})?;
```

The CSS `backdrop-filter` serves as the cross-platform fallback and is always applied. On platforms where native vibrancy is active, the CSS filter and the native effect work together.
