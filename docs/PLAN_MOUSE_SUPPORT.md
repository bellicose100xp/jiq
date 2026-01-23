# Mouse Support Enhancement Plan

## Overview

This document outlines a phased implementation plan to enhance mouse support in JIQ. The goal is to provide intuitive mouse interactions that complement the existing keyboard-driven interface.

---

## Implementation Guidelines

1. **Commit after each phase** - Each phase should be committed separately with a descriptive commit message
2. **Test coverage** - All new logic must have test coverage before committing
3. **Manual TUI testing** - Verify functionality manually before marking phase complete
4. **Update docs for deviations** - Any changes made during implementation that differ from the original plan must be documented

After each phase:
1. Run `cargo build --release` - must pass
2. Run `cargo clippy --all-targets --all-features` - zero errors
3. Run `cargo fmt --all --check` - zero formatting issues
4. Run `cargo test` - all tests pass
5. Manual TUI testing with explicit test steps
6. Verify test coverage for new code logic

---

## Phase Checklist

- [x] Phase 1: Region Tracking Infrastructure
- [x] Phase 1A: Scrollable Trait & Implementations
- [x] Phase 2: Mouse Event Router
- [x] Phase 3: Visual Scrollbars (INDEPENDENT)
- [x] Phase 4: Click-to-Focus
- [x] Phase 5: AI Window Mouse Interactions
- [x] Phase 6: Snippet Manager Mouse Interactions
- [x] Phase 7: Dismiss Popups on Outside Click

---

## Current State

### Existing Mouse Functionality
- **Location:** `src/app/app_events.rs:279-289`
- Mouse capture is enabled in terminal initialization
- Only handles `ScrollDown` and `ScrollUp` events (scrolls results pane by 3 lines)
- All other mouse events (`Down`, `Up`, `Moved`) are ignored
- No position awareness - scrolling always affects results pane regardless of cursor location

### Existing Scrollbar
- Results pane has a visual scrollbar (commit `074784c`)
- Uses ratatui's `Scrollbar` widget with `ScrollbarState`
- Located in `src/results/results_render.rs:56-70`

### Layout Structure
```
Terminal Area
├── Results Pane (Constraint::Min(3))
│   └── Search Bar (3 lines, at BOTTOM of results area via Layout split)
├── Input Field (3 lines, hidden in overlay mode)
└── Help Line (1 line)

Popups (rendered on top of base layout):
├── AI Window (right side, above input, reserves space for autocomplete on left)*
├── Autocomplete Popup (above input, limited width, LEFT side)*
├── History Popup (above input, full width)*
├── Tooltip Popup (right side, above input)*
├── Error Overlay (centered in results area, when error visible)
├── Snippet Manager (full overlay over results area)
└── Help Popup (full screen overlay)

* Only rendered when input field is visible (not in search/snippet overlay mode)

Note: Search bar is NOT an overlay - it splits the results area vertically when visible.
```

### Focus System
- Two focus states: `InputField` and `ResultsPane`
- Keyboard-only switching (Tab, i key)
- No mouse-based focus changes

---

## Requirements Summary

From the feature request:

1. **AI Window**: Focus entry on hover, scroll suggestions, click to apply
2. **Click-to-focus**: Clicking in result or input box moves focus to that box
3. **Search input click**: Clicking on search input allows editing (same as pressing `/`)
4. **Snippet manager**: Scroll snippet list, highlight/focus snippet by clicking
5. **Position-aware scrolling**: Scroll the component under the mouse cursor
6. **Visual scrollbars**: Add scrollbars to all scrollable areas for discoverability

---

## Existing Infrastructure Analysis

### What Already Exists

| Component | Scroll State | Position Tracking | Scroll Methods |
|-----------|-------------|-------------------|----------------|
| **Results Pane** | `ScrollState` in `app.results_scroll` | N/A | ✓ `scroll_up(n)`, `scroll_down(n)` |
| **AI Window** | `SelectionState` in `app.ai.selection` | ✓ `suggestion_y_positions`, `suggestion_heights` | ✗ Only `navigate_next/prev` |
| **Snippets** | `scroll_offset: usize` | ✗ Fixed-height items (1 line) | ✗ Only `select_next/prev` |
| **Autocomplete** | `scroll_offset: usize` | ✗ Fixed-height items (1 line) | ✗ Only `select_next/prev` |
| **History** | `scroll_offset: usize` | ✗ Fixed-height items (1 line) | ✗ Only `select_next/prev` |
| **Help Popup** | `ScrollState` per tab | N/A | ✓ via `current_scroll_mut()` |

### Key Insight: AI Position Tracking Already Exists

The `SelectionState` struct (`src/ai/selection/state.rs`) already tracks:
```rust
pub struct SelectionState {
    suggestion_y_positions: Vec<u16>,  // Y position of each suggestion
    suggestion_heights: Vec<u16>,      // Height of each suggestion
    scroll_offset: u16,
    viewport_height: u16,
    // ...
}
```

This means finding which suggestion is under the cursor is straightforward:
```rust
fn suggestion_at_y(&self, y: u16) -> Option<usize> {
    let content_y = y.saturating_add(self.scroll_offset);
    for (i, &pos) in self.suggestion_y_positions.iter().enumerate() {
        let height = self.suggestion_heights.get(i).copied().unwrap_or(1);
        if content_y >= pos && content_y < pos + height {
            return Some(i);
        }
    }
    None
}
```

### Design Decision: Scroll vs Selection

**Question**: When user scrolls with mouse wheel in AI/Snippet/History, should it:
- A) Scroll the view (selection may scroll off-screen)
- B) Change the selection (like keyboard up/down)

**Recommendation: Option A (Scroll view only)**

Rationale:
- Mouse users expect scroll to move the view, not change selection
- Allows browsing without accidentally changing selection
- Click can be used to select specific item
- Matches behavior of most desktop applications

**Implementation**: Add `scroll_view_up/down` methods separate from `select_next/prev`.

---

## Code Organization Principles

### DRY: Avoid Duplicated Logic

**Problem:** Without abstractions, we'd duplicate region-recording and scroll logic across many files.

**Solutions:**

1. **Scrollable Trait** - Common interface for scrollable components:
```rust
// src/scroll/scrollable.rs
pub trait Scrollable {
    fn scroll_view_up(&mut self, lines: usize);
    fn scroll_view_down(&mut self, lines: usize);
    fn scroll_offset(&self) -> usize;
    fn max_scroll(&self) -> usize;
    fn viewport_size(&self) -> usize;
}
```

2. **Region Recording via Return Value** - Render functions return their area instead of mutating global state:
```rust
// Instead of: app.layout_regions.ai_window = Some(area);
// Return the area from render:
pub fn render_popup(...) -> Option<Rect> {
    // ... render logic ...
    Some(popup_area)
}

// Caller collects regions:
if let Some(area) = ai_render::render_popup(...) {
    regions.ai_window = Some(area);
}
```

### Focused Files: Split Large Modules

**Mouse events module structure** (follows existing `app_events.rs` pattern):
```
src/app/
├── app_events.rs           # Existing - calls mouse_events
├── mouse_events.rs         # Dispatcher: handle_mouse_event() (~30 lines)
├── mouse_scroll.rs         # handle_scroll() (~50 lines)
├── mouse_click.rs          # handle_click() (~80 lines)
├── mouse_hover.rs          # handle_hover() (~60 lines)
├── mouse_events_tests.rs   # Dispatcher tests
├── mouse_scroll_tests.rs   # Scroll routing tests
├── mouse_click_tests.rs    # Click handling tests
└── mouse_hover_tests.rs    # Hover detection tests
```

**Layout module structure** (new top-level module like `scroll.rs`):
```
src/
├── layout.rs               # Main module with re-exports
└── layout/
    ├── layout_regions.rs       # LayoutRegions struct, Region enum (~60 lines)
    ├── layout_hit_test.rs      # region_at() with priority logic (~80 lines)
    ├── layout_regions_tests.rs # Region struct tests
    └── layout_hit_test_tests.rs # Hit testing tests
```

### Test Coverage Requirements

Each new module must have corresponding test file:
- `regions.rs` → `regions_tests.rs`
- `scroll.rs` → `scroll_tests.rs`
- `click.rs` → `click_tests.rs`

**Test categories:**
1. **Unit tests** - Individual function behavior
2. **Edge case tests** - Boundaries, empty states, overlapping regions
3. **Integration tests** - Full event flow from mouse event to state change

---

## Architecture Changes

### 1. Region Tracking System

To support position-aware interactions, we need to track where UI components are rendered.

**New struct:** `src/layout/regions.rs`
```rust
#[derive(Default, Clone)]
pub struct LayoutRegions {
    // Base layout
    pub results_pane: Option<Rect>,
    pub input_field: Option<Rect>,
    pub search_bar: Option<Rect>,         // At bottom of results when visible

    // Popups (only populated when visible)
    pub ai_window: Option<Rect>,
    pub autocomplete: Option<Rect>,
    pub history_popup: Option<Rect>,
    pub tooltip: Option<Rect>,
    pub error_overlay: Option<Rect>,
    pub help_popup: Option<Rect>,

    // Snippet manager sub-regions
    pub snippet_manager: Option<Rect>,
    pub snippet_list: Option<Rect>,       // List area within snippet manager
    pub snippet_preview: Option<Rect>,    // Preview area within snippet manager
}

impl LayoutRegions {
    pub fn region_at(&self, x: u16, y: u16) -> Option<Region> {
        // Returns the topmost region containing the point
        // Check overlays first (help > snippet > history > ai > autocomplete)
        // Then check base regions (results > input)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    // Base layout
    ResultsPane,
    InputField,
    SearchBar,

    // Popups
    AiWindow,
    Autocomplete,
    HistoryPopup,
    Tooltip,
    ErrorOverlay,
    HelpPopup,

    // Snippet manager sub-regions
    SnippetList,
    SnippetPreview,
}
```

**Integration:** Store `LayoutRegions` in `App` state, populate during render.

**Important visibility rule:** AI, Autocomplete, History, and Tooltip popups are only rendered when the input field is visible (i.e., not in search or snippet overlay mode). The `LayoutRegions` will naturally reflect this since regions are only populated when components are rendered.

### 2. Enhanced Mouse Event Handler

**New module:** `src/app/mouse_events.rs`

```rust
pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    let region = app.layout_regions.region_at(mouse.column, mouse.row);

    match mouse.kind {
        MouseEventKind::ScrollDown => handle_scroll(app, region, ScrollDirection::Down),
        MouseEventKind::ScrollUp => handle_scroll(app, region, ScrollDirection::Up),
        MouseEventKind::Down(MouseButton::Left) => handle_click(app, region, mouse),
        MouseEventKind::Moved => handle_hover(app, region, mouse),
        _ => {}
    }
}
```

---

## Implementation Phases

### Phase Dependencies

```
Phase 1: Region Tracking ─────────┬──> Phase 2: Mouse Event Router ──> Phase 4: Click-to-Focus
                                  │                                ──> Phase 5: AI Window Mouse
                                  │                                ──> Phase 6: Snippet Mouse
                                  │
Phase 1A: Scroll Methods ─────────┘

Phase 3: Visual Scrollbars (INDEPENDENT - can run in parallel with Phase 1)
```

**Critical insight:** Phase 3 (Scrollbars) does NOT depend on region tracking. It's purely visual and can be implemented in parallel with Phase 1 to deliver value faster.

---

### Phase 1: Region Tracking Infrastructure

**Goal:** Establish the foundation for position-aware mouse interactions.
**Dependency:** None (foundation)

**Tasks:**
1. Create `src/layout/` module with `regions.rs`
2. Add `LayoutRegions` struct to track rendered areas
3. Add `layout_regions: LayoutRegions` field to `App` struct
4. Update render functions to record their areas:
   - `results_render::render_pane()` → record `results_pane`, `search_bar`
   - `input_render::render_field()` → record `input_field`
   - `ai_render::render_popup()` → record `ai_window`
   - `autocomplete_render::render_popup()` → record `autocomplete`
   - `history_render::render_popup()` → record `history_popup`
   - `tooltip_render::render_popup()` → record `tooltip`
   - `results_render::render_error_overlay()` → record `error_overlay`
   - `snippet_render::render_popup()` → record `snippet_manager`, `snippet_list`, `snippet_preview`
   - `help_popup_render::render_popup()` → record `help_popup`
5. Implement `region_at()` with proper overlay priority

**Files to modify:**
- `src/app/app_state.rs` - Add `layout_regions` field
- `src/app/app_render.rs` - Pass regions to render functions
- `src/results/results_render.rs` - Record results_pane, search_bar, error_overlay
- `src/input/input_render.rs` - Record input_field
- `src/ai/ai_render.rs` - Record ai_window
- `src/autocomplete/autocomplete_render.rs` - Record autocomplete
- `src/history/history_render.rs` - Record history_popup
- `src/tooltip/tooltip_render.rs` - Record tooltip
- `src/snippets/snippet_render.rs` - Record snippet areas
- `src/help/help_popup_render.rs` - Record help_popup

**New files:**
- `src/layout.rs` - Main module with re-exports
- `src/layout/layout_regions.rs` - LayoutRegions struct, Region enum (~60 lines)
- `src/layout/layout_hit_test.rs` - `region_at()` with priority logic (~80 lines)
- `src/layout/layout_regions_tests.rs` - Region struct tests
- `src/layout/layout_hit_test_tests.rs` - Hit testing with mock regions

**Design decision:** Render functions return their rendered `Rect` instead of mutating global state. The caller (`app_render.rs`) collects all regions. This keeps render functions focused and testable.

---

### Phase 1A: Scrollable Trait & Implementations (Foundation)

**Goal:** Create a common `Scrollable` trait and implement it for all scrollable components.
**Dependency:** None (foundation, can run in parallel with Phase 1)

**Rationale:** Using a trait ensures consistent scroll behavior and enables generic scroll handling in Phase 2. Avoids duplicating scroll logic in each component.

**Tasks:**
1. Create `Scrollable` trait in `src/scroll/scroll_trait.rs`:
   ```rust
   pub trait Scrollable {
       fn scroll_view_up(&mut self, lines: usize);
       fn scroll_view_down(&mut self, lines: usize);
       fn scroll_offset(&self) -> usize;
       fn max_scroll(&self) -> usize;
       fn viewport_size(&self) -> usize;
   }
   ```

2. Implement `Scrollable` for each component:
   - `SelectionState` (AI) - scroll suggestions view
   - `SnippetState` - scroll snippet list view
   - `HistoryState` - scroll history list view
   - `AutocompleteState` - scroll autocomplete list view
   - Note: `HelpPopupState` uses `ScrollState` which already has scroll methods

3. Add unit tests for trait implementations

**New files:**
- `src/scroll/scroll_trait.rs` - Trait definition (~30 lines)
- `src/scroll/scroll_trait_tests.rs` - Trait behavior tests

**Files to modify:**
- `src/scroll.rs` - Export new trait (add `pub mod scroll_trait;`)
- `src/ai/selection/state.rs` - `impl Scrollable for SelectionState`
- `src/snippets/snippet_state.rs` - `impl Scrollable for SnippetState`
- `src/history/history_state.rs` - `impl Scrollable for HistoryState`
- `src/autocomplete/autocomplete_state.rs` - `impl Scrollable for AutocompleteState`

---

### Phase 2: Mouse Event Router

**Goal:** Create position-aware mouse event routing with focused, testable modules.
**Dependency:** Phase 1 (region tracking) + Phase 1A (Scrollable trait)

**Tasks:**
1. Create mouse event files in `src/app/`:
   ```
   src/app/
   ├── mouse_events.rs         # Dispatcher: handle_mouse_event()
   ├── mouse_scroll.rs         # Scroll routing logic
   └── mouse_scroll_tests.rs   # Scroll routing tests
   ```

2. Implement `handle_mouse_event()` dispatcher in `mouse_events.rs` (~30 lines):
   ```rust
   pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
       let region = app.layout_regions.region_at(mouse.column, mouse.row);
       match mouse.kind {
           MouseEventKind::ScrollDown => scroll::handle_scroll(app, region, Direction::Down),
           MouseEventKind::ScrollUp => scroll::handle_scroll(app, region, Direction::Up),
           _ => {} // Click/hover added in later phases
       }
   }
   ```

3. Implement `mouse_scroll.rs` with region-to-component routing (~50 lines):
   - Use `Scrollable` trait for uniform handling where possible
   - Handle fallback (outside regions → scroll results pane)

4. Update `app_events.rs` to call new handler

**Files to modify:**
- `src/app/app_events.rs` - Replace inline handler with `mouse_events::handle_mouse_event()`
- `src/app.rs` - Add `pub mod mouse_events; pub mod mouse_scroll;`

**New files:**
- `src/app/mouse_events.rs` - Dispatcher (~30 lines)
- `src/app/mouse_scroll.rs` - Scroll handling (~50 lines)
- `src/app/mouse_scroll_tests.rs` - Scroll tests

---

### Phase 3: Visual Scrollbars (INDEPENDENT)

**Goal:** Add scrollbar indicators to all scrollable components.
**Dependency:** None (can run in parallel with Phase 1/1A/2)

**Why independent:** Scrollbars are purely visual widgets that read existing scroll state. They don't need region tracking or new scroll methods.

**Scrollbar utility function:**
```rust
// src/widgets/scrollbar.rs
pub fn render_vertical_scrollbar(
    frame: &mut Frame,
    area: Rect,
    total_items: usize,
    viewport_size: usize,
    scroll_offset: usize,
) {
    if total_items <= viewport_size {
        return; // No scrollbar needed
    }

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None);

    let mut state = ScrollbarState::new(total_items)
        .position(scroll_offset)
        .viewport_content_length(viewport_size);

    frame.render_stateful_widget(scrollbar, area, &mut state);
}
```

**Tasks:**
1. Create reusable scrollbar utility in `src/widgets/scrollbar.rs`
2. Add scrollbar to AI Window suggestions:
   - Render after suggestions content
   - Use `ai_state.selection.scroll_offset()` and suggestion count
3. Add scrollbar to Snippet Manager list:
   - In `render_list()` function
   - Use `state.scroll_offset()` and `state.filtered_count()`
4. Add scrollbar to Help Popup:
   - Use existing scroll state
5. Add scrollbar to History Popup:
   - Use history entry count and scroll offset
6. Add scrollbar to Autocomplete:
   - Use suggestion count and selected index

**Files to modify:**
- `src/ai/ai_render.rs` - Add scrollbar after suggestions
- `src/snippets/snippet_render.rs` - Add scrollbar to list
- `src/help/help_popup_render.rs` - Add scrollbar
- `src/history/history_render.rs` - Add scrollbar
- `src/autocomplete/autocomplete_render.rs` - Add scrollbar

**New files:**
- `src/widgets/scrollbar.rs`
- `src/widgets/scrollbar_tests.rs`

---

### Phase 4: Click-to-Focus

**Goal:** Clicking in result or input box moves focus to that box.
**Dependency:** Phase 1 (region tracking) + Phase 2 (mouse event router)

**Tasks:**
1. Add click handling files to `src/app/`:
   ```
   src/app/
   ├── mouse_click.rs          # Click handling (~80 lines)
   └── mouse_click_tests.rs    # Click tests
   ```

2. Implement `handle_click()` in `mouse_click.rs`:
   ```rust
   fn handle_click(app: &mut App, region: Option<Region>, mouse: MouseEvent) {
       match region {
           Some(Region::ResultsPane) => {
               if app.focus != Focus::ResultsPane {
                   app.focus = Focus::ResultsPane;
               }
           }
           Some(Region::InputField) => {
               if app.focus != Focus::InputField {
                   app.focus = Focus::InputField;
                   app.input.editor_mode = EditorMode::Insert;
               }
           }
           // ... other regions
           _ => {}
       }
   }
   ```
2. Handle click in Search Bar:
   - If search is visible and confirmed (navigating results), click makes it editable
   - Equivalent to pressing `/` to edit search query
3. Test focus transitions with mouse

**Files to modify:**
- `src/app/mouse_events.rs` - Add click handler
- `src/search/search_state.rs` - May need method to switch to edit mode

---

### Phase 5: AI Window Mouse Interactions

**Goal:** Hover to highlight, click to apply suggestions.
**Dependency:** Phase 1 (region tracking) + Phase 2 (mouse event router) + Phase 4 (click module)

**Tasks:**
1. Add hover handling files to `src/app/`:
   ```
   src/app/
   ├── mouse_hover.rs          # Hover handling (~60 lines)
   └── mouse_hover_tests.rs    # Hover tests
   ```

2. Track hovered suggestion index in AI state:
   ```rust
   // In AiState or SelectionState
   pub hovered_index: Option<usize>,
   ```
2. Implement hover detection in `handle_hover()`:
   - Calculate which suggestion is under cursor using `suggestion_y_positions`
   - Update `hovered_index`
   - If navigation not active, also update `selected_index` on hover
3. Implement click-to-apply:
   - On click in AI window, apply the clicked suggestion
   - Use existing `apply_suggestion()` logic
4. Update AI render to show hover highlighting:
   - Highlight hovered suggestion differently from selected
   - Consider: subtle highlight for hover, stronger for selected

**Hover vs Selection behavior:**
- If user has used Alt+Up/Down (navigation_active=true): hover only shows visual feedback, doesn't change selection
- If user hasn't navigated: hover changes selection (for quick mouse-only usage)

**Files to modify:**
- `src/ai/selection/state.rs` - Add `hovered_index`
- `src/ai/ai_render.rs` - Render hover state
- `src/app/mouse_events.rs` - Handle AI window hover and click
- `src/ai/ai_events.rs` - May need click-to-apply method

---

### Phase 6: Snippet Manager Mouse Interactions

**Goal:** Click to select snippet, scroll to navigate.
**Dependency:** Phase 1 (region tracking) + Phase 2 (mouse event router)

**Tasks:**
1. Implement click-to-select in snippet list:
   - Calculate which snippet is under cursor
   - Update `selected_index`
2. Double-click to apply (optional enhancement):
   - Track last click time and position
   - Apply snippet on double-click
3. Scroll support (from Phase 2)
4. Visual feedback:
   - Hover highlighting for snippet entries (optional)

**Files to modify:**
- `src/snippets/snippet_state.rs` - Add method to select by index
- `src/snippets/snippet_render.rs` - Track item positions
- `src/app/mouse_events.rs` - Handle snippet interactions

---

### Phase 7: Dismiss Popups on Outside Click

**Goal:** Clicking outside an open popup dismisses it.
**Dependency:** Phase 4 (click handling infrastructure)

**Behavior:**
- When Help popup is open and user clicks outside its boundary → close the popup
- When Error overlay is open and user clicks outside → close the overlay
- Other popups (AI, Autocomplete, History, Tooltip) are contextual and may have different dismiss behaviors

**Tasks:**
1. In `handle_click()`, check if Help popup is visible:
   - If click is NOT on `Region::HelpPopup` → close help popup
2. Similarly for Error overlay:
   - If click is NOT on `Region::ErrorOverlay` → close error overlay
3. Add tests for dismiss behavior

**Files to modify:**
- `src/app/mouse_click.rs` - Add dismiss logic in click handler

---

## Testing Strategy

### Unit Tests
Each phase should include unit tests:

1. **Region tracking tests:**
   - Test `region_at()` with various coordinates
   - Test overlay priority (popup over base regions)
   - Test edge cases (borders, empty areas)

2. **Scroll routing tests:**
   - Test scroll events route to correct component
   - Test scroll bounds are respected
   - Test scroll when component is not visible (should no-op)

3. **Click handling tests:**
   - Test focus changes on click
   - Test click outside all regions (should no-op)
   - Test click in overlapping regions (topmost wins)

4. **AI interaction tests:**
   - Test hover index calculation
   - Test click-to-apply
   - Test hover behavior with/without navigation active

### Integration Tests
- Full mouse event flow from terminal to component
- Verify visual feedback renders correctly
- Test interaction sequences (hover then click, scroll then click)

### Manual Testing Checklist
- [ ] Scroll in results pane
- [ ] Scroll in AI window
- [ ] Scroll in snippet list
- [ ] Scroll in help popup
- [ ] Click to focus results pane
- [ ] Click to focus input field
- [ ] Click search bar to edit
- [ ] Hover AI suggestions
- [ ] Click AI suggestion to apply
- [ ] Click snippet to select
- [ ] Scrollbars visible when content overflows
- [ ] Scrollbars hidden when content fits
- [ ] Click outside help popup to dismiss
- [ ] Click outside error overlay to dismiss

---

## Risk Assessment

### Low Risk
- Adding scrollbars (purely visual, no state changes)
- Position-aware scrolling (extends existing functionality)
- Region tracking (new infrastructure, isolated)

### Medium Risk
- Click-to-focus (changes focus behavior, may conflict with keyboard flow)
- Search bar click-to-edit (needs careful state transition handling)

### Lower Than Expected Complexity
- AI hover interactions - position tracking already exists in `SelectionState` (`suggestion_y_positions`, `suggestion_heights`)
- Snippet click-to-select - fixed-height items make index calculation trivial

### Mitigation
- Implement phases incrementally
- Each phase is independently useful
- Can stop after Phase 3 (scrolling + scrollbars) and have significant value
- Extensive testing for focus-related changes

---

## Dependencies

- ratatui `Scrollbar` and `ScrollbarState` widgets (already in use)
- crossterm `MouseEvent`, `MouseEventKind`, `MouseButton` (already enabled)
- No external dependencies needed

---

## Estimated Scope

| Phase | Complexity | Files Modified | New Files | Depends On |
|-------|-----------|----------------|-----------|------------|
| 1. Region Tracking | Medium | 10 | 5 | None |
| 1A. Scrollable Trait | Low | 5 | 2 | None |
| 2. Mouse Event Router | Low | 2 | 3 | 1, 1A |
| 3. Visual Scrollbars | Low | 6 | 2 | **None** |
| 4. Click-to-Focus | Medium | 1 | 2 | 1, 2 |
| 5. AI Window Mouse | Medium | 3 | 2 | 1, 2, 4 |
| 6. Snippet Manager Mouse | Low | 2 | 0 | 1, 2, 4 |
| 7. Dismiss on Outside Click | Low | 1 | 0 | 4 |

**File size targets (per CLAUDE.md):**
- No file over 1000 lines
- Each module focused on single responsibility
- Tests in separate `*_tests.rs` files

**Notes:**
- Phase 5 complexity reduced from Medium-High to Medium because position tracking already exists in `SelectionState`
- Phase 6 is Low because snippet items have fixed height (1 line each)
- Phase 3 can run **in parallel** with Phases 1/1A/2

**Recommended implementation order:**

```
Week 1 (Parallel tracks):
├── Track A: Phase 1 (Region Tracking) → Phase 1A (Scroll Methods) → Phase 2 (Router)
└── Track B: Phase 3 (Scrollbars) ← Can be done independently!

Week 2 (Sequential, needs Phase 1+2):
└── Phase 4 (Click-to-Focus) → Phase 5 (AI Mouse) → Phase 6 (Snippet Mouse) → Phase 7 (Dismiss)
```

**Minimum Viable Mouse Support:** Phase 1 + 1A + 2 + 3 delivers:
- Position-aware scrolling in all components
- Visual scrollbar indicators everywhere
- No breaking changes to existing keyboard workflow

---

## Open Questions

1. **Hover delay for AI suggestions?**
   - Should hover immediately highlight, or wait ~100ms to avoid flickering?

2. **Click vs double-click for apply?**
   - Single click could accidentally apply when user just wants to scroll
   - Double-click is safer but less discoverable
   - Alternative: click to select, Enter to apply (current keyboard behavior)

3. **Visual feedback for clickable areas?**
   - Should cursor change when hovering clickable regions? (Terminal may not support)
   - Should borders highlight on hover?

4. **Scroll speed?**
   - Currently 3 lines per scroll event for results pane
   - Recommended per component:
     - Results pane: 3 lines (current behavior)
     - AI window: 1 suggestion at a time (variable height)
     - Snippet/History/Autocomplete: 1 item (fixed height)
     - Help popup: 3 lines (content-heavy)

---

## References

- Current mouse handler: `src/app/app_events.rs:279-289`
- Scrollbar implementation: `src/results/results_render.rs:56-70`
- AI selection state: `src/ai/selection/state.rs`
- Snippet state: `src/snippets/snippet_state.rs`
- ratatui Scrollbar docs: https://docs.rs/ratatui/latest/ratatui/widgets/struct.Scrollbar.html
