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

## Rust Module Structure
- Use `{module_name}.rs`, never `mod.rs`
- Place tests in `{module_name}_tests.rs` files
- Never co-locate tests with implementation
- Split large test files into `{module_name}_tests/` directory with focused test modules
- Refactor files exceeding 1000 lines into multiple focused modules
- Extract repeated logic into reusable functions/modules (DRY principle)
- Split complex business logic into separate files for clarity
