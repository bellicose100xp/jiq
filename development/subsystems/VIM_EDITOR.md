# VIM Modal Editing

**File:** `src/editor/mode.rs` (46 lines, simple enum)

## Mode Enum

```rust
pub enum EditorMode {
    Insert,          // Regular typing
    Normal,          // VIM navigation
    Operator(char),  // d/c + awaiting motion
}

impl Default for EditorMode {
    fn default() -> Self { EditorMode::Insert }  // Beginner-friendly
}
```

## Mode State Machine

```
Start → INSERT ←──────────────────────────┐
          │                              │
       ESC (no autocomplete)             │
          ▼                              │
       NORMAL ──────────────────────────┘
          │     i, a, I, A, C
          ├─ x, X, D → execute → stay NORMAL
          │
          ├─ d → OPERATOR('d') ─┬─ motion → execute → NORMAL
          │                     ├─ dd → execute → NORMAL
          │                     └─ invalid/ESC → cancel → NORMAL
          │
          └─ c → OPERATOR('c') ─┬─ motion → execute → INSERT
                                ├─ cc → execute → INSERT
                                └─ invalid/ESC → cancel → NORMAL
```

## Mode Indicators

**Border colors:**
- INSERT: Cyan
- NORMAL: Yellow
- OPERATOR: Green

**Title:** `[INPUT FIELD - {MODE} MODE]`

## Delegation to tui-textarea

jiq does NOT reimplement text editing:

**tui-textarea provides:**
- Text buffer, cursor, selection
- Undo/redo stack
- Word boundaries
- All text operations

**jiq adds:**
- Mode state tracking
- Mode-based key routing
- Operator+motion composition

## Key Implementation

### Navigation (NORMAL mode)

```rust
KeyCode::Char('h') => textarea.move_cursor(CursorMove::Back)
KeyCode::Char('w') => textarea.move_cursor(CursorMove::WordForward)
// etc. - all delegate to tui-textarea
```

### Operators (OPERATOR mode)

```rust
// Enter operator mode
KeyCode::Char('d') => {
    editor_mode = Operator('d');
    textarea.start_selection();  // Begin visual selection
}

// Apply motion
KeyCode::Char('w') => {
    textarea.move_cursor(CursorMove::WordForward);  // Extends selection
}

// Execute
textarea.cut();  // Delete selected text
editor_mode = if op == 'c' { Insert } else { Normal };
```

**Double operator (dd, cc):**
```rust
if key == operator {
    textarea.delete_line_by_head();
    textarea.delete_line_by_end();
    // Result: entire line deleted
}
```

## Supported Commands

### NORMAL Mode

| Command | Effect |
|---------|--------|
| h/l, ←/→ | Move left/right |
| 0, Home | Line start |
| $, End | Line end |
| w/b/e | Word forward/back/end |
| i/a/I/A | Enter INSERT (various positions) |
| x/X | Delete char/char before |
| D/C | Delete/change to end |
| d/c | Enter OPERATOR mode |
| u | Undo |
| Ctrl+r | Redo |

### OPERATOR Mode

**Motions:** w, b, e, $, 0, h, l
**Special:** dd (delete line), cc (change line)

## Design Decisions

**Why default to INSERT?**
- Accessible to non-VIM users
- VIM users press ESC to get NORMAL
- Inclusive design

**Why limited VIM subset?**
- Single-line input field (limited use case)
- 80/20 rule - most valuable commands covered
- Can extend later if needed

**Not implemented:**
- Visual mode (v/V)
- Search (/, n, N)
- Registers/yank
- Repeat (.)
- Many others

**Why execute query after every edit?**
- Consistency with INSERT mode
- Real-time feedback is core feature

## Performance

**Mode switching:** Zero cost (enum comparison, no allocations)
**Cursor movement:** Delegated to tui-textarea (well-optimized)
**Query execution:** Subprocess spawn (~50-100ms), mode adds <1ms overhead
