# Agent Dashboard Components

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Overview

This document specifies the individual components used inside the agent sidebar (right sidebar). Each component is designed to be self-contained with clear data inputs, visual states, and interaction behaviors. Components receive data from Layer 2 (output parsing adapters) or Layer 4 (Wit Protocol) and render it in a consistent, readable format.

---

## 2. Token/Cost Counter

The token and cost counters appear in the sidebar header and provide real-time usage tracking for the active agent session.

### 2.1. Layout

```
Tokens: 12.4k | $0.03
```

- **Position**: inline within the sidebar header, center group
- **Font**: `--text-xs`, monospace for numeric values
- **Separator**: `|` character, color `--color-border`

### 2.2. Data Format

| Value         | Format                                                  | Examples                      |
|---------------|---------------------------------------------------------|-------------------------------|
| Tokens < 1k  | Exact integer                                           | `142`, `999`                  |
| Tokens >= 1k | Abbreviated with one decimal                            | `1.2k`, `12.4k`, `142.8k`    |
| Tokens >= 1M | Abbreviated with one decimal                            | `1.2M`                        |
| Cost          | Dollar sign, two decimal places                         | `$0.00`, `$0.03`, `$12.50`   |

### 2.3. Update Behavior

- **Source**: Layer 2 `TokenUpdate` events or Layer 4 `usage` messages
- **Frequency**: real-time as events arrive, no artificial throttle
- **Animation**: no animation on update (instant value replacement)

### 2.4. Color Thresholds

| State     | Condition     | Token color               | Cost color                |
|-----------|---------------|---------------------------|---------------------------|
| Normal    | Cost <= $1.00 | `--color-text-secondary`  | `--color-text-secondary`  |
| Warning   | Cost > $1.00  | `--color-text-secondary`  | `--color-warning`         |
| Danger    | Cost > $5.00  | `--color-text-secondary`  | `--color-error`           |

### 2.5. Hidden State

- If the adapter cannot extract token/cost data, the counters are hidden entirely (not shown as zero)
- If only one value is available (e.g., tokens but not cost), display only the available value without the separator

---

## 3. Activity Timeline

Vertical timeline displaying the agent's actions in chronological order. This is the primary view for understanding what the agent is doing in real time.

### 3.1. Container

- **Padding**: `--sp-3` horizontal, `--sp-2` vertical
- **Overflow**: vertical scroll
- **Scrollbar**: thin (6px), auto-hide
- **Background**: `--color-bg`

### 3.2. Timeline Entry

```
  [icon]  Action title                    2m ago
     |    Optional detail content
     |    (collapsible for some types)
     |
```

#### Layout

- **Icon**: 16px, positioned left, with a vertical line (`--color-border-muted`, 1px) connecting entries
- **Title**: font `--text-sm`, weight 500, color `--color-text`
- **Timestamp**: font `--text-xs`, color `--color-text-muted`, right-aligned
- **Detail content**: font `--text-xs`, color `--color-text-secondary`, indented below the title
- **Entry padding**: `--sp-2` vertical
- **Entry gap**: 0 (connected by the vertical line)

### 3.3. Entry Types

| Type       | Icon          | Icon color               | Detail behavior              |
|------------|---------------|--------------------------|------------------------------|
| Thinking   | Brain         | `--color-text-muted`     | Collapsible, collapsed by default |
| Tool Use   | Wrench        | `--color-primary`        | Collapsible, shows tool name + args |
| File Edit  | File          | `--color-warning`        | Shows file path, not collapsible   |
| Error      | X (circle)    | `--color-error`          | Always expanded, red background tint |

### 3.4. Current Entry Highlight

- The most recent (current) entry has a 2px left border colored `--color-primary`
- Background: `--color-surface-active` (subtle highlight)
- If the entry is collapsible, it is expanded by default when it is the current entry

### 3.5. Auto-Scroll

- **Default**: auto-scroll to the latest entry as new entries are appended
- **Pause**: if the user scrolls up manually (scroll position > 50px from bottom), auto-scroll pauses
- **Resume**: auto-scroll resumes when the user scrolls back to the bottom (within 50px)
- **Indicator**: when auto-scroll is paused, a small "New activity" pill appears at the bottom of the timeline; clicking it scrolls to the latest entry and resumes auto-scroll

### 3.6. Timestamps

| Age            | Format          | Example        |
|----------------|-----------------|----------------|
| < 60 seconds   | "just now"      | just now        |
| 1-59 minutes   | "{n}m ago"      | 2m ago          |
| 60 minutes     | Absolute time   | 14:32           |
| > 60 minutes   | Absolute time   | 14:32           |

- Timestamps update every 30 seconds (not per-second) to avoid unnecessary re-renders

---

## 4. File Change List

Displays all files created, modified, or deleted by the agent during the current session.

### 4.1. Summary Bar

```
+----------------------------------------------+
|  3 files changed  (+45 -12)                   |
+----------------------------------------------+
```

- **Position**: top of the Files tab, sticky (does not scroll)
- **Font**: `--text-xs`, color `--color-text-secondary`
- **Additions**: green color `--color-success`
- **Deletions**: red color `--color-error`
- **Background**: `--color-surface`
- **Border bottom**: 1px solid `--color-border-muted`
- **Height**: 32px
- **Padding**: `--sp-2` vertical, `--sp-3` horizontal

### 4.2. File Entry

```
  [+]  src/components/Button.tsx              [Undo]
```

#### Layout

- **Height**: 32px
- **Padding**: `--sp-1` vertical, `--sp-3` horizontal
- **Icon**: 14px, left-positioned
- **Path**: font `--text-sm`, monospace (`--font-mono`), truncate with ellipsis from the left (show filename, truncate directory path)
- **Undo button**: ghost button, `--text-xs`, visible on hover only

#### Icons

| Status   | Icon | Color              |
|----------|------|--------------------|
| Created  | `+`  | `--color-success`  |
| Modified | `~`  | `--color-warning`  |
| Deleted  | `-`  | `--color-error`    |

### 4.3. Inline Diff View

When a file entry is clicked, an inline diff view expands below it.

```
  [~]  src/utils/helpers.ts                   [Undo]
       ┌────────────────────────────────────┐
       │ - const old = getValue();          │
       │ + const result = getNewValue();    │
       │   return result;                   │
       └────────────────────────────────────┘
```

#### Styling

- **Removed lines**: background `--color-error` at 10% opacity, text `--color-error`
- **Added lines**: background `--color-success` at 10% opacity, text `--color-success`
- **Context lines**: no background, text `--color-text-secondary`
- **Syntax highlighting**: applied using the current terminal theme colors
- **Font**: monospace (`--font-mono`), `--text-xs`
- **Max height**: 300px, scrollable if longer
- **Border**: 1px solid `--color-border-muted`, `--radius-sm`
- **Margin**: `--sp-2` left indent, `--sp-1` vertical

### 4.4. Git Baseline Comparison

- **Baseline snapshot**: taken at the moment the agent session starts (Layer 3)
- **Comparison method**: diff between baseline and current file state
- **Untracked files**: files not in git at baseline time are shown as "Created" if they now exist
- **Deleted files**: files in baseline that no longer exist are shown as "Deleted"

### 4.5. Undo Button

- **Behavior**: restores the file to its baseline state
- **Method**: `git checkout -- {path}` for modified files, `git restore --staged {path}` for staged files, `rm {path}` for created files, `git checkout -- {path}` for deleted files
- **Confirmation**: no confirmation dialog for individual file undo (action is easily reversible by the agent re-running)
- **Disabled state**: grayed out if the file is already at baseline
- **After undo**: file entry is removed from the list, summary bar updates

---

## 5. Conversation View

Displays the agent conversation in a readable chat format, separate from the raw terminal output.

### 5.1. Container

- **Padding**: `--sp-3` horizontal, `--sp-2` vertical
- **Overflow**: vertical scroll
- **Scrollbar**: thin (6px), auto-hide
- **Background**: `--color-bg`

### 5.2. Message Bubbles

#### User Messages

- **Alignment**: right-aligned
- **Background**: `--color-surface-hover`
- **Border-radius**: `--radius-md`, with bottom-right corner squared (`--radius-xs`)
- **Max width**: 85% of container
- **Font**: `--text-sm`, color `--color-text`
- **Padding**: `--sp-2` vertical, `--sp-3` horizontal

#### Assistant Messages

- **Alignment**: left-aligned
- **Background**: `--color-surface`
- **Border-radius**: `--radius-md`, with bottom-left corner squared (`--radius-xs`)
- **Max width**: 85% of container
- **Font**: `--text-sm`, color `--color-text`
- **Padding**: `--sp-2` vertical, `--sp-3` horizontal

### 5.3. Code Blocks

- **Background**: `--color-bg`
- **Border**: 1px solid `--color-border-muted`
- **Border-radius**: `--radius-sm`
- **Font**: monospace (`--font-mono`), `--text-xs`
- **Language label**: top-right corner, `--text-xs`, `--color-text-muted`
- **Copy button**: icon button, top-right, visible on hover
- **Syntax highlighting**: using current theme colors
- **Max height**: 400px, scrollable if longer
- **Padding**: `--sp-3`

### 5.4. Thinking Blocks

- **Collapsible**: collapsed by default, with a "Thinking..." toggle label
- **Toggle label**: font `--text-xs`, italic, color `--color-text-muted`, with a caret icon
- **Expanded content**: font `--text-xs`, italic, color `--color-text-muted`
- **Background**: none (inline within the message flow)
- **Left border**: 2px solid `--color-border-muted` when expanded (blockquote style)
- **Padding**: `--sp-2` left when expanded

### 5.5. Tool Call Blocks

- **Compact display**: single line showing tool name and brief summary
- **Format**: `[wrench icon] tool_name(arg1, arg2)` in `--text-xs`, monospace
- **Click to expand**: reveals full input parameters and output result
- **Expanded background**: `--color-surface-hover`
- **Border-radius**: `--radius-sm`

### 5.6. Message Spacing

- **Gap between messages**: `--sp-3`
- **Gap between same-role consecutive messages**: `--sp-1`
- **Timestamp between message groups**: shown between messages that are > 2 minutes apart, centered, `--text-xs`, `--color-text-muted`

---

## 6. Approval Request Card

Appears when a Layer 4 agent sends an `approval_request` message through the Wit Protocol. This component takes priority over normal tab content.

### 6.1. Layout

```
+----------------------------------------------+
|  ⚠ Approval Required                         |
+----------------------------------------------+
|                                               |
|  Action:  Delete file                         |
|  Target:  src/legacy/old-module.ts            |
|  Reason:  "File is no longer referenced       |
|            anywhere in the codebase"          |
|                                               |
|  ⏱ Auto-timeout in 28s                       |
|                                               |
|  [  Approve  ]         [  Reject  ]           |
+----------------------------------------------+
```

### 6.2. Styling

- **Background**: `--color-surface`
- **Border**: 2px solid `--color-warning`
- **Border-radius**: `--radius-md`
- **Margin**: `--sp-3`
- **Padding**: `--sp-4`
- **Shadow**: `--shadow-md`

### 6.3. Header

- **Icon**: warning triangle, 16px, `--color-warning`
- **Text**: "Approval Required", font `--text-md`, weight 600, color `--color-text`

### 6.4. Details

| Field  | Style                                               |
|--------|-----------------------------------------------------|
| Label  | `--text-xs`, weight 600, color `--color-text-muted` |
| Value  | `--text-sm`, color `--color-text`, monospace for paths |

### 6.5. Buttons

| Button   | Background         | Text color     | Width  |
|----------|--------------------|----------------|--------|
| Approve  | `--color-success`  | white          | 50%    |
| Reject   | `--color-error`    | white          | 50%    |

- **Height**: 36px
- **Border-radius**: `--radius-sm`
- **Gap**: `--sp-2` between buttons
- **Font**: `--text-sm`, weight 600

### 6.6. Timeout Display

- **Format**: "Auto-timeout in {n}s"
- **Font**: `--text-xs`, color `--color-text-muted`
- **Countdown**: updates every second
- **Icon**: clock icon, 12px
- **Hidden**: if the request has no timeout
- **On timeout**: card auto-dismisses, request is auto-rejected, a toast notification appears

### 6.7. Interaction

- **Focus**: Approve button is focused by default when the card appears
- **Keyboard**: Enter to approve, Escape to reject
- **After action**: card dismisses with a brief success/error confirmation inline

---

## 7. Session Ended State

Displayed when the agent process exits. Replaces the active content with a summary while keeping historical data browsable.

### 7.1. Summary Card

```
+----------------------------------------------+
|  Session Complete                             |
|                                               |
|  Duration     12m 34s                         |
|  Tokens       142.8k                          |
|  Cost         $1.23                            |
|  Files        7 changed (+128 -45)            |
|                                               |
|  [Browse History]                             |
+----------------------------------------------+
```

### 7.2. Styling

- **Position**: top of the content area, above the tabs
- **Background**: `--color-surface`
- **Border**: 1px solid `--color-border-muted`
- **Border-radius**: `--radius-md`
- **Margin**: `--sp-3`
- **Padding**: `--sp-4`

### 7.3. Fields

| Field    | Format                          | Example              |
|----------|---------------------------------|----------------------|
| Duration | `{m}m {s}s` or `{h}h {m}m`     | 12m 34s, 1h 5m       |
| Tokens   | Abbreviated (same as counter)   | 142.8k               |
| Cost     | Dollar, two decimal places      | $1.23                |
| Files    | Count + net additions/deletions | 7 changed (+128 -45) |

### 7.4. Browse History Link

- **Style**: text link, `--color-primary`, underline on hover
- **Behavior**: opens a history view (future feature) or keeps the current tabs browsable
- **Tabs**: Activity, Files, and Conversation tabs remain fully functional and browsable after session end

---

## 8. Empty State (No Agent)

Displayed when no agent has been detected in the current terminal session.

### 8.1. Layout

```
+----------------------------------------------+
|                                               |
|                                               |
|            [Bot Icon - 48px]                  |
|                                               |
|      No AI agent detected.                    |
|      Run an agent CLI to activate             |
|      the dashboard.                           |
|                                               |
|                                               |
+----------------------------------------------+
```

### 8.2. Styling

- **Icon**: Bot/Robot, 48px, `--color-text-muted`
- **Heading**: "No AI agent detected.", font `--text-base`, weight 500, color `--color-text-secondary`
- **Subtext**: "Run an agent CLI to activate the dashboard.", font `--text-sm`, color `--color-text-muted`
- **Alignment**: centered both horizontally and vertically in the content area
- **Gap**: `--sp-3` between icon and heading, `--sp-1` between heading and subtext

### 8.3. Visibility

- This state is shown when the sidebar is manually opened but no agent is running
- If the sidebar was never opened and no agent was detected, the sidebar remains hidden entirely (the user never sees this state unless they explicitly open the sidebar)
