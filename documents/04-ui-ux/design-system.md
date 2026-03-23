# Design System

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Overview

This document defines the design system for the Wit terminal emulator. Every UI component is built from shared design tokens, ensuring visual language consistency across the entire application.

**Design principles:**
- **Minimal**: only display what is necessary, reduce visual noise
- **Terminal-first**: every design decision serves the terminal experience first
- **Accessible**: support keyboard navigation, screen reader, high contrast
- **Consistent**: every component uses shared tokens, no hardcoded values

---

## 2. Design Tokens

### 2.1. Colors

Refer to `branding.md` for detailed information. Summary of main color tokens:

#### Primary Palette

| Token                  | Hex       | Usage                            |
|------------------------|-----------|----------------------------------|
| `--color-primary`      | `#58E6D9` | Main accent, links, focus rings  |
| `--color-primary-hover`| `#6AEEE2` | Hover state of primary           |
| `--color-primary-muted`| `#58E6D920` | Light background for primary   |
| `--color-accent`       | `#D4A857` | Cursor, highlights, badges       |
| `--color-accent-hover` | `#E0B96A` | Hover state of accent            |
| `--color-accent-muted` | `#D4A85720` | Light background for accent    |

#### Background & Surface

| Token                    | Hex       | Usage                          |
|--------------------------|-----------|--------------------------------|
| `--color-bg`             | `#0D1117` | Main app background            |
| `--color-surface`        | `#161B22` | Panels, sidebars, cards        |
| `--color-surface-hover`  | `#1C2129` | Hover on surface elements      |
| `--color-surface-active` | `#22272E` | Active/pressed state           |
| `--color-border`         | `#30363D` | Borders, dividers              |
| `--color-border-muted`   | `#21262D` | Subtle borders                 |

#### Text

| Token                  | Hex       | Usage                          |
|------------------------|-----------|--------------------------------|
| `--color-text`         | `#F0F6FC` | Primary text                   |
| `--color-text-secondary`| `#8B949E`| Secondary text, descriptions   |
| `--color-text-muted`   | `#6E7681` | Placeholder, disabled text     |
| `--color-text-inverse` | `#0D1117` | Text on light background       |

#### Semantic Colors

| Token                  | Hex       | Usage                          |
|------------------------|-----------|--------------------------------|
| `--color-success`      | `#3FB950` | Success, git clean              |
| `--color-warning`      | `#D29922` | Warning                         |
| `--color-error`        | `#F85149` | Error, git conflicts            |
| `--color-info`         | `#58A6FF` | Information                     |

### 2.2. Spacing Scale

Base unit: **4px**. Every spacing value is a multiple of 4.

| Token      | Value | Pixels | Common usage                    |
|------------|-------|--------|---------------------------------|
| `--sp-1`   | 1     | 4px    | Small icon gap, inline spacing  |
| `--sp-2`   | 2     | 8px    | Internal component padding      |
| `--sp-3`   | 3     | 12px   | Gap between items in a list     |
| `--sp-4`   | 4     | 16px   | Standard panel padding          |
| `--sp-5`   | 5     | 20px   | Spacing between small sections  |
| `--sp-6`   | 6     | 24px   | Margin between groups           |
| `--sp-8`   | 8     | 32px   | Large spacing between sections  |
| `--sp-10`  | 10    | 40px   | Header height, large gaps       |
| `--sp-12`  | 12    | 48px   | Toolbar height                  |
| `--sp-16`  | 16    | 64px   | Maximum spacing                 |

**Rule:** Always use spacing tokens, do not use arbitrary values. If a value outside the scale is needed, prefer the nearest value.

### 2.3. Border Radius

| Token           | Value | Usage                           |
|-----------------|-------|---------------------------------|
| `--radius-xs`   | 2px   | Small tags, badges              |
| `--radius-sm`   | 4px   | Buttons, inputs                 |
| `--radius-md`   | 6px   | Cards, dropdowns                |
| `--radius-lg`   | 8px   | Modals, panels                  |
| `--radius-xl`   | 12px  | Large cards                     |
| `--radius-2xl`  | 16px  | Feature panels                  |
| `--radius-full` | 9999px| Avatars, pills                  |

### 2.4. Shadows

| Token              | Value                                         | Usage            |
|--------------------|-----------------------------------------------|------------------|
| `--shadow-sm`      | `0 1px 2px rgba(0, 0, 0, 0.3)`               | Subtle lift      |
| `--shadow-md`      | `0 4px 8px rgba(0, 0, 0, 0.4)`               | Dropdowns, popups|
| `--shadow-lg`      | `0 8px 24px rgba(0, 0, 0, 0.5)`              | Modals, overlays |
| `--shadow-focus`   | `0 0 0 2px var(--color-primary)`              | Focus ring       |

**Note:** Shadows are relatively strong due to the dark background. In light mode, opacity should be reduced.

---

## 3. Typography

### 3.1. Font Families

| Token               | Font           | Usage                       |
|----------------------|----------------|-----------------------------|
| `--font-mono`        | JetBrains Mono | Terminal output, code       |
| `--font-ui`          | Inter          | UI labels, menus, buttons   |
| `--font-fallback-mono`| Cascadia Code, Consolas, monospace | Fallback mono  |
| `--font-fallback-ui` | system-ui, -apple-system, sans-serif | Fallback UI |

### 3.2. Typography Scale - UI (Inter)

| Token          | Size  | Weight | Line Height | Usage                     |
|----------------|-------|--------|-------------|---------------------------|
| `--text-xs`    | 11px  | 400    | 16px        | Badges, captions          |
| `--text-sm`    | 12px  | 400    | 18px        | Secondary text, labels    |
| `--text-base`  | 13px  | 400    | 20px        | Body text, menu items     |
| `--text-md`    | 14px  | 500    | 22px        | Emphasized body           |
| `--text-lg`    | 16px  | 600    | 24px        | Section headers           |
| `--text-xl`    | 20px  | 600    | 28px        | Page titles               |
| `--text-2xl`   | 24px  | 700    | 32px        | Hero/onboarding           |

### 3.3. Typography Scale - Terminal (JetBrains Mono)

| Token               | Size  | Weight | Line Height | Usage                |
|----------------------|-------|--------|-------------|----------------------|
| `--term-text-sm`     | 12px  | 400    | 1.2         | Compact mode         |
| `--term-text-base`   | 14px  | 400    | 1.3         | Default terminal     |
| `--term-text-lg`     | 16px  | 400    | 1.3         | Large terminal       |
| `--term-text-xl`     | 18px  | 400    | 1.3         | Extra large          |

**Note:** Terminal line height uses a ratio (unitless) to ensure accurate character grid alignment.

---

## 4. Component Library

### 4.1. Buttons

#### Primary Button
- Background: `--color-primary`
- Text: `--color-text-inverse`
- Border: none
- Hover: `--color-primary-hover`, increase brightness 10%
- Active: decrease brightness 5%
- Disabled: opacity 0.5, cursor not-allowed
- Padding: `--sp-2` vertical, `--sp-4` horizontal
- Border-radius: `--radius-sm`
- Font: `--text-sm`, weight 500

#### Secondary Button
- Background: transparent
- Text: `--color-text`
- Border: 1px solid `--color-border`
- Hover: background `--color-surface-hover`
- Active: background `--color-surface-active`
- Disabled: opacity 0.5
- Same padding and border-radius as Primary

#### Ghost Button
- Background: transparent
- Text: `--color-text-secondary`
- Border: none
- Hover: background `--color-surface-hover`, text `--color-text`
- Active: background `--color-surface-active`
- Commonly used for toolbar actions, close buttons

#### Icon Button
- Size: 28x28px (small), 32x32px (medium), 36x36px (large)
- Border-radius: `--radius-sm`
- Padding: `--sp-1`
- Contains only an icon, no text
- Tooltip is required for accessibility

### 4.2. Inputs

#### Text Input
- Background: `--color-bg`
- Border: 1px solid `--color-border`
- Border-radius: `--radius-sm`
- Padding: `--sp-2` vertical, `--sp-3` horizontal
- Focus: border-color `--color-primary`, box-shadow `--shadow-focus`
- Placeholder: `--color-text-muted`
- Height: 32px (small), 36px (medium)
- Font: `--text-sm`

#### Search Input
- Same as Text Input but with a search icon on the left
- Clear button (x) on the right when there is a value
- Shortcut hint on the right (e.g., "Ctrl+F")

### 4.3. Dropdowns / Select

- Trigger: similar to Text Input with a chevron icon on the right
- Menu: background `--color-surface`, border 1px `--color-border`
- Menu shadow: `--shadow-md`
- Item padding: `--sp-2` vertical, `--sp-3` horizontal
- Item hover: background `--color-surface-hover`
- Item selected: background `--color-primary-muted`, text `--color-primary`
- Max height: 240px, scrollable
- Border-radius: `--radius-md`

### 4.4. Tooltips

- Background: `--color-surface` (or `#2D333B` for more prominence)
- Text: `--color-text`, `--text-xs`
- Padding: `--sp-1` vertical, `--sp-2` horizontal
- Border-radius: `--radius-sm`
- Shadow: `--shadow-sm`
- Delay: 500ms before showing
- Position: auto (prefer top, fallback bottom/left/right)
- Arrow: 6px triangle
- Max width: 240px

### 4.5. Scroll Areas

- Scrollbar width: 6px (thin)
- Scrollbar thumb: `--color-border`, border-radius `--radius-full`
- Scrollbar track: transparent
- Thumb hover: `--color-text-muted`
- Auto-hide: hide after 1.5s without scrolling, show when hovering over scroll area
- Terminal scrollbar: hidden by default, shown when scrolling (see terminal-view.md)

### 4.6. Dividers

- Horizontal: 1px solid `--color-border-muted`
- Vertical: 1px solid `--color-border-muted`
- Spacing: `--sp-2` margin top and bottom (horizontal) or left and right (vertical)
- Can have a label (e.g., "OR") with background `--color-surface` to bisect the divider

---

## 5. Icon System

### 5.1. Icon Library

Uses **Phosphor Icons** (https://phosphoricons.com/) as the primary icon set.

**Reasons for choosing Phosphor:**
- 6 weight variants (thin, light, regular, bold, fill, duotone)
- 1000+ icons
- MIT license
- Consistent style
- Good React component support

### 5.2. Icon Sizes

| Token          | Size  | Usage                            |
|----------------|-------|----------------------------------|
| `--icon-sm`    | 16px  | Inline icons, badges, tags       |
| `--icon-md`    | 20px  | Button icons, menu items         |
| `--icon-lg`    | 24px  | Section headers, empty states    |

### 5.3. Icon Colors

- Default: `currentColor` (inherits from parent text color)
- Interactive: changes color on hover/focus (e.g., `--color-text` -> `--color-primary`)
- Semantic: use semantic colors for status icons (success/error/warning)
- Disabled: `--color-text-muted`

### 5.4. Common Icons

| Icon Name         | Usage                       | Phosphor Name      |
|-------------------|-----------------------------|---------------------|
| Terminal          | Session icon, shell type    | `Terminal`          |
| Plus              | New session, add action     | `Plus`              |
| X                 | Close, dismiss              | `X`                 |
| MagnifyingGlass   | Search                      | `MagnifyingGlass`   |
| GearSix           | Settings                    | `GearSix`           |
| GitBranch         | Git branch info             | `GitBranch`         |
| Folder            | Directory, project          | `Folder`            |
| File              | File paths                  | `File`              |
| Clock             | History, recent             | `Clock`             |
| CaretRight        | Expand, chevron             | `CaretRight`        |
| CaretDown         | Collapse, chevron           | `CaretDown`         |
| Copy              | Copy to clipboard           | `Copy`              |
| ArrowsOutSimple   | Fullscreen                  | `ArrowsOutSimple`   |
| SidebarSimple     | Toggle sidebar              | `SidebarSimple`     |

---

## 6. Animation & Motion

### 6.1. Transition Defaults

| Token                  | Value              | Usage                       |
|------------------------|--------------------|-----------------------------|
| `--transition-fast`    | `100ms ease`       | Hover states, toggles       |
| `--transition-base`    | `150ms ease`       | Most UI transitions         |
| `--transition-slow`    | `250ms ease-out`   | Panel slides, expand/collapse|

### 6.2. Principles

- **No heavy animations**: a terminal emulator should feel fast and snappy
- **Functional motion only**: animations only convey state changes (e.g., panel open/close), not decoration
- **Short duration**: maximum 250ms for any animation
- **Ease curves**: `ease` for most cases, `ease-out` for elements appearing, `ease-in` for elements disappearing

### 6.3. Reduce Motion

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

When `prefers-reduced-motion` is enabled:
- Disable all animations
- Transitions switch to instant (duration ~0)
- Cursor blink is disabled
- Panel slide transitions switch to instant show/hide

---

## 7. Responsive Behavior

### 7.1. Window Size

| Constraint     | Value    | Notes                           |
|----------------|----------|---------------------------------|
| Min width      | 600px    | Cannot resize below this        |
| Min height     | 400px    | Cannot resize below this        |
| Default size   | 1024x768 | Size when first opened          |
| Max size       | Unbounded| Depends on screen               |

### 7.2. Layout Adaptation

- **< 800px width**: Left sidebar auto-collapses, right sidebar hidden
- **800-1200px**: Left sidebar visible, right sidebar hidden by default
- **> 1200px**: Both sidebars can be visible simultaneously
- Terminal area always occupies all remaining space

### 7.3. Sidebar Behavior

- Left sidebar: collapsible to 0px, state saved to preferences
- Right sidebar: collapsible to 0px, hidden by default
- When window is too small, sidebar auto-collapses with animation
- Drag handle to resize sidebar width

---

## 8. Accessibility

### 8.1. Keyboard Navigation

- **Tab order**: logical and consistent (sidebar -> terminal -> completion popup)
- **Focus ring**: 2px solid `--color-primary`, offset 2px
- **Focus visible**: only show focus ring when navigating with keyboard (`:focus-visible`)
- **Skip navigation**: not necessary since terminal is the main focus

### 8.2. ARIA Labels

- Every interactive element must have an accessible name
- Icon buttons: `aria-label` is required
- Sidebar sections: `role="region"` with `aria-label`
- Session list: `role="listbox"` with `role="option"` for each item
- Completion popup: `role="listbox"` with `aria-activedescendant`
- Terminal: `role="document"` or appropriate custom role

### 8.3. Color Contrast

| Context                | Required ratio | Actual ratio (estimated)   |
|------------------------|----------------|--------------------------|
| Text on bg             | >= 4.5:1       | ~15:1 (#F0F6FC / #0D1117)|
| Secondary text on bg   | >= 4.5:1       | ~6.5:1 (#8B949E / #0D1117)|
| Primary on bg          | >= 3:1 (large) | ~8:1 (#58E6D9 / #0D1117) |
| Accent on bg           | >= 3:1 (large) | ~7:1 (#D4A857 / #0D1117) |

All color pairs meet WCAG AA. Some pairs meet AAA.

### 8.4. Screen Reader

- Live regions for terminal output (be careful not to be overwhelming)
- `aria-live="polite"` for status updates
- `aria-live="assertive"` for error messages
- Completion popup announcements: "N completions available"

---

## 9. Dark Mode & Light Mode

### 9.1. Dark Mode (Default)

This is Wit's default mode. All color tokens above are for dark mode.

### 9.2. Light Mode (Optional)

Light mode overrides the color tokens:

| Token (dark)          | Dark Value  | Light Value |
|-----------------------|-------------|-------------|
| `--color-bg`          | `#0D1117`   | `#FFFFFF`   |
| `--color-surface`     | `#161B22`   | `#F6F8FA`   |
| `--color-surface-hover`| `#1C2129`  | `#EAEEF2`   |
| `--color-border`      | `#30363D`   | `#D0D7DE`   |
| `--color-text`        | `#F0F6FC`   | `#1F2328`   |
| `--color-text-secondary`| `#8B949E` | `#656D76`   |
| `--color-text-muted`  | `#6E7681`   | `#8B949E`   |

**Primary and accent remain the same** in both modes, only adjusting opacity if needed.

### 9.3. Implementation

- Use CSS custom properties, switch via `data-theme` attribute on root element
- Theme preference saved in user settings
- Respect `prefers-color-scheme` media query if user has not explicitly chosen
- Transition when switching theme: `--transition-slow` for background, instant for text

---

## 10. Design Token Export

All tokens are exported in the following formats:
- **CSS Custom Properties**: file `tokens.css` - imported into the app
- **TypeScript constants**: file `tokens.ts` - used in styled-components or inline styles
- **Tailwind config**: extend Tailwind theme with custom tokens (if using Tailwind)

```css
/* Example tokens.css */
:root {
  --color-primary: #58E6D9;
  --color-bg: #0D1117;
  --color-surface: #161B22;
  --sp-1: 4px;
  --sp-2: 8px;
  /* ... */
}

[data-theme="light"] {
  --color-bg: #FFFFFF;
  --color-surface: #F6F8FA;
  /* ... */
}
```
