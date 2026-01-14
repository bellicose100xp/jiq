# Contributing to jiq

## Prerequisites

- Rust (latest stable, 2024 edition)
- jq v1.8.1+ ([jqlang.org/download](https://jqlang.org/download/)) - required for snapshot tests
- cargo-insta (`cargo install cargo-insta`)

## Architecture

### Module Structure

Features are self-contained modules:

```
src/feature_name/
├── mod.rs      # Exports
├── state.rs    # State struct
├── events.rs   # Event handlers (optional)
```

### Key Rules

1. **Self-contained modules** - Features define their state and event logic in their own module
2. **Integration pattern** - `App` holds feature state, main dispatcher calls feature event handlers
3. **Small files** - Easy to reason about, single responsibility

Example:
```rust
// Feature owns its state and events
// src/clipboard/state.rs
pub struct ClipboardState { ... }

// src/clipboard/events.rs
pub fn handle_clipboard_key(app: &mut App, key: KeyEvent) -> bool { ... }

// Main app integrates the feature
// src/app/state.rs
pub struct App {
    pub clipboard: ClipboardState,
    ...
}

// src/app/events.rs - Dispatcher calls feature handler
if clipboard::events::handle_clipboard_key(self, key) {
    return;
}
```

### Why This Pattern

1. **Separation of Concerns** - Each module owns its domain logic, main app is just a coordinator
2. **Testability** - Feature modules tested independently without full app context
3. **Maintainability** - Changes to clipboard don't touch history or search
4. **Rust Ownership** - App owns state, feature functions borrow mutably, clear ownership
5. **Scalability** - Adding features doesn't bloat existing files

## Standards

### Code Quality

```bash
cargo build              # Zero warnings required
cargo clippy -D warnings # Must pass
```

### Rust 2024 Edition

- Use `#[derive(Default)]` with `#[default]` for enums
- Prefer derive macros over manual implementations
- Document public APIs with `///`

## Testing

### Required

- **Unit tests** for all business logic
- **Snapshot tests** for visual components (`cargo-insta`)
- **Debug logging** for TUI interactions (`log::debug!`)

```bash
cargo test           # All tests
cargo insta test     # Snapshots
cargo insta review   # Accept snapshots
```

## Pull Request Checklist

- [ ] Feature in self-contained module
- [ ] Unit tests for logic
- [ ] Snapshot tests for visuals
- [ ] Zero warnings (`cargo build` + `cargo clippy`)
- [ ] Manual TUI testing done

## Commits

```
type(scope): description

feat(clipboard): add OSC 52 support
fix(autocomplete): handle empty suggestions
refactor(help): move to dedicated module
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`
