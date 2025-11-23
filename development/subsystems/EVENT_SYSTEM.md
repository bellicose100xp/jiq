# Event System

**File:** `src/app/events.rs` (413 lines, 54 tests)

## Event Flow

```
Keyboard → handle_events() → handle_key_event()
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
            handle_global_keys()          Focus-based routing
            (Ctrl+C, q, Enter, Tab)              │
                    │                   ┌─────────┴──────────┐
                    │                   ▼                    ▼
            Returns true/false    InputField           ResultsPane
                                       │                    │
                                  Mode dispatch         j/k scroll
                                       │
                    ┌──────────────────┼──────────────────┐
                    ▼                  ▼                  ▼
                 INSERT             NORMAL            OPERATOR
              (type text)        (vim nav/cmd)       (d/c+motion)
```

## Dispatch Priority

1. **Global keys** (work everywhere): Ctrl+C, q, Enter, Shift+Enter, Shift+Tab, Tab
2. **Focus-specific**: InputField vs ResultsPane
3. **Mode-specific**: INSERT/NORMAL/OPERATOR (input field only)

## Key Handlers

### Global Keys (`events.rs:36-91`)

| Key | Action |
|-----|--------|
| Ctrl+C, q | Quit |
| Enter | Quit + output results |
| Shift+Enter | Quit + output query |
| Shift+Tab | Toggle focus |
| Tab | Accept autocomplete (if visible) |

**Tab handling:** Only works when autocomplete visible, returns `false` otherwise to allow tui-textarea handling.

### Input Field - INSERT Mode (`events.rs:129-149`)

```rust
fn handle_insert_mode_key(&mut self, key: KeyEvent) {
    let content_changed = self.textarea.input(key);
    if content_changed {
        self.execute_query();           // Real-time execution
        self.results_scroll = 0;        // Reset scroll
    }
    self.update_autocomplete();         // Always update
}
```

### Input Field - NORMAL Mode (`events.rs:152-256`)

| Keys | Action |
|------|--------|
| h/l/←/→, 0/$, w/b/e | Cursor movement |
| i, a, I, A | Enter INSERT mode |
| x, X, D | Delete operations |
| C | Delete to end + INSERT |
| d, c | Enter OPERATOR mode |
| u, Ctrl+r | Undo/redo |

### Input Field - OPERATOR Mode (`events.rs:259-343`)

State machine: `NORMAL → d/c → OPERATOR(char) → motion/dd/cc → execute → NORMAL/INSERT`

**Visual selection mechanism:**
```rust
start_selection()   // Mark start
move_cursor(...)    // Extends selection
cut()              // Execute delete/change
```

**Motions:** w, b, e, $, 0, h, l
**Double operator:** dd (delete line), cc (change line)

### Results Pane (`events.rs:359-411`)

| Keys | Scroll Amount |
|------|---------------|
| j/k, ↑/↓ | 1 line |
| J/K | 10 lines |
| g, Home | Top |
| G | Bottom |
| Ctrl+d/u, PgDn/Up | Half page |

## ESC Key Priority

```rust
if ESC pressed:
    if autocomplete.is_visible():
        autocomplete.hide()      // Priority 1
        return
    editor_mode = Normal         // Priority 2
```

Prevents frustrating UX where ESC doesn't close popup.

## Query Execution Triggers

Every content change in any mode:
- INSERT mode typing
- NORMAL mode: x, X, D, C
- OPERATOR completion: dw, dd, etc.
- Undo/redo

## Design Decisions

**Why global keys have priority?**
- User must always be able to quit
- Consistent behavior regardless of focus

**Why execute on every keystroke?**
- Real-time feedback is core value
- jq latency acceptable (~50-100ms)

**Why saturating arithmetic for scroll?**
- No bounds checking needed
- Idiomatic Rust
- Zero runtime cost

## Test Coverage

54 tests across:
- VIM operators (17): dw, db, de, d$, dd, cw, cc, etc.
- Mode transitions (8): INSERT↔NORMAL, OPERATOR handling
- VIM commands (9): x, X, D, C, u, Ctrl+r
- Navigation (8): h, l, 0, $, w, b, e
- Autocomplete (10): ESC, arrows, Tab
- Results scroll (14): All scroll commands + bounds
- Global keys (10): Quit, output modes, focus switch
