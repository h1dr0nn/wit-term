# Session Sidebar (Left)

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Overview

The left sidebar contains the list of sessions (open terminal sessions). Users can create, switch, reorder, and manage sessions from here. The sidebar is designed to not take up too much space while remaining easy to use when needed.

---

## 2. Layout & Dimensions

### 2.1. Dimensions

| Property        | Value                          |
|-----------------|--------------------------------|
| Default width   | 240px                          |
| Min width       | 180px (when resizing)          |
| Max width       | 360px (when resizing)          |
| Collapsed width | 0px (completely hidden)        |

### 2.2. Resize

- **Drag handle**: right border of sidebar, 4px hit area
- **Cursor**: `col-resize` when hovering over drag handle
- **Visual**: drag handle shows a 2px vertical line colored `--color-border` on hover
- **Snap**: no snap points, free resize within the 180-360px range
- **Persist**: width is saved to user preferences

### 2.3. Structure

```
+------------------------+
|  [Logo] Wit    [<]     |  <- Header
+------------------------+
|                        |
|  Session 1 (active)    |  <- Session List
|  Session 2             |
|  Session 3             |
|                        |
|                        |
|                        |
+------------------------+
|  [+] New Session       |  <- Footer
+------------------------+
```

---

## 3. Toggle (Collapse/Expand)

### 3.1. Toggle Methods

| Method            | Description                        |
|-------------------|------------------------------------|
| Ctrl+B            | Toggle sidebar visibility          |
| Collapse button   | Icon button [<] in header          |
| Auto-collapse     | When window width < 800px          |

### 3.2. Animation

- **Duration**: 200ms ease-out
- **Effect**: slide left (collapse), slide right (expand)
- **Terminal resize**: terminal view expands/shrinks accordingly, triggers grid recalculation
- **Reduce motion**: instant show/hide instead of slide

### 3.3. State

- **Collapsed**: sidebar width = 0, takes up no space
- **Expanded**: sidebar width = last saved width (or default 240px)
- **State is saved**: persists across sessions/restarts

---

## 4. Header

### 4.1. Layout

```
+-----------------------------+
|  [icon] Wit        [<]      |
+-----------------------------+
```

- **Logo/Icon**: Wit app icon, 20x20px
- **App name**: "Wit", font `--text-md`, weight 600, color `--color-text`
- **Collapse button**: icon button (CaretLeft), 28x28px, ghost style
- **Height**: 48px (--sp-12)
- **Padding**: 0 `--sp-3`
- **Background**: `--color-surface`
- **Border bottom**: 1px solid `--color-border-muted`

### 4.2. Interactions

- Click collapse button: toggle sidebar
- Collapse button icon: CaretLeft when expanded, CaretRight when collapsed (if mini-mode exists)

---

## 5. Session List

### 5.1. Container

- **Padding**: `--sp-2` top and bottom
- **Overflow**: vertical scroll when sessions exceed viewport
- **Scrollbar**: thin scrollbar (6px), auto-hide
- **Background**: `--color-surface`

### 5.2. Session Item

```
+-----------------------------+
|  [>] bash - ~/projects  [x] |
+-----------------------------+
```

#### Layout
- **Height**: 36px
- **Padding**: `--sp-2` vertical, `--sp-3` horizontal
- **Gap**: `--sp-2` between items (or 0 gap, using hover state to differentiate)
- **Border-radius**: `--radius-sm` (4px)
- **Margin**: 0 `--sp-2` (creates spacing from sidebar edges)

#### Content
- **Icon**: shell type icon (Terminal), size 16px, color `--color-text-secondary`
  - bash/zsh: Terminal icon
  - PowerShell: Terminal icon (variant)
  - cmd: Terminal icon (variant)
  - Custom: can set icon per session
- **Title**: session name or shell + cwd, font `--text-sm`, truncate with ellipsis
  - Default title: `{shell_name} - {cwd_basename}`
  - Custom title: user can rename
- **Close button**: X icon, 16px, only visible when hovering over session item

### 5.3. Session States

#### Default (Inactive)
- Background: transparent
- Text: `--color-text-secondary`
- Icon: `--color-text-muted`

#### Hover
- Background: `--color-surface-hover`
- Text: `--color-text`
- Close button: visible, color `--color-text-muted`, hover `--color-error`

#### Active (Current Session)
- Background: `--color-surface-active`
- Text: `--color-text`
- Icon: `--color-primary`
- **Left border**: 2px solid `--color-primary` (indicator for active session)
- Font weight: 500 (slightly bolder)

#### Dragging
- Opacity: 0.7
- Shadow: `--shadow-md`
- Background: `--color-surface`
- Drag placeholder: 2px line colored `--color-primary` at the drop target position

---

## 6. Context Menu (Right-Click)

When right-clicking on a session item:

| Menu Item       | Shortcut         | Description                     |
|-----------------|------------------|---------------------------------|
| Rename          | F2               | Opens inline edit for session title |
| Duplicate       | -                | Creates a new session with the same settings |
| Close           | Ctrl+Shift+W     | Closes this session             |
| Close Others    | -                | Closes all other sessions       |
| Close to Right  | -                | Closes sessions below           |
| ---             |                  | Divider                         |
| Move Up         | Alt+Up           | Moves session up                |
| Move Down       | Alt+Down         | Moves session down              |

### Context Menu Appearance
- Background: `--color-surface`
- Border: 1px solid `--color-border`
- Border-radius: `--radius-md`
- Shadow: `--shadow-md`
- Item padding: `--sp-2` vertical, `--sp-3` horizontal
- Shortcut text: `--color-text-muted`, `--text-xs`, right-aligned

---

## 7. Drag to Reorder

### 7.1. Drag Behavior

- **Initiate**: click and hold (200ms) on a session item, then move
- **Visual**: item "lifts up" with shadow and reduced opacity
- **Drop indicator**: horizontal line colored `--color-primary` (2px) at the drop position
- **Drop**: release mouse to place session at new position
- **Cancel**: press Escape or release outside the sidebar area

### 7.2. Constraints

- Can only drag within the session list (cannot drag outside the sidebar)
- Dragging is not allowed when there is only 1 session

---

## 8. New Session Button (Footer)

### 8.1. Layout

```
+-----------------------------+
|  [+]  New Session           |
+-----------------------------+
```

- **Position**: bottom of sidebar, fixed (does not scroll with list)
- **Height**: 40px
- **Padding**: `--sp-2` vertical, `--sp-3` horizontal
- **Border top**: 1px solid `--color-border-muted`
- **Background**: `--color-surface`

### 8.2. Appearance

- **Icon**: Plus, 16px, `--color-text-secondary`
- **Text**: "New Session", `--text-sm`, `--color-text-secondary`
- **Hover**: background `--color-surface-hover`, text `--color-text`, icon `--color-primary`
- **Active**: background `--color-surface-active`

### 8.3. Interactions

- **Click**: creates a new session with the default shell
- **Long press / Right-click**: menu to choose shell type (bash, zsh, PowerShell, etc.)
- **Shortcut**: Ctrl+Shift+T

---

## 9. Inline Rename

When user selects "Rename" from the context menu or presses F2:

- Session title transforms into a text input
- Input auto-selects all current text
- Enter: saves the new name
- Escape: cancels, keeps the old name
- Click outside: saves the new name
- Max length: 50 characters
- Input style: no border, background `--color-bg`, blends into the session item

---

## 10. Keyboard Navigation

### 10.1. Session Switching

| Shortcut           | Action                             |
|--------------------|------------------------------------|
| Ctrl+1 ... Ctrl+9  | Switch to session 1-9             |
| Ctrl+Tab           | Switch to next session             |
| Ctrl+Shift+Tab     | Switch to previous session         |
| Ctrl+Shift+T       | Create new session                 |
| Ctrl+Shift+W       | Close current session              |

### 10.2. Within Sidebar

When the sidebar is focused (Tab into sidebar):
- **Arrow Up/Down**: move highlight between sessions
- **Enter**: switch to the highlighted session
- **Delete**: close the highlighted session (with confirmation if there is unsaved state)
- **F2**: rename the highlighted session
- **Escape**: return focus to terminal

---

## 11. Session Groups (Future)

> **Note**: This is a future feature, not implemented immediately.

### 11.1. Concept

- Group sessions by project or category
- Folder icon with expand/collapse
- Drag sessions into a group
- Groups can have their own name and color

### 11.2. UI (Draft)

```
+-----------------------------+
|  v Project Alpha            |
|    Session 1                |
|    Session 2                |
|  > Project Beta             |
|  Session 3 (ungrouped)      |
+-----------------------------+
```

---

## 12. Empty State

When there are no sessions:

```
+-----------------------------+
|                             |
|      [Terminal Icon]        |
|                             |
|   No sessions open.         |
|   Create one to get started.|
|                             |
|   [+ New Session]           |
|                             |
+-----------------------------+
```

- **Icon**: Terminal, 48px, `--color-text-muted`
- **Text**: `--text-base`, `--color-text-secondary`, centered
- **Button**: Primary button style
- **This state is rare**: the app typically auto-creates the first session on startup

---

## 13. Accessibility

- **Role**: sidebar has `role="complementary"` and `aria-label="Sessions"`
- **Session list**: `role="listbox"`
- **Session item**: `role="option"`, `aria-selected` for active session
- **Close button**: `aria-label="Close session: {name}"`
- **New session button**: `aria-label="Create new terminal session"`
- **Collapse button**: `aria-label="Collapse sidebar"` / `aria-label="Expand sidebar"`, `aria-expanded`
- **Keyboard**: all interactions can be performed via keyboard
- **Focus management**: when closing the active session, focus moves to the nearest session
