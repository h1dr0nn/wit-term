# Git Workflow

> **Status:** Active
> **Last Updated:** 2026-03-23
> **Owner:** Wit Team

Git workflow for the Wit project. Uses trunk-based development with short-lived feature branches.

---

## Table of Contents

- [Branching Model](#branching-model)
- [Branch Naming](#branch-naming)
- [Commit Message Format](#commit-message-format)
- [Pull Request Workflow](#pull-request-workflow)
- [Code Review Guidelines](#code-review-guidelines)
- [Merge Strategy](#merge-strategy)
- [Release Tagging](#release-tagging)
- [Protected Branch Rules](#protected-branch-rules)

---

## Branching Model

Wit uses **trunk-based development**:

- **`main`** is the primary branch - always in a deployable state.
- Every change is made on **short-lived feature branches** (lasting 1-3 days, maximum 1 week).
- No `develop` or `staging` branches. Main is the source of truth.

```
main ─────────●───────●───────●───────●───────→
               \     /         \     /
                feat/x          fix/y
```

---

## Branch Naming

Use prefixes to categorize branches:

| Prefix | Purpose | Example |
|--------|---------|---------|
| `feature/` | New feature | `feature/split-pane` |
| `fix/` | Bug fix | `fix/cursor-overflow` |
| `refactor/` | Code restructuring | `refactor/parser-state-machine` |
| `docs/` | Documentation update | `docs/setup-guide` |
| `test/` | Add/fix tests | `test/parser-edge-cases` |
| `chore/` | Build, tooling, config | `chore/update-dependencies` |
| `perf/` | Performance improvement | `perf/grid-rendering` |

**Rules:**

- Use **kebab-case** for branch names: `feature/tab-completion`, not `feature/tabCompletion`.
- Keep names short but descriptive: `fix/cursor-overflow` is better than `fix/bug`.
- One branch does **one thing**. If doing multiple things, split into multiple branches.

---

## Commit Message Format

Use the **Conventional Commits** specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

| Type | Description | Example |
|------|-------------|---------|
| `feat` | New feature | `feat(parser): add OSC sequence support` |
| `fix` | Bug fix | `fix(grid): prevent cursor from exceeding bounds` |
| `refactor` | Restructuring (no behavior change) | `refactor(pty): simplify read loop` |
| `docs` | Documentation change | `docs: add development setup guide` |
| `test` | Add/fix tests | `test(parser): add CSI parameter edge cases` |
| `chore` | Build, CI, tooling | `chore: update Tauri to v2.1` |
| `perf` | Performance improvement | `perf(grid): use ring buffer for scrollback` |
| `style` | Formatting, whitespace (no logic change) | `style: apply rustfmt` |
| `ci` | CI/CD config change | `ci: add Windows build to matrix` |

### Scope (Optional)

Scope is the module/area affected: `parser`, `grid`, `pty`, `ui`, `completion`, `config`.

### Rules

- Description uses **lowercase**, does not end with a period.
- Description is in **imperative mood**: "add support" not "added support" or "adds support".
- Body (if any) is separated from description by a blank line.
- Limit the first line to **72 characters**.

### Full Examples

```
feat(completion): add context-aware path completion

Implement path completion that understands the current command context.
When the user types a command like `cd` or `cat`, the completer now
suggests only directories or files respectively.

Closes #42
```

```
fix(parser): handle malformed CSI sequences gracefully

Previously, a CSI sequence with more than 16 parameters would cause
a panic due to buffer overflow. Now we truncate excess parameters
and log a warning.
```

```
refactor(grid): extract cell operations into separate module

The Grid struct was over 500 lines. Move cell-level operations
(set_char, get_char, clear_region) into a new CellOps module
to improve readability.
```

---

## Pull Request Workflow

### Process

1. **Create branch** from the latest `main`:
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feature/my-feature
   ```

2. **Implement** changes, commit following Conventional Commits.

3. **Push branch:**
   ```bash
   git push -u origin feature/my-feature
   ```

4. **Create Pull Request** on GitHub with full description.

5. **Review** - wait for at least 1 approval (or self-review for solo maintainer in early stages).

6. **Address feedback** - push additional commits, or fixup.

7. **Squash merge** into `main`.

### PR Template

Create file `.github/PULL_REQUEST_TEMPLATE.md`:

```markdown
## What

<!-- Brief description of the change. -->

## Why

<!-- Why is this change needed? Link to issue if applicable. -->

## How

<!-- Approach/implementation. Note design decisions. -->

## Testing

<!-- How was this tested? Steps to verify. -->

## Checklist

- [ ] Code follows coding standards
- [ ] Tests added/updated
- [ ] Documentation updated (if applicable)
- [ ] Self-reviewed the diff
- [ ] No unrelated changes included
```

---

## Code Review Guidelines

### Reviewers should check

- **Correctness:** Is the logic correct? Are there missed edge cases?
- **Design:** Is the code easy to understand and maintain? Is it over-engineered?
- **Performance:** Are there potential bottlenecks? Especially on hot paths (parser, rendering).
- **Security:** Are there security concerns? Input validation, buffer bounds?
- **Testing:** Are tests sufficient and meaningful?
- **Style:** Follows coding standards? Is naming clear?

### Reviewers should NOT

- Nitpick about style that is already autoformatted (rustfmt, prettier).
- Request changes unrelated to the PR.
- Block PR for preference differences (not correctness).

### Response Time

- **Target:** Review within **24 hours** (business days).
- If more time is needed, comment: "Will review by [date]."
- PRs should not go more than **3 days** without a response.

### Comment Conventions

- `nit:` - Minor suggestion, not mandatory. E.g.: `nit: consider renaming to X for clarity`
- `suggestion:` - Improvement suggestion, worth considering. E.g.: `suggestion: extract this into a helper`
- `question:` - Needs explanation. E.g.: `question: why not use Y here?`
- `blocker:` - Must fix before merge. E.g.: `blocker: this will panic on empty input`

---

## Merge Strategy

### Squash and Merge

- Use **squash and merge** for all PRs.
- All commits in a PR are combined into **one commit** on `main`.
- The squash merge commit message follows Conventional Commits format.
- The `main` history will be clean and easy to read - each commit is a complete change.

```
main: ──── feat(parser): add OSC support ──── fix(grid): bounds check ──── ...
```

### When NOT to squash

- **No exceptions.** Always squash merge. If a PR is too large to squash, it should be split into smaller PRs.

---

## Release Tagging

### Versioning

Use **Semantic Versioning** (semver):

```
v<major>.<minor>.<patch>
```

| Part | When to increment | Example |
|------|-------------------|---------|
| `major` | Breaking changes | `v1.0.0` -> `v2.0.0` |
| `minor` | New features (backward-compatible) | `v0.1.0` -> `v0.2.0` |
| `patch` | Bug fixes | `v0.1.0` -> `v0.1.1` |

> During the `v0.x.y` stage, minor versions may include breaking changes.

### Creating a tag

```bash
# Update version numbers (Cargo.toml, package.json, tauri.conf.json)
# Commit version bump
git commit -m "chore: bump version to v0.2.0"

# Create annotated tag
git tag -a v0.2.0 -m "Release v0.2.0"

# Push tag (will trigger release CI)
git push origin v0.2.0
```

---

## Protected Branch Rules

Configuration for the `main` branch on GitHub:

### Required

- **Require pull request before merging** - No direct pushes to `main`.
- **Require status checks to pass** - CI (lint, build, test) must pass.
- **Require linear history** - Ensures squash merge, no merge commits.

### Recommended

- **Require conversation resolution** - All review comments must be resolved.
- **Require signed commits** - GPG signed commits (optional in early stages).
- **Do not allow bypassing** - Even admins must follow rules.

> **Note:** In the early stages (solo maintainer), some rules can be relaxed. When more contributors join, enable all rules above.
