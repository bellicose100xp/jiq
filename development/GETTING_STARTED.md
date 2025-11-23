# Getting Started

Welcome! This guide will help you set up your development environment for working with jiq.

## Prerequisites

### Required

- **Rust** (1.80 or later) - Install from [rustup.rs](https://rustup.rs/)
- **jq** - JSON processor ([installation guide](https://jqlang.org/download/))
- **Git** - Version control

### Recommended

- **rust-analyzer** - LSP for IDE support ([setup guide](https://rust-analyzer.github.io/))
- **clippy** - Linting tool (comes with Rust)
- **cargo-watch** - Auto-rebuild on file changes (`cargo install cargo-watch`)

### Platform-Specific Notes

**Linux:**
```bash
# Ubuntu/Debian
sudo apt-get install build-essential jq

# Fedora
sudo dnf install gcc jq
```

**macOS:**
```bash
# Using Homebrew
brew install jq
```

**Windows:**
```powershell
# Using Chocolatey
choco install jq

# Or using Scoop
scoop install jq
```

## Setup Your Development Environment

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/jiq.git
cd jiq

# Add upstream remote
git remote add upstream https://github.com/bellicose100xp/jiq.git
```

### 2. Verify Prerequisites

```bash
# Check Rust version (should be 1.80+)
rustc --version

# Check jq is installed
jq --version

# Check cargo is available
cargo --version
```

### 3. Build the Project

```bash
# Build in debug mode (faster compilation)
cargo build

# Run the binary
cargo run -- tests/fixtures/simple.json

# Build optimized release version
cargo build --release
```

### 4. Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_autocomplete

# Run with verbose output
cargo test -- --test-threads=1 --nocapture
```

### 5. Try It Out

```bash
# Run jiq with sample data
echo '{"name": "Alice", "age": 30}' | cargo run

# Or use a test fixture
cargo run -- tests/fixtures/nested.json
```

**Expected behavior:**
- You should see a TUI with input field (top) and results (bottom)
- Start typing a jq query like `.name`
- Press `Tab` to see autocomplete suggestions
- Press `Enter` to output results
- Press `q` or `Ctrl+C` to quit

## Development Workflow

### Hot Reload During Development

Use `cargo-watch` for automatic rebuilds:

```bash
# Install cargo-watch (one-time)
cargo install cargo-watch

# Auto-rebuild and run on file changes
cargo watch -x 'run -- tests/fixtures/simple.json'

# Auto-run tests on changes
cargo watch -x test
```

### IDE Setup

**VS Code:**
1. Install "rust-analyzer" extension
2. Install "CodeLLDB" for debugging
3. Open the `jiq` folder

**Vim/Neovim:**
```vim
" Add to your config for rust-analyzer LSP
:CocInstall coc-rust-analyzer
```

**IntelliJ IDEA / CLion:**
1. Install "Rust" plugin
2. Open project - it will auto-detect Cargo.toml

### Code Formatting

```bash
# Format all code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check
```

### Linting

```bash
# Run clippy lints
cargo clippy

# Fix auto-fixable issues
cargo clippy --fix

# Strict mode (treat warnings as errors)
cargo clippy -- -D warnings
```

## Working with the Code

### Step 1: Create a Branch

```bash
# Update your fork
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b fix/autocomplete-bug
# or
git checkout -b feat/add-fuzzy-matching
```

**Branch naming convention:**
- `fix/description` - Bug fixes
- `feat/description` - New features
- `docs/description` - Documentation
- `refactor/description` - Code refactoring
- `test/description` - Test additions

### Step 2: Make Your Changes

```bash
# Edit files in your IDE
vim src/autocomplete/state.rs

# Test your changes
cargo test

# Format and lint
cargo fmt
cargo clippy
```

### Step 3: Write Tests

All code changes should include tests. See [TESTING.md](TESTING.md) for details.

```rust
#[test]
fn test_my_new_feature() {
    // Arrange
    let input = "test data";

    // Act
    let result = my_function(input);

    // Assert
    assert_eq!(result, expected_value);
}
```

### Step 4: Commit Your Changes

```bash
# Add your changes
git add .

# Commit with a descriptive message
git commit -m "fix: correct autocomplete after pipe character

- Fixed context detection for queries ending with |
- Added test case for pipe character edge case
- Closes #123"
```

**Commit message format:**
```
<type>: <short summary>

<longer description>

<footer with issue references>
```

**Types:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `test:` - Adding tests
- `refactor:` - Code refactoring
- `perf:` - Performance improvement
- `chore:` - Build/tooling changes

### Step 5: Push Changes

```bash
# Push to your branch
git push origin fix/autocomplete-bug
```

## Common Tasks for Beginners

### Task 1: Add a jq Function to Autocomplete

1. Open `src/autocomplete/jq_functions.rs`
2. Find the appropriate category in `JQ_BUILTINS`
3. Add your function:

```rust
Suggestion::new("your_func", SuggestionType::Function)
    .with_description("Brief description"),
```

4. Add a test in the same file:

```rust
#[test]
fn test_new_function_in_list() {
    let results = filter_builtins("your");
    assert!(results.iter().any(|s| s.text() == "your_func"));
}
```

5. Run tests: `cargo test`

### Task 2: Fix a Typo in Documentation

1. Edit the relevant file (README.md, code comments, etc.)
2. Run spell check: `cargo doc --no-deps`
3. Commit: `git commit -m "docs: fix typo in autocomplete description"`
4. Push and create PR

### Task 3: Add a Test Case

1. Open `tests/integration_tests.rs`
2. Add a new test function:

```rust
#[test]
fn test_my_scenario() {
    // Your test here
}
```

3. Run: `cargo test test_my_scenario`
4. Commit and push

## Troubleshooting

### Build Fails

**Error: `cannot find -ljq` or similar linking errors**
- Solution: Install jq binary (see Prerequisites)

**Error: `error: toolchain '1.80' is not installed`**
- Solution: `rustup update`

**Error: `failed to run custom build command for 'crossterm'`**
- Linux: `sudo apt-get install build-essential`
- macOS: `xcode-select --install`

### Tests Fail

**Error: Tests hang or terminal is corrupted**
- Solution: Don't run interactive TUI tests (we avoid these in CI)

**Error: jq command not found in tests**
- Solution: Ensure jq is in your PATH

### IDE Issues

**rust-analyzer is slow**
- Run: `cargo clean && cargo check`
- Restart LSP server

**Code completion not working**
- Ensure rust-analyzer is installed
- Check: `cargo check` runs successfully

## Learning Resources

### Rust

- [The Rust Book](https://doc.rust-lang.org/book/) - Comprehensive Rust guide
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Learn by doing
- [Rustlings](https://github.com/rust-lang/rustlings) - Interactive exercises

### TUI Development

- [Ratatui Book](https://ratatui.rs/introduction/) - TUI framework guide
- [Ratatui Examples](https://github.com/ratatui/ratatui/tree/main/examples) - Sample apps
- [Crossterm Docs](https://docs.rs/crossterm) - Terminal manipulation

### jq Language

- [jq Manual](https://jqlang.github.io/jq/manual/) - Official jq documentation
- [jq Playground](https://jqplay.org/) - Try jq queries online

## Getting Help

Stuck? Here's how to get help:

1. **Check existing documentation** - Search this folder and README.md
2. **Search issues** - Someone may have had the same problem
3. **Ask in Discussions** - https://github.com/bellicose100xp/jiq/discussions
4. **Join community chat** - (if applicable)
5. **File an issue** - For bugs or unclear documentation

## Next Steps

Now that you're set up:

1. **Explore the code** - Read [ARCHITECTURE.md](ARCHITECTURE.md) to understand the system
2. **Review the workflow** - Check [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) for best practices
3. **Understand testing** - Read [TESTING.md](TESTING.md) for testing guidelines
4. **Join the community** - Introduce yourself in Discussions

**Happy coding!** Enjoy exploring jiq.

---

**Questions?** Open a discussion: https://github.com/bellicose100xp/jiq/discussions
