# Agent Sidebar (Right)

> **Status:** draft
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## 1. Overview

The right sidebar is dedicated to agent monitoring. It provides real-time visibility into AI coding agents (Claude Code, Aider, Codex CLI, etc.) running inside a terminal session. The sidebar auto-opens when the system detects an agent process and displays activity, file changes, conversation history, and cost tracking. It does not require any configuration from the user — detection and display are fully automatic.

---

## 2. Layout & Dimensions

### 2.1. Dimensions

| Property        | Value                          |
|-----------------|--------------------------------|
| Default width   | 360px                          |
| Min width       | 280px (when resizing)          |
| Max width       | 50% of viewport width          |
| Collapsed width | 0px (completely hidden)        |

### 2.2. Resize

- **Drag handle**: left border of sidebar, 4px hit area
- **Cursor**: `col-resize` when hovering over drag handle
- **Visual**: drag handle shows a 2px vertical line colored `--color-border` on hover
- **Snap**: no snap points, free resize within the 280px–50% range
- **Persist**: width is saved to user preferences, per-user (not per-session)

### 2.3. Structure

```
+----------------------------------+
|  [icon] Claude Code    [$] [X]   |  <- Header (agent name, cost, close)
+----------------------------------+
|  [Activity] [Files] [Conversation]|  <- Tab Bar
+----------------------------------+
|                                  |
|                                  |
|  (Tab Content Area)              |  <- Scrollable content
|                                  |
|                                  |
|                                  |
+----------------------------------+
|  [Pause] [Undo] [Stop] [Approve]|  <- Actions Bar
+----------------------------------+
```

---

## 3. Open/Close Behavior

### 3.1. Auto-Open

- **Trigger**: Layer 1 emits an `AgentDetected` event for the active terminal session
- **Animation**: slide in from right, 200ms ease-out
- **Condition**: sidebar must not have been manually closed by the user during this session; if the user closed it manually, auto-open is suppressed until the next agent session

### 3.2. Auto-Close

- **Never**: the sidebar does not auto-close when an agent exits
- **Rationale**: the user may want to review the session summary, file changes, or conversation history after the agent finishes
- **State**: transitions to "Session Ended" state (see Section 8)

### 3.3. Manual Toggle

| Method            | Description                              |
|-------------------|------------------------------------------|
| Ctrl+Shift+A      | Toggle agent sidebar visibility          |
| Button             | Icon button in the terminal toolbar area |
| Collapse button   | Close button [X] in sidebar header       |

### 3.4. Animation

- **Duration**: 200ms ease-out
- **Effect**: slide right (collapse), slide left (expand)
- **Terminal resize**: terminal view expands/shrinks accordingly, triggers grid recalculation
- **Reduce motion**: instant show/hide instead of slide

### 3.5. Per-Tab Isolation

- Each terminal tab maintains its own independent agent sidebar state
- Switching tabs switches the sidebar content (or hides it if that tab has no agent)
- Width preference is shared across all tabs

---

## 4. Header Section

### 4.1. Layout

```
+----------------------------------------------+
|  [icon] Claude Code   gpt-4  12.4k  $0.03 [X]|
+----------------------------------------------+
   ^      ^              ^      ^      ^      ^
   |      |              |      |      |      Close button
   |      |              |      |      Cost counter
   |      |              |      Token counter
   |      |              Model badge
   |      Agent name
   Agent icon
```

### 4.2. Left Group

- **Agent icon**: 20x20px, agent-specific icon (e.g., Claude icon, Aider icon, generic bot icon)
- **Agent name**: detected agent name, font `--text-md`, weight 600, color `--color-text`

### 4.3. Center Group

- **Model badge**: pill-shaped badge, font `--text-xs`, background `--color-surface-hover`, color `--color-text-secondary`. Displays the model name when available (e.g., "opus", "sonnet"). Hidden if unknown.
- **Token counter**: font `--text-xs`, color `--color-text-secondary`. Format: abbreviated (e.g., "12.4k"). Hidden if data unavailable.
- **Cost counter**: font `--text-xs`, color `--color-text-secondary`. Format: "$0.03". Hidden if data unavailable.

### 4.4. Right Group

- **Close button**: icon button (X), 28x28px, ghost style, `aria-label="Close agent sidebar"`

### 4.5. Data Source

- Token and cost data comes from Layer 2 (output parsing) or Layer 4 (Wit Protocol)
- If neither source provides data, the counters are hidden (not shown as zero)
- **Height**: 48px (`--sp-12`)
- **Padding**: 0 `--sp-3`
- **Background**: `--color-surface`
- **Border bottom**: 1px solid `--color-border-muted`

---

## 5. Tabs

### 5.1. Tab Bar

```
+----------------------------------------------+
|  [Activity]    [Files]    [Conversation]      |
+----------------------------------------------+
```

- **Height**: 36px
- **Style**: underline indicator on active tab, 2px solid `--color-primary`
- **Font**: `--text-sm`, weight 500
- **Active tab text**: `--color-text`
- **Inactive tab text**: `--color-text-secondary`
- **Hover**: `--color-text`, background `--color-surface-hover`
- **Badge**: each tab may show a count badge (e.g., Files tab shows "3" for 3 changed files)

### 5.2. Activity Tab

Vertical timeline showing the agent's actions in chronological order.

#### Timeline Entry Structure

```
  [icon]  Action description            2m ago
          Optional detail text (collapsible)
     |
  [icon]  Next action                   1m ago
     |
  [icon]  Current action (highlighted)    now
```

#### Entry Types

| Type       | Icon        | Description                           |
|------------|-------------|---------------------------------------|
| Thinking   | Brain       | Agent reasoning, collapsible content  |
| Tool Use   | Wrench      | Tool/command invocation               |
| File Edit  | File        | File path shown alongside icon        |
| Error      | X (red)     | Error message, always expanded        |

#### Behavior

- **Current entry**: highlighted with `--color-primary` accent on the left border (2px)
- **Auto-scroll**: scrolls to the latest entry as new entries appear; pauses auto-scroll if user has scrolled up manually
- **Timestamps**: relative format ("2m ago", "just now") switching to absolute format ("14:32") after 1 hour
- **Collapsible**: Thinking and Tool Use entries can be expanded/collapsed; collapsed by default except the current entry

### 5.3. Files Tab

List of files changed by the agent during the current session.

#### File Entry Structure

```
  [+]  src/components/Button.tsx        [Undo]
  [~]  src/utils/helpers.ts             [Undo]
  [-]  src/old-module.ts                [Undo]
```

#### Behavior

- **Icons**: green `+` (created), yellow `~` (modified), red `-` (deleted)
- **Path**: relative to project root, font `--text-sm`, monospace
- **Click to expand**: shows inline diff view below the entry
- **Diff view**: red/green line coloring for removed/added lines, syntax highlighted using the current theme
- **Git baseline**: comparison is against the git state at the moment the agent session started (Layer 3 snapshot)
- **"Undo" button**: per-file, appears on hover, restores the file to its baseline state via `git checkout` or `git restore`
- **Summary bar** (top of Files tab): "3 files changed (+45 -12)", font `--text-xs`, color `--color-text-secondary`

### 5.4. Conversation Tab

Readable view of the agent conversation, distinct from the raw terminal output.

#### Message Styling

| Role       | Alignment    | Background                  |
|------------|--------------|------------------------------|
| User       | Right-aligned| `--color-surface-hover`      |
| Assistant  | Left-aligned | `--color-surface`            |

#### Content Rendering

- **Code blocks**: syntax highlighted, with language label and copy button
- **Thinking blocks**: collapsible, dimmed text (`--color-text-muted`), italic, prefixed with "Thinking..." label
- **Tool calls**: compact single-line display (tool name + summary), click to expand full input/output
- **Markdown**: rendered inline (bold, italic, lists, links)

---

## 6. Actions Bar

Always visible at the bottom of the sidebar, regardless of which tab is active.

### 6.1. Layout

```
+----------------------------------------------+
|  [Pause]  [Undo Last]  [Stop]  [Approve]     |
+----------------------------------------------+
```

- **Height**: 48px
- **Padding**: `--sp-2` vertical, `--sp-3` horizontal
- **Background**: `--color-surface`
- **Border top**: 1px solid `--color-border-muted`
- **Button style**: compact icon+label buttons, `--text-xs`
- **Button gap**: `--sp-2`

### 6.2. Actions

| Button       | Icon    | Behavior                                                                 |
|--------------|---------|--------------------------------------------------------------------------|
| Pause/Resume | Pause   | Sends SIGTSTP to pause, SIGCONT to resume (Layer 1). Toggles label.     |
| Undo Last    | Undo    | Reverts the most recent file change (Layer 3). Disabled if no changes.   |
| Stop         | Square  | Sends SIGTERM to the agent process. Confirmation dialog before sending.  |
| Approve      | Check   | Visible only when a Layer 4 approval request is pending. Green style.    |
| Reject       | X       | Visible only when a Layer 4 approval request is pending. Red style.      |

### 6.3. Button States

- **Disabled**: grayed out (`opacity: 0.4`), no pointer events. Pause is disabled when no agent is running. Undo Last is disabled when no file changes exist.
- **Active/Toggle**: Pause button shows "Resume" label and Play icon when the agent is paused
- **Approve/Reject**: these two buttons replace the normal actions bar when an approval request is pending, with a description of the request shown above them

---

## 7. Approval Request Card

When a Layer 4 agent sends an approval request, a card appears above the actions bar (or replaces the tab content area, depending on priority).

### 7.1. Layout

```
+----------------------------------------------+
|  Approval Required                            |
|                                               |
|  Action: Delete file                          |
|  Target: src/legacy/old-module.ts             |
|  Reason: "File is no longer referenced"       |
|                                               |
|  Timeout: 28s remaining                       |
|                                               |
|  [Approve]                    [Reject]        |
+----------------------------------------------+
```

- **Background**: `--color-surface`, border 1px solid `--color-warning`
- **Border-radius**: `--radius-md`
- **Approve button**: green (`--color-success`), full width half
- **Reject button**: red (`--color-error`), full width half
- **Timeout**: countdown timer, `--color-warning`, hidden if no timeout set
- **Auto-timeout**: if the timeout expires, the request is auto-rejected and the card dismisses

---

## 8. States

### 8.1. Loading

- **When**: agent detected but adapter is initializing
- **Display**: spinner + "Connecting to {agent name}..."
- **Duration**: typically < 1 second

### 8.2. Active

- **When**: agent is running and being monitored
- **Display**: full sidebar with header, tabs, and actions bar
- **Header**: shows live token/cost counters

### 8.3. Session Ended

- **When**: agent process exits
- **Display**: summary card at the top of the content area
- **Summary card content**: duration, total tokens, total cost, number of files changed
- **Tabs**: remain browsable (Activity, Files, Conversation still contain session data)
- **Actions bar**: all buttons disabled except a "Browse History" link
- **Header**: agent name with "(ended)" suffix, counters show final totals

### 8.4. No Agent

- **When**: no agent detected in the current terminal session
- **Display**: centered message in the content area

```
+----------------------------------------------+
|                                               |
|           [Bot Icon - 48px]                   |
|                                               |
|     No AI agent detected.                     |
|     Run an agent CLI to activate              |
|     the dashboard.                            |
|                                               |
+----------------------------------------------+
```

- **Icon**: Bot/Robot, 48px, `--color-text-muted`
- **Text**: `--text-base`, `--color-text-secondary`, centered
- **Sidebar may be hidden**: if no agent has ever been detected in this tab, the sidebar remains closed

---

## 9. Accessibility

- **Role**: sidebar has `role="complementary"` and `aria-label="Agent monitoring"`
- **Tab bar**: `role="tablist"`, each tab has `role="tab"` and `aria-selected`
- **Tab content**: `role="tabpanel"`, `aria-labelledby` referencing the corresponding tab
- **Actions bar**: buttons have descriptive `aria-label` values (e.g., `aria-label="Pause agent"`, `aria-label="Stop agent process"`)
- **Close button**: `aria-label="Close agent sidebar"`
- **Approval card**: `role="alertdialog"`, `aria-label="Approval request"`, auto-focused when it appears
- **Keyboard navigation**:
  - Tab key moves between interactive elements within the sidebar
  - Arrow Left/Right switches between tabs
  - Escape returns focus to the terminal
  - Enter activates the focused button or expands a collapsed entry
- **Focus management**: when the sidebar opens (auto or manual), focus moves to the first tab; when it closes, focus returns to the terminal
- **Screen reader**: activity timeline entries are announced as they appear (via `aria-live="polite"` on the timeline container)
