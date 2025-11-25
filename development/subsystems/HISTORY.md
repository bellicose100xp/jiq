# Query History

Persistent storage and recall of successful jq queries.

**Status:** Implemented v2.7.0 | **Code Quality:** A

## Features

- Persistent storage (survives sessions)
- Fuzzy search filtering (fzf-style)
- Quick cycling (Ctrl+P/Ctrl+N)
- Automatic deduplication (moves to top on reuse)
- Full-width popup display
- Platform-aware paths

**Keybindings:** Ctrl+R=search popup, Ctrl+P/N=cycle, ↑=open when empty

## Architecture

```
src/history/
├── mod.rs      # Public exports
├── state.rs    # HistoryState, popup & cycling logic
├── storage.rs  # File I/O (load/save)
└── matcher.rs  # Fuzzy matching (SkimMatcherV2)
```

## Data Flow

```
App exit (Enter) → add_entry() → storage::add_entry()
                                        ↓
                                  ~/.local/share/jiq/history
                                        ↓
App init → HistoryState::new() → storage::load_history()
              ↓
    In-memory entries Vec<String>
```

## Storage Format

Plain text file, one query per line, newest first:
```
.items[] | select(.active)
.users[0].name
.
```

**Path resolution:** `dirs::data_dir()` + `jiq/history`
- Linux: `~/.local/share/jiq/history`
- macOS: `~/Library/Application Support/jiq/history`
- Windows: `%APPDATA%\jiq\history`

## Implementation Details

### Two Interaction Modes

**1. Quick Cycling (Ctrl+P/Ctrl+N)**
```rust
cycling_index: Option<usize>  // Tracks position during cycling

Ctrl+P → cycle_previous() → Load older, increment index
Ctrl+N → cycle_next() → Load newer, decrement index
Typing → reset_cycling() → Clear index
```

**2. Search Popup (Ctrl+R)**
```rust
visible: bool
search_query: String
filtered_indices: Vec<usize>  // Indices of matches

Type → push_search_char() → update_filter() → fuzzy match
↑/↓ → select_previous/next() → Wrap-around navigation
Enter/Tab → selected_entry() → Replace input
```

### Fuzzy Matching

Uses `fuzzy-matcher` crate (SkimMatcherV2 algorithm):
```rust
matcher.fuzzy_match(entry, query) → Option<i64>
Sort by score descending
Empty query returns all entries
```

Example: `"itm"` matches `".items[] | .name"`

### Reversed Display

Most recent at bottom (closer to input):
```rust
pub fn visible_entries(&self) -> impl Iterator {
    self.filtered_indices
        .iter()
        .take(15)
        .enumerate()
        .collect::<Vec>()
        .into_iter()
        .rev()  // Display reversed
}
```

Navigation inverted to match: Up→older, Down→newer

### Deduplication

```rust
entries.retain(|e| e != query);  // Remove old occurrence
entries.insert(0, query);        // Add to top
```

O(n) linear scan, acceptable for 1000 entries.

## Rendering

**Popup Layout:**
```
┌─ History (3/15) ────────────────────────┐
│   .oldest_entry                         │
│   .middle_entry                         │
│ ► .newest_entry                         │ ← Selected
├─────────────────────────────────────────┤
│ Search: itm                             │
└─────────────────────────────────────────┘
```

**Dynamic truncation:** `(width - 6)` chars to fill screen
**Selected:** Black on Cyan, bold
**Unselected:** White on Black

## Testing Strategy

**242 tests total, 13 for history:**

**Test isolation:** `persist_to_disk: bool` flag
- Production: `true` → writes to disk
- Tests: `false` → in-memory only
- All tests use `app_with_query()` helper → automatic isolation

**Coverage:**
- Popup: open/close, navigation, selection, search
- Cycling: boundaries, wrap, reset
- UTF-8: emoji, accented chars, fuzzy search
- Edge cases: empty, single entry, no matches

**Design note:** Integration tests for file I/O removed (dirs crate caches paths). Trait-based DI considered over-engineering for single-user CLI tool.

## Design Decisions

**Why fuzzy match?**
- Users remember query content, not exact syntax
- fzf behavior is familiar to developers

**Why reverse display order?**
- Most recent queries are most likely to be reused
- Bottom position reduces eye travel to input

**Why two interaction modes?**
- Ctrl+P/N: Fast, no visual clutter (like bash)
- Ctrl+R: Exploratory, when you forgot what you typed

**Why save only successful queries?**
- Failed queries are mistakes, not useful history
- Reduces noise in history list

**Why 1000 entry limit?**
- Balance between useful history and file size
- ~100KB worst case (100 char queries)
- O(n) operations remain fast

## Concurrency Note

Read-modify-write without file locking. Last writer wins if multiple jiq instances run simultaneously. Acceptable for single-user CLI tool. Document limitations:

```rust
/// ## Concurrency Note
/// This function uses a read-modify-write pattern without file locking.
```

## Configuration Constants

Edit `src/history/state.rs`:
```rust
pub const MAX_VISIBLE_HISTORY: usize = 15;  // Popup height
```

Edit `src/history/storage.rs`:
```rust
const MAX_HISTORY_ENTRIES: usize = 1000;  // Max saved entries
```

## Known Limitations

- No file locking (concurrent writes may lose data)
- No timestamps (pure recency-based)
- No query metadata (execution time, result count, etc.)
- No cross-device sync
- Fixed 1000 entry limit (not configurable)
