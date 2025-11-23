# Developer Documentation

Welcome to the **jiq** developer documentation! This guide will help you understand and work with the jiq codebase, whether you're a first-time Rust developer or a seasoned expert.

## What is jiq?

**jiq** is an interactive JSON query tool built in Rust that provides:
- Real-time jq query execution with instant feedback
- VIM-style keybindings for power users
- Context-aware autocomplete for jq functions and JSON fields
- Beautiful terminal UI built with Ratatui

## Quick Navigation

### For Developers

Start here if you're new to the project:

1. **[Getting Started](GETTING_STARTED.md)** - Set up your development environment
2. **[Architecture](ARCHITECTURE.md)** - Understand how jiq works

### For Active Developers

Day-to-day development resources:

- **[Development Guide](DEVELOPMENT_GUIDE.md)** - Common workflows and best practices
- **[Testing Guide](TESTING.md)** - How to write and run tests
- **[Deployment](DEPLOYMENT.md)** - Release process and distribution

### Deep Dives

Detailed feature documentation:

- **[Autocomplete Feature](features/AUTOCOMPLETE.md)** - Implementation details of the autocomplete system
- **[Distribution Strategy](features/DEPLOYMENT.md)** - How we package and distribute jiq

## Documentation Map by Developer Experience

### Beginner Developers (New to Rust or TUIs)

Focus on these documents in order:

1. [Getting Started](GETTING_STARTED.md) - Setup and first contribution
2. [Architecture](ARCHITECTURE.md#visual-overview) - Visual diagrams of system
3. [Testing Guide](TESTING.md#running-tests) - How to run tests
4. [Development Guide](DEVELOPMENT_GUIDE.md#common-tasks) - Simple tasks to start with

**Good Starting Points:**
- Explore documentation
- Run and understand tests
- Trace code execution with debugger

### Intermediate Developers (Familiar with Rust)

1. [Architecture](ARCHITECTURE.md) - Full system design
2. [Development Guide](DEVELOPMENT_GUIDE.md) - Development workflows
3. [Testing Guide](TESTING.md) - Test patterns used

**Focus Areas:**
- Feature enhancements
- Performance improvements
- UI/UX improvements

### Senior Developers (Rust experts)

1. [Architecture](ARCHITECTURE.md#design-decisions) - Design rationale
2. [Autocomplete Feature](features/AUTOCOMPLETE.md) - Complex feature deep-dive
3. [Development Guide](DEVELOPMENT_GUIDE.md#advanced-workflows) - Advanced patterns
4. [Deployment](DEPLOYMENT.md) - Release engineering

**Recommended Work:**
- Architectural improvements
- New major features
- Performance optimization
- Release management

## Key Technologies

- **Language:** Rust 2024 Edition (MSRV: 1.80+)
- **TUI Framework:** [Ratatui](https://ratatui.rs/) 0.29
- **Terminal:** [Crossterm](https://github.com/crossterm-rs/crossterm) 0.28
- **JSON Processing:** External `jq` binary
- **Testing:** Built-in test framework + [assert_cmd](https://docs.rs/assert_cmd)

## Project Structure

```
jiq/
├── src/
│   ├── main.rs              # Entry point
│   ├── error.rs             # Error types
│   ├── app/                 # Application state and UI
│   │   ├── mod.rs           # Public API
│   │   ├── state.rs         # App state management
│   │   ├── events.rs        # Event handling
│   │   └── render.rs        # UI rendering
│   ├── autocomplete/        # Autocomplete system
│   │   ├── mod.rs
│   │   ├── state.rs         # Autocomplete state
│   │   ├── context.rs       # Context detection
│   │   ├── jq_functions.rs  # jq built-ins database
│   │   └── json_analyzer.rs # JSON field extraction
│   ├── editor/              # VIM-style editor
│   │   ├── mod.rs
│   │   └── mode.rs          # Editor modes (INSERT/NORMAL)
│   ├── input/               # Input handling
│   │   ├── mod.rs
│   │   └── reader.rs        # JSON input reader
│   └── query/               # Query execution
│       ├── mod.rs
│       └── executor.rs      # jq process executor
├── tests/
│   ├── integration_tests.rs # Integration tests
│   └── fixtures/            # Test data
├── development/             # This directory!
│   ├── README.md            # You are here
│   ├── GETTING_STARTED.md
│   ├── ARCHITECTURE.md
│   ├── DEVELOPMENT_GUIDE.md
│   ├── TESTING.md
│   ├── DEPLOYMENT.md
│   └── features/            # Detailed feature docs
└── Cargo.toml
```

## Getting Help

- **Questions?** Open a [GitHub Discussion](https://github.com/bellicose100xp/jiq/discussions)
- **Bugs?** File an [Issue](https://github.com/bellicose100xp/jiq/issues)

## License

jiq is dual-licensed under MIT OR Apache-2.0. See [LICENSE-MIT](../LICENSE-MIT) and [LICENSE-APACHE](../LICENSE-APACHE) for details.

---

**Ready to dive in?** Start with [Getting Started](GETTING_STARTED.md) →
