# Contributing to jiq

Thank you for considering contributing to jiq! This document provides guidelines and best practices for contributing.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [How Can I Contribute?](#how-can-i-contribute)
3. [Development Process](#development-process)
4. [Code Style](#code-style)
5. [Commit Guidelines](#commit-guidelines)
6. [Pull Request Process](#pull-request-process)
7. [Review Process](#review-process)
8. [Community](#community)

## Code of Conduct

### Our Standards

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Accept constructive criticism gracefully
- Focus on what's best for the community
- Show empathy towards other community members

### Unacceptable Behavior

- Harassment, discrimination, or offensive comments
- Trolling or insulting comments
- Publishing others' private information
- Other conduct that's inappropriate in a professional setting

## How Can I Contribute?

### Reporting Bugs

**Before submitting a bug report:**
1. Check existing issues to avoid duplicates
2. Ensure you're using the latest version
3. Verify the bug is reproducible

**Submitting a bug report:**
1. Use the GitHub issue tracker
2. Include a clear title and description
3. Provide steps to reproduce
4. Include sample JSON and query (if applicable)
5. Mention your environment (OS, Rust version, jq version)

**Good bug report example:**
```markdown
**Title:** Autocomplete doesn't appear after pipe character

**Description:**
When typing a query with a pipe character, autocomplete suggestions don't appear.

**Steps to reproduce:**
1. Run: `echo '{"users": []}' | cargo run`
2. Type: `.users | `
3. Notice autocomplete doesn't show

**Expected:** Autocomplete should suggest jq functions

**Environment:**
- OS: Ubuntu 22.04
- Rust: 1.80
- jq: 1.7
- jiq: 2.3.0

**Sample data:**
```json
{"users": [{"name": "Alice"}]}
```
```

### Suggesting Features

**Before suggesting:**
1. Check if it's already proposed
2. Consider if it fits jiq's scope
3. Think about implementation complexity

**Feature request template:**
```markdown
**Title:** Add fuzzy matching to autocomplete

**Problem:**
Currently autocomplete only matches prefixes. Sometimes I remember part of a function name but not the start.

**Proposed solution:**
Add fuzzy matching so typing "mapv" suggests "map_values"

**Alternatives:**
- Substring matching
- Regex search

**Additional context:**
Similar to how VSCode autocomplete works
```

### Contributing Code

We welcome contributions of all sizes:

**Good First Issues:**
- Documentation improvements
- Adding jq functions to autocomplete
- Test coverage improvements
- Bug fixes with clear reproduction

**Larger Features:**
- Performance optimizations
- New autocomplete features
- VIM keybinding enhancements
- UI improvements

**Check the issues tagged with:**
- `good first issue` - Great for newcomers
- `help wanted` - Community contributions welcome
- `enhancement` - New features
- `bug` - Bug fixes needed

### Improving Documentation

Documentation improvements are always welcome:

- Fix typos or unclear explanations
- Add examples
- Improve code comments
- Update outdated information
- Add diagrams or visualizations

### Reviewing Pull Requests

Help review open pull requests:

- Test the changes locally
- Provide constructive feedback
- Check code style and tests
- Verify documentation is updated

## Development Process

### 1. Fork and Clone

```bash
# Fork on GitHub, then:
git clone https://github.com/YOUR_USERNAME/jiq.git
cd jiq
git remote add upstream https://github.com/bellicose100xp/jiq.git
```

### 2. Create a Branch

```bash
# Update main
git checkout main
git pull upstream main

# Create feature branch
git checkout -b fix/issue-123-autocomplete-bug
```

**Branch naming:**
- `fix/description` - Bug fixes
- `feat/description` - New features
- `docs/description` - Documentation
- `test/description` - Tests
- `refactor/description` - Code refactoring
- `perf/description` - Performance improvements

### 3. Make Your Changes

Follow the development workflow in [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md).

**Checklist:**
- [ ] Code compiles without warnings
- [ ] Tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Lints pass (`cargo clippy`)
- [ ] Documentation updated (if needed)
- [ ] Tests added for new code

### 4. Write Tests

All code changes should include tests. See [TESTING.md](TESTING.md).

**Minimum requirements:**
- Unit tests for new functions
- Integration tests for CLI changes
- Regression tests for bug fixes

### 5. Commit Your Changes

See [Commit Guidelines](#commit-guidelines) below.

### 6. Push and Create PR

```bash
# Push to your fork
git push origin fix/issue-123-autocomplete-bug

# Create PR on GitHub
```

## Code Style

### Rust Style Guidelines

**Follow standard Rust conventions:**

```rust
// Use rustfmt (enforced)
cargo fmt

// Fix clippy warnings (enforced)
cargo clippy -- -D warnings
```

### Formatting Rules

**Enforced by rustfmt:**
- 4 spaces for indentation (no tabs)
- 100 character line length
- Trailing commas in multi-line items

**Manual style choices:**

```rust
// Use descriptive variable names
// Good
let autocomplete_suggestions = get_suggestions();

// Bad
let x = get_suggestions();
let sug = get_suggestions();

// Prefer early returns
// Good
fn validate(input: &str) -> Result<(), Error> {
    if input.is_empty() {
        return Err(Error::Empty);
    }

    // Main logic here
    Ok(())
}

// Bad
fn validate(input: &str) -> Result<(), Error> {
    if !input.is_empty() {
        // Main logic nested
        Ok(())
    } else {
        Err(Error::Empty)
    }
}

// Use ? for error propagation
// Good
fn process() -> Result<Value, Error> {
    let data = read_file()?;
    let parsed = parse_json(&data)?;
    Ok(parsed)
}

// Bad
fn process() -> Result<Value, Error> {
    match read_file() {
        Ok(data) => match parse_json(&data) {
            Ok(parsed) => Ok(parsed),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}
```

### Documentation Standards

**Document all public APIs:**

```rust
/// Analyzes the query context to determine what type of suggestions to show.
///
/// This function examines the query string and cursor position to understand
/// whether the user is typing a field name, function name, or operator.
///
/// # Arguments
///
/// * `query` - The current jq query string
/// * `cursor` - The cursor position within the query
///
/// # Returns
///
/// A tuple containing:
/// - The suggestion context (field, function, etc.)
/// - The partial word being typed (for filtering)
///
/// # Examples
///
/// ```
/// use jiq::autocomplete::analyze_context;
///
/// let (context, partial) = analyze_context(".users.na", 9);
/// assert_eq!(partial, "na");
/// ```
pub fn analyze_context(query: &str, cursor: usize) -> (SuggestionContext, String) {
    // Implementation
}
```

**Module-level documentation:**

```rust
//! Context detection for autocomplete suggestions.
//!
//! This module provides functionality to analyze a jq query and determine
//! what type of autocomplete suggestions should be shown (fields, functions,
//! operators, etc.).

pub mod context;
```

### Error Handling

**Use proper error types:**

```rust
// Good - specific error type
fn parse_json(input: &str) -> Result<Value, JiqError> {
    serde_json::from_str(input)
        .map_err(|e| JiqError::InvalidJson(e.to_string()))
}

// Bad - generic error
fn parse_json(input: &str) -> Result<Value, String> {
    // Don't use String for errors
}
```

**Don't use unwrap/expect in library code:**

```rust
// Bad - panics in production
let value = some_operation().unwrap();

// Good - propagate error
let value = some_operation()?;

// Acceptable in tests
#[test]
fn test_something() {
    let value = some_operation().unwrap();  // OK in tests
}
```

### Performance Considerations

**Hot path optimizations:**

```rust
// Prefer borrowing over cloning
// Good
fn process(data: &str) { }

// Bad (if not necessary)
fn process(data: String) { }

// Use iterators instead of collecting
// Good
let filtered: Vec<_> = items.iter().filter(|x| x.is_valid()).collect();

// Better
let filtered = items.iter().filter(|x| x.is_valid());

// Avoid unnecessary allocations
// Good
static DATA: LazyLock<Vec<Item>> = LazyLock::new(|| {
    // Built once
});

// Bad
fn get_data() -> Vec<Item> {
    // Allocated on every call
}
```

## Commit Guidelines

### Commit Message Format

```
<type>: <short summary>

<optional longer description>

<optional footer>
```

**Types:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `style:` - Code style (formatting, no logic change)
- `refactor:` - Code refactoring (no functional changes)
- `perf:` - Performance improvement
- `test:` - Adding or updating tests
- `chore:` - Build process or auxiliary tools

**Examples:**

```
feat: add fuzzy matching to autocomplete

Implemented fuzzy search algorithm for suggestion filtering.
Users can now type partial function names like "mapv" to find "map_values".

Closes #123
```

```
fix: autocomplete not showing after pipe character

The context detection was incorrectly identifying the context
after pipe characters, causing autocomplete to not appear.

Fixed by trimming whitespace and checking the last non-whitespace
character for context detection.

Fixes #456
```

```
docs: update ARCHITECTURE.md with autocomplete flow

Added diagram showing the autocomplete data flow and context
detection algorithm.
```

### Commit Best Practices

**One logical change per commit:**

```bash
# Good - separate commits
git commit -m "feat: add fuzzy matching"
git commit -m "test: add tests for fuzzy matching"
git commit -m "docs: document fuzzy matching"

# Bad - everything in one commit
git commit -m "add fuzzy matching, tests, and docs"
```

**Write clear commit messages:**

```
# Good
fix: prevent autocomplete crash on empty query

The autocomplete popup would crash when the query was empty
because it tried to access query[0] without checking length.

Fixed by adding a length check before accessing.

# Bad
fix stuff
fixed bug
updates
```

## Pull Request Process

### Before Creating PR

- [ ] Code compiles: `cargo build`
- [ ] Tests pass: `cargo test`
- [ ] Format code: `cargo fmt`
- [ ] No lint warnings: `cargo clippy`
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if user-facing change)

### PR Title and Description

**Title format:**

```
<type>: <short description>
```

**Description template:**

```markdown
## Summary
Brief description of changes

## Motivation
Why is this change needed?

## Changes
- List of specific changes made
- Another change

## Testing
How was this tested?

## Screenshots (if UI changes)
![Before](url)
![After](url)

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No clippy warnings
- [ ] Formatted with cargo fmt

Closes #123
```

**Example:**

```markdown
## Summary
Add fuzzy matching to autocomplete suggestions

## Motivation
Users often remember part of a function name but not the exact prefix.
Fuzzy matching makes autocomplete more forgiving and discoverable.

## Changes
- Implemented fuzzy match algorithm using Levenshtein distance
- Updated suggestion filtering to use fuzzy match
- Added tests for fuzzy matching edge cases
- Updated documentation

## Testing
- Unit tests for fuzzy match algorithm
- Integration tests with various queries
- Manual testing with real-world queries

## Checklist
- [x] Tests added/updated
- [x] Documentation updated
- [x] CHANGELOG.md updated
- [x] No clippy warnings
- [x] Formatted with cargo fmt

Closes #123
```

### PR Size Guidelines

**Ideal PR size:**
- **Small:** <200 lines changed (preferred)
- **Medium:** 200-500 lines
- **Large:** >500 lines (consider splitting)

**If your PR is large:**
- Split into smaller, logical PRs if possible
- Provide extra context in description
- Consider creating a tracking issue with task list

### Draft PRs

Use draft PRs for:
- Work in progress
- Seeking early feedback
- Demonstrating an approach

```markdown
## Status: Draft / Work in Progress

This PR is not ready for review yet. I'm seeking feedback on:
- The overall approach
- The autocomplete algorithm choice
- The UI/UX design

TODO before review:
- [ ] Add tests
- [ ] Update documentation
- [ ] Fix performance issues
```

## Review Process

### What Reviewers Look For

**Code Quality:**
- Correctness and logic
- Error handling
- Performance considerations
- Code readability

**Testing:**
- Adequate test coverage
- Edge cases covered
- Tests are clear and maintainable

**Documentation:**
- Public APIs documented
- README updated if needed
- CHANGELOG updated

**Style:**
- Follows Rust conventions
- Passes cargo fmt
- No clippy warnings

### Responding to Feedback

**Be responsive and professional:**

```markdown
# Good response
Thanks for catching that! You're right, this could panic if the vector is empty.
I'll add a check and a test case.

# Also good
I considered that approach, but chose this one because [reason].
What do you think?

# Not helpful
That's just your opinion.
It works fine for me.
```

**Making changes:**

```bash
# Make changes based on feedback
git add .
git commit -m "address review feedback: add bounds check"
git push origin feat/my-feature
```

**Resolving conversations:**
- Mark conversations as resolved when addressed
- Explain your changes in replies
- Ask for clarification if needed

### CI Checks

All PRs must pass:
- [ ] Build succeeds
- [ ] All tests pass
- [ ] cargo fmt check passes
- [ ] cargo clippy passes with no warnings

**If CI fails:**
1. Read the error message carefully
2. Fix locally and test
3. Push fix
4. Wait for CI to rerun

## Community

### Getting Help

- **Questions:** [GitHub Discussions](https://github.com/bellicose100xp/jiq/discussions)
- **Bugs:** [GitHub Issues](https://github.com/bellicose100xp/jiq/issues)
- **PRs:** Tag maintainers if no response after 1 week

### Maintainer Response Time

- **Issues:** Usually within 1-3 days
- **PRs:** Usually within 3-7 days
- **Security issues:** Within 24 hours

### Recognition

Contributors are recognized in:
- Git commit history
- Release notes
- CHANGELOG.md (for significant contributions)
- README.md contributors section (if applicable)

## License

By contributing, you agree that your contributions will be licensed under the same MIT OR Apache-2.0 dual license that covers the project.

---

**Thank you for contributing to jiq!** Your efforts help make this tool better for everyone.

If you have questions about contributing, feel free to ask in [Discussions](https://github.com/bellicose100xp/jiq/discussions).
