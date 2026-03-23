# Testing Strategy

> **Status:** Active
> **Last Updated:** 2026-03-23
> **Owner:** Wit Team

Testing strategy for the Wit terminal emulator. Goal: ensure correctness of core logic (parser, grid, completion) while keeping the test suite fast and maintainable.

---

## Table of Contents

- [Testing Pyramid](#testing-pyramid)
- [Rust Testing](#rust-testing)
- [Frontend Testing](#frontend-testing)
- [E2E Testing (Future)](#e2e-testing-future)
- [What to Test, What Not to Test](#what-to-test-what-not-to-test)
- [Test Coverage Targets](#test-coverage-targets)
- [CI Test Requirements](#ci-test-requirements)

---

## Testing Pyramid

Wit follows the traditional testing pyramid:

```
        /\
       / E2E \          Few - slow, fragile, high confidence
      /--------\
     /Integration\      Moderate - test module boundaries
    /--------------\
   /   Unit Tests    \   Many - fast, focused, stable
  /--------------------\
```

- **Unit tests** (most): Test individual functions/methods. Run fast (<1 second per test).
- **Integration tests** (moderate): Test interactions between modules. E.g.: parser + grid, PTY + parser.
- **E2E tests** (fewest): Test the entire app from the UI. Only for the most critical flows.

---

## Rust Testing

### Unit Tests

Placed in the same file as the code, under the `#[cfg(test)]` module:

```rust
pub fn parse_params(input: &[u8]) -> Vec<u16> {
    // implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_param() {
        assert_eq!(parse_params(b"42"), vec![42]);
    }

    #[test]
    fn test_parse_multiple_params() {
        assert_eq!(parse_params(b"1;2;3"), vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_empty_params() {
        assert_eq!(parse_params(b""), vec![]);
    }

    #[test]
    fn test_parse_default_param() {
        // Missing param should default to 0
        assert_eq!(parse_params(b";5"), vec![0, 5]);
    }
}
```

**Rules:**

- Every public function should have at least 1 unit test.
- Test both happy path and edge cases (empty input, max values, invalid input).
- Test names describe behavior, not implementation: `test_cursor_wraps_at_line_end`, not `test_cursor_wrap_logic`.

### Integration Tests

Placed in the `tests/` directory at the crate root:

```
src-tauri/
├── src/
│   ├── parser/
│   └── grid/
└── tests/
    ├── parser_integration.rs    # Parser + Action execution
    ├── grid_operations.rs       # Full grid operations
    └── pty_communication.rs     # PTY read/write flows
```

```rust
// tests/parser_integration.rs

use wit_term::parser::Parser;
use wit_term::grid::Grid;

#[test]
fn test_parser_moves_cursor_on_grid() {
    let mut grid = Grid::new(80, 24);
    let mut parser = Parser::new();

    // Simulate writing "Hello" then moving cursor
    let input = b"Hello\x1b[3D"; // Write "Hello", move left 3
    let actions = parser.parse(input);

    for action in actions {
        grid.execute(action);
    }

    assert_eq!(grid.cursor_position(), (2, 0)); // Column 2, Row 0
}
```

### Property-Based Testing

Use `proptest` for modules that need testing with many different inputs:

```rust
// Cargo.toml
// [dev-dependencies]
// proptest = "1"

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parser_never_panics(input in prop::collection::vec(any::<u8>(), 0..1024)) {
        let mut parser = Parser::new();
        // Parser must not panic with any input
        let _ = parser.parse(&input);
    }

    #[test]
    fn test_fuzzy_match_is_symmetric(
        pattern in "[a-z]{1,10}",
        candidate in "[a-z]{1,20}"
    ) {
        let score = fuzzy_match(&pattern, &candidate);
        // Score must always be >= 0 or None
        if let Some(s) = score {
            prop_assert!(s >= 0);
        }
    }
}
```

**Use for:**

- ANSI parser - ensure it never panics with any byte sequence
- Fuzzy matcher - ensure scoring consistency
- Grid operations - ensure bounds are never violated

### Snapshot Testing

Use `insta` for output-based testing, particularly useful for the parser:

```rust
// Cargo.toml
// [dev-dependencies]
// insta = { version = "1", features = ["yaml"] }

use insta::assert_yaml_snapshot;

#[test]
fn test_parse_color_sequence() {
    let mut parser = Parser::new();
    let actions = parser.parse(b"\x1b[31;1mHello\x1b[0m");
    assert_yaml_snapshot!(actions);
}
```

Run `cargo insta review` to view and approve new snapshots.

**Use for:**

- Parser output - ensure escape sequences are parsed correctly
- Grid state - ensure grid state is correct after processing sequences

### Benchmarks

Use `criterion` for performance-critical paths:

```rust
// benches/parser_bench.rs

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use wit_term::parser::Parser;

fn bench_parse_large_output(c: &mut Criterion) {
    let input = include_bytes!("fixtures/large_output.bin");
    let mut parser = Parser::new();

    c.bench_function("parse_large_output", |b| {
        b.iter(|| {
            parser.parse(black_box(input));
        });
    });
}

fn bench_parse_color_heavy(c: &mut Criterion) {
    let input = include_bytes!("fixtures/color_heavy.bin");
    let mut parser = Parser::new();

    c.bench_function("parse_color_heavy", |b| {
        b.iter(|| {
            parser.parse(black_box(input));
        });
    });
}

criterion_group!(benches, bench_parse_large_output, bench_parse_color_heavy);
criterion_main!(benches);
```

**Benchmark for:**

- ANSI parser throughput (MB/s)
- Grid scroll operations
- Fuzzy matching with large lists
- Cell rendering preparation

### Test Fixtures

Place test data in `tests/fixtures/`:

```
tests/fixtures/
├── ansi/
│   ├── simple_text.bin       # Plain text without escape sequences
│   ├── basic_colors.bin      # SGR color sequences
│   ├── cursor_movement.bin   # Cursor movement sequences
│   ├── scroll_region.bin     # Scroll region operations
│   ├── unicode_mixed.bin     # Mixed ASCII + Unicode + escape sequences
│   └── malformed.bin         # Intentionally malformed sequences
└── completions/
    ├── commands.txt          # Test command list
    └── paths/                # Test directory structure
```

---

## Frontend Testing

### Unit Tests

Use **Vitest** for stores, hooks, and utility functions:

```tsx
// src/hooks/__tests__/useKeyBindings.test.ts
import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useKeyBindings } from "../useKeyBindings";

describe("useKeyBindings", () => {
  it("should register and trigger key binding", () => {
    const handler = vi.fn();
    const { result } = renderHook(() => useKeyBindings());

    act(() => {
      result.current.register("Ctrl+T", handler);
    });

    act(() => {
      result.current.trigger("Ctrl+T");
    });

    expect(handler).toHaveBeenCalledOnce();
  });
});
```

### Component Tests

Use **React Testing Library** to test behavior, not implementation:

```tsx
// src/components/__tests__/TabBar.test.tsx
import { render, screen, fireEvent } from "@testing-library/react";
import { TabBar } from "../TabBar";

describe("TabBar", () => {
  it("should display tab titles", () => {
    const tabs = [
      { id: "1", title: "bash" },
      { id: "2", title: "vim" },
    ];

    render(<TabBar tabs={tabs} activeTabId="1" onTabClick={() => {}} />);

    expect(screen.getByText("bash")).toBeInTheDocument();
    expect(screen.getByText("vim")).toBeInTheDocument();
  });

  it("should call onTabClick when tab is clicked", () => {
    const onTabClick = vi.fn();
    const tabs = [{ id: "1", title: "bash" }];

    render(<TabBar tabs={tabs} activeTabId="1" onTabClick={onTabClick} />);

    fireEvent.click(screen.getByText("bash"));
    expect(onTabClick).toHaveBeenCalledWith("1");
  });
});
```

### Do Not Use UI Snapshot Tests

- **Do not use snapshot tests for UI components.** They are very fragile - any small CSS/markup change causes failures.
- Instead, test **behavior** (user interactions, state changes) with React Testing Library.

---

## E2E Testing (Future)

> This section will be implemented when the app is more stable. The plan is documented here for reference.

### Tools

- **Tauri's built-in testing framework** or **Playwright** with Tauri driver.

### Test Scenarios

```
E2E Test: Basic Terminal Usage
1. Launch app
2. Verify terminal prompt is displayed
3. Type "echo hello"
4. Press Enter
5. Verify output contains "hello"
6. Verify prompt is displayed again

E2E Test: Tab Management
1. Launch app (1 tab)
2. Press Ctrl+T (new tab)
3. Verify 2 tabs are displayed
4. Press Ctrl+W (close tab)
5. Verify 1 tab remains

E2E Test: Tab Completion
1. Launch app
2. Type "gi" then press Tab
3. Verify completion suggestions are displayed
4. Verify "git" is in the suggestions
5. Select "git", verify input is updated
```

---

## What to Test, What Not to Test

### Should test (priority high -> low)

| Module | Reason | Test type |
|--------|--------|-----------|
| ANSI parser | Core logic, many edge cases, easy to get wrong | Unit, property-based, snapshot |
| Grid operations | Complex state, bounds checking | Unit, integration |
| Completion matching | Fuzzy logic, scoring, ranking | Unit, property-based |
| Context detection | Command parsing, arg detection | Unit |
| Key binding resolution | Modifier combinations, conflicts | Unit |
| PTY communication | Byte stream handling, encoding | Integration |
| Configuration loading | File parsing, defaults, validation | Unit |

### Should not test

| Module | Reason |
|--------|--------|
| Tauri framework internals | Already tested by the Tauri team |
| React rendering details | Test behavior, not render output |
| CSS/visual appearance | Use visual review, not automated tests |
| Third-party crate logic | Already tested by crate authors |
| Trivial getters/setters | No logic to test |

---

## Test Coverage Targets

| Scope | Target | Reason |
|-------|--------|--------|
| Core modules (parser, grid, completion) | **80%+** | Complex logic, many edge cases |
| Tauri commands | **70%+** | Middle layer, test happy paths |
| Frontend hooks/stores | **70%+** | State logic needs correctness assurance |
| Frontend components | **50%+** | Test behavior, not rendering |
| Overall | **60%+** | Balance between confidence and velocity |

> Coverage is a reference metric, not an absolute target. 80% coverage with meaningful tests is better than 100% coverage with meaningless tests.

### Measuring coverage

```bash
# Rust coverage (using cargo-tarpaulin or llvm-cov)
cargo tarpaulin --out html

# Frontend coverage
pnpm test -- --coverage
```

---

## CI Test Requirements

All tests must pass before merging a PR:

### Required Checks

- `cargo test` - All Rust unit + integration tests
- `cargo clippy` - No warnings
- `cargo fmt --check` - Code is formatted
- `pnpm test` - All frontend tests
- `pnpm lint` - ESLint passes

### Performance

- Total test suite should run in **under 5 minutes** on CI.
- If slower, optimize or parallelize.
- Benchmarks run **separately**, do not block PR merge.

### Flaky Tests

- Flaky tests (pass/fail randomly) must be **fixed immediately** or **temporarily disabled** with a note.
- "Retry until pass" is not an acceptable solution.

> See also: [CI/CD Pipeline](ci-cd.md) for details on CI configuration.
