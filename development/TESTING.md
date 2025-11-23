# Testing

## Test Structure

```
src/              # Unit tests (#[cfg(test)] modules)
tests/            # Integration tests
  ├── integration_tests.rs
  └── fixtures/   # Test JSON files
```

## Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test autocomplete

# With output
cargo test -- --nocapture

# Watch mode
cargo watch -x test

# Coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Writing Tests

**Unit test pattern:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let input = "test";

        // Act
        let result = function(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

**Integration test pattern:**
```rust
use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn test_cli_with_file() {
    cargo_bin_cmd!()
        .arg("tests/fixtures/simple.json")
        .assert()
        .success();
}
```

## Coverage

**Current:** 47 tests across all modules

**Priority:**
- High: Public APIs, complex logic, error handling
- Medium: Helper functions, state transitions
- Low: UI rendering, simple getters

## Best Practices

- Name tests: `test_<what>_<when>_<expected>`
- Test edge cases: empty input, special chars, boundaries
- Test error paths
- Add regression tests when fixing bugs
- Use test fixtures in `tests/fixtures/`

## Reference

See [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
