# Snippet Library Feature - Implementation Plan

## Implementation Guidelines

1. **Commit after each phase** - Each phase should be committed separately with a descriptive commit message
2. **100% test coverage** - All new code must have complete test coverage before committing
3. **Manual TUI testing** - Verify functionality manually before marking phase complete

## Phase Checklist

- [ ] Phase 1: Empty Popup Shell
- [ ] Phase 2: Load and Display Snippets
- [ ] Phase 3: List Navigation and Selection
- [ ] Phase 4: Preview Pane
- [ ] Phase 5: Apply Snippet
- [ ] Phase 6: Fuzzy Search
- [ ] Phase 7: Create New Snippet (Name Entry)
- [ ] Phase 8: Create with Description
- [ ] Phase 9: Rename Snippet
- [ ] Phase 10: Edit Snippet Query
- [ ] Phase 11: Delete Snippet with Confirmation
- [ ] Phase 12: Scroll Support for Long Lists
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

### Browse Mode
```
┌─ Snippets (3/5) ─────────────────────────────────────────────────┐
│ Search: [____________________________]                           │
├─────────────────────────────┬────────────────────────────────────┤
│ ► Select all keys           │ Query:                             │
│   Filter by type            │ keys                               │
│   Flatten arrays            │                                    │
│                             │ Description:                       │
│                             │ Returns array of all keys          │
├─────────────────────────────┴────────────────────────────────────┤
│ [Enter] Apply  [n]ew  [e]dit  [r]ename  [d]elete  [Esc] Close    │
└──────────────────────────────────────────────────────────────────┘
```

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

**Files to create/modify**:
- `src/snippets.rs` - module root
- `src/snippets/snippet_state.rs` - minimal SnippetState (visible flag only)
- `src/snippets/snippet_events.rs` - handle Esc to close
- `src/snippets/snippet_render.rs` - render empty bordered box with title
- `src/app/app_state.rs` - add `snippets: SnippetState` field
- `src/app/app_events/global.rs` - add `Ctrl+S` trigger
- `src/app/app_render.rs` - call snippet render when visible

**Manual test**: Run jiq, press `Ctrl+S`, see empty popup, press `Esc`, popup closes.

**Tests**: State visibility toggle, Esc closes popup.

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

**Manual test**: Open popup, use arrow keys to navigate, see selection move.

**Tests**: Navigation bounds, wrap-around behavior (or boundary stop).

---

### Phase 4: Preview Pane
**Goal**: Show selected snippet's query and description in preview pane.

**Files to modify**:
- `src/snippets/snippet_render.rs` - split layout 40/60, render preview pane

**Manual test**: Navigate list, see preview update with query text.

**Tests**: Render snapshot tests for preview.

---

### Phase 5: Apply Snippet
**Goal**: Press Enter to apply selected snippet (replace current query).

**Files to modify**:
- `src/snippets/snippet_events.rs` - handle Enter key, replace query, close popup

**Manual test**: Select snippet, press Enter, query input replaced, popup closes.

**Tests**: Event test for Enter applying snippet.

---

### Phase 6: Fuzzy Search
**Goal**: Type to filter snippets by name.

**Files to create/modify**:
- `src/snippets/snippet_matcher.rs` - SnippetMatcher with fuzzy matching
- `src/snippets/snippet_state.rs` - add search_textarea, filtered_indices
- `src/snippets/snippet_events.rs` - route typing to search textarea
- `src/snippets/snippet_render.rs` - render search bar

**Manual test**: Type partial name, see list filter in real-time.

**Tests**: Matcher tests, filter state tests.

---

### Phase 7: Create New Snippet (Name Entry)
**Goal**: Press `n` to enter create mode, type name, press Enter to save.

**Files to modify**:
- `src/snippets/snippet_state.rs` - add SnippetMode::CreateName, name_textarea, pending_query
- `src/snippets/snippet_events.rs` - handle `n` key, CreateName mode events
- `src/snippets/snippet_render.rs` - render create mode UI
- `src/snippets/snippet_storage.rs` - save_snippets() function

**Manual test**: Type query, press `Ctrl+S`, press `n`, type name, press Enter, snippet saved.

**Tests**: Mode transition tests, save tests.

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

**Files to modify**:
- `src/snippets/snippet_state.rs` - add SnippetMode::EditName
- `src/snippets/snippet_events.rs` - handle `r` key, rename mode events

**Manual test**: Select snippet, press `r`, change name, press Enter, name updated.

**Tests**: Rename event tests, duplicate name handling.

---

### Phase 10: Edit Snippet Query
**Goal**: Press `e` to edit selected snippet's query.

**Files to modify**:
- `src/snippets/snippet_state.rs` - add SnippetMode::EditQuery
- `src/snippets/snippet_events.rs` - handle `e` key, edit mode events
- `src/snippets/snippet_render.rs` - render query editor

**Manual test**: Select snippet, press `e`, modify query, press Enter, query updated.

**Tests**: Edit event tests.

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
**Goal**: Handle lists longer than viewport with scroll offset.

**Files to modify**:
- `src/snippets/snippet_state.rs` - add scroll_offset, viewport calculations
- `src/snippets/snippet_render.rs` - render visible slice with scroll indicators

**Manual test**: Add 20+ snippets, verify scrolling works smoothly.

**Tests**: Scroll offset tests, boundary tests.

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
- Empty snippets (show "No snippets yet. Press 'n' to create one.")
- Invalid TOML file (log warning, use empty list)
- Very long query (wrap in preview)
- Duplicate names (show error, prevent save)
- Special characters in names
- Missing config directory (create on first save)

**Tests**: Edge case unit tests.

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
