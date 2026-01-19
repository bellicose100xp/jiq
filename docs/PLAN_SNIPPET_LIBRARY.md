# Snippet Library Feature - Implementation Plan

## Implementation Guidelines

1. **Commit after each phase** - Each phase should be committed separately with a descriptive commit message
2. **100% test coverage** - All new code must have complete test coverage before committing
3. **Manual TUI testing** - Verify functionality manually before marking phase complete
4. **Update docs for deviations** - Any changes made during implementation that differ from the original plan must be documented. Update architecture decisions and modify affected later phases to account for these changes

## Phase Checklist

- [x] Phase 1: Empty Popup Shell
- [x] Phase 2: Load and Display Snippets
- [x] Phase 3: List Navigation and Selection
- [x] Phase 4: Preview Pane
- [x] Phase 5: Apply Snippet
- [x] Phase 6: Fuzzy Search
- [x] Phase 7: Create New Snippet (Name Entry)
- [ ] Phase 8: Create with Description
- [ ] Phase 9: Rename Snippet
- [ ] Phase 10: Edit Snippet Query
- [ ] Phase 11: Delete Snippet with Confirmation
- [x] Phase 12: Scroll Support for Long Lists (implemented in Phase 4)
- [ ] Phase 13: Visual Polish
- [ ] Phase 14: Edge Cases and Error Handling

---

## Overview

Add a Snippet Library feature to jiq that allows users to save, manage, and reuse jq queries. The feature is triggered by `Ctrl+S` and provides a popup interface for snippet management.

## User Requirements Summary

- **Trigger**: `Ctrl+S` opens Snippet Manager popup
- **Storage**: `~/.config/jiq/snippets.toml` (TOML format)
- **Snippet fields**: name (required), query (required), description (optional)
- **No tags** in v1 - keep it simple
- **Flat list** - no folders/categories
- **Fuzzy search** to filter snippets by name
- **Preview pane** showing full query text of selected snippet
- **Apply mode**: Replace current query entirely when snippet selected
- **Save flow**: `Ctrl+S` opens manager → press `n` to create new snippet from current query
- **Edit inline** in TUI (no external editor)
- **Operations**: Add (`n`), Remove (`d`/`x`), Rename (`r`), Edit query (`e`)
- **100% test coverage** with unit tests and snapshot tests

## Architecture Decisions

### Event Routing and Popup Priority
- When snippets popup is visible, it captures most keystrokes (similar to history popup)
- Event flow: Check `snippets.is_visible()` early in event handling, before global keys
- Route events to `snippet_events::handle_event()` when visible, short-circuiting other handlers
- `Ctrl+S` global trigger added in `app_events/global.rs`, gated to not fire when snippets already visible

### Truly Global Keys
Certain keys work regardless of popup state for essential user control:
- **`F1`** - Help popup toggle (users should always be able to see keybindings)
- **`?`** - Help popup toggle (when not in a text editing context)
- **`Ctrl+C`** - Quit application (users should always be able to exit)

Note: `Shift+Tab` (BackTab) is allowed through for history popup (to switch focus and close it), but is captured by snippets popup since snippets is a modal that may use Tab/Shift+Tab for its own navigation.

Note: `?` toggles help when snippets popup is visible and not in editing mode (CreateName, CreateDescription, EditName, EditQuery). When editing, `?` is captured as a character input.

### Popup Stacking and Visibility
- **Render order** (back to front): AI/tooltip → autocomplete → history → **snippets**
- Snippets popup renders on top of all other popups when visible
- **On open**: Hide autocomplete popup (same as history behavior)
- **On close**: Return focus to query input field
- Snippets and history popups are mutually exclusive (opening one closes the other)

### Storage Location
- **Path**: `~/.config/jiq/snippets.toml` (intentionally different from history)
- **Rationale**: Snippets are reusable and worth syncing across machines; history is ephemeral and machine-specific
- **Concurrency**: "Last writer wins" - no file locking (same as history)
- **Missing directory**: Create `~/.config/jiq/` on first save if it doesn't exist

### Apply Snippet Behavior
When user presses Enter to apply a snippet:
1. Replace query input text with snippet query
2. Execute the query immediately
3. Reset JSON output scroll position to top
4. Clear any existing error overlay
5. Close snippets popup
6. Return focus to query input

### List Navigation
- **No wrap-around**: Navigation stops at list boundaries
- Up at first item: stays at first item
- Down at last item: stays at last item

### Fuzzy Search Behavior
- Mirror `HistoryMatcher` pattern: multi-term AND matching, score-based sorting
- Results sorted by fuzzy score descending (best matches first)
- Use same `TextArea` configuration as history's `create_search_textarea`

### Layout Design (Updated in Phase 4)
- **Vertical layout**: List on top, query preview on bottom (both use full width)
- **Original plan**: Horizontal 40/60 split was changed because queries benefit from full terminal width
- **Preview pane**: Shows only the query (no description), titled "Query Preview"
- **Description display**: Shown inline next to snippet name in gray, truncated if too long
- **Dynamic preview height**: Adjusts based on wrapped query content, capped at 50% of available height
- **Scroll support**: Implemented early (originally Phase 12) to keep selection visible in long lists

### Long Query Preview
- Use `wrap_text` utility (from AI rendering) for long queries in preview pane
- Prevents layout overflow and maintains readability
- Full terminal width allows longer queries to display with fewer wrapped lines

### Test Coverage
- Unit tests for all state transitions and business logic
- Snapshot tests for all render states
- Event handling tests for all keybindings
- Storage tests for TOML read/write edge cases
- Coverage verified via `cargo test` - all new code paths must have corresponding tests

### Validation and Notification Patterns (Established in Phase 7)
- **Name validation**: Empty check, whitespace trimming, case-insensitive duplicate checking
- **Query validation**: Empty check, whitespace trimming
- **Error notifications**: Use `show_warning()` for validation errors (yellow, auto-dismiss after 10 seconds)
- **Sort order**: New snippets inserted at beginning for newest-first ordering
- **Validation failures**: Keep user in editing mode with notification, don't lose input

These patterns should be followed in Phase 9 (Rename) and Phase 10 (Edit Query).

## Module Structure

```
src/
  snippets.rs                    # Module root (pub mod declarations)
  snippets/
    snippet_state.rs             # SnippetState struct, Snippet struct, SnippetMode enum
    snippet_events.rs            # Event handling (keybindings)
    snippet_render.rs            # Popup rendering
    snippet_storage.rs           # TOML file I/O
    snippet_matcher.rs           # Fuzzy search matcher
    snippet_state_tests.rs       # State unit tests
    snippet_events_tests.rs      # Event handling tests
    snippet_render_tests.rs      # Render snapshot tests
    snippet_storage_tests.rs     # TOML I/O tests
    snippet_matcher_tests.rs     # Fuzzy search tests
```

## Data Structures

### Snippet

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snippet {
    pub name: String,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
```

### TOML Format

```toml
[[snippets]]
name = "Select all keys"
query = "keys"
description = "Returns array of all keys in an object"

[[snippets]]
name = "Flatten nested arrays"
query = "flatten"
```

### SnippetMode (State Machine)

```rust
pub enum SnippetMode {
    Browse,                              // Browsing/searching snippets
    CreateName,                          // Creating new snippet - editing name
    CreateDescription,                   // Creating new snippet - editing description
    EditName { original_name: String },  // Editing existing snippet's name
    EditQuery { snippet_name: String },  // Editing existing snippet's query
    ConfirmDelete { snippet_name: String }, // Confirming deletion
}
```

## UI Layout

Note: The snippets popup fills the entire results pane area, replacing the JSON output while visible. The popup height adapts to the terminal size.

### Browse Mode
```
┌─ Snippets (3) ───────────────────────────────────────────────────┐
│ ► Select all keys - Returns array of all keys in an object       │
│   Filter by type - Filters items matching a specific type        │
│   Flatten arrays                                                 │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
┌─ Query Preview ──────────────────────────────────────────────────┐
│ keys                                                             │
└──────────────────────────────────────────────────────────────────┘
```

Note: Description appears inline next to snippet name (in gray). Preview shows only the query.
Search bar will be added in Phase 6.

### Create Mode
```
┌─ New Snippet ────────────────────────────────────────────────────┐
│ Name:                                                            │
│ [AWS Log Flattener_____________]                                 │
│                                                                  │
│ Query (from current):                                            │
│ .Records[] | {source, detail}                                    │
│                                                                  │
│ Description (optional):                                          │
│ [Extracts source and detail____]                                 │
├──────────────────────────────────────────────────────────────────┤
│ [Enter] Save  [Tab] Next Field  [Esc] Cancel                     │
└──────────────────────────────────────────────────────────────────┘
```

## Keybindings

### Global
| Key | Action |
|-----|--------|
| `Ctrl+S` | Open Snippet Manager |

### Browse Mode
| Key | Action |
|-----|--------|
| `Up` / `k` | Select previous |
| `Down` / `j` | Select next |
| `Enter` | Apply snippet (replace query) |
| `n` | Create new snippet |
| `e` | Edit selected snippet's query |
| `r` | Rename selected snippet |
| `d` / `x` | Delete (with confirmation) |
| `Esc` | Close popup |
| Other | Search input |

### Create/Edit Mode
| Key | Action |
|-----|--------|
| `Enter` | Save / Next field |
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `Esc` | Cancel |

### ConfirmDelete Mode
| Key | Action |
|-----|--------|
| `y` / `Enter` | Confirm delete |
| `n` / `Esc` | Cancel |

---

## Phased Implementation

Each phase delivers the smallest testable feature. Manual TUI testing after each phase.

### Phase 1: Empty Popup Shell
**Goal**: `Ctrl+S` opens an empty popup, `Esc` closes it.

**Event routing setup** (per Architecture Decisions):
- Add `snippets.is_visible()` check early in event handling
- Route to `snippet_events::handle_event()` when visible, short-circuiting other handlers
- On open: hide autocomplete, close history popup if open
- On close: return focus to query input

**Files to create/modify**:
- `src/snippets.rs` - module root
- `src/snippets/snippet_state.rs` - minimal SnippetState (visible flag only)
- `src/snippets/snippet_events.rs` - handle Esc to close
- `src/snippets/snippet_render.rs` - render empty bordered box with title
- `src/app/app_state.rs` - add `snippets: SnippetState` field
- `src/app/app_events/global.rs` - add `Ctrl+S` trigger (gated when snippets already visible)
- `src/app/app_events.rs` - add snippets visibility check and event routing
- `src/app/app_render.rs` - call snippet render when visible (renders on top of other popups)

**Manual test**: Run jiq, press `Ctrl+S`, see empty popup, press `Esc`, popup closes. Verify autocomplete hidden when popup opens.

**Tests**: State visibility toggle, Esc closes popup, event routing short-circuits.

---

### Phase 2: Load and Display Snippets
**Goal**: Load snippets from TOML file and display as a list.

**Files to create/modify**:
- `src/snippets/snippet_storage.rs` - load_snippets() from TOML
- `src/snippets/snippet_state.rs` - add Snippet struct, snippets Vec
- `src/snippets/snippet_render.rs` - render list of snippet names

**Manual test**: Create `~/.config/jiq/snippets.toml` manually with a few entries, open popup, see list.

**Tests**: Storage load tests (empty, valid, invalid TOML), render snapshot.

---

### Phase 3: List Navigation and Selection
**Goal**: Navigate list with arrow keys, show selection indicator.

**Files to modify**:
- `src/snippets/snippet_state.rs` - add selected_index, select_next/prev methods
- `src/snippets/snippet_events.rs` - handle Up/Down/j/k keys
- `src/snippets/snippet_render.rs` - highlight selected item with `►`

**Manual test**: Open popup, use arrow keys to navigate, see selection move. Verify navigation stops at boundaries (no wrap-around).

**Tests**: Navigation bounds, boundary stop behavior.

---

### Phase 4: Preview Pane
**Goal**: Show selected snippet's query in preview pane with description inline in list.

**Deviations from original plan**:
- Changed from horizontal 40/60 split to vertical layout (list top, preview bottom)
- Preview shows only query, not description (titled "Query Preview")
- Description now displays inline next to snippet name in gray
- Preview height is dynamic based on wrapped query content
- Scroll support implemented early (originally Phase 12) to keep selection visible

**Files modified**:
- `src/snippets/snippet_render.rs` - vertical layout, query-only preview, inline descriptions
- `src/snippets/snippet_state.rs` - added scroll_offset, visible_count, set_visible_count(), visible_snippets()
- `src/app/app_render.rs` - changed to pass `&mut SnippetState` for scroll updates

**Manual test**: Navigate list, see preview update with query text. Test with long query to verify wrapping. Verify scroll keeps selection visible with 30+ snippets.

**Tests**: Render snapshot tests for preview, scroll behavior tests.

---

### Phase 5: Apply Snippet
**Goal**: Press Enter to apply selected snippet with full execution flow.

**Apply behavior** (per Architecture Decisions):
1. Replace query input text with snippet query
2. Execute the query immediately
3. Reset JSON output scroll position to top
4. Clear any existing error overlay
5. Close snippets popup
6. Return focus to query input

**Files to modify**:
- `src/snippets/snippet_events.rs` - handle Enter key, implement full apply flow

**Manual test**: Select snippet, press Enter. Verify: query replaced, results update immediately, scroll resets, popup closes.

**Tests**: Event test for Enter applying snippet, verify all state changes.

---

### Phase 6: Fuzzy Search
**Goal**: Type to filter snippets by name.

**Implementation** (per Architecture Decisions):
- Mirror `HistoryMatcher` pattern: multi-term AND matching
- Sort results by fuzzy score descending (best matches first)
- Use `create_search_textarea` style from history

**Files to create/modify**:
- `src/snippets/snippet_matcher.rs` - SnippetMatcher with fuzzy matching (mirror HistoryMatcher)
- `src/snippets/snippet_state.rs` - add search_textarea, filtered_indices
- `src/snippets/snippet_events.rs` - route typing to search textarea
- `src/snippets/snippet_render.rs` - render search bar

**Manual test**: Type partial name, see list filter in real-time. Verify best matches appear first.

**Tests**: Matcher tests (multi-term, scoring, empty query), filter state tests.

---

### Phase 7: Create New Snippet (Name Entry)
**Goal**: Press `n` to enter create mode, type name, press Enter to save.

**Implementation notes**:
- Added `SnippetMode` enum with `Browse` and `CreateName` variants
- Name validation: rejects empty names, trims whitespace
- Query validation: rejects empty queries (not in original plan)
- Duplicate check: case-insensitive comparison ("Keys" vs "keys" are duplicates)
- Sort order: new snippets inserted at beginning (newest-first, not in original plan)
- Error notifications: use `show_warning()` for auto-dismiss after 10 seconds
- Vertical layout: Name input → Query display → Hints bar
- Minimal height fallback: Shows only name input when space limited

**Files modified**:
- `src/snippets/snippet_state.rs` - SnippetMode enum, create mode methods, validation
- `src/snippets/snippet_events.rs` - mode dispatcher, `n` key handler, CreateName event handlers
- `src/snippets/snippet_render.rs` - create mode UI rendering
- `src/snippets/snippet_storage.rs` - save_snippets() and serialize_snippets_toml()
- `src/snippets.rs` - export SnippetMode

**Tests added**: 23 new tests (state transitions, validation, notifications, rendering)

**Manual test**: Type query, press `Ctrl+S`, press `n`, type name, press Enter → snippet saved at top of list. Verify empty name/query show yellow warning notification that auto-dismisses.

---

### Phase 8: Create with Description
**Goal**: After entering name, optionally enter description.

**Files to modify**:
- `src/snippets/snippet_state.rs` - add SnippetMode::CreateDescription
- `src/snippets/snippet_events.rs` - Tab/Enter to move between fields
- `src/snippets/snippet_render.rs` - render description field

**Manual test**: Create snippet, add description, verify in TOML file.

**Tests**: Field navigation tests.

---

### Phase 9: Rename Snippet
**Goal**: Press `r` to rename selected snippet.

**Implementation requirements** (based on Phase 7):
- Use case-insensitive duplicate checking when validating new name
- Show warning notification for validation errors (empty name, duplicate)
- Trim whitespace from new name
- Keep snippet in same position (don't move to top like create)

**Files to modify**:
- `src/snippets/snippet_state.rs` - add SnippetMode::EditName, rename_snippet() with validation
- `src/snippets/snippet_events.rs` - handle `r` key, rename mode events

**Manual test**: Select snippet, press `r`, change name, press Enter, name updated. Try duplicate name (case-insensitive), verify warning notification.

**Tests**: Rename event tests, case-insensitive duplicate name handling, notification tests.

---

### Phase 10: Edit Snippet Query
**Goal**: Press `e` to edit selected snippet's query.

**Implementation requirements** (based on Phase 7):
- Validate query is not empty before saving
- Trim whitespace from query
- Show warning notification for validation errors
- Keep snippet in same position

**Files to modify**:
- `src/snippets/snippet_state.rs` - add SnippetMode::EditQuery, edit_query() with validation
- `src/snippets/snippet_events.rs` - handle `e` key, edit mode events
- `src/snippets/snippet_render.rs` - render query editor

**Manual test**: Select snippet, press `e`, modify query, press Enter, query updated. Try empty query, verify warning notification.

**Tests**: Edit event tests, query validation tests, notification tests.

---

### Phase 11: Delete Snippet with Confirmation
**Goal**: Press `d` to delete with confirmation dialog.

**Files to modify**:
- `src/snippets/snippet_state.rs` - add SnippetMode::ConfirmDelete
- `src/snippets/snippet_events.rs` - handle `d` key, confirm mode events
- `src/snippets/snippet_render.rs` - render confirmation dialog

**Manual test**: Select snippet, press `d`, see confirmation, press `y` to delete.

**Tests**: Delete confirmation flow tests.

---

### Phase 12: Scroll Support for Long Lists
**Status**: ✅ Implemented in Phase 4

**Goal**: Handle lists longer than viewport with scroll offset.

**Implementation** (completed in Phase 4):
- `scroll_offset` and `visible_count` added to SnippetState
- `set_visible_count()` called by render to update visible item count based on list area height
- `visible_snippets()` returns iterator over only visible items
- `adjust_scroll_to_selection()` keeps selected item in view during navigation
- Visible count dynamically calculated from available list height minus borders

**Files modified** (in Phase 4):
- `src/snippets/snippet_state.rs` - scroll state and navigation adjustments
- `src/snippets/snippet_render.rs` - renders only visible slice, calculates visible count

---

### Phase 13: Visual Polish
**Goal**: Improve visual design (colors, borders, hints bar).

**Files to modify**:
- `src/snippets/snippet_render.rs` - add context-sensitive hints bar, improve styling

**Manual test**: Verify UI looks polished and consistent with jiq style.

**Tests**: Updated snapshot tests.

---

### Phase 14: Edge Cases and Error Handling
**Goal**: Handle all edge cases gracefully.

**Edge cases**:
- ✅ Empty snippets (show "No snippets yet. Press 'n' to create one.") - implemented in Phase 2
- ✅ Invalid TOML file (log warning, use empty list) - implemented in Phase 2
- ✅ Very long query (wrap in preview) - implemented in Phase 4
- ✅ Duplicate names (case-insensitive, show warning, prevent save) - implemented in Phase 7
- ✅ Special characters in names - handled by TOML serialization in Phase 7
- ✅ Missing config directory (create on first save) - implemented in Phase 7
- ✅ Empty name validation - implemented in Phase 7
- ✅ Empty query validation - implemented in Phase 7
- ✅ Whitespace trimming (name and query) - implemented in Phase 7
- Remaining: Additional edge cases for rename/edit/delete operations

**Tests**: Most edge case tests already implemented across previous phases.

---

## Critical Files Reference

| Purpose | File Path |
|---------|-----------|
| Pattern: State struct | `src/history/history_state.rs` |
| Pattern: Events handling | `src/history/history_events.rs` |
| Pattern: Popup render | `src/help/help_popup_render.rs` |
| Pattern: Storage I/O | `src/history/storage.rs` |
| Pattern: Matcher | `src/history/matcher.rs` |
| Add Ctrl+S trigger | `src/app/app_events/global.rs` |
| Add snippets to App | `src/app/app_state.rs` |
| Add render call | `src/app/app_render.rs` |
| Pattern: Test helpers | `src/test_utils.rs` |

## Verification Plan

After each phase:
1. Run `cargo build --release` - must pass
2. Run `cargo clippy --all-targets --all-features` - zero warnings
3. Run `cargo fmt --all --check` - zero formatting issues
4. Run `cargo test` - all tests pass
5. Manual TUI testing with explicit test steps
6. Verify 100% test coverage for new code
