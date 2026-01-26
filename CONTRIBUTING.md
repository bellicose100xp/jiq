# Contributing to jiq

## Prerequisites

- Rust (latest stable, 2024 edition)
- jq v1.8.1+ ([jqlang.org/download](https://jqlang.org/download/)) - required for snapshot tests
- cargo-insta (`cargo install cargo-insta`)

## Architecture

### Module Structure

**Never use `mod.rs`** - Use `{module_name}.rs` for the main module file (Rust 2018+ style).

Features are self-contained modules with prefixed filenames:

```
src/
├── feature_name.rs                    # Main module - re-exports public API
└── feature_name/
    ├── feature_name_state.rs          # State struct
    ├── feature_name_events.rs         # Event handlers (optional)
    ├── feature_name_render.rs         # Rendering (optional)
    ├── feature_name_state_tests.rs    # Tests for state
    └── feature_name_events_tests.rs   # Tests for events
```

### Key Rules

1. **No mod.rs** - Use `{module_name}.rs` as main module file, never `mod.rs`
2. **Prefix filenames** - Files in a module directory are prefixed with module name (e.g., `history_state.rs` not `state.rs`)
3. **Separate test files** - Tests go in `{module}_tests.rs`, not co-located with implementation
4. **Self-contained modules** - Features define their state and event logic in their own module
5. **Integration pattern** - `App` holds feature state, main dispatcher calls feature event handlers
6. **Small files** - Under 1000 lines (including tests), single responsibility

Example:
```rust
// src/clipboard.rs - Main module file (NOT mod.rs)
pub mod clipboard_state;
pub mod clipboard_events;

pub use clipboard_state::ClipboardState;
pub use clipboard_events::handle_clipboard_key;

// src/clipboard/clipboard_state.rs - State struct
pub struct ClipboardState { ... }

#[cfg(test)]
#[path = "clipboard_state_tests.rs"]
mod clipboard_state_tests;

// src/clipboard/clipboard_events.rs - Event handlers
pub fn handle_clipboard_key(app: &mut App, key: KeyEvent) -> bool { ... }

// Main app integrates the feature
// src/app/app_state.rs
pub struct App {
    pub clipboard: ClipboardState,
    ...
}

// src/app/app_events.rs - Dispatcher calls feature handler
if clipboard::handle_clipboard_key(self, key) {
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

**DRY Principles:**
- Extract repeated logic into reusable functions or modules
- Use traits for shared behavior across types
- Create utility modules for common operations

**Focused Code:**
- Functions do one thing well
- Clear, self-explanatory naming
- Prefer early returns over deep nesting
- Extract helpers for complex conditionals

**Theme & Styling:**
- All colors are centralized in `src/theme.rs`
- Use `theme::module::CONSTANT` in render files
- Never hardcode `Color::*` in render files
- Add new colors to `theme.rs` before using them

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
