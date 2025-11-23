# Architecture

High-level system design of jiq.

## System Overview

jiq = TUI for real-time jq query execution with VIM editing and autocomplete.

**Core components:**
1. Input handling (JSON from file/stdin)
2. Query editing (VIM-style with tui-textarea)
3. Query execution (external jq subprocess)
4. Autocomplete (context-aware suggestions)
5. UI rendering (Ratatui two-pane layout)

**Tech stack:**
- Ratatui 0.29 (TUI framework)
- Crossterm 0.28 (terminal manipulation)
- tui-textarea 0.7 (text editor widget)
- serde_json 1.0 (JSON parsing)
- External jq binary (query execution)

## Component Diagram

```
main.rs (entry, CLI, event loop)
    │
    ├──→ InputReader  ──→ JSON ──┐
    │                            │
    └──→ JqExecutor   ──→ Results┤
                                 │
                                 ▼
                              App (state)
                              │  │  │
                ┌─────────────┼──┼──┴──────────┐
                ▼             ▼  ▼             ▼
            Editor      Autocomplete      QueryResult
            (VIM        (suggestions)     (jq output)
             modes)
                │             │              │
                └─────────────┴──────────────┘
                              │
                         Rendering
                              │
                        ┌─────┴─────┐
                        ▼           ▼
                    Ratatui    Crossterm
```

## UI Layout

```
┌─────────────── Terminal ───────────────┐
│ ┌─ Input (30%) ───────────[INSERT]──┐ │
│ │ .users[] | select(.active)        │ │
│ │  ┌─ Autocomplete ────┐            │ │
│ │  │ ► select   [fn]   │            │ │
│ │  │   .status  [field]│            │ │
│ │  └───────────────────┘            │ │
│ └───────────────────────────────────┘ │
│ ┌─ Results (70%) ──────────────────┐ │
│ │ [                                │ │
│ │   { "name": "Alice",            │ │
│ │     "active": true }            │ │
│ │ ]                                │ │
│ └───────────────────────────────────┘ │
│ Tab: Focus | Enter: Output           │
└───────────────────────────────────────┘
```

## Module Structure

```
src/
├── main.rs           # Entry point, CLI, event loop
├── error.rs          # JiqError enum
├── app/
│   ├── state.rs      # App struct, Focus, OutputMode
│   ├── events.rs     # Event dispatch by focus/mode
│   └── render.rs     # UI rendering, autocomplete popup
├── autocomplete/
│   ├── state.rs      # Suggestions, selection
│   ├── context.rs    # Field vs function detection
│   ├── jq_functions.rs   # LazyLock<Vec<Suggestion>>
│   └── json_analyzer.rs  # Extract JSON fields
├── editor/
│   └── mode.rs       # EditorMode enum
├── input/
│   └── reader.rs     # Read JSON from file/stdin
└── query/
    └── executor.rs   # Spawn jq subprocess
```

## Data Flow

### Application Lifecycle

```
1. Parse CLI args, validate jq exists
2. Read JSON input (file or stdin)
3. Initialize terminal (Ratatui)
4. Create App with JSON
5. Event loop:
   - Render UI (draw frame)
   - Poll for keyboard event
   - Dispatch to handler
   - Update state
   - Check should_quit
6. Restore terminal
7. Output results (if requested)
```

### Query Execution (per keystroke)

```
User types → INSERT mode handler
                  │
    ┌─────────────┴────────────┐
    ▼                          ▼
textarea.input(key)    update_autocomplete()
    │                          │
content_changed?        analyze_context()
    │                          │
    ▼                    ┌─────┴──────┐
execute_query()          ▼            ▼
    │              Field context  Function context
    ▼                   │              │
Spawn jq subprocess     ▼              ▼
    │              JSON fields    jq built-ins
    ├─ success         │              │
    │  └─> Display     └──────┬───────┘
    └─ error                  │
       └─> Show in red   Filter by prefix
                              │
                         Update popup
```

## Key Design Decisions

### External jq Binary

**Choice:** Spawn external jq process instead of native Rust implementation.

**Rationale:**
- Correctness (15+ years of development)
- Zero maintenance for query logic
- All jq features supported
- Users already have jq (prerequisite)

**Trade-off:** ~50ms subprocess overhead (acceptable for interactive use)

### Real-Time Execution

**Choice:** Execute jq on every keystroke.

**Rationale:**
- Real-time feedback is core value proposition
- User expectation from README
- Latency acceptable (<100ms)

**Could add:** Debouncing if large files cause issues (not needed yet)

### LazyLock for Static Data

**Choice:** Use LazyLock<Vec<Suggestion>> for jq built-ins.

**Rationale:**
- Built once at first access
- Zero runtime cost after initialization
- 90% reduction in allocations

### tui-textarea Delegation

**Choice:** Use tui-textarea for all text editing, only add VIM layer.

**Rationale:**
- Don't reinvent text editing
- Well-tested cursor/undo/selection
- Simple integration
- Independent upgrades

### Two-Pane 30/70 Split

**Choice:** Input 30%, Results 70%.

**Rationale:**
- Queries usually short
- Maximize result visibility
- Standard in similar tools

## Performance Characteristics

**Event loop:** ~60 FPS rendering
**Query execution:** ~50-100ms per keystroke
  - Process spawn: ~20-50ms
  - jq execution: ~10-50ms (query dependent)
**Autocomplete:** <1ms (filters static data)
**Rendering:** ~10-15ms (Ratatui double-buffering)

**Bottleneck:** jq subprocess spawn (acceptable)

## Extension Points

**Adding autocomplete suggestion types:**
1. Add variant to `SuggestionType` enum
2. Add variant to `SuggestionContext` enum
3. Implement detection in `context::analyze_context()`
4. Add color in `render.rs`

**Adding VIM commands:**
1. Add handler in appropriate mode function in `events.rs`
2. Delegate to tui-textarea methods
3. Call `execute_query()` if content changes

**Adding output formats:**
1. Add variant to `OutputMode` enum
2. Handle in `main.rs::handle_output()`

## See Also

- [Event System](subsystems/EVENT_SYSTEM.md) - Detailed event handling
- [VIM Editor](subsystems/VIM_EDITOR.md) - Modal editing system
- [Autocomplete](subsystems/AUTOCOMPLETE.md) - Suggestion system
- [Query Execution](subsystems/QUERY_EXECUTION.md) - jq subprocess
- [Rendering](subsystems/RENDERING.md) - UI layout and rendering
