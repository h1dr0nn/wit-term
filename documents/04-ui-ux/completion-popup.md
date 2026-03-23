# Completion Popup UI

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Overview

The completion popup is the component that displays auto-complete suggestions when the user is entering commands in the terminal. The popup must appear quickly, not obscure important content, and be easy to navigate with the keyboard.

**Design principles:**
- **Fast**: display within 50ms after trigger
- **Non-intrusive**: does not obscure important output, easy to dismiss
- **Keyboard-first**: all actions can be done with the keyboard
- **Informative**: displays enough information for the user to choose the right completion

---

## 2. Position & Anchor

### 2.1. Position

- **Anchor point**: current cursor position in the terminal
- **Direction**: popup displays **above** the cursor (preferred)
- **Fallback**: if there is not enough space above, display **below** the cursor
- **Horizontal align**: left-aligned with cursor position

### 2.2. Offset

- **Vertical offset**: 4px from cursor (--sp-1)
- **Horizontal offset**: 0px (aligned with cursor)
- **Boundary**: popup does not extend beyond terminal view bounds
- **Reposition**: if popup is clipped on the right edge, shift to the left

### 2.3. Position Example

```
  $ git che|                    <- cursor here
  +---------------------+
  | > checkout           |      <- popup below (fallback)
  |   cherry-pick        |
  |   clean              |
  +---------------------+
```

```
  +---------------------+
  |   cherry-pick        |      <- popup above (preferred)
  |   clean              |
  | > checkout           |
  +---------------------+
  $ git che|                    <- cursor here
```

---

## 3. Dimensions

### 3.1. Size Constraints

| Property       | Value                           |
|----------------|---------------------------------|
| Max height     | 300px (~10 items visible)       |
| Min height     | 1 item (36px)                   |
| Max width      | 500px                           |
| Min width      | 200px                           |
| Dynamic width  | Grows with content, within min-max range |

### 3.2. Scrollbar

- Appears when items exceed max height
- Style: same as design system scrollbar (6px, auto-hide)
- Scroll with mouse wheel or keyboard (Arrow keys)

---

## 4. Item Layout

### 4.1. Item Structure

```
+--------------------------------------------------+
|  [icon]  command-name        description text     |
+--------------------------------------------------+
```

- **Height**: 30px per item
- **Padding**: `--sp-1` vertical, `--sp-2` horizontal
- **Icon**: 16px, left-aligned, color by kind
- **Text (name)**: `--text-sm`, monospace (`--font-mono`), `--color-text`
- **Description**: `--text-xs`, `--color-text-muted`, right-aligned, truncated with ellipsis
- **Gap**: `--sp-2` between icon and text, flexible space between text and description

### 4.2. Item Kinds & Icons

| Kind       | Icon (Phosphor)    | Color              | Description              |
|------------|--------------------|--------------------|--------------------------|
| Command    | Terminal           | `--color-primary`  | Executable commands      |
| Flag       | Flag               | `--color-accent`   | Command flags/options    |
| Argument   | CaretRight         | `--color-info`     | Command arguments        |
| Path       | File / Folder      | `--color-success`  | File/directory paths     |
| History    | Clock              | `--color-text-muted`| Command history items   |
| Variable   | At (or Code)       | `#C678DD`          | Environment variables    |
| Alias      | ArrowBendUpRight   | `--color-warning`  | Shell aliases            |

- File vs Folder: use File icon for files, Folder icon for directories
- Icon can be hidden if popup is too narrow (< 250px width)

---

## 5. Selection & Highlight

### 5.1. Selected Item

- **Background**: `--color-accent` with opacity 20% (`#D4A85733`)
- **Text**: `--color-text` (unchanged)
- **Left border**: 2px solid `--color-accent`
- **Scroll into view**: when navigating with keyboard, selected item is always visible

### 5.2. Hover Item

- **Background**: `--color-surface-hover`
- **Hover and selected are different**: selected has accent background, hover only has surface-hover

### 5.3. No Selection

- When popup first appears: first item is pre-selected
- User can deselect by pressing Up when at the first item (deselects all)

---

## 6. Fuzzy Match Highlighting

### 6.1. Matched Characters

When user types "gco" and it matches "git checkout":
```
  g-it  c-heck-o-ut
  ^     ^       ^        <- matched characters
```

- **Matched characters**: `--color-primary`, font-weight 600 (bold)
- **Unmatched characters**: keep normal style
- **Continuous match**: if match is contiguous (e.g., "che" in "checkout"), underline the matched segment

### 6.2. Scoring & Ordering

- Items are sorted by match score (highest first)
- Score priority: prefix match > word boundary match > fuzzy match
- Same score: prioritize by frequency of use > alphabetical

---

## 7. Source Badge

### 7.1. Purpose

A small badge showing the origin of the completion (so the user knows where the data comes from).

### 7.2. Appearance

- **Position**: right side of item, after description
- **Style**: small tag, `--text-xs`, padding 2px 4px
- **Border-radius**: `--radius-xs`
- **Background**: `--color-surface-active`
- **Text**: `--color-text-muted`

### 7.3. Source Types

| Source      | Badge text  | Description                    |
|-------------|-------------|--------------------------------|
| History     | `hist`      | From command history           |
| Parse       | `parse`     | From CLI spec parsing          |
| FS          | `fs`        | From file system scan          |
| Plugin      | `plug`      | From plugin/provider           |
| Man         | `man`       | From man page parsing          |

- **Optional**: badges can be hidden if popup is too narrow or in settings

---

## 8. Header (Optional)

### 8.1. When Displayed

- Displayed when there are many completions (> 20 items)
- Displays filter mode indicator

### 8.2. Layout

```
+--------------------------------------------------+
|  15 completions              [filter: fuzzy]      |
+--------------------------------------------------+
|  [icon]  item 1              description          |
|  ...                                              |
```

- **Height**: 24px
- **Background**: `--color-surface-active`
- **Text**: `--text-xs`, `--color-text-muted`
- **Count**: "15 completions" on the left
- **Filter mode**: "fuzzy" / "prefix" / "exact" on the right
- **Border bottom**: 1px solid `--color-border-muted`

---

## 9. Keyboard Navigation

### 9.1. Trigger Completion

| Shortcut        | Action                               |
|-----------------|--------------------------------------|
| Tab             | Trigger completion popup (if not open) |
| Typing          | Auto-trigger after 2 characters (configurable) |

### 9.2. Navigate & Select

| Shortcut        | Action                               |
|-----------------|--------------------------------------|
| Arrow Down      | Move to next item                    |
| Arrow Up        | Move to previous item                |
| Tab             | Accept selected completion           |
| Enter           | Accept selected completion           |
| Escape          | Dismiss popup without selecting      |
| Ctrl+Space      | Force re-trigger completion          |

### 9.3. Scroll

| Shortcut        | Action                               |
|-----------------|--------------------------------------|
| Page Down       | Scroll down 10 items                 |
| Page Up         | Scroll up 10 items                   |
| Home            | Jump to first item                   |
| End             | Jump to last item                    |

---

## 10. Animation

### 10.1. Appearance

- **Fade in**: opacity 0 -> 1, duration 100ms, ease
- **No slide**: popup appears at its final position, does not slide up/down
- **Reduce motion**: instant appear (no fade)

### 10.2. Disappearance

- **Fade out**: opacity 1 -> 0, duration 80ms, ease
- **Instant dismiss** when accepting completion (no fade delay)

---

## 11. Dismiss Behavior

### 11.1. Dismiss Triggers

| Trigger                        | Action                       |
|--------------------------------|------------------------------|
| Escape                         | Dismiss, do not accept       |
| Click outside popup            | Dismiss, do not accept       |
| Type non-matching character    | Dismiss if no items match    |
| Accept completion (Tab/Enter)  | Dismiss + insert completion  |
| Cursor moves (left/right arrows) | Dismiss                    |
| Terminal loses focus           | Dismiss                      |

### 11.2. Does Not Dismiss When

- Continuing to type matching characters (popup updates filter)
- Navigating with Arrow Up/Down
- Scrolling within the popup

---

## 12. Single Completion Behavior

When there is **only one** matching completion:
- **Do not show popup**: instead, display an inline hint
- **Inline hint**: the remaining part of the completion shown in `--color-text-muted` color right after the cursor
- **Tab**: accepts the inline hint
- **Any other key**: the inline hint disappears

Example:
```
$ git chec|kout              <- "kout" is the inline hint, faded color
```

---

## 13. No Results State

When the user types but no completions match:
- Popup auto-closes (fades out)
- No "No results" message is displayed (not necessary, popup simply disappears)

---

## 14. Performance

| Metric                  | Target                         |
|-------------------------|--------------------------------|
| Popup appear latency    | < 50ms after trigger           |
| Filter/re-render        | < 16ms (60fps)                 |
| Max items rendered      | Virtual scroll if > 100 items  |
| Memory                  | < 5MB for completion cache     |

- **Virtual scrolling**: only render items in viewport (+ buffer of 5 items above/below)
- **Debounce filtering**: 16ms debounce when user types fast
- **Background computation**: completion list is precomputed before user types

---

## 15. Accessibility

- **Role**: popup has `role="listbox"`, `aria-label="Completions"`
- **Items**: `role="option"`, `aria-selected` for selected item
- **Terminal input**: `aria-activedescendant` points to selected item ID
- **Announcement**: when popup opens "N completions available", when navigating "{item name}, {kind}"
- **Focus**: focus remains on terminal input, popup is only controlled via keyboard
- **High contrast**: ensure matched characters and selected state are clearly visible in high contrast mode
