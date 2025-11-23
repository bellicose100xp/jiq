# Testing Guide

Comprehensive guide to testing in jiq, including test structure, best practices, and how to write effective tests.

## Table of Contents

1. [Testing Philosophy](#testing-philosophy)
2. [Test Structure](#test-structure)
3. [Running Tests](#running-tests)
4. [Writing Unit Tests](#writing-unit-tests)
5. [Writing Integration Tests](#writing-integration-tests)
6. [Test Coverage](#test-coverage)
7. [Best Practices](#best-practices)
8. [Debugging Tests](#debugging-tests)

## Testing Philosophy

### Goals

- **Correctness** - Catch bugs before they reach users
- **Regression Prevention** - Ensure bugs don't reappear
- **Documentation** - Tests show how code should be used
- **Refactoring Confidence** - Change code without fear

### Test Pyramid

```
         ╱╲
        ╱  ╲      Few
       ╱ E2E╲     (Integration Tests)
      ╱──────╲
     ╱ Integ. ╲   Some
    ╱──────────╲  (Integration Tests)
   ╱   Unit     ╲ Many
  ╱──────────────╲(Unit Tests)
```

**jiq's approach:**
- **Many unit tests** - Test individual functions and modules
- **Some integration tests** - Test CLI interface
- **No E2E tests** - Interactive TUI is difficult to test

## Test Structure

### Directory Layout

```
jiq/
├── src/
│   ├── app/
│   │   ├── state.rs
│   │   └── state.rs         # Contains #[cfg(test)] mod tests
│   ├── autocomplete/
│   │   ├── context.rs
│   │   └── context.rs       # Contains #[cfg(test)] mod tests
│   └── ...
├── tests/
│   ├── integration_tests.rs # CLI integration tests
│   └── fixtures/            # Test data
│       ├── simple.json
│       ├── array.json
│       ├── nested.json
│       └── invalid.json
└── Cargo.toml
```

### Unit Tests vs Integration Tests

**Unit Tests** (in `src/` files):
- Test individual functions
- Access private functions
- Fast execution
- Use `#[cfg(test)]` module

**Integration Tests** (in `tests/` directory):
- Test public API
- Test CLI behavior
- Slower execution
- Use `assert_cmd` crate

## Running Tests

### Basic Test Commands

```bash
# Run all tests
cargo test

# Run with output shown
cargo test -- --nocapture

# Run specific test
cargo test test_autocomplete_context

# Run tests in a specific module
cargo test autocomplete::

# Run tests matching a pattern
cargo test autocomplete

# Run with verbose output
cargo test -- --test-threads=1 --nocapture

# Run only unit tests (skip integration tests)
cargo test --lib

# Run only integration tests
cargo test --test integration_tests
```

### Watch Mode

```bash
# Install cargo-watch
cargo install cargo-watch

# Auto-run tests on file changes
cargo watch -x test

# Auto-run specific test
cargo watch -x 'test test_autocomplete'

# Clear screen before each run
cargo watch -c -x test
```

### Running Tests in CI

```bash
# Simulate CI environment
cargo test --all-features --no-fail-fast

# With coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Writing Unit Tests

### Basic Structure

```rust
// In src/autocomplete/context.rs

/// Analyzes query context
pub fn analyze_context(query: &str, cursor: usize) -> (SuggestionContext, String) {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_context_detection() {
        // Arrange
        let query = ".users.name";
        let cursor = query.len();

        // Act
        let (context, partial) = analyze_context(query, cursor);

        // Assert
        assert_eq!(context, SuggestionContext::FieldContext);
        assert_eq!(partial, "name");
    }
}
```

### Test Organization

**Pattern: Arrange-Act-Assert**

```rust
#[test]
fn test_descriptive_name() {
    // Arrange - Set up test data
    let input = "test data";

    // Act - Execute the code under test
    let result = function_to_test(input);

    // Assert - Verify the outcome
    assert_eq!(result, expected_value);
}
```

### Common Assertions

```rust
// Equality
assert_eq!(actual, expected);
assert_ne!(actual, unexpected);

// Boolean conditions
assert!(condition);
assert!(!condition);

// Panic testing
#[test]
#[should_panic(expected = "error message")]
fn test_panic() {
    panic!("error message");
}

// Result testing
#[test]
fn test_result() -> Result<(), JiqError> {
    let result = fallible_function()?;
    assert_eq!(result, expected);
    Ok(())
}
```

### Testing Edge Cases

```rust
#[test]
fn test_empty_input() {
    let result = analyze_context("", 0);
    // Verify behavior with empty input
}

#[test]
fn test_cursor_at_start() {
    let result = analyze_context(".field", 0);
    // Verify behavior with cursor at position 0
}

#[test]
fn test_special_characters() {
    let result = analyze_context(".field-name", 11);
    // Verify handling of special characters
}

#[test]
fn test_very_long_input() {
    let long_query = "a".repeat(10000);
    let result = analyze_context(&long_query, 100);
    // Verify performance with large input
}
```

### Testing Error Conditions

```rust
#[test]
fn test_invalid_json_returns_error() {
    let result = InputReader::read_json(Some(Path::new("invalid.json")));
    assert!(result.is_err());

    if let Err(JiqError::InvalidJson(msg)) = result {
        assert!(msg.contains("expected"));
    } else {
        panic!("Wrong error type");
    }
}
```

### Testing Async Code (if needed in future)

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert_eq!(result, expected);
}
```

## Writing Integration Tests

### CLI Integration Tests

Integration tests use `assert_cmd` to test the CLI interface.

**File:** `tests/integration_tests.rs`

```rust
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn test_cli_with_valid_json_file() {
    cargo_bin_cmd!()
        .arg("tests/fixtures/simple.json")
        .assert()
        .success();
}

#[test]
fn test_cli_with_nonexistent_file() {
    cargo_bin_cmd!()
        .arg("nonexistent.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));
}

#[test]
fn test_cli_help_flag() {
    cargo_bin_cmd!()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Interactive JSON query tool"));
}

#[test]
fn test_cli_version_flag() {
    cargo_bin_cmd!()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
```

### Testing with Fixtures

**Create test fixtures:**

```bash
# tests/fixtures/simple.json
{
  "name": "Alice",
  "age": 30,
  "city": "Seattle"
}

# tests/fixtures/array.json
[
  {"id": 1, "name": "Item 1"},
  {"id": 2, "name": "Item 2"}
]

# tests/fixtures/invalid.json
{invalid json}
```

**Use in tests:**

```rust
fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn test_with_fixture() {
    let path = fixture_path("simple.json");
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("Alice"));
}
```

### Stdin Testing (Piped Input)

```rust
#[test]
fn test_stdin_input() {
    let mut cmd = cargo_bin_cmd!();
    cmd.write_stdin(r#"{"name": "test"}"#)
        .assert()
        .success();
}
```

### TUI Testing Challenges

**Problem:** Interactive TUI applications are hard to test.

**Why:**
- Requires terminal emulation
- Keyboard simulation is complex
- Tests can corrupt terminal state

**Solutions:**

1. **Extract business logic** - Test logic separately from UI

```rust
// Good - testable
fn calculate_scroll_offset(current: usize, max: usize) -> usize {
    // Logic here
}

#[test]
fn test_scroll_calculation() {
    assert_eq!(calculate_scroll_offset(10, 100), expected);
}

// Bad - not testable (tied to TUI)
fn handle_scroll_key(&mut self, key: KeyCode) {
    // Direct TUI manipulation - hard to test
}
```

2. **Test state transitions** - Test app state without rendering

```rust
#[test]
fn test_mode_transition() {
    let mut app = App::new("{}".to_string());

    // Start in INSERT mode
    assert_eq!(app.editor_mode(), EditorMode::Insert);

    // Simulate ESC key (would be in event handler)
    // app.handle_escape();
    // assert_eq!(app.editor_mode(), EditorMode::Normal);
}
```

3. **Use conditional compilation** - Skip TUI tests in CI

```rust
#[test]
#[cfg_attr(not(feature = "interactive-tests"), ignore)]
fn test_interactive_feature() {
    // Only runs with: cargo test --features interactive-tests
}
```

## Test Coverage

### Measuring Coverage

**Install cargo-tarpaulin (Linux):**
```bash
cargo install cargo-tarpaulin
```

**Run coverage:**
```bash
# Generate HTML report
cargo tarpaulin --out Html

# Open in browser
xdg-open tarpaulin-report.html

# Generate multiple formats
cargo tarpaulin --out Html --out Lcov
```

**Coverage goals:**
- **Core logic:** 80%+ coverage
- **UI code:** Lower coverage acceptable (hard to test)
- **Error handling:** All error paths tested

### What to Test

**High Priority:**
- Public API functions
- Complex logic (autocomplete context detection)
- Error handling
- Edge cases

**Medium Priority:**
- Helper functions
- Data transformations
- State transitions

**Low Priority (or skip):**
- Direct UI rendering code
- Simple getters/setters
- Generated code

### Coverage Gaps

**Identify untested code:**
```bash
cargo tarpaulin --out Html --exclude-files 'tests/*'
# Check report for red/yellow lines
```

**Add tests for critical paths:**
```rust
// If coverage shows this line is never tested:
if critical_condition {
    // Add a test that exercises this branch
}
```

## Best Practices

### Test Naming

**Use descriptive names:**

```rust
// Good
#[test]
fn test_autocomplete_suggests_fields_after_dot() { }

#[test]
fn test_autocomplete_suggests_functions_without_dot() { }

// Bad
#[test]
fn test_autocomplete() { }

#[test]
fn test1() { }
```

**Name pattern:** `test_<what>_<when>_<expected>`

```rust
#[test]
fn test_executor_with_invalid_query_returns_error() { }

#[test]
fn test_json_analyzer_with_nested_objects_extracts_all_fields() { }
```

### Test Independence

**Each test should be independent:**

```rust
// Bad - tests share state
static mut COUNTER: usize = 0;

#[test]
fn test_a() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 1);  // Fails if test_b runs first!
}

#[test]
fn test_b() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 1);
}
```

```rust
// Good - tests are independent
#[test]
fn test_a() {
    let counter = 0;
    let result = counter + 1;
    assert_eq!(result, 1);  // Always passes
}

#[test]
fn test_b() {
    let counter = 0;
    let result = counter + 1;
    assert_eq!(result, 1);  // Always passes
}
```

### Test Data Management

**Use constants for common data:**

```rust
#[cfg(test)]
mod tests {
    const SIMPLE_JSON: &str = r#"{"name": "test"}"#;
    const ARRAY_JSON: &str = r#"[1, 2, 3]"#;

    #[test]
    fn test_with_simple_json() {
        let result = parse(SIMPLE_JSON);
        assert!(result.is_ok());
    }

    #[test]
    fn test_with_array_json() {
        let result = parse(ARRAY_JSON);
        assert!(result.is_ok());
    }
}
```

**Use helper functions:**

```rust
#[cfg(test)]
fn create_test_app() -> App {
    App::new(r#"{"test": "data"}"#.to_string())
}

#[test]
fn test_feature_a() {
    let app = create_test_app();
    // Test feature A
}

#[test]
fn test_feature_b() {
    let app = create_test_app();
    // Test feature B
}
```

### Assertions

**Use specific assertions:**

```rust
// Good - clear what's being tested
assert_eq!(result.len(), 3);
assert!(result.contains(&item));
assert!(result.starts_with("prefix"));

// Bad - unclear
assert!(result.len() == 3);  // Use assert_eq! instead
assert!(result.contains(&item) == true);  // Redundant
```

**Custom error messages:**

```rust
assert_eq!(
    result,
    expected,
    "Expected result to be {} for input '{}', but got {}",
    expected,
    input,
    result
);
```

### Regression Tests

**When fixing a bug:**
1. Write a test that reproduces the bug
2. Verify the test fails
3. Fix the bug
4. Verify the test passes
5. Commit both test and fix together

```rust
// Issue #123: Autocomplete doesn't work after pipe
#[test]
fn test_autocomplete_after_pipe() {
    let query = ".users | ";
    let (ctx, _) = analyze_context(query, query.len());

    // This test failed before the fix
    assert_eq!(ctx, SuggestionContext::FunctionContext);
}
```

## Debugging Tests

### Test Output

```bash
# Show println! output even for passing tests
cargo test -- --nocapture

# Show output for specific test
cargo test test_name -- --nocapture

# Run tests sequentially (easier to debug)
cargo test -- --test-threads=1
```

### Debugging Individual Tests

**Add debug output:**

```rust
#[test]
fn test_with_debug() {
    let input = "test";
    println!("Input: {:?}", input);

    let result = function(input);
    println!("Result: {:?}", result);

    assert_eq!(result, expected);
}
```

**Use dbg! macro:**

```rust
#[test]
fn test_with_dbg() {
    let input = "test";
    let result = function(dbg!(input));  // Prints to stderr
    dbg!(&result);
    assert_eq!(result, expected);
}
```

### Conditional Test Execution

```rust
// Only run on Linux
#[test]
#[cfg(target_os = "linux")]
fn test_linux_specific() { }

// Only run in release mode
#[test]
#[cfg(not(debug_assertions))]
fn test_release_only() { }

// Skip test (with reason)
#[test]
#[ignore = "slow test, run manually"]
fn test_slow_operation() { }
```

Run ignored tests:
```bash
cargo test -- --ignored
cargo test -- --include-ignored  # Run all tests
```

## Continuous Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install jq
        run: sudo apt-get install -y jq
      - name: Run tests
        run: cargo test --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
```

### Test Matrix (Multiple Rust Versions)

```yaml
strategy:
  matrix:
    rust: [stable, beta, nightly]
    os: [ubuntu-latest, macos-latest, windows-latest]
runs-on: ${{ matrix.os }}
```

## Resources

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [assert_cmd Documentation](https://docs.rs/assert_cmd)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)

---

**Questions about testing?** Ask in discussions: https://github.com/bellicose100xp/jiq/discussions
