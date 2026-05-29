# JIQ Project Instructions

## Prerequisites
- **jq v1.8.1+** is required for running tests. Snapshot tests depend on specific jq error message formats.
  - Install: `curl -Lo /tmp/jq https://github.com/jqlang/jq/releases/download/jq-1.8.1/jq-linux-amd64 && chmod +x /tmp/jq && sudo mv /tmp/jq /usr/local/bin/jq`
  - Verify: `jq --version` should show `jq-1.8.1` or higher

## Testing
- Run tests in background mode - execution may be lengthy.
- Always run the full test suite with `cargo test`. Never use `--lib`.

## Pre-Commit Requirements
Execute in order:
1. Strip implementation detail comments; retain business logic documentation only
2. Ensure 100% test coverage for all new logic added
3. Run `cargo build --release` (zero warnings)
4. Request user validation of TUI functionality with explicit test steps — STOP and wait for user to drive the steps before continuing
5. Verify zero linting errors (`cargo clippy --all-targets --all-features -- -D warnings`)
6. Verify zero formatting issues (`cargo fmt --all --check`)
7. Run `cargo build` (debug; zero warnings)
8. Run full test suite (`cargo test` — never `--lib`)

All checks must pass before staging files. After all green:
- **Sync with remote first** to minimize PR merge conflicts:
  - `git fetch origin`
  - If you've been working on `main` and have local commits: `git rebase origin/main` (resolve conflicts now, locally, instead of in the PR)
  - If no local commits yet: `git pull --ff-only origin main`
- Stage specific files by name (never `git add -A` / `git add .`)
- Commit with a single-line lowercase Conventional Commit message (no body, no issue refs)
- **Do not push.** Pushing/tagging/publishing is deferred to the `jiq-release` skill, invoked only when the user explicitly asks to release/ship/publish.

## Documentation Site

`docs/` is the canonical user-facing reference (Jekyll + just-the-docs, GitHub Pages, <https://bellicose100xp.github.io/jiq/>). README is one-liner intent only.

User-visible feature/shortcut/config change → update docs in the same change set:
- Feature → `docs/features/<page>.md` + `docs/quick-reference.md`
- Shortcut → both above
- Config → `docs/configuration.md`

Visuals: inline SVG / HTML via helpers in `docs/_sass/custom/custom.scss` (`.tui-mockup`, `.io-pair`, `.drill-chain`, `.feature-grid`, `.shortcuts`, `<kbd>`).

## Releasing a Change

When the user asks to release / ship / publish / tag, invoke the **`jiq-release`** skill (`.claude/skills/jiq-release/SKILL.md`). Pass `patch` / `minor` / `major` if specified; otherwise the skill infers. Never reinvent the flow inline.

## Performance Testing

Pieces:
- Generator: `src/bin/gen_perf_fixture.rs` — produces fixtures in `tests/perf_fixtures/` (gitignored). Three shapes (`wide`, `deep`, `keys`) × four sizes (`1k`/`10k`/`100k`/`1m`).
- Perf module: `src/perf.rs` — RAII stopwatches, percentile summary at exit. Gated by `--debug`.
- Bench-script flag: `--bench-script <path>` — replays `text`/`key`/`wait` directives through the real input dispatcher. Scripts in `tests/perf_scripts/` (`typical`, `heavy_search`, `scroll_navigation`, `deep_drilling`, `autocomplete_burst`).

Read [`tests/perf_fixtures/README.md`](tests/perf_fixtures/README.md) in full when: generating a fixture, running a benchmark (manual TUI or scripted), wiring a new timer, choosing a shape/size for a new scenario, or troubleshooting `No such device or address` (TTY/PTY workaround). It owns the commands; do not duplicate them here.

Rules:
- Always benchmark on a release build (debug is 5-10× slower with different bottlenecks).
- No stopwatches inside `terminal.draw` or any per-frame / per-line path.
- In-memory tally during session; only the summary writes on exit (no per-stopwatch logging).

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
