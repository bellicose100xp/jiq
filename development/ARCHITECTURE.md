# Architecture

This document describes the high-level architecture of jiq, explaining how the different components work together to provide an interactive JSON query experience.

## Table of Contents

1. [System Overview](#system-overview)
2. [Visual Architecture](#visual-architecture)
3. [Module Structure](#module-structure)
4. [Data Flow](#data-flow)
5. [Component Details](#component-details)
6. [Design Decisions](#design-decisions)
7. [Performance Considerations](#performance-considerations)

## System Overview

**jiq** is a terminal-based interactive application that provides real-time jq query execution with VIM-style editing and context-aware autocomplete.

### Core Responsibilities

1. **Input Handling** - Read JSON from file or stdin
2. **Query Editing** - VIM-style text editor with modal editing
3. **Query Execution** - Execute jq queries via external process
4. **Autocomplete** - Context-aware suggestions for queries
5. **UI Rendering** - Two-pane TUI with syntax highlighting
6. **Output** - Export results or query string

### Technology Stack

```
┌─────────────────────────────────────────────────┐
│                   jiq (Rust)                    │
├─────────────────────────────────────────────────┤
│  UI Layer          │  Ratatui 0.29              │
│  Terminal          │  Crossterm 0.28            │
│  Text Editor       │  tui-textarea 0.7          │
│  JSON Processing   │  serde_json 1.0            │
│  Query Execution   │  jq (external binary)      │
│  Error Handling    │  color-eyre 0.6            │
└─────────────────────────────────────────────────┘
```

## Visual Architecture

### High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                            main.rs                              │
│                        (Entry Point)                            │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   ├──────────────────────────────────────────────┐
                   │                                              │
           ┌───────▼────────┐                            ┌────────▼────────┐
           │  InputReader   │                            │   JqExecutor    │
           │  (input/)      │                            │   (query/)      │
           └───────┬────────┘                            └────────┬────────┘
                   │                                              │
                   │  JSON                                        │  Results
                   │                                              │
           ┌───────▼──────────────────────────────────────────────▼────────┐
           │                                                                │
           │                         App (app/)                             │
           │                      Application State                         │
           │                                                                │
           │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
           │  │   Editor     │  │ Autocomplete │  │    Query     │        │
           │  │  (editor/)   │  │(autocomplete)│  │  Executor    │        │
           │  │              │  │              │  │              │        │
           │  │ - Mode       │  │ - State      │  │ - Execute    │        │
           │  │ - VIM cmds   │  │ - Context    │  │ - Results    │        │
           │  │              │  │ - Suggestions│  │              │        │
           │  └──────────────┘  └──────────────┘  └──────────────┘        │
           │                                                                │
           └────────────────────────────┬───────────────────────────────────┘
                                        │
                                        │  UI Rendering
                                        │
                        ┌───────────────▼────────────────┐
                        │         Ratatui                │
                        │    Terminal Rendering          │
                        └───────────────┬────────────────┘
                                        │
                        ┌───────────────▼────────────────┐
                        │        Crossterm               │
                        │   Terminal Manipulation        │
                        └────────────────────────────────┘
```

### UI Layout

```
┌────────────────────────────────────────────────────────────────┐
│                        Terminal Window                         │
├────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Query Input (30%)                        [INSERT MODE]   │  │
│  ├──────────────────────────────────────────────────────────┤  │
│  │ .users[] | select(.active == true)                       │  │
│  │                                                           │  │
│  │ ┌─ Autocomplete Popup ─────────┐                         │  │
│  │ │ ► select      [fn]            │                         │  │
│  │ │   select_values [fn]          │                         │  │
│  │ │   .status      [field]        │                         │  │
│  │ └───────────────────────────────┘                         │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Results (70%)                                            │  │
│  ├──────────────────────────────────────────────────────────┤  │
│  │ [                                                         │  │
│  │   {                                                       │  │
│  │     "name": "Alice",                                      │  │
│  │     "active": true,                                       │  │
│  │     "email": "alice@example.com"                          │  │
│  │   },                                                      │  │
│  │   {                                                       │  │
│  │     "name": "Bob",                                        │  │
│  │     "active": true,                                       │  │
│  │     "email": "bob@example.com"                            │  │
│  │   }                                                       │  │
│  │ ]                                                         │  │
│  └──────────────────────────────────────────────────────────┘  │
│  Tab: Switch Focus | Enter: Output | Shift+Enter: Query Only  │
└────────────────────────────────────────────────────────────────┘
```

## Module Structure

### Source Directory Layout

```
src/
├── main.rs                    # Entry point, CLI parsing, main loop
├── error.rs                   # Custom error types (JiqError)
│
├── app/                       # Application state and coordination
│   ├── mod.rs                 # Public API (re-exports)
│   ├── state.rs               # App struct, state management
│   ├── events.rs              # Event handling (keyboard, etc.)
│   └── render.rs              # UI rendering logic
│
├── autocomplete/              # Autocomplete system
│   ├── mod.rs                 # Public API
│   ├── state.rs               # Autocomplete state, suggestions
│   ├── context.rs             # Context detection (field vs function)
│   ├── jq_functions.rs        # Static jq built-ins database
│   └── json_analyzer.rs       # Extract fields from JSON
│
├── editor/                    # VIM-style editor
│   ├── mod.rs                 # Public API
│   └── mode.rs                # Editor modes (INSERT/NORMAL/OPERATOR)
│
├── input/                     # Input handling
│   ├── mod.rs                 # Public API
│   └── reader.rs              # Read JSON from file or stdin
│
└── query/                     # Query execution
    ├── mod.rs                 # Public API
    └── executor.rs            # Execute jq via subprocess
```

### Module Responsibilities

| Module | Purpose | Key Types | Dependencies |
|--------|---------|-----------|--------------|
| `main.rs` | Entry point, orchestration | `Args`, `main()` | All modules |
| `error.rs` | Error handling | `JiqError` | thiserror |
| `app` | UI state, event loop | `App`, `Focus`, `OutputMode` | All subsystems |
| `autocomplete` | Suggestion system | `AutocompleteState`, `Suggestion` | serde_json |
| `editor` | Text editing modes | `EditorMode` | tui-textarea |
| `input` | JSON input | `InputReader` | std::fs, std::io |
| `query` | jq execution | `JqExecutor` | std::process |

## Data Flow

### Application Lifecycle

```
1. INITIALIZATION
   ┌──────────────────┐
   │   Parse CLI Args │
   └────────┬─────────┘
            │
            ▼
   ┌──────────────────┐
   │  Validate jq     │
   │  binary exists   │
   └────────┬─────────┘
            │
            ▼
   ┌──────────────────┐
   │  Read JSON input │
   │  (file or stdin) │
   └────────┬─────────┘
            │
            ▼
   ┌──────────────────┐
   │  Initialize      │
   │  Terminal (TUI)  │
   └────────┬─────────┘
            │
            ▼
   ┌──────────────────┐
   │  Create App with │
   │  initial state   │
   └────────┬─────────┘
            │
            ▼

2. EVENT LOOP
   ┌──────────────────┐
   │  Render UI       │ ◄──────────────┐
   │  (draw frame)    │                │
   └────────┬─────────┘                │
            │                          │
            ▼                          │
   ┌──────────────────┐                │
   │  Wait for event  │                │
   │  (keyboard, etc.)│                │
   └────────┬─────────┘                │
            │                          │
            ▼                          │
   ┌──────────────────┐                │
   │  Handle event    │                │
   │  - Update state  │                │
   │  - Execute query │                │
   │  - Autocomplete  │                │
   └────────┬─────────┘                │
            │                          │
            ▼                          │
   ┌──────────────────┐                │
   │  Should quit?    │────No──────────┘
   └────────┬─────────┘
            │ Yes
            ▼

3. SHUTDOWN
   ┌──────────────────┐
   │  Restore terminal│
   └────────┬─────────┘
            │
            ▼
   ┌──────────────────┐
   │  Output results  │
   │  (if requested)  │
   └──────────────────┘
```

### Query Execution Flow

```
User types character
        │
        ▼
┌───────────────────┐
│  Event: KeyPress  │
└─────────┬─────────┘
          │
          ▼
┌─────────────────────────────┐
│  app::events::handle_events │
└─────────┬───────────────────┘
          │
          ▼
┌──────────────────────────────┐     ┌─────────────────────────┐
│  Update query text           │────►│  Update autocomplete    │
│  (tui-textarea)              │     │  suggestions            │
└─────────┬────────────────────┘     └─────────────────────────┘
          │                                      │
          │                          ┌───────────▼────────────┐
          │                          │  Analyze context       │
          │                          │  - Field vs Function   │
          │                          │  - Extract prefix      │
          │                          └───────────┬────────────┘
          │                                      │
          │                          ┌───────────▼────────────┐
          │                          │  Generate suggestions  │
          │                          │  - jq functions        │
          │                          │  - JSON fields         │
          │                          │  - Filter by prefix    │
          │                          └────────────────────────┘
          │
          ▼
┌──────────────────────────────┐
│  JqExecutor::execute()       │
└─────────┬────────────────────┘
          │
          ▼
┌──────────────────────────────┐
│  Spawn jq subprocess         │
│  - Pass query as arg         │
│  - Pipe JSON to stdin        │
└─────────┬────────────────────┘
          │
          ▼
┌──────────────────────────────┐
│  Capture stdout/stderr       │
└─────────┬────────────────────┘
          │
          ├─── Success ─────►┌──────────────────────┐
          │                  │  Parse ANSI colors   │
          │                  │  Display results     │
          │                  └──────────────────────┘
          │
          └─── Error ──────►┌──────────────────────┐
                            │  Display error msg   │
                            │  (red text)          │
                            └──────────────────────┘
```

### Autocomplete Flow

```
Query text changes
        │
        ▼
┌────────────────────────┐
│  autocomplete::update  │
└──────────┬─────────────┘
           │
           ▼
┌──────────────────────────────┐
│  context::analyze_context    │
│  Input: ".users.na"          │
│  Output: FieldContext, "na"  │
└──────────┬───────────────────┘
           │
           ▼
     ┌────┴──────┐
     │           │
     ▼           ▼
Field Context   Function Context
     │               │
     ▼               ▼
┌─────────────┐  ┌──────────────────┐
│  Analyze    │  │  Filter jq       │
│  JSON       │  │  built-ins       │
│  structure  │  │  (static data)   │
└─────┬───────┘  └────────┬─────────┘
      │                   │
      ▼                   ▼
┌─────────────┐  ┌──────────────────┐
│  Extract    │  │  Match prefix    │
│  fields:    │  │  "ma" →          │
│  - name     │  │  - map           │
│  - email    │  │  - map_values    │
│  - active   │  │  - match         │
└─────┬───────┘  └────────┬─────────┘
      │                   │
      └───────┬───────────┘
              ▼
     ┌────────────────┐
     │  Merge and     │
     │  sort results  │
     └────────┬───────┘
              │
              ▼
     ┌────────────────┐
     │  Update popup  │
     │  - Max 10      │
     │  - Color coded │
     │  - Scrollable  │
     └────────────────┘
```

## Component Details

### 1. Main Entry Point (`main.rs`)

**Responsibilities:**
- Parse CLI arguments using clap
- Validate jq binary exists
- Read JSON input (file or stdin)
- Initialize and run TUI
- Handle output after exit

**Key Functions:**
```rust
fn main() -> Result<()>
fn validate_jq_exists() -> Result<(), JiqError>
fn run(terminal: DefaultTerminal, json: String) -> Result<App>
fn handle_output(app: &App, json: &str) -> Result<()>
```

**Flow:**
1. `main()` orchestrates entire lifecycle
2. `validate_jq_exists()` checks for jq binary
3. `run()` contains event loop
4. `handle_output()` prints results after terminal restored

### 2. Application State (`app/state.rs`)

**Core Type:**
```rust
pub struct App {
    // UI state
    query_input: TextArea<'static>,
    result_text: String,
    scroll_offset: usize,
    focus: Focus,

    // Data
    json_input: String,
    json_analyzer: JsonAnalyzer,

    // Subsystems
    autocomplete: AutocompleteState,
    editor_mode: EditorMode,

    // Control
    should_quit: bool,
    output_mode: Option<OutputMode>,
}
```

**Key Methods:**
- `new(json_input: String) -> Self` - Initialize app
- `query() -> &str` - Get current query text
- `execute_query()` - Run jq and update results
- `should_quit() -> bool` - Exit condition

**State Enums:**
```rust
pub enum Focus {
    Input,   // User typing in query field
    Results, // User scrolling results
}

pub enum OutputMode {
    Results, // Output filtered JSON
    Query,   // Output query string only
}
```

### 3. Event Handling (`app/events.rs`)

**Responsibilities:**
- Read keyboard events from Crossterm
- Dispatch to appropriate handlers based on:
  - Current focus (Input vs Results)
  - Editor mode (INSERT vs NORMAL)
  - Global keys (Tab, Enter, Quit)

**Event Priority:**
1. Global keys (work anywhere)
2. Focus-specific keys (Input vs Results)
3. Mode-specific keys (VIM modes)

**Key Functions:**
```rust
fn handle_events(&mut self) -> Result<()>
fn handle_input_events(&mut self, event: Event) -> Result<()>
fn handle_results_events(&mut self, event: Event) -> Result<()>
fn handle_insert_mode_key(&mut self, key: KeyEvent) -> Result<()>
fn handle_normal_mode_key(&mut self, key: KeyEvent) -> Result<()>
```

### 4. Rendering (`app/render.rs`)

**Responsibilities:**
- Draw two-pane layout (input + results)
- Render autocomplete popup
- Syntax highlighting for JSON
- Mode indicators (INSERT/NORMAL)

**Layout:**
```rust
Layout::vertical([
    Constraint::Percentage(30), // Input pane
    Constraint::Percentage(70), // Results pane
])
```

**Visual Indicators:**
- Border colors match editor mode:
  - Cyan = INSERT
  - Yellow = NORMAL
  - Green = OPERATOR
- Autocomplete popup overlays input pane
- Help text at bottom

### 5. Autocomplete System (`autocomplete/`)

See [features/AUTOCOMPLETE.md](features/AUTOCOMPLETE.md) for detailed documentation.

**High-level:**
- `state.rs` - Manages suggestion list, selection
- `context.rs` - Detects whether to suggest fields or functions
- `jq_functions.rs` - Static database of jq built-ins (LazyLock)
- `json_analyzer.rs` - Extracts field names from input JSON

**Performance:**
- Static data with `LazyLock` (built once)
- Filtering instead of rebuilding
- Minimal allocations

### 6. Editor Modes (`editor/mode.rs`)

**VIM Modal Editing:**
```rust
pub enum EditorMode {
    Insert,   // Regular typing
    Normal,   // VIM navigation
    Operator, // After 'd' or 'c' (waiting for motion)
}
```

**Mode Transitions:**
```
INSERT ←──i,a,I,A──→ NORMAL ←──d,c──→ OPERATOR
  │                    │                │
  └────────ESC─────────┘                │
                                        │
                                        └──motion──→ Execute → NORMAL
```

**Supported Operations:**
- Navigation: h, l, 0, $, w, b, e
- Insert: i, a, I, A
- Delete: x, X, dw, db, de, d$, dd
- Change: cw, cb, ce, c$, cc
- Undo/Redo: u, Ctrl+r

### 7. Query Executor (`query/executor.rs`)

**Responsibilities:**
- Spawn jq subprocess
- Pass query and JSON
- Capture output
- Handle errors

**Implementation:**
```rust
pub struct JqExecutor {
    json_input: String,
}

impl JqExecutor {
    pub fn execute(&self, query: &str) -> Result<String, JiqError> {
        let output = Command::new("jq")
            .arg(query)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()?;

        // Parse output, handle errors
    }
}
```

**Error Handling:**
- Invalid query → Show jq error message
- jq not found → JiqError::JqNotFound
- Process spawn failure → Propagate error

### 8. Input Reader (`input/reader.rs`)

**Responsibilities:**
- Read JSON from file path or stdin
- Validate JSON syntax
- Return as String

**Implementation:**
```rust
pub struct InputReader;

impl InputReader {
    pub fn read_json(path: Option<&Path>) -> Result<String, JiqError> {
        let content = match path {
            Some(p) => fs::read_to_string(p)?,
            None => Self::read_from_stdin()?,
        };

        // Validate JSON
        serde_json::from_str::<serde_json::Value>(&content)?;

        Ok(content)
    }
}
```

**Special Handling:**
- Stdin uses Crossterm's `use-dev-tty` feature
- Allows reading from pipe while TUI runs

## Design Decisions

### Why External jq Instead of Native Rust?

**Decision:** Use external `jq` binary instead of implementing jq in Rust.

**Rationale:**
1. **Correctness** - jq has 15+ years of development and testing
2. **Completeness** - Supports all jq features without reimplementation
3. **Maintenance** - Upstream handles bugs and new features
4. **Scope** - Implementing a full jq parser/executor is a massive project

**Trade-offs:**
- ✅ Guaranteed correctness
- ✅ Zero maintenance for query logic
- ❌ Requires jq installation
- ❌ Subprocess overhead (negligible for interactive use)

### Why LazyLock for Static Data?

**Decision:** Use `LazyLock` for jq built-ins database instead of const initialization.

**Rationale:**
1. **Performance** - Built once at first access, zero runtime cost
2. **Ergonomics** - Allows complex initialization logic
3. **Modern Rust** - Uses Rust 1.80+ feature

**Impact:**
- 90% reduction in allocations during typing
- Instant autocomplete responses

### Why Two-Pane Layout?

**Decision:** Split screen into input (30%) and results (70%).

**Rationale:**
1. **Feedback** - See results while typing query
2. **Context** - Both query and output visible simultaneously
3. **Standard** - Matches user expectations from similar tools

**Alternatives Considered:**
- Single pane with toggle (rejected: too much switching)
- Three panes with JSON structure (rejected: too cluttered)

### Why VIM Keybindings?

**Decision:** Implement full VIM modal editing for power users.

**Rationale:**
1. **Target Audience** - Command-line users often know VIM
2. **Efficiency** - Navigate/edit without leaving home row
3. **Expectations** - jq users likely familiar with VIM

**Trade-off:**
- ✅ Power users love it
- ❌ Learning curve for beginners
- **Mitigation:** Default to INSERT mode, show hints

### Why Ratatui Over Other TUI Libraries?

**Decision:** Use Ratatui instead of cursive, tui-rs (archived), or termion.

**Rationale:**
1. **Active Development** - tui-rs fork with ongoing maintenance
2. **Modern API** - Clean, ergonomic design
3. **Ecosystem** - Good widget library (tui-textarea)
4. **Performance** - Efficient rendering

## Performance Considerations

### Hot Paths

The following code paths execute on every keystroke:

1. **Event handling** (`app/events.rs`)
   - Minimize allocations
   - Early returns for common cases

2. **Query execution** (`query/executor.rs`)
   - Spawns subprocess on every keystroke
   - Acceptable for interactive use (<100ms)
   - Could debounce for large JSON files (future optimization)

3. **Autocomplete filtering** (`autocomplete/`)
   - Uses static data (LazyLock)
   - Filters instead of rebuilding
   - Limited to 10 suggestions

### Memory Usage

**Baseline:**
- JSON input stored as String (~1× input size)
- Query results stored as String (~1× output size)
- Autocomplete suggestions: ~100 static items

**Total:** ~2-3× input JSON size in memory

### Optimization Opportunities (Future)

1. **Debounce query execution** - Wait 100ms before executing
2. **Incremental rendering** - Only redraw changed regions
3. **Virtual scrolling** - Lazy load large result sets
4. **Stream processing** - Handle JSON streams

## Testing Strategy

### Unit Tests

Each module has comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_detection() {
        // Test isolated component logic
    }
}
```

**Coverage:**
- Autocomplete context detection
- JSON field extraction
- Editor mode transitions
- Error handling

### Integration Tests

Tests in `tests/integration_tests.rs`:

```rust
#[test]
fn test_cli_with_invalid_json() {
    cargo_bin_cmd!()
        .arg("invalid.json")
        .assert()
        .failure();
}
```

**Coverage:**
- CLI argument parsing
- File input handling
- Error messages

### Testing Challenges

**Interactive TUI:**
- Difficult to test rendering
- Can't easily simulate keyboard input
- Terminal state management

**Mitigation:**
- Extract business logic from UI code
- Test logic functions independently
- Use `#[cfg(test)]` for test helpers

## Extension Points

### Adding New Autocomplete Suggestion Types

1. Add variant to `SuggestionType` enum
2. Add variant to `SuggestionContext` enum
3. Implement detection in `context::analyze_context()`
4. Add color in `render.rs`

### Adding New Editor Modes

1. Add variant to `EditorMode` enum
2. Add handler in `app/events.rs`
3. Add visual indicator in `render.rs`

### Adding New Output Formats

1. Add variant to `OutputMode` enum
2. Handle in `main.rs::handle_output()`

## Further Reading

- [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) - Day-to-day workflows
- [TESTING.md](TESTING.md) - Testing practices
- [features/AUTOCOMPLETE.md](features/AUTOCOMPLETE.md) - Autocomplete deep dive

---

**Questions about architecture?** Open a discussion: https://github.com/bellicose100xp/jiq/discussions
