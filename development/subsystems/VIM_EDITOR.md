# VIM Modal Editing System

Deep dive into jiq's VIM-style modal editing implementation using tui-textarea.

## Overview

jiq implements a subset of VIM modal editing for the query input field, providing familiar keybindings for VIM users while remaining accessible for beginners (defaults to INSERT mode).

**Location:**
- Mode definitions: `src/editor/mode.rs` (46 lines, simple enum)
- Mode handling: `src/app/events.rs` (integrated with event system)

**Key Design:** Thin wrapper around tui-textarea
- Delegates actual text editing to tui-textarea
- Adds VIM modal behavior on top
- Minimal custom code (mode tracking + dispatch)

## Mode System

### EditorMode Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Insert,          // Regular typing
    Normal,          // VIM navigation
    Operator(char),  // Waiting for motion after 'd' or 'c'
}
```

**Default:** INSERT mode
- Beginners can use jiq without knowing VIM
- VIM users press ESC to enter NORMAL mode
- Best of both worlds

### Mode State Machine

```
Application Start
        │
        ▼
    ┌────────┐
    │ INSERT │◄────────────────────────────────┐
    └────┬───┘                                 │
         │                                     │
         │ ESC (no autocomplete)               │
         ▼                                     │
    ┌────────┐                                 │
    │ NORMAL │                                 │
    └────┬───┘                                 │
         │                                     │
         ├─ i, a, I, A ─────────────────────────┘
         │
         ├─ x, X, D ──► Execute ──► Stay NORMAL
         │
         ├─ C ──► Execute ──► INSERT
         │
         ├─ d ──► OPERATOR('d')
         │         │
         │         ├─ Valid motion ──► Execute ──► NORMAL
         │         ├─ dd (same key) ──► Execute ──► NORMAL
         │         └─ Invalid/ESC ──► Cancel ──► NORMAL
         │
         └─ c ──► OPERATOR('c')
                   │
                   ├─ Valid motion ──► Execute ──► INSERT
                   ├─ cc (same key) ──► Execute ──► INSERT
                   └─ Invalid/ESC ──► Cancel ──► NORMAL
```

### Mode Display

```rust
impl EditorMode {
    pub fn display(&self) -> String {
        match self {
            EditorMode::Insert => "INSERT".to_string(),
            EditorMode::Normal => "NORMAL".to_string(),
            EditorMode::Operator(op) => format!("OPERATOR({})", op),
        }
    }
}
```

**Usage:** Shown in input field border title
- `[INSERT MODE]` - Cyan border
- `[NORMAL MODE]` - Yellow border
- `[OPERATOR(d)]` - Green border

## INSERT Mode

**Entry:** Application start, or from NORMAL via i/a/I/A/c

**Behavior:** Standard text editing
- All printable characters insert at cursor
- Backspace/Delete work normally
- Arrow keys move cursor
- Delegated entirely to tui-textarea

**Special keys:**
- ESC → NORMAL mode (unless autocomplete visible)
- Tab → Accept autocomplete (if visible)
- Up/Down → Navigate autocomplete (if visible)

**Query execution:** Every keystroke that changes content
```rust
let content_changed = self.textarea.input(key);
if content_changed {
    self.execute_query();
    self.results_scroll = 0;
}
self.update_autocomplete();
```

## NORMAL Mode

**Entry:** ESC from INSERT mode

**Purpose:** VIM navigation and commands without inserting text

### Navigation Commands

| Key | tui-textarea Action | Description |
|-----|---------------------|-------------|
| h, ← | `CursorMove::Back` | Move cursor left |
| l, → | `CursorMove::Forward` | Move cursor right |
| 0, Home | `CursorMove::Head` | Jump to line start |
| $, End | `CursorMove::End` | Jump to line end |
| w | `CursorMove::WordForward` | Next word start |
| b | `CursorMove::WordBack` | Previous word start |
| e | `CursorMove::WordEnd` | Current/next word end |

**Implementation pattern:**
```rust
KeyCode::Char('h') => {
    self.textarea.move_cursor(CursorMove::Back);
}
```

No custom cursor logic - delegates to tui-textarea's well-tested cursor movement.

### Insert Mode Entry Commands

| Key | Action | Cursor Movement |
|-----|--------|-----------------|
| i | Enter INSERT | None (insert at cursor) |
| a | Enter INSERT | Forward 1 (append after cursor) |
| I | Enter INSERT | To line start |
| A | Enter INSERT | To line end |

**Implementation:**
```rust
KeyCode::Char('i') => {
    self.editor_mode = EditorMode::Insert;
}

KeyCode::Char('a') => {
    self.textarea.move_cursor(CursorMove::Forward);
    self.editor_mode = EditorMode::Insert;
}
```

### Simple Delete Commands

| Key | Action | Mode After |
|-----|--------|------------|
| x | Delete char at cursor | NORMAL |
| X | Delete char before cursor | NORMAL |
| D | Delete to end of line | NORMAL |

```rust
KeyCode::Char('x') => {
    self.textarea.delete_next_char();
    self.execute_query();
}

KeyCode::Char('D') => {
    self.textarea.delete_line_by_end();
    self.execute_query();
}
```

### Change Command

| Key | Action | Mode After |
|-----|--------|------------|
| C | Delete to end + INSERT | INSERT |

```rust
KeyCode::Char('C') => {
    self.textarea.delete_line_by_end();
    self.textarea.cancel_selection();
    self.editor_mode = EditorMode::Insert;
    self.execute_query();
}
```

### Undo/Redo

| Key | Action | tui-textarea Method |
|-----|--------|---------------------|
| u | Undo | `textarea.undo()` |
| Ctrl+r | Redo | `textarea.redo()` |

**Note:** Undo/redo provided by tui-textarea
- No custom undo stack needed
- Handles text history automatically
- Works across mode changes

## OPERATOR Mode

**Entry:** Press 'd' or 'c' in NORMAL mode

**Purpose:** Compose operator + motion (VIM-style)
- `dw` = delete word
- `d$` = delete to end
- `dd` = delete line
- `cw` = change word
- etc.

### Visual Selection Mechanism

```rust
// When entering OPERATOR mode
self.textarea.start_selection();

// User presses motion key (e.g., 'w')
self.textarea.move_cursor(CursorMove::WordForward);
// ^ This extends the selection

// Execute operator
self.textarea.cut(); // Deletes selected text
```

**tui-textarea selection API:**
- `start_selection()` - Mark selection start at cursor
- Cursor movement extends selection
- `cut()` - Delete selected text
- `cancel_selection()` - Abort operation

### Operator Execution

```rust
match operator {
    'd' => {
        // Delete - cut and stay in NORMAL
        self.textarea.cut();
        self.editor_mode = EditorMode::Normal;
    }
    'c' => {
        // Change - cut and enter INSERT
        self.textarea.cut();
        self.editor_mode = EditorMode::Insert;
    }
    _ => {
        // Unknown operator - cancel
        self.textarea.cancel_selection();
        self.editor_mode = EditorMode::Normal;
    }
}
self.execute_query();
```

### Double Operator (dd, cc)

Special case: Pressing operator key twice deletes entire line.

```rust
if key.code == KeyCode::Char(operator) {
    // dd or cc - delete entire line
    self.textarea.delete_line_by_head();
    self.textarea.delete_line_by_end();
    self.editor_mode = if operator == 'c' {
        EditorMode::Insert  // cc → INSERT
    } else {
        EditorMode::Normal  // dd → NORMAL
    };
    self.execute_query();
    return;
}
```

**Why two method calls?**
```rust
self.textarea.delete_line_by_head(); // Delete from start to cursor
self.textarea.delete_line_by_end();  // Delete from cursor to end
```
Result: Entire line deleted regardless of cursor position.

### Supported Motions

| Motion | tui-textarea | Description |
|--------|--------------|-------------|
| w | `WordForward` | To next word start |
| b | `WordBack` | To previous word start |
| e | `WordEnd` + `Forward` | To word end (inclusive) |
| $ | `End` | To line end |
| 0 | `Head` | To line start |
| h | `Back` | One char left |
| l | `Forward` | One char right |

**Why `Forward` after `e`?**
```rust
KeyCode::Char('e') => {
    self.textarea.move_cursor(CursorMove::WordEnd);
    self.textarea.move_cursor(CursorMove::Forward); // Include char at cursor
    true
}
```
- `WordEnd` positions cursor ON last char
- `Forward` includes it in selection
- Matches VIM behavior

### Invalid Motion Handling

```rust
let motion_applied = match key.code {
    KeyCode::Char('w') => { /* ... */ true }
    // ...
    _ => false,  // Unknown motion
};

if !motion_applied {
    // Cancel operator and return to NORMAL
    self.textarea.cancel_selection();
    self.editor_mode = EditorMode::Normal;
}
```

**No error message** - just cancels operation
- Matches VIM behavior
- User can try again
- Doesn't disrupt flow

## Integration with tui-textarea

### Delegation Strategy

jiq does NOT reimplement text editing. Instead:

**tui-textarea provides:**
- Text buffer management
- Cursor positioning
- Selection handling
- Undo/redo stack
- Word boundary detection
- Character deletion

**jiq adds:**
- Mode state (INSERT/NORMAL/OPERATOR)
- Mode-based key routing
- Operator+motion composition
- Query execution triggers

### Key Methods Used

```rust
// Cursor movement
textarea.move_cursor(CursorMove::Back)
textarea.move_cursor(CursorMove::Forward)
textarea.move_cursor(CursorMove::Head)
textarea.move_cursor(CursorMove::End)
textarea.move_cursor(CursorMove::WordForward)
textarea.move_cursor(CursorMove::WordBack)
textarea.move_cursor(CursorMove::WordEnd)

// Selection
textarea.start_selection()
textarea.cancel_selection()
textarea.cut()

// Editing
textarea.input(key)           // INSERT mode text input
textarea.delete_next_char()   // 'x' command
textarea.delete_char()        // 'X' command
textarea.delete_line_by_head()
textarea.delete_line_by_end()

// History
textarea.undo()
textarea.redo()

// Query
textarea.lines()[0]    // Get current query text
textarea.cursor()      // Get cursor position
```

### Why This Works

**Separation of concerns:**
- tui-textarea = text editing engine
- jiq = VIM behavior layer

**Benefits:**
- Don't reinvent text editing
- tui-textarea is well-tested
- Easy to maintain
- Can upgrade tui-textarea independently

## Visual Indicators

### Border Colors

Mode is indicated by input field border color:

```rust
// In render.rs
let border_color = match self.editor_mode {
    EditorMode::Insert => Color::Cyan,
    EditorMode::Normal => Color::Yellow,
    EditorMode::Operator(_) => Color::Green,
};
```

**Color choices:**
- **Cyan (INSERT)** - Default, calming
- **Yellow (NORMAL)** - Warning/attention
- **Green (OPERATOR)** - Awaiting completion

### Mode Label

Title includes mode name:

```
INSERT mode: [INPUT FIELD - INSERT MODE]
NORMAL mode: [INPUT FIELD - NORMAL MODE - Press i to edit]
OPERATOR:    [INPUT FIELD - OPERATOR(d)]
```

**Helper text in NORMAL:**
- Reminds users how to start editing
- Prevents confusion for beginners
- Removed in INSERT/OPERATOR (unnecessary)

## Design Decisions

### Why Default to INSERT Mode?

**Decision:** `EditorMode::default() = Insert`

**Rationale:**
- Accessible to non-VIM users
- Can start typing immediately
- VIM users can ESC to NORMAL
- Inclusive design

**Alternative considered:** Default to NORMAL
- Rejected: Too VIM-specific
- Would confuse beginners

### Why Operator('c') Enters INSERT After Cut?

**Decision:** Change operator enters INSERT mode after executing.

**Rationale:**
- Matches VIM semantics
- "Change" means "replace with new text"
- Delete + INSERT is the expected workflow

**Implementation:**
```rust
'c' => {
    self.textarea.cut();
    self.editor_mode = EditorMode::Insert;  // Ready to type
}
```

### Why Not Implement More VIM Commands?

**Current subset:**
- Basic navigation (h/l/w/b/e/0/$)
- Insert mode entry (i/a/I/A)
- Delete (x/X/d/D)
- Change (c/C)
- Undo/redo (u/Ctrl+r)

**Not implemented:**
- Visual mode (v/V)
- Search (/, ?, n, N)
- Registers ("/yank)
- Repeat (.)
- Marks (m, ')
- Many more...

**Rationale:**
- Single-line input field (limited use case)
- Diminishing returns (80/20 rule)
- Complexity vs benefit
- Can add later if needed

**Most valuable VIM features covered:**
- Modal editing
- Efficient navigation
- Operator composition
- Undo/redo

### Why Execute Query After Every Edit?

**Decision:** Re-execute jq after every text change, even in NORMAL mode.

**Rationale:**
- Consistency with INSERT mode
- Real-time feedback is core value prop
- User expects to see results immediately
- Acceptable performance (<100ms)

**Commands that trigger execution:**
- x, X (delete char)
- D, C (delete/change to end)
- dd, cc (delete line)
- dw, db, de, etc. (operator+motion)
- u, Ctrl+r (undo/redo)

## Testing

### Mode Transition Tests

```rust
#[test]
fn test_escape_from_insert_to_normal() {
    let mut app = app_with_query(".name");
    app.editor_mode = EditorMode::Insert;

    app.handle_key_event(key(KeyCode::Esc));

    assert_eq!(app.editor_mode, EditorMode::Normal);
}

#[test]
fn test_i_enters_insert_mode_at_cursor() {
    let mut app = app_with_query(".name");
    app.editor_mode = EditorMode::Normal;
    let cursor_before = app.textarea.cursor();

    app.handle_key_event(key(KeyCode::Char('i')));

    assert_eq!(app.editor_mode, EditorMode::Insert);
    assert_eq!(app.textarea.cursor(), cursor_before);
}
```

### Operator Tests

```rust
#[test]
fn test_operator_dw_deletes_word() {
    let mut app = app_with_query(".name.first");
    app.editor_mode = EditorMode::Normal;

    // Enter operator mode
    app.handle_key_event(key(KeyCode::Char('d')));
    assert!(matches!(app.editor_mode, EditorMode::Operator('d')));

    // Execute motion
    app.handle_key_event(key(KeyCode::Char('w')));

    assert_eq!(app.editor_mode, EditorMode::Normal);
    assert!(app.query().contains("first")); // Word deleted
}
```

### Edge Cases

```rust
#[test]
fn test_operator_invalid_motion_cancels() {
    let mut app = app_with_query(".name");
    app.editor_mode = EditorMode::Normal;
    let original_query = app.query().to_string();

    app.handle_key_event(key(KeyCode::Char('d')));
    app.handle_key_event(key(KeyCode::Char('z'))); // Invalid

    assert_eq!(app.editor_mode, EditorMode::Normal);
    assert_eq!(app.query(), original_query); // Unchanged
}
```

## Future Enhancements

### 1. Visual Mode (v/V)

```rust
pub enum EditorMode {
    Insert,
    Normal,
    Operator(char),
    Visual,      // Character-wise visual
    VisualLine,  // Line-wise visual
}
```

Could use tui-textarea's selection for highlighting.

### 2. Repeat Command (.)

```rust
struct RepeatInfo {
    last_command: Vec<KeyEvent>,
    last_operator: Option<(char, CursorMove)>,
}
```

Record last change and replay on '.'.

### 3. Multi-line Editing

Currently single-line only. Could support:
- Multiple query lines
- Ctrl+Enter for newline
- More complex jq queries

Requires switching from single-line TextArea to multi-line.

### 4. Search (/, ?, n, N)

Limited usefulness in single-line input, but could search:
- Autocomplete suggestions
- jq function list
- Previous queries (if history added)

## Performance Notes

**Mode switching:** Zero cost
- Enum comparison
- No allocations
- Inline functions

**Cursor movement:** Delegated to tui-textarea
- Well-optimized
- No redraws unless needed
- Minimal state changes

**Query execution:** Subprocess spawn
- Main cost is jq process
- ~50-100ms per execution
- Mode system adds <1ms overhead

## Related Code

- **Event handling:** `src/app/events.rs`
- **Rendering:** `src/app/render.rs` (mode colors)
- **App state:** `src/app/state.rs` (editor_mode field)

---

**Maintainer Notes:**
- Mode system is simple by design (46 lines)
- Most complexity in event handling
- tui-textarea does the heavy lifting
- Adding new VIM commands: Update appropriate event handler
- Mode transitions tested thoroughly (8 tests)
