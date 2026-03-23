# Coding Standards

> **Status:** Active
> **Last Updated:** 2026-03-23
> **Owner:** Wit Team

Conventions and code standards for the Wit project. All contributors must follow these rules to ensure codebase consistency.

---

## Table of Contents

- [Rust Code Style](#rust-code-style)
- [TypeScript/React Code Style](#typescriptreact-code-style)
- [Shared Conventions](#shared-conventions)

---

## Rust Code Style

### Formatting

- Use **rustfmt** with default configuration. If customization is needed, add to `rustfmt.toml`:

```toml
# rustfmt.toml
edition = "2021"
max_width = 100
use_field_init_shorthand = true
use_try_shorthand = true
```

- Run `cargo fmt` before every commit. CI will reject unformatted code.

### Clippy Lints

- All Clippy warnings are enabled by default.
- Lints that are **denied** (not allowed):

```rust
// src-tauri/src/main.rs or lib.rs
#![deny(clippy::unwrap_used)]       // Do not use .unwrap() in production code
#![deny(clippy::expect_used)]       // Use Result/Option handling instead
#![deny(clippy::panic)]             // Do not use panic! in production
#![warn(clippy::all)]               // Warn for all other lints
#![warn(clippy::pedantic)]          // Warn for pedantic lints
```

> **Exception:** `unwrap()` and `expect()` are allowed in test code (`#[cfg(test)]`).

### Naming Conventions

| Type | Convention | Example |
|------|-----------|---------|
| Functions, methods | `snake_case` | `parse_escape_sequence()` |
| Variables, fields | `snake_case` | `cursor_position` |
| Types, structs, enums | `CamelCase` | `TerminalGrid`, `AnsiColor` |
| Enum variants | `CamelCase` | `Color::BrightRed` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_SCROLLBACK_LINES` |
| Modules | `snake_case` | `mod ansi_parser` |
| Type parameters | Single uppercase | `T`, `E`, `R` |

### Error Handling

- **Library errors** (modules like parser, grid): use `thiserror` to define structured errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("invalid escape sequence: {0}")]
    InvalidEscapeSequence(String),
    #[error("unexpected end of input")]
    UnexpectedEof,
}
```

- **Application errors** (Tauri commands, main): use `anyhow` for convenience:

```rust
use anyhow::{Context, Result};

fn load_config() -> Result<Config> {
    let content = std::fs::read_to_string("config.toml")
        .context("failed to read config file")?;
    // ...
}
```

- **Never** use `.unwrap()` or `.expect()` in production code. Use the `?` operator or handle explicitly.

### Documentation

- `///` for all public items (functions, structs, enums, traits):

```rust
/// Parse a single ANSI escape sequence from the input buffer.
///
/// Returns the parsed action and the number of bytes consumed.
/// Returns `None` if the buffer does not contain a complete sequence.
pub fn parse_sequence(input: &[u8]) -> Option<(Action, usize)> {
    // ...
}
```

- `//!` for module-level documentation:

```rust
//! ANSI/VT terminal parser.
//!
//! This module implements a state-machine-based parser for ANSI escape
//! sequences following the VT100/VT220/xterm specifications.
```

### Testing

- **Unit tests** placed in the same file, under the `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_csi() {
        let result = parse_sequence(b"\x1b[1;2H");
        assert_eq!(result, Some((Action::CursorPosition(1, 2), 6)));
    }
}
```

- **Integration tests** placed in the `tests/` directory at the crate root.
- Name tests to describe behavior: `test_cursor_moves_to_correct_position()`, not `test_1()`.

### Unsafe Code

- **Avoid `unsafe`** unless truly necessary (FFI, performance-critical code that has been benchmarked).
- If using `unsafe`, **mandatory** to document safety invariants:

```rust
// SAFETY: `ptr` is guaranteed to be valid and aligned because it comes
// from a Vec allocation that has not been deallocated. The length is
// bounds-checked above.
unsafe {
    std::ptr::copy_nonoverlapping(src, dst, len);
}
```

### Dependencies

- Prefer crates that are well-maintained, have many downloads, and few dependencies.
- When adding a new dependency, note the reason in the PR description.
- Check license compatibility (MIT/Apache-2.0 preferred).
- Use `cargo deny` to audit dependencies.

---

## TypeScript/React Code Style

### TypeScript Configuration

- Enable **strict mode** in `tsconfig.json`:

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true
  }
}
```

### Components

- Use only **functional components**, do not use class components.
- One component per file. File name matches component name:

```
components/
├── Terminal.tsx           # <Terminal /> component
├── TabBar.tsx             # <TabBar /> component
└── CommandPalette.tsx     # <CommandPalette /> component
```

- Define props with `interface`, do not use `type` for props:

```tsx
interface TerminalProps {
  sessionId: string;
  onClose: () => void;
  className?: string;
}

export function Terminal({ sessionId, onClose, className }: TerminalProps) {
  // ...
}
```

### Hooks

- Custom hooks placed in the `hooks/` directory:

```
hooks/
├── useTerminal.ts
├── useKeyBindings.ts
└── useCompletions.ts
```

- Hook names always start with `use`.
- Each hook does one thing, keep it small and focused.

### Naming Conventions

| Type | Convention | Example |
|------|-----------|---------|
| Variables, functions | `camelCase` | `cursorPosition`, `handleKeyDown` |
| Components | `PascalCase` | `Terminal`, `TabBar` |
| Interfaces, types | `PascalCase` | `TerminalProps`, `SessionState` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_TABS`, `DEFAULT_FONT_SIZE` |
| Files (components) | `PascalCase.tsx` | `Terminal.tsx` |
| Files (utilities) | `camelCase.ts` | `parseAnsi.ts` |
| CSS classes | Tailwind utilities | `className="flex items-center"` |

### Imports

Order imports as follows, separated by blank lines:

```tsx
// 1. External packages
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

// 2. Internal modules (absolute imports)
import { useTerminal } from "@/hooks/useTerminal";
import { Terminal } from "@/components/Terminal";

// 3. Types (use type import when only importing types)
import type { SessionState } from "@/types/session";
```

- Use absolute imports from `src/` (configure the `@/` alias in `vite.config.ts` and `tsconfig.json`).

### Type Safety

- **Do not use `any`**. If the exact type is unknown, use `unknown` and narrow:

```tsx
// WRONG
function processData(data: any) { ... }

// CORRECT
function processData(data: unknown) {
  if (typeof data === "string") {
    // data is a string here
  }
}
```

- Use discriminated unions for state management:

```tsx
type ConnectionState =
  | { status: "disconnected" }
  | { status: "connecting" }
  | { status: "connected"; sessionId: string }
  | { status: "error"; message: string };
```

### Tailwind CSS

- **Utility-first**: use Tailwind classes directly in JSX.
- When a pattern repeats **3 or more times**, extract into a component.
- Use the `cn()` utility (clsx + tailwind-merge) to merge conditional classes:

```tsx
import { cn } from "@/lib/utils";

<div className={cn(
  "flex items-center gap-2 px-3 py-1",
  isActive && "bg-accent text-accent-foreground",
  className
)} />
```

- Do not write custom CSS unless Tailwind does not support it (very rare).

---

## Shared Conventions

### File Length

- **Limit to ~300 lines** per file. If a file is longer, split into smaller modules/components.
- This is a guideline, not a hard rule - flexibility is allowed if splitting would break logical coherence.

### Variable Naming

- Use **full descriptive names**, avoid abbreviations:

```
// WRONG
let cb = getCompletions();
let pos = getCursorPos();
let buf = readInput();

// CORRECT
let completions = getCompletions();
let cursorPosition = getCursorPos();
let inputBuffer = readInput();
```

- **Exceptions** - common abbreviations that are accepted:
  - `id` (identifier)
  - `url` (uniform resource locator)
  - `ctx` (context - only in Rust)
  - `i`, `j`, `k` (loop indices)
  - `tx`, `rx` (channel sender/receiver - only in Rust)

### Comments

- Comments explain **WHY**, not **WHAT**:

```rust
// WRONG - reading the code makes it obvious
// Increment counter by 1
counter += 1;

// CORRECT - explains the reason
// We skip the first byte because the escape character has already been
// consumed by the caller.
let params = &input[1..];
```

- Do not leave **commented-out code** in the main branch. Use version control to preserve history.

### TODO Format

- Use a consistent format to make them easy to find and track:

```rust
// TODO(username): Implement scrollback buffer compression
// TODO(username): This is O(n^2), optimize when grid size > 1000 rows
```

```tsx
// TODO(username): Add keyboard shortcut for this action
// TODO(username): Extract to shared hook
```

- TODOs should be linked to GitHub issues when possible.

### Logging

- **Rust:** Use the `tracing` crate with appropriate levels:
  - `error!` - Errors requiring attention
  - `warn!` - Potential issues
  - `info!` - Important information (startup, shutdown)
  - `debug!` - Information useful for debugging
  - `trace!` - Detailed information (every escape sequence, every keystroke)

- **TypeScript:** Use `console.error`, `console.warn`, `console.info`. Avoid `console.log` in production code.
