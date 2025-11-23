# jiq - Technical Documentation

Technical documentation and architecture notes for the jiq codebase.

## What is jiq?

**jiq** is an interactive JSON query tool built in Rust that provides:
- Real-time jq query execution with instant feedback
- VIM-style keybindings for power users
- Context-aware autocomplete for jq functions and JSON fields
- Beautiful terminal UI built with Ratatui

## Documentation Index

### Core Documentation

**[ARCHITECTURE.md](ARCHITECTURE.md)** - System design and component interactions
- High-level system overview
- Component diagrams and data flow
- Module structure and responsibilities
- Design decisions and rationale

**[DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** - Development workflows and patterns
- Common development tasks
- Code organization principles
- Best practices and idioms
- Debugging techniques

**[TESTING.md](TESTING.md)** - Testing strategy and patterns
- Test structure and organization
- Running tests
- Writing effective tests
- Coverage goals

**[DEPLOYMENT.md](DEPLOYMENT.md)** - Release process
- cargo-dist automation
- Version management
- Distribution channels
- Release checklist

### Subsystem Documentation

**[subsystems/EVENT_SYSTEM.md](subsystems/EVENT_SYSTEM.md)** - Event handling architecture
- Event flow and dispatching
- Keyboard event handling
- Mode-specific event routing
- Focus management

**[subsystems/AUTOCOMPLETE.md](subsystems/AUTOCOMPLETE.md)** - Autocomplete system deep dive
- Context detection algorithm
- Suggestion generation
- Performance optimizations
- Future improvements

**[subsystems/VIM_EDITOR.md](subsystems/VIM_EDITOR.md)** - VIM modal editing
- Mode system (INSERT/NORMAL/OPERATOR)
- Command parsing and execution
- State transitions
- tui-textarea integration

**[subsystems/QUERY_EXECUTION.md](subsystems/QUERY_EXECUTION.md)** - Query execution pipeline
- jq process management
- Input/output handling
- Error parsing
- Performance considerations

**[subsystems/RENDERING.md](subsystems/RENDERING.md)** - UI rendering system
- Ratatui layout management
- Syntax highlighting
- Popup rendering
- Performance optimization

**[subsystems/SYNTAX_HIGHLIGHTING.md](subsystems/SYNTAX_HIGHLIGHTING.md)** - jq syntax highlighting
- Color scheme and token types
- Character-by-character parser
- Overlay rendering approach
- Edge cases and test coverage

### Feature Documentation

**[features/AUTOCOMPLETE.md](features/AUTOCOMPLETE.md)** - Original autocomplete feature notes

## Project Structure

```
jiq/
├── src/
│   ├── main.rs              # Entry point, CLI, main loop
│   ├── error.rs             # Error types
│   │
│   ├── app/                 # Application coordination
│   │   ├── mod.rs           # Public API
│   │   ├── state.rs         # App state, focus management
│   │   ├── events.rs        # Event dispatch and handling
│   │   └── render.rs        # UI rendering logic
│   │
│   ├── autocomplete/        # Autocomplete system
│   │   ├── mod.rs
│   │   ├── state.rs         # Suggestion state
│   │   ├── context.rs       # Context detection
│   │   ├── jq_functions.rs  # Built-in function database
│   │   └── json_analyzer.rs # JSON field extraction
│   │
│   ├── editor/              # VIM modal editing
│   │   ├── mod.rs
│   │   └── mode.rs          # Mode definitions
│   │
│   ├── input/               # Input handling
│   │   ├── mod.rs
│   │   └── reader.rs        # JSON input reader
│   │
│   └── query/               # Query execution
│       ├── mod.rs
│       └── executor.rs      # jq subprocess executor
│
├── tests/
│   ├── integration_tests.rs
│   └── fixtures/
│
└── development/             # This directory
    ├── README.md            # You are here
    ├── ARCHITECTURE.md
    ├── DEVELOPMENT_GUIDE.md
    ├── TESTING.md
    ├── DEPLOYMENT.md
    ├── subsystems/          # Detailed subsystem docs
    └── features/            # Feature-specific notes
```

## Key Technologies

- **Rust 2024 Edition** (MSRV: 1.80+)
- **Ratatui 0.29** - TUI framework
- **Crossterm 0.28** - Terminal manipulation
- **tui-textarea 0.7** - Text editor widget
- **serde_json 1.0** - JSON parsing
- **External jq binary** - Query execution

## Quick Reference

### Build & Test
```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test               # Run all tests
cargo clippy             # Linting
cargo fmt                # Format code
```

### Development
```bash
cargo watch -x test      # Auto-run tests
cargo watch -x 'run -- tests/fixtures/simple.json'
```

### Release
```bash
git tag v2.x.x
git push origin v2.x.x   # Triggers CI release
```

---

**Note:** This documentation is for maintainer reference. It focuses on understanding the codebase architecture and internal design decisions.
