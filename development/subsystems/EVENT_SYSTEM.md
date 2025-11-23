# Event System Architecture

Deep dive into jiq's event handling, dispatching, and keyboard input management.

## Overview

The event system is the core coordination layer that routes keyboard input to appropriate handlers based on:
- Current **focus** (Input field vs Results pane)
- Current **editor mode** (INSERT/NORMAL/OPERATOR)
- **Global shortcuts** that work anywhere

**Location:** `src/app/events.rs` - 413 lines, 54 test cases

## Event Flow Architecture

```
Terminal Keyboard Event
        │
        ▼
┌───────────────────────┐
│  handle_events()      │  Poll crossterm for events
│  - Read from terminal │
│  - Filter KeyPress    │
└───────────┬───────────┘
            │
            ▼
┌───────────────────────┐
│  handle_key_event()   │  Route to handler
└───────────┬───────────┘
            │
            ▼
┌───────────────────────┐
│ handle_global_keys()  │  Try global shortcuts first
│ Returns: bool         │  (Ctrl+C, Tab, Enter, q, etc.)
└───────┬───────────────┘
        │
        ├─ true ─────────► Event handled, done
        │
        └─ false ────────► Delegate to focused pane
                            │
            ┌───────────────┴────────────────┐
            │                                │
            ▼                                ▼
  ┌─────────────────┐            ┌────────────────────┐
  │ Input Field     │            │  Results Pane      │
  │  Focused        │            │  Focused           │
  └────────┬────────┘            └────────┬───────────┘
           │                              │
           ▼                              ▼
  ┌─────────────────┐            ┌────────────────────┐
  │ ESC handling    │            │ Scroll commands    │
  │ Autocomplete    │            │ - j/k (1 line)     │
  │ navigation      │            │ - J/K (10 lines)   │
  └────────┬────────┘            │ - g/G (top/bottom) │
           │                     │ - Ctrl+d/u (½ page)│
           ▼                     └────────────────────┘
  ┌─────────────────┐
  │ Mode dispatch   │
  └────────┬────────┘
           │
  ┌────────┴─────────────────────────┐
  │                                  │
  ▼                                  ▼
┌─────────────┐        ┌─────────────┐        ┌──────────────┐
│ INSERT mode │        │ NORMAL mode │        │ OPERATOR mode│
│             │        │             │        │              │
│ - Type text │        │ - h/l/0/$   │        │ - Await      │
│ - Execute   │        │ - w/b/e     │        │   motion     │
│   query     │        │ - i/a/I/A   │        │ - Execute    │
│ - Update    │        │ - x/X       │        │   d/c        │
│   autocomplete       │ - d/c ops   │        │   operation  │
└─────────────┘        │ - u/Ctrl+r  │        └──────────────┘
                       │ - D/C       │
                       └─────────────┘
```

## Key Event Dispatch

### Three-Layer Priority

```rust
fn handle_key_event(&mut self, key: KeyEvent) {
    // Priority 1: Global keys (work anywhere)
    if self.handle_global_keys(key) {
        return; // Handled
    }

    // Priority 2: Focus-specific
    match self.focus {
        Focus::InputField => self.handle_input_field_key(key),
        Focus::ResultsPane => self.handle_results_pane_key(key),
    }
}
```

**Why this order?**
- Global keys (Ctrl+C, Enter, q) should *always* work
- Prevents focus from blocking critical actions
- Consistent UX regardless of where cursor is

### Global Keys Handler

**Location:** `events.rs:36-91`

```rust
fn handle_global_keys(&mut self, key: KeyEvent) -> bool {
    match key {
        Ctrl+C      → quit
        q           → quit
        Enter       → quit with Results output
        Shift+Enter → quit with Query output
        Shift+Tab   → toggle focus
        Tab         → autocomplete (if visible in input)
        _           → false (not handled)
    }
}
```

**Design decision:** Tab is special
- Only works when autocomplete visible in input field
- Returns `false` if not applicable
- Prevents interfering with tui-textarea's Tab handling

## Input Field Event Handling

### Special Keys (Pre-Mode)

Before mode dispatch, these keys are always checked:

```rust
fn handle_input_field_key(&mut self, key: KeyEvent) {
    // 1. ESC - context-aware
    if key.code == KeyCode::Esc {
        if self.autocomplete.is_visible() {
            self.autocomplete.hide(); // Close popup first
            return;
        }
        self.editor_mode = EditorMode::Normal; // Then switch mode
        return;
    }

    // 2. Autocomplete navigation (INSERT mode only)
    if self.editor_mode == EditorMode::Insert
       && self.autocomplete.is_visible() {
        match key.code {
            KeyCode::Down => self.autocomplete.select_next(),
            KeyCode::Up => self.autocomplete.select_previous(),
            _ => {}
        }
    }

    // 3. Dispatch by mode
    match self.editor_mode {
        EditorMode::Insert => ...
        EditorMode::Normal => ...
        EditorMode::Operator(_) => ...
    }
}
```

**ESC priority:**
1. Close autocomplete if open
2. Switch to NORMAL mode if no autocomplete

This prevents frustrating UX where ESC doesn't close the popup.

### INSERT Mode Handling

**Location:** `events.rs:129-149`

```rust
fn handle_insert_mode_key(&mut self, key: KeyEvent) {
    // Delegate to tui-textarea for actual text editing
    let content_changed = self.textarea.input(key);

    if content_changed {
        // Re-execute query
        let query = self.textarea.lines()[0].as_ref();
        self.query_result = self.executor.execute(query);

        // Cache successful results
        if let Ok(result) = &self.query_result {
            self.last_successful_result = Some(result.clone());
        }

        // Reset scroll position
        self.results_scroll = 0;
    }

    // Always update autocomplete (even if no change)
    self.update_autocomplete();
}
```

**Key insight:** Query executes on *every keystroke*
- Real-time feedback as you type
- Acceptable latency for jq subprocess spawn
- Could add debouncing if needed for large files

**Autocomplete updates every time:**
- Detects context changes (field vs function)
- Filters suggestions based on new prefix
- Updates even if content didn't change (cursor moved)

### NORMAL Mode Handling

**Location:** `events.rs:152-256`

VIM navigation and commands:

| Category | Keys | Action |
|----------|------|--------|
| **Cursor movement** | h, l, ←, → | Move left/right |
| **Line extent** | 0, $, Home, End | Jump to start/end |
| **Word movement** | w, b, e | Forward/back/end of word |
| **Insert mode** | i, a, I, A | Enter INSERT at various positions |
| **Delete** | x, X, D | Delete char/char before/to end |
| **Change** | C | Delete to end + INSERT |
| **Operators** | d, c | Enter OPERATOR mode |
| **Undo/Redo** | u, Ctrl+r | Undo/redo changes |

**tui-textarea integration:**
```rust
// All cursor movement delegates to tui-textarea
KeyCode::Char('h') => {
    self.textarea.move_cursor(CursorMove::Back);
}

// Delete operations update query immediately
KeyCode::Char('x') => {
    self.textarea.delete_next_char();
    self.execute_query(); // Re-run jq
}
```

**Why re-execute on every delete?**
- Maintains real-time feedback consistency
- User sees immediate results
- Matches INSERT mode behavior

### OPERATOR Mode Handling

**Location:** `events.rs:259-343`

Handles `d` (delete) and `c` (change) operators with motions.

**State machine:**
```
NORMAL mode
    │
    │ Press 'd' or 'c'
    ▼
OPERATOR('d') or OPERATOR('c')
    │
    │ Start visual selection
    │ textarea.start_selection()
    │
    ├─ Press same operator (dd, cc)
    │  └─> Delete entire line
    │      → NORMAL (for d) or INSERT (for c)
    │
    ├─ Press valid motion (w, b, e, $, 0, h, l)
    │  └─> Move cursor (extends selection)
    │      Execute: textarea.cut()
    │      → NORMAL (for d) or INSERT (for c)
    │
    └─ Press invalid key
       └─> Cancel: textarea.cancel_selection()
           → NORMAL
```

**Implementation:**
```rust
fn handle_operator_mode_key(&mut self, key: KeyEvent) {
    let operator = match self.editor_mode {
        EditorMode::Operator(op) => op,
        _ => return,
    };

    // Check for double operator (dd, cc)
    if key.code == KeyCode::Char(operator) {
        self.textarea.delete_line_by_head();
        self.textarea.delete_line_by_end();
        self.editor_mode = if operator == 'c' {
            EditorMode::Insert
        } else {
            EditorMode::Normal
        };
        self.execute_query();
        return;
    }

    // Apply motion
    let motion_applied = match key.code {
        KeyCode::Char('w') => {
            self.textarea.move_cursor(CursorMove::WordForward);
            true
        }
        // ... other motions
        _ => false,
    };

    if motion_applied {
        // Execute operator
        match operator {
            'd' => {
                self.textarea.cut();
                self.editor_mode = EditorMode::Normal;
            }
            'c' => {
                self.textarea.cut();
                self.editor_mode = EditorMode::Insert;
            }
            _ => {
                self.textarea.cancel_selection();
                self.editor_mode = EditorMode::Normal;
            }
        }
        self.execute_query();
    } else {
        // Invalid motion - cancel
        self.textarea.cancel_selection();
        self.editor_mode = EditorMode::Normal;
    }
}
```

**Key insight:** Visual selection
- `start_selection()` marks selection start
- Cursor movement extends selection
- `cut()` deletes selected text
- `cancel_selection()` aborts operation

## Results Pane Event Handling

**Location:** `events.rs:359-411`

Much simpler - just scrolling:

```rust
fn handle_results_pane_key(&mut self, key: KeyEvent) {
    match key.code {
        // 1 line scroll
        Up | k    → results_scroll -= 1
        Down | j  → results_scroll += 1

        // 10 line scroll
        K         → results_scroll -= 10
        J         → results_scroll += 10

        // Jump
        Home | g  → results_scroll = 0
        G         → results_scroll = max_scroll()

        // Half page
        PageUp | Ctrl+u    → results_scroll -= viewport_height / 2
        PageDown | Ctrl+d  → results_scroll += viewport_height / 2

        _ → ignore
    }
}
```

**Using `saturating_sub/add`:**
```rust
self.results_scroll = self.results_scroll.saturating_sub(1);
```
- Never goes below 0
- No need for bounds checking
- Elegant Rust idiom

## Autocomplete Integration

### Tab Key Handling

Tab is a global key, but context-aware:

```rust
// In handle_global_keys()
if key.code == KeyCode::Tab && !key.modifiers.contains(KeyModifiers::CONTROL) {
    if self.focus == Focus::InputField && self.autocomplete.is_visible() {
        if let Some(suggestion) = self.autocomplete.selected() {
            let text = suggestion.text.clone();
            self.insert_autocomplete_suggestion(&text);
        }
        return true; // Handled
    }
    return false; // Not handled - pass to textarea
}
```

**Why return false when not visible?**
- Allows tui-textarea to handle Tab normally
- Flexible: could indent in future
- Non-intrusive design

### Autocomplete Update Trigger

Updated after every INSERT mode keystroke:

```rust
fn handle_insert_mode_key(&mut self, key: KeyEvent) {
    let content_changed = self.textarea.input(key);

    if content_changed {
        // ... execute query ...
    }

    // Always update (even if content didn't change)
    self.update_autocomplete();
}
```

Why always update?
- Cursor position might have changed
- Context might have changed (`.field|` → function context)
- Cost is minimal (just filters static data)

## Query Execution

### Execution Triggers

Query re-execution happens on:

1. **Content change in INSERT mode** (events.rs:134)
2. **Any edit in NORMAL mode** (x, X, D, C)
3. **Any operator completion** (dw, cc, etc.)

**Common pattern:**
```rust
self.execute_query();

// Which does:
fn execute_query(&mut self) {
    let query = self.textarea.lines()[0].as_ref();
    self.query_result = self.executor.execute(query);

    if let Ok(result) = &self.query_result {
        self.last_successful_result = Some(result.clone());
    }

    self.results_scroll = 0; // Reset scroll
}
```

### Result Caching

```rust
// Cache successful results
if let Ok(result) = &self.query_result {
    self.last_successful_result = Some(result.clone());
}
```

**Why cache?**
- Could show last valid result when query is invalid
- Currently unused, but prepared for future enhancement
- Minimal cost (String clone)

### Scroll Reset

Every query execution resets scroll to top:

```rust
self.results_scroll = 0;
```

**Rationale:**
- New results likely different size
- User wants to see beginning of output
- Prevents confusing UX (scrolled past new results)

## Testing Strategy

### Test Categories (54 tests total)

1. **VIM Operator Tests** (17 tests)
   - dw, db, de, d$, d0, dd
   - cw, cb, ce, cc
   - Invalid motions, cancellation
   - Edge cases

2. **Mode Transition Tests** (8 tests)
   - INSERT ↔ NORMAL
   - NORMAL → OPERATOR
   - Operator completion → mode

3. **Simple VIM Commands** (9 tests)
   - x, X, D, C
   - u, Ctrl+r

4. **VIM Navigation** (8 tests)
   - h, l, 0, $, w, b, e

5. **Autocomplete Interaction** (10 tests)
   - ESC closes popup vs switches mode
   - Arrow navigation
   - Tab acceptance
   - Mode restrictions

6. **Results Scrolling** (14 tests)
   - j/k, J/K, g/G
   - Page Up/Down, Ctrl+u/d
   - Bounds checking

7. **Global Keys** (10 tests)
   - Ctrl+C, q, Enter, Shift+Enter
   - Focus switching
   - Global key priority

### Testing Patterns

**Helper functions:**
```rust
fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

fn key_with_mods(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

fn app_with_query(query: &str) -> App {
    let mut app = App::new(TEST_JSON.to_string());
    app.textarea.insert_str(query);
    app
}
```

**Test structure:**
```rust
#[test]
fn test_operator_dw_deletes_word_from_start() {
    // Arrange
    let mut app = app_with_query(".name.first");
    app.textarea.move_cursor(CursorMove::Head);
    app.editor_mode = EditorMode::Normal;

    // Act
    app.handle_key_event(key(KeyCode::Char('d')));
    app.handle_key_event(key(KeyCode::Char('w')));

    // Assert
    assert!(app.query().contains("first"));
    assert_eq!(app.editor_mode, EditorMode::Normal);
}
```

## Design Decisions

### Why Global Keys Have Priority?

**Decision:** Check global keys before focus-specific keys.

**Rationale:**
- User should *always* be able to quit (Ctrl+C, q)
- Enter should *always* output results
- Focus shouldn't trap user
- More predictable UX

**Alternative considered:** Focus-first routing
- Rejected: Could block critical actions
- Example: If results pane consumed 'q', can't quit

### Why ESC Closes Autocomplete First?

**Decision:** ESC closes autocomplete popup before switching to NORMAL mode.

**Rationale:**
- User wants to dismiss popup
- Switching mode is secondary action
- Matches IDE behavior (VSCode, etc.)

**Implementation:**
```rust
if key.code == KeyCode::Esc {
    if self.autocomplete.is_visible() {
        self.autocomplete.hide();
        return; // Don't switch mode
    }
    self.editor_mode = EditorMode::Normal;
}
```

### Why Execute Query on Every Keystroke?

**Decision:** Re-execute jq on every content change in INSERT mode.

**Rationale:**
- Real-time feedback is jiq's core value proposition
- jq execution is fast (<100ms typically)
- Matches user expectation from README
- Could debounce if needed (not needed yet)

**Alternative considered:** Debounce (wait 100ms after typing stops)
- Rejected: Delays feedback
- Could add as option if large files cause issues

### Why Saturating Arithmetic for Scroll?

**Decision:** Use `saturating_sub()` and `saturating_add()` for scroll position.

**Rationale:**
- Prevents underflow/overflow
- Cleaner than manual bounds checking
- Idiomatic Rust
- Zero runtime cost (optimized to same assembly)

**Before:**
```rust
// Manual bounds checking
if self.results_scroll > 0 {
    self.results_scroll -= 1;
}
```

**After:**
```rust
// Saturating arithmetic
self.results_scroll = self.results_scroll.saturating_sub(1);
```

## Performance Considerations

### Hot Paths

**Every keystroke:**
1. Poll event (crossterm)
2. Dispatch (3 function calls max)
3. Execute query (jq subprocess)
4. Update autocomplete (filter static data)
5. Render UI (Ratatui)

**Bottleneck:** jq subprocess spawn
- ~50-100ms per execution
- Acceptable for interactive use
- Future: Could debounce for very large files

### Memory Allocation

**Minimal allocations in event path:**
- Query string extraction (zero-copy slice)
- Result caching (String clone only on success)
- Autocomplete suggestions (filters Vec, doesn't rebuild)

**No heap allocation:**
- Cursor movement
- Mode transitions
- Scroll updates
- Focus changes

## Future Enhancements

### 1. Debounced Query Execution

```rust
struct QueryDebouncer {
    last_query: String,
    timer: Option<Instant>,
    delay: Duration,
}

impl QueryDebouncer {
    fn should_execute(&mut self, query: &str) -> bool {
        if query != self.last_query {
            self.timer = Some(Instant::now());
            self.last_query = query.to_string();
            false // Wait for delay
        } else if let Some(start) = self.timer {
            start.elapsed() >= self.delay // Execute after delay
        } else {
            false
        }
    }
}
```

### 2. Async Query Execution

```rust
// Don't block event loop while jq runs
async fn execute_query_async(&mut self, query: String) {
    let result = tokio::spawn(async move {
        JqExecutor::execute(&query)
    }).await;

    self.query_result = result;
}
```

### 3. Multi-line Queries

Currently limited to single line. Could support:
- Ctrl+Enter for newline
- Multi-line text area
- More complex jq queries

### 4. Macro Recording

VIM-style macro recording:
```rust
// Press 'q' + register to start recording
// Replay with '@' + register
struct MacroRecorder {
    recording: Option<char>,
    macros: HashMap<char, Vec<KeyEvent>>,
}
```

## Related Code

- **Editor modes:** `src/editor/mode.rs`
- **App state:** `src/app/state.rs`
- **Autocomplete:** `src/autocomplete/state.rs`
- **Rendering:** `src/app/render.rs`

---

**Maintainer Notes:**
- Event system is well-tested (54 test cases)
- Adding new global keys: Update `handle_global_keys()`
- Adding new VIM commands: Update appropriate mode handler
- ESC behavior is nuanced - check tests before modifying
