# Development Guide

This guide covers day-to-day development workflows, best practices, and common tasks for jiq contributors.

## Table of Contents

1. [Development Workflow](#development-workflow)
2. [Code Organization](#code-organization)
3. [Common Tasks](#common-tasks)
4. [Best Practices](#best-practices)
5. [Debugging](#debugging)
6. [Performance](#performance)
7. [Advanced Workflows](#advanced-workflows)

## Development Workflow

### Daily Development Cycle

```bash
# 1. Update your local repository
git checkout main
git pull upstream main

# 2. Create a feature branch
git checkout -b feat/my-feature

# 3. Make changes and test frequently
cargo watch -x 'test'  # Auto-run tests on save

# 4. Before committing
cargo fmt              # Format code
cargo clippy           # Run lints
cargo test             # Run all tests

# 5. Commit and push
git add .
git commit -m "feat: add my feature"
git push origin feat/my-feature

# 6. Create PR on GitHub
```

### Recommended Tools

**cargo-watch** - Auto-rebuild on file changes
```bash
# Install
cargo install cargo-watch

# Watch and run tests
cargo watch -x test

# Watch and run with specific fixture
cargo watch -x 'run -- tests/fixtures/nested.json'

# Watch tests with clear screen
cargo watch -c -x test
```

**cargo-expand** - See macro expansions
```bash
cargo install cargo-expand
cargo expand app::state  # See expanded code
```

**cargo-edit** - Manage dependencies
```bash
cargo install cargo-edit
cargo add ratatui@0.29   # Add dependency
cargo rm old-crate       # Remove dependency
```

**bacon** - Alternative to cargo-watch (faster)
```bash
cargo install bacon
bacon           # Run default job (check)
bacon test      # Run tests
bacon clippy    # Run clippy
```

## Code Organization

### Module Organization Principles

1. **One concern per module** - Each module has a single responsibility
2. **Private by default** - Only expose what's necessary
3. **Clear public API** - Document all public items
4. **Re-export at top level** - Use `pub use` in `mod.rs`

**Example:**
```rust
// src/autocomplete/mod.rs
mod state;
mod context;
mod jq_functions;
mod json_analyzer;

// Re-export only public API
pub use state::{AutocompleteState, Suggestion, SuggestionType};
pub use context::{get_suggestions, SuggestionContext};
```

### File Structure Conventions

**Small modules** - Single file
```
src/error.rs           # Single file for error types
```

**Larger modules** - Directory with mod.rs
```
src/autocomplete/
├── mod.rs             # Public API, re-exports
├── state.rs           # Main types
├── context.rs         # Helper logic
└── ...
```

### Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Modules | snake_case | `json_analyzer` |
| Structs | PascalCase | `AutocompleteState` |
| Enums | PascalCase | `EditorMode` |
| Functions | snake_case | `analyze_context` |
| Constants | SCREAMING_SNAKE | `MAX_SUGGESTIONS` |
| Static | SCREAMING_SNAKE | `JQ_BUILTINS` |

## Common Tasks

### Adding a New jq Function to Autocomplete

**File:** `src/autocomplete/jq_functions.rs`

```rust
static JQ_BUILTINS: LazyLock<Vec<Suggestion>> = LazyLock::new(|| {
    let mut builtins = Vec::new();

    // Add to appropriate category
    builtins.extend(vec![
        // Array functions
        Suggestion::new("your_func", SuggestionType::Function)
            .with_description("Brief description of what it does"),
    ]);

    builtins.sort_by(|a, b| a.text().cmp(b.text()));
    builtins
});
```

**Add test:**
```rust
#[test]
fn test_your_func_in_autocomplete() {
    let results = filter_builtins("your");
    assert!(results.iter().any(|s| s.text() == "your_func"));
}
```

**Run test:**
```bash
cargo test test_your_func_in_autocomplete
```

### Adding a New Keybinding

**File:** `src/app/events.rs`

**Step 1:** Determine where the key should be handled

```rust
// Global keys (work anywhere)
fn handle_events(&mut self, event: Event) -> Result<()> {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::F(1) => {
                // Your handler here
            }
            // ...
        }
    }
}

// Input-specific keys
fn handle_input_events(&mut self, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Ctrl+X handler
        }
        // ...
    }
}
```

**Step 2:** Update documentation in README.md

**Step 3:** Add test if possible
```rust
#[test]
fn test_new_keybinding() {
    let mut app = App::new("{}".to_string());
    // Simulate key press and verify behavior
}
```

### Adding a New Suggestion Type

**Step 1:** Add to `SuggestionType` enum

**File:** `src/autocomplete/state.rs`
```rust
pub enum SuggestionType {
    Function,
    Field,
    Operator,
    Pattern,
    YourNewType,  // Add here
}
```

**Step 2:** Add context detection

**File:** `src/autocomplete/context.rs`
```rust
pub enum SuggestionContext {
    FieldContext,
    FunctionContext,
    YourNewContext,  // Add here
}

pub fn analyze_context(query: &str, cursor: usize) -> (SuggestionContext, String) {
    // Add detection logic
}
```

**Step 3:** Add color in rendering

**File:** `src/app/render.rs`
```rust
let type_color = match suggestion.suggestion_type() {
    SuggestionType::Function => Color::Yellow,
    SuggestionType::Field => Color::Cyan,
    SuggestionType::YourNewType => Color::Magenta,  // Choose color
    // ...
};
```

### Improving Performance

**Step 1:** Profile first
```bash
# Build with profiling symbols
cargo build --release --profile=release-with-debug

# Run with a profiler (Linux)
perf record --call-graph dwarf ./target/release/jiq data.json
perf report
```

**Step 2:** Common optimizations

**Use iterators instead of collecting:**
```rust
// Slower - allocates Vec
let results: Vec<_> = items.iter().filter(|x| x.is_valid()).collect();

// Faster - lazy iterator
let results = items.iter().filter(|x| x.is_valid());
```

**Cache expensive computations:**
```rust
// Use LazyLock for static data
use std::sync::LazyLock;

static EXPENSIVE_DATA: LazyLock<Vec<Item>> = LazyLock::new(|| {
    // Computed once at first access
    compute_expensive_data()
});
```

**Avoid unnecessary clones:**
```rust
// Slower
fn process(data: String) { ... }

// Faster - borrow instead
fn process(data: &str) { ... }
```

### Adding a New Error Type

**File:** `src/error.rs`
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JiqError {
    // Existing errors...

    #[error("Your error message: {0}")]
    YourNewError(String),

    #[error("Another error with context")]
    AnotherError {
        source: #[from] SomeOtherError,
    },
}
```

**Usage:**
```rust
if something_wrong {
    return Err(JiqError::YourNewError("details".to_string()));
}
```

## Best Practices

### Code Style

**Follow Rust conventions:**
- Use `cargo fmt` to format code
- Run `cargo clippy` and fix all warnings
- Document all public APIs

**Example of well-documented code:**
```rust
/// Analyzes the query context to determine suggestion type.
///
/// # Arguments
/// * `query` - The current jq query string
/// * `cursor` - The cursor position in the query
///
/// # Returns
/// A tuple of (context type, partial word being typed)
///
/// # Examples
/// ```
/// let (ctx, partial) = analyze_context(".users.na", 9);
/// assert_eq!(ctx, SuggestionContext::FieldContext);
/// assert_eq!(partial, "na");
/// ```
pub fn analyze_context(query: &str, cursor: usize) -> (SuggestionContext, String) {
    // Implementation
}
```

### Error Handling

**Use Result types:**
```rust
// Good
fn parse_json(input: &str) -> Result<Value, JiqError> {
    serde_json::from_str(input)
        .map_err(|e| JiqError::InvalidJson(e.to_string()))
}

// Bad - panics on error
fn parse_json(input: &str) -> Value {
    serde_json::from_str(input).unwrap()  // Don't do this!
}
```

**Propagate errors with `?`:**
```rust
fn read_and_parse(path: &Path) -> Result<Value, JiqError> {
    let content = fs::read_to_string(path)?;  // Propagates IO error
    let value = serde_json::from_str(&content)?;  // Propagates parse error
    Ok(value)
}
```

**Handle errors gracefully in UI:**
```rust
match executor.execute(&query) {
    Ok(result) => self.result_text = result,
    Err(e) => self.result_text = format!("Error: {}", e),  // Show error, don't crash
}
```

### Testing Best Practices

**Write tests for:**
- All public APIs
- Edge cases (empty input, special characters)
- Error conditions
- Regression bugs (add test when fixing)

**Test structure (Arrange-Act-Assert):**
```rust
#[test]
fn test_autocomplete_with_prefix() {
    // Arrange
    let prefix = "ma";

    // Act
    let results = filter_builtins(prefix);

    // Assert
    assert!(!results.is_empty());
    assert!(results.iter().all(|s| s.text().starts_with(prefix)));
}
```

**Use descriptive test names:**
```rust
// Good - describes what it tests
#[test]
fn test_autocomplete_shows_fields_after_dot() { }

// Bad - vague
#[test]
fn test_autocomplete() { }
```

### Performance Guidelines

**Avoid in hot paths:**
- String allocations (use `&str` when possible)
- Vec cloning (use iterators)
- Unnecessary HashMap lookups

**Prefer:**
- Stack allocation over heap
- Borrowing over cloning
- Iterators over collecting
- Static data over runtime construction

**Example:**
```rust
// Slower - allocates on every call
fn get_functions() -> Vec<Suggestion> {
    vec![
        Suggestion::new("map", SuggestionType::Function),
        Suggestion::new("select", SuggestionType::Function),
        // ...
    ]
}

// Faster - static data, built once
static FUNCTIONS: LazyLock<Vec<Suggestion>> = LazyLock::new(|| {
    vec![
        Suggestion::new("map", SuggestionType::Function),
        Suggestion::new("select", SuggestionType::Function),
    ]
});
```

### Code Review Checklist

Before submitting a PR:

- [ ] Code compiles without warnings
- [ ] All tests pass (`cargo test`)
- [ ] New code has tests
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy is happy (`cargo clippy`)
- [ ] Documentation updated (if needed)
- [ ] CHANGELOG.md updated (if user-facing)
- [ ] No unwrap() or expect() in production code
- [ ] Error handling is appropriate
- [ ] Performance considered for hot paths

## Debugging

### Debug Logging

**Option 1:** Use `dbg!()` macro (quick and dirty)
```rust
let result = some_function();
dbg!(&result);  // Prints to stderr
```

**Option 2:** Add env_logger for structured logging
```bash
# Add to Cargo.toml
# [dependencies]
# env_logger = "0.11"
# log = "0.4"

# Use in code
log::debug!("Query executed: {}", query);
log::warn!("Slow query: {}ms", elapsed);

# Run with logging
RUST_LOG=debug cargo run
```

### Debugging TUI Applications

**Problem:** Can't see print statements when TUI is running

**Solution 1:** Write to a file
```rust
use std::fs::OpenOptions;
use std::io::Write;

fn debug_log(msg: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/jiq-debug.log")
        .unwrap();
    writeln!(file, "{}", msg).unwrap();
}

// Usage
debug_log(&format!("Query: {}", query));
```

**Solution 2:** Use a separate terminal
```bash
# Terminal 1: tail the debug log
tail -f /tmp/jiq-debug.log

# Terminal 2: run jiq
cargo run -- data.json
```

### Using a Debugger

**VS Code with CodeLLDB:**

1. Install "CodeLLDB" extension
2. Add to `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug jiq",
      "cargo": {
        "args": ["build", "--bin=jiq"]
      },
      "args": ["tests/fixtures/simple.json"],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

3. Set breakpoints and press F5

**GDB (command line):**
```bash
# Build with debug symbols
cargo build

# Run with gdb
rust-gdb ./target/debug/jiq

# Set breakpoint
(gdb) break main.rs:42
(gdb) run tests/fixtures/simple.json
```

### Common Issues

**Issue:** "jq not found" in tests
```bash
# Solution: ensure jq is in PATH
which jq
export PATH=/usr/local/bin:$PATH
```

**Issue:** Tests hang
```bash
# Solution: likely a TUI test, skip them
cargo test -- --skip interactive
```

**Issue:** Clippy warnings about unused code
```rust
// Solution: use #[cfg(test)] for test-only code
#[cfg(test)]
fn test_helper() {
    // Only compiled during tests
}
```

## Performance

### Profiling

**CPU profiling (Linux):**
```bash
# Install perf
sudo apt-get install linux-tools-generic

# Build release with debug symbols
cargo build --release

# Profile
perf record --call-graph dwarf ./target/release/jiq large.json
perf report
```

**Memory profiling:**
```bash
# Install valgrind
sudo apt-get install valgrind

# Run with massif
valgrind --tool=massif ./target/release/jiq data.json

# Analyze
ms_print massif.out.*
```

**Benchmarking:**
```rust
// Add to Cargo.toml: [dev-dependencies]
// criterion = "0.5"

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_autocomplete(c: &mut Criterion) {
    c.bench_function("filter_builtins", |b| {
        b.iter(|| filter_builtins(black_box("ma")))
    });
}

criterion_group!(benches, benchmark_autocomplete);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo bench
```

### Optimization Checklist

When optimizing:

1. **Measure first** - Profile before optimizing
2. **Focus on hot paths** - Optimize what matters
3. **Test after** - Ensure correctness
4. **Document why** - Explain non-obvious optimizations

## Advanced Workflows

### Working with Multiple Branches

```bash
# Create a worktree for parallel work
git worktree add ../jiq-feature-x feature-x

# Work in separate directories
cd ../jiq-feature-x
cargo test

# Remove worktree when done
git worktree remove ../jiq-feature-x
```

### Bisecting to Find Regressions

```bash
# Find commit that introduced a bug
git bisect start
git bisect bad                  # Current version is bad
git bisect good v2.0.0          # v2.0.0 was good

# Git will checkout commits, test each:
cargo test
git bisect good   # or 'git bisect bad'

# Git finds the breaking commit
git bisect reset
```

### Creating a Minimal Reproduction

When debugging, create a minimal example:

```rust
// tests/minimal_repro.rs
#[test]
fn minimal_reproduction() {
    // Simplest code that shows the bug
    let input = r#"{"field": "value"}"#;
    let result = reproduce_bug(input);
    assert_eq!(result, expected);
}
```

### Dependency Updates

```bash
# Check for outdated dependencies
cargo outdated

# Update dependencies (respecting Cargo.toml constraints)
cargo update

# Update and test
cargo update
cargo test
cargo build --release
```

### Release Checklist

See [DEPLOYMENT.md](DEPLOYMENT.md) for full release process.

Quick checklist:
1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Run full test suite
4. Create git tag
5. Push tag (triggers CI)

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Ratatui Documentation](https://ratatui.rs/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)

---

**Questions?** Ask in discussions: https://github.com/bellicose100xp/jiq/discussions
