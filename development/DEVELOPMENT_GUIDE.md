# Development Guide

Quick reference for maintainer workflows and best practices.

## Quick Commands

```bash
# Build
cargo build              # Debug
cargo build --release    # Release

# Test
cargo test               # All tests
cargo test --lib         # Unit tests only
cargo test autocomplete  # Specific module

# Quality
cargo fmt                # Format code
cargo clippy             # Lints
cargo watch -x test      # Auto-run tests

# Run
cargo run -- data.json   # Run with file
echo '{}' | cargo run    # Run with stdin
```

## Recommended Tools

| Tool | Purpose | Install |
|------|---------|---------|
| cargo-watch | Auto-rebuild on changes | `cargo install cargo-watch` |
| cargo-expand | View macro expansions | `cargo install cargo-expand` |
| bacon | Faster alternative to cargo-watch | `cargo install bacon` |

## Code Organization

**Module principles:**
- One concern per module
- Private by default
- Re-export in `mod.rs`
- Document public APIs

**Naming conventions:**

| Type | Convention | Example |
|------|------------|---------|
| Modules | snake_case | `json_analyzer` |
| Structs/Enums | PascalCase | `AutocompleteState` |
| Functions | snake_case | `analyze_context` |
| Constants/Static | SCREAMING_SNAKE | `MAX_SUGGESTIONS` |

## Common Tasks

### Add jq Function to Autocomplete

`src/autocomplete/jq_functions.rs`:
```rust
Suggestion::new("func_name", SuggestionType::Function)
    .with_description("Description")
```

### Add Keybinding

`src/app/events.rs` → Add to appropriate handler (global/INSERT/NORMAL/OPERATOR)

### Add Suggestion Type

1. Add to `SuggestionType` enum (`src/autocomplete/state.rs`)
2. Add context detection (`src/autocomplete/context.rs`)
3. Add color mapping (`src/app/render.rs`)

### Add Error Type

`src/error.rs`:
```rust
#[derive(Error, Debug)]
pub enum JiqError {
    #[error("Message: {0}")]
    NewError(String),
}
```

## Best Practices

**Code quality:**
- Run `cargo fmt` and `cargo clippy` before committing
- Document public APIs with doc comments
- Use `Result` types, avoid `unwrap()` in production

**Performance:**
- Prefer `&str` over `String` where possible
- Use `LazyLock` for static data
- Use iterators instead of collecting

**Testing:**
- Test public APIs, edge cases, errors
- Name tests descriptively: `test_<what>_<when>_<expected>`
- Use Arrange-Act-Assert pattern

## Debugging

**TUI debugging:** Write to file (`/tmp/jiq-debug.log`) or use `dbg!()` macro

**Profiling:**
```bash
# CPU (Linux)
perf record --call-graph dwarf ./target/release/jiq data.json
perf report

# Benchmarks
cargo bench
```

## Reference

- [Rust Book](https://doc.rust-lang.org/book/)
- [Ratatui Docs](https://ratatui.rs/)
- [DEPLOYMENT.md](DEPLOYMENT.md) - Release process
