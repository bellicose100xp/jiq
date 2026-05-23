# JIQ Project Instructions

## Prerequisites
- **jq v1.8.1+** is required for running tests. Snapshot tests depend on specific jq error message formats.
  - Install: `curl -Lo /tmp/jq https://github.com/jqlang/jq/releases/download/jq-1.8.1/jq-linux-amd64 && chmod +x /tmp/jq && sudo mv /tmp/jq /usr/local/bin/jq`
  - Verify: `jq --version` should show `jq-1.8.1` or higher

## Testing
Run tests in background mode - execution may be lengthy.

## Pre-Commit Requirements
Execute in order:
1. Strip implementation detail comments; retain business logic documentation only
2. Ensure 100% test coverage for all new logic added
3. Run `cargo build --release`
4. Request user validation of TUI functionality with explicit test steps
5. Verify zero linting errors (`cargo clippy --all-targets --all-features`)
6. Verify zero formatting issues (`cargo fmt --all --check`)
7. Verify zero build warnings

All checks must pass before staging files.

## Releasing a Change

Whenever the user asks to release, ship, or publish a change — for example "release the change", "release patch", "release minor", "release major", "ship jiq", "publish a new version", "tag a release" — invoke the **`jiq-release`** skill (project-local, in `.claude/skills/jiq-release/SKILL.md`). It drives the full flow end-to-end: branch, PR, CI, squash-merge, version bump, tag, `cargo publish`, and Homebrew tap update.

The skill accepts an optional bump argument:

| Phrasing | Skill argument |
|---|---|
| "release the change", "release patch" | `patch` (default) |
| "release minor", "release as a minor" | `minor` |
| "release major" (only with explicit user authorization) | `major` |

If the user does not say which bump, the skill infers from the change set (bug fix → patch, new feature → minor, breaking change → major-with-confirmation).

Do not reinvent the release flow inline. Reach for the skill.

## Rust Module Structure
- Use `{module_name}.rs`, never `mod.rs`
- Place tests in `{module_name}_tests.rs` files
- Never co-locate tests with implementation
- Split large test files into `{module_name}_tests/` directory with focused test modules

## Code Quality Principles

### File Organization
- **Max 1000 lines per file** - Applies to all files including tests; refactor into focused modules
- **Single responsibility** - Each file should have one clear purpose
- **Logical grouping** - Related functionality stays together, unrelated code gets its own file

### DRY (Don't Repeat Yourself)
- Extract repeated logic into reusable functions or modules
- Use traits for shared behavior across types
- Create utility modules for common operations
- If you copy-paste code, consider abstracting it

### Functions & Methods
- **Keep functions focused** - Each function does one thing well
- **Easy to read** - Code should be self-explanatory; avoid clever tricks
- **Easy to reason about** - Reader should understand behavior without tracing through many files
- **Clear naming** - Function names describe what they do, not how

### Complexity Management
- Split complex business logic into separate files for clarity
- Prefer composition over deep nesting
- Extract helper functions for complex conditionals
- Use early returns to reduce indentation levels

## Theme & Styling

All colors and styles are centralized in `src/theme.rs`. When adding or modifying UI components:

- **DO** add new colors to the appropriate module in `theme.rs`
- **DO** use `theme::module::CONSTANT` in render files
- **DON'T** hardcode `Color::*` values directly in render files
- **DON'T** import `ratatui::style::Color` in render files (use theme constants instead)

Example:
```rust
// Good
use crate::theme;
let style = Style::default().fg(theme::input::MODE_INSERT);

// Bad
use ratatui::style::Color;
let style = Style::default().fg(Color::Cyan);
```
