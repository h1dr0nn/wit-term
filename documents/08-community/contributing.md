# Contributing Guide

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Welcome to Wit!

Thank you for your interest in contributing to the Wit terminal emulator. Whether you are an experienced developer or just getting started, there are ways for you to participate and make a difference.

### Project Philosophy

Wit is built on these principles:

- **Context-aware by default** - the terminal should understand your working context
- **Community-driven completions** - completion rules are data (TOML), not code, so anyone can contribute
- **Open and inclusive** - MIT license, all contributions are valued
- **Quality over quantity** - fewer features that work well are better than many half-baked ones

---

## Ways to Contribute

### 1. Completion Rules (most encouraged!)

This is the easiest and most impactful way to contribute. You only need to know TOML and understand the CLI tool you want to add completions for. No Rust or TypeScript knowledge required.

> See details: [Completion Contribution Guide](./completion-contribution.md)

### 2. Bug Reports

Found a bug? Please report it! A good bug report helps us fix issues much faster.

### 3. Code Contributions

Contribute code to the frontend (React/TypeScript) or backend (Rust/Tauri).

### 4. Documentation

Improve docs, fix typos, add examples, write tutorials.

### 5. Testing

Try it on different platforms, test edge cases, write test cases.

### 6. Design

Propose UX/UI improvements, design themes, mockup new features.

---

## Getting Started

### Step 1: Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/<your-username>/wit-term.git
cd wit-term
```

### Step 2: Setup Development Environment

Follow the instructions in the [Setup Guide](../02-getting-started/setup.md) to install:

- Rust toolchain (rustup)
- Node.js (v18+)
- Tauri v2 prerequisites (per OS)
- pnpm

```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev
```

### Step 3: Find an Issue to Work On

On GitHub Issues, look for these labels:

- **`good first issue`** - suitable for newcomers, clearly described
- **`help wanted`** - needs help from the community
- **`completion-rule`** - needs completion rules for a specific command
- **`documentation`** - needs documentation improvements

If you want to work on something that doesn't have an issue yet, create a new issue first to discuss.

### Step 4: Create a Feature Branch

```bash
git checkout -b feature/add-curl-completions
# or
git checkout -b fix/parser-edge-case
```

Naming convention:
- `feature/` - new feature
- `fix/` - bug fix
- `docs/` - documentation
- `completions/` - adding completion rules

### Step 5: Implement and Test

```bash
# Run tests
pnpm test

# Run Rust tests
cargo test

# Lint
pnpm lint
```

### Step 6: Submit Pull Request

```bash
git add .
git commit -m "feat: add curl completion rules"
git push origin feature/add-curl-completions
```

Then create a Pull Request on GitHub with:
- Clear description of the changes
- Link to related issue (if any)
- Screenshots (if changing UI)
- Completed checklist

---

## Contribution Areas by Skill Level

### Beginner - No Rust Knowledge Required

| Area | Skills needed | Description |
|------|--------------|-------------|
| Completion rules | TOML, CLI knowledge | Write TOML files describing commands and flags |
| Documentation | Markdown | Fix typos, add examples, improve docs |
| Bug reports | Terminal usage | Find and report bugs in detail |
| Translation | Languages | Translate documentation or UI strings |

**Completion rules are an excellent entry point** - you can contribute immediately without setting up the Rust toolchain. See the [Completion Contribution Guide](./completion-contribution.md).

### Intermediate - React/TypeScript

| Area | Skills needed | Description |
|------|--------------|-------------|
| Frontend components | React, TypeScript | Improve completion popup, settings UI |
| Themes | CSS, TOML | Create new themes for the terminal |
| UI improvements | React, CSS | Responsive design, accessibility |
| Integration tests | TypeScript, Testing | Write E2E tests for frontend |

### Advanced - Rust/Systems

| Area | Skills needed | Description |
|------|--------------|-------------|
| Rust core | Rust | Parser improvements, PTY handling |
| Completion engine | Rust | Ranking algorithm, fuzzy matching |
| Context providers | Rust, TOML | Detect new project types |
| Performance | Rust, Profiling | Optimize startup time, memory usage |
| New providers | Rust | Shell history, man page parsing |

---

## Code Review Process

### Expectations

- **Response time**: Maintainers will review PRs within 3-5 business days
- **Constructive feedback**: Review comments are always constructive
- **Iteration**: 1-2 rounds of revisions may be needed before merge
- **CI must pass**: All automated tests and lints must pass

### Review Checklist (for reviewers)

- Code follows project conventions
- Tests cover the changes
- Documentation is updated if needed
- No security concerns
- Performance is not negatively affected

### After PR is Merged

- Branch will be automatically deleted
- Changes will appear in the next release
- Your name will be added to CONTRIBUTORS.md

---

## Issue Reporting

### Bug Report Template

When reporting a bug, please provide:

```markdown
## Bug Description
Brief description of what went wrong.

## Steps to Reproduce
1. Open Wit terminal
2. Type `...`
3. Press Tab
4. See error: ...

## Expected Behavior
What should have happened.

## Actual Behavior
What actually happened.

## Environment
- OS: [e.g., Ubuntu 24.04, macOS 15, Windows 11]
- Wit version: [e.g., 0.1.0]
- Shell: [e.g., bash 5.2, zsh 5.9, fish 3.7]

## Screenshots / Logs
If available.
```

### Feature Request Template

```markdown
## Feature Description
Describe the feature you want.

## Use Case
Why is this feature useful? When would you use it?

## Proposed Solution
If you have an idea about how to implement it.

## Alternatives Considered
Other solutions you have considered.
```

---

## Communication Channels

| Channel | Status | Purpose |
|---------|--------|---------|
| GitHub Issues | **Active** | Bug reports, feature requests |
| GitHub Discussions | Planned | Q&A, ideas, community chat |
| Discord | Planned | Real-time communication |

Currently, all discussions take place on GitHub Issues. We will open additional channels as the community grows.

---

## Code of Conduct

The Wit project is committed to creating a friendly and inclusive environment for everyone, regardless of experience, gender, identity, background, or any personal characteristic.

### Core Principles

- **Respect** - Treat each other with respect and courtesy
- **Constructive** - Provide constructive feedback, help each other grow
- **Inclusive** - Welcome everyone, especially newcomers
- **Patient** - Not everyone has the same skill level, be patient when explaining
- **Professional** - Focus on technical matters, avoid personal arguments

### Not Accepted

- Offensive language, harassment, or discrimination
- Personal attacks or trolling
- Spam or unrelated advertising

Violations will be handled by maintainers, ranging from warnings to banning from the project.

---

## License

Wit is released under the **MIT License**. When you submit a contribution, you agree that the contribution is also licensed under the MIT License.

This means:
- Your code will be open source
- Anyone can use, modify, and distribute it
- You will still be credited in CONTRIBUTORS.md and commit history

---

## Get Started Now!

The fastest ways to contribute:

1. **Completion rules** - Pick a CLI tool you use daily and [write completion rules](./completion-contribution.md) for it
2. **Bug reports** - Try out Wit and report any issues
3. **Documentation** - See a typo or something confusing? Fix it!

Every contribution, no matter how small, is valued. Thank you!
