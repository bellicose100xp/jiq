# Text Selection in Results Pane Plan

## Overview

This document outlines the implementation plan for mouse-based text selection in the results pane. Users will be able to click and drag to select text, copy the selection with `Y` or `c` keys, and auto-scroll when dragging beyond viewport boundaries.

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

- [ ] Phase 1: Selection State Data Structure
- [ ] Phase 2: Screen-to-Text Coordinate Mapping
- [ ] Phase 3: Mouse Event Handling (Click & Drag)
- [ ] Phase 4: Selection Rendering (Visual Highlight)
- [ ] Phase 5: Auto-Scroll During Drag
- [ ] Phase 6: Copy Selected Text to Clipboard
- [ ] Phase 7: Integration & Edge Cases

---

## Current State Analysis

### Existing Infrastructure

| Component | Location | Description |
|-----------|----------|-------------|
| **Results Rendering** | `src/results/results_render.rs` | Renders results using ratatui `Paragraph` with `Text<'static>` |
| **Scroll State** | `src/scroll/scroll_state.rs` | Tracks `offset`, `h_offset`, `viewport_height`, `max_offset` |
| **Mouse Events** | `src/app/mouse_events.rs` | Dispatcher routing mouse events to handlers |
| **Mouse Click** | `src/app/mouse_click.rs` | Handles left click for focus changes |
| **Layout Regions** | `src/layout/layout_regions.rs` | Tracks `ResultsPane` bounds via `Rect` |
| **Clipboard** | `src/clipboard/clipboard_events.rs` | Handles `Y` key to copy entire results |
| **Query State** | `src/query/query_state.rs` | Stores rendered text and unformatted text |

### Key Rendering Architecture

```
Results Pane (Rect from LayoutRegions)
  ├── Block: borders + title with status
  ├── Paragraph: renders Text<'static>
  │   ├── Vertical scroll: viewport slicing (lines[offset..offset+height])
  │   └── Horizontal scroll: Paragraph::scroll((0, h_offset))
  └── Scrollbar: rendered separately on right edge
```

### Existing Text Storage

```rust
// In QueryState (src/query/query_state.rs)
pub last_successful_result_rendered: Option<Text<'static>>,    // Styled ratatui text
pub last_successful_result_unformatted: Option<Arc<String>>,   // Plain text for copying
pub cached_line_count: u32,
pub cached_max_line_width: u16,
```

### Search Highlighting Pattern (Reference)

The search feature already implements per-character styling:
- `Match` struct: `{ line: u32, col: u16, len: u16 }`
- Applied via `apply_highlights_to_line()` function
- Only processes visible viewport for performance

---

## Phase Dependencies

```
Phase 1: Selection State ──────────┐
                                   │
Phase 2: Coordinate Mapping ───────┼──> Phase 3: Mouse Events ──> Phase 5: Auto-Scroll
                                   │           │
                                   │           └──> Phase 4: Selection Rendering
                                   │
                                   └──────────────> Phase 6: Copy to Clipboard

Phase 7: Integration (depends on all above)
```

---

## Phase 1: Selection State Data Structure

**Goal:** Define data structures to represent text selection state.

**Dependency:** None (foundation)

### New File: `src/results/selection.rs`

```rust
/// Represents a text selection in the results pane using absolute coordinates.
/// Start and end are stored in document order (start <= end).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionBounds {
    /// Starting line (0-indexed, absolute in full document)
    pub start_line: u32,
    /// Starting column (0-indexed, character position)
    pub start_col: u16,
    /// Ending line (0-indexed, absolute in full document)
    pub end_line: u32,
    /// Ending column (0-indexed, exclusive - character AFTER last selected)
    pub end_col: u16,
}

impl SelectionBounds {
    /// Create from anchor and current positions, normalizing order
    pub fn from_positions(
        anchor: (u32, u16),
        current: (u32, u16),
    ) -> Self {
        let (start, end) = if anchor.0 < current.0
            || (anchor.0 == current.0 && anchor.1 <= current.1) {
            (anchor, current)
        } else {
            (current, anchor)
        };

        Self {
            start_line: start.0,
            start_col: start.1,
            end_line: end.0,
            end_col: end.1,
        }
    }

    /// Check if selection is empty (zero characters)
    pub fn is_empty(&self) -> bool {
        self.start_line == self.end_line && self.start_col == self.end_col
    }

    /// Check if a given line is within selection
    pub fn contains_line(&self, line: u32) -> bool {
        line >= self.start_line && line <= self.end_line
    }

    /// Get column range for a specific line (returns None if line not in selection)
    pub fn columns_for_line(&self, line: u32, line_length: u16) -> Option<(u16, u16)> {
        if !self.contains_line(line) {
            return None;
        }

        let start_col = if line == self.start_line {
            self.start_col
        } else {
            0
        };

        let end_col = if line == self.end_line {
            self.end_col
        } else {
            line_length
        };

        Some((start_col, end_col))
    }
}

/// Tracks the active selection state during and after mouse drag
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    /// The anchor point where selection started (line, col)
    /// None if no selection in progress
    anchor: Option<(u32, u16)>,

    /// Current completed selection bounds
    /// None if no selection exists
    selection: Option<SelectionBounds>,

    /// Whether a drag is currently in progress
    drag_active: bool,
}

impl SelectionState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new selection at the given position
    pub fn start_selection(&mut self, line: u32, col: u16) {
        self.anchor = Some((line, col));
        self.selection = None;
        self.drag_active = true;
    }

    /// Update selection as drag continues
    pub fn update_selection(&mut self, line: u32, col: u16) {
        if let Some(anchor) = self.anchor {
            self.selection = Some(SelectionBounds::from_positions(anchor, (line, col)));
        }
    }

    /// Finish the drag operation
    pub fn finish_selection(&mut self) {
        self.drag_active = false;
        // Keep selection visible for copying
        // Clear empty selections
        if let Some(sel) = &self.selection {
            if sel.is_empty() {
                self.clear();
            }
        }
    }

    /// Clear all selection state
    pub fn clear(&mut self) {
        self.anchor = None;
        self.selection = None;
        self.drag_active = false;
    }

    /// Get current selection bounds (if any)
    pub fn selection(&self) -> Option<&SelectionBounds> {
        self.selection.as_ref()
    }

    /// Check if drag is in progress
    pub fn is_dragging(&self) -> bool {
        self.drag_active
    }

    /// Check if a selection exists (for copy operation)
    pub fn has_selection(&self) -> bool {
        self.selection.is_some()
    }
}
```

### Integration Points

**File:** `src/app/app_state.rs`
- Add `results_selection: SelectionState` field to `App` struct

**File:** `src/results.rs`
- Add `pub mod selection;` export

### Test File: `src/results/selection_tests.rs`

Test cases:
1. `SelectionBounds::from_positions` normalizes reverse selections
2. `SelectionBounds::is_empty` returns true for zero-length selection
3. `SelectionBounds::contains_line` boundary conditions
4. `SelectionBounds::columns_for_line` returns correct ranges
5. `SelectionState` lifecycle: start -> update -> finish -> clear
6. Empty selections are cleared on finish

---

## Phase 2: Screen-to-Text Coordinate Mapping

**Goal:** Convert mouse screen coordinates to absolute document line/column positions.

**Dependency:** Phase 1 (uses `SelectionBounds`)

### Coordinate Systems

```
Screen Coordinates (from crossterm MouseEvent):
┌────────────────────────────────────────┐
│ (0,0)                            (w,0) │
│                                        │
│   Results Pane Area (from layout)      │
│   ┌──────────────────────────────────┐ │
│   │ border (y = area.y)              │ │
│   │ ┌──────────────────────────────┐ │ │
│   │ │ content line 0 (y = area.y+1)│ │ │
│   │ │ content line 1               │ │ │
│   │ │ ...                          │ │ │
│   │ └──────────────────────────────┘ │ │
│   │ border                           │ │
│   └──────────────────────────────────┘ │
│                                        │
└────────────────────────────────────────┘

Text Coordinates (in document):
- line: 0-indexed absolute line in full document
- col: 0-indexed character position in line (NOT byte offset)
```

### New File: `src/results/selection_coords.rs`

```rust
use ratatui::layout::Rect;
use ratatui::text::Text;
use crate::scroll::ScrollState;

/// Result of coordinate conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextPosition {
    pub line: u32,
    pub col: u16,
}

/// Convert screen position to absolute text position
/// Returns None if position is outside the content area
pub fn screen_to_text_position(
    mouse_col: u16,
    mouse_row: u16,
    results_area: Rect,
    scroll: &ScrollState,
    text: &Text<'_>,
) -> Option<TextPosition> {
    // Check if mouse is within content area (inside borders)
    let content_x = results_area.x + 1;  // +1 for left border
    let content_y = results_area.y + 1;  // +1 for top border
    let content_width = results_area.width.saturating_sub(2);  // -2 for borders
    let content_height = results_area.height.saturating_sub(2); // -2 for borders

    // Check bounds
    if mouse_col < content_x || mouse_col >= content_x + content_width {
        return None;
    }
    if mouse_row < content_y || mouse_row >= content_y + content_height {
        return None;
    }

    // Calculate relative position within content
    let relative_row = mouse_row - content_y;
    let relative_col = mouse_col - content_x;

    // Convert to absolute document position
    let absolute_line = scroll.offset as u32 + relative_row as u32;

    // Check if line exists in document
    if absolute_line >= text.lines.len() as u32 {
        // Clicked below content - clamp to last line
        let last_line = text.lines.len().saturating_sub(1) as u32;
        let last_line_len = line_char_count(text, last_line);
        return Some(TextPosition {
            line: last_line,
            col: last_line_len,
        });
    }

    // Calculate character column accounting for horizontal scroll
    let absolute_col = scroll.h_offset + relative_col;

    // Clamp to line length
    let line_len = line_char_count(text, absolute_line);
    let clamped_col = absolute_col.min(line_len);

    Some(TextPosition {
        line: absolute_line,
        col: clamped_col,
    })
}

/// Count the number of displayed characters in a line
/// This counts actual characters, not bytes, handling multi-byte UTF-8
fn line_char_count(text: &Text<'_>, line_idx: u32) -> u16 {
    text.lines
        .get(line_idx as usize)
        .map(|line| {
            line.spans.iter()
                .map(|span| span.content.chars().count())
                .sum::<usize>() as u16
        })
        .unwrap_or(0)
}

/// Get the character at a specific position for bounds checking
pub fn char_at_position(text: &Text<'_>, line: u32, col: u16) -> Option<char> {
    let line_content = text.lines.get(line as usize)?;
    let mut char_idx = 0u16;

    for span in &line_content.spans {
        for ch in span.content.chars() {
            if char_idx == col {
                return Some(ch);
            }
            char_idx += 1;
        }
    }

    None
}
```

### Edge Cases to Handle

1. **Click below content**: Clamp to last line, end of line
2. **Click right of line content**: Clamp to line length
3. **Click in border area**: Return None
4. **Click in scrollbar area**: Return None (handled by region hit test)
5. **Horizontal scroll offset**: Add `h_offset` to relative column
6. **Multi-byte characters**: Count chars, not bytes

### Test File: `src/results/selection_coords_tests.rs`

Test cases:
1. Click in middle of content - correct position
2. Click on first character - (line, 0)
3. Click past end of line - clamps to line length
4. Click below content - clamps to last line
5. Click in top border - returns None
6. Click in left border - returns None
7. Horizontal scroll offset applied correctly
8. Vertical scroll offset applied correctly
9. Multi-byte UTF-8 characters counted correctly

---

## Phase 3: Mouse Event Handling (Click & Drag)

**Goal:** Capture mouse down, drag, and up events to create and update selections.

**Dependency:** Phase 1 (SelectionState), Phase 2 (coordinate mapping)

### New File: `src/app/results_selection_events.rs`

```rust
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use crate::app::App;
use crate::layout::Region;
use crate::results::selection_coords::{screen_to_text_position, TextPosition};

/// Handle mouse down in results pane - start selection
pub fn handle_mouse_down(app: &mut App, mouse: MouseEvent) {
    let Some(results_area) = app.layout_regions.results_pane else {
        return;
    };

    let Some(query_state) = &app.query else {
        return;
    };

    let Some(text) = &query_state.last_successful_result_rendered else {
        return;
    };

    if let Some(pos) = screen_to_text_position(
        mouse.column,
        mouse.row,
        results_area,
        &app.results_scroll,
        text,
    ) {
        app.results_selection.start_selection(pos.line, pos.col);
    }
}

/// Handle mouse drag in results pane - update selection
pub fn handle_mouse_drag(app: &mut App, mouse: MouseEvent) {
    if !app.results_selection.is_dragging() {
        return;
    }

    let Some(results_area) = app.layout_regions.results_pane else {
        return;
    };

    let Some(query_state) = &app.query else {
        return;
    };

    let Some(text) = &query_state.last_successful_result_rendered else {
        return;
    };

    // Even if mouse is outside content, we still want to update selection
    // For edge positions (outside viewport), we handle auto-scroll in Phase 5

    let pos = screen_to_text_position_extended(
        mouse.column,
        mouse.row,
        results_area,
        &app.results_scroll,
        text,
    );

    app.results_selection.update_selection(pos.line, pos.col);
}

/// Handle mouse up - finish selection
pub fn handle_mouse_up(app: &mut App, _mouse: MouseEvent) {
    app.results_selection.finish_selection();
}

/// Extended coordinate mapping that handles positions outside viewport
/// Used during drag to allow selection beyond visible area
fn screen_to_text_position_extended(
    mouse_col: u16,
    mouse_row: u16,
    results_area: Rect,
    scroll: &ScrollState,
    text: &Text<'_>,
) -> TextPosition {
    let content_y = results_area.y + 1;
    let content_height = results_area.height.saturating_sub(2);
    let total_lines = text.lines.len() as u32;

    // Handle vertical position
    let line = if mouse_row < content_y {
        // Above viewport - use scroll offset or 0
        scroll.offset.saturating_sub(1) as u32
    } else if mouse_row >= content_y + content_height {
        // Below viewport - use last visible line + 1 or max
        ((scroll.offset as u32) + content_height as u32).min(total_lines.saturating_sub(1))
    } else {
        // Within viewport
        scroll.offset as u32 + (mouse_row - content_y) as u32
    };

    let line = line.min(total_lines.saturating_sub(1));

    // Handle horizontal position
    let content_x = results_area.x + 1;
    let col = if mouse_col < content_x {
        0
    } else {
        let relative_col = mouse_col - content_x;
        let absolute_col = scroll.h_offset + relative_col;
        let line_len = line_char_count(text, line);
        absolute_col.min(line_len)
    };

    TextPosition { line, col }
}
```

### Integration with Mouse Event Router

**File:** `src/app/mouse_events.rs`

Update `handle_mouse_event()`:

```rust
pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    let region = app.layout_regions.region_at(mouse.column, mouse.row);

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if region == Some(Region::ResultsPane) {
                // Start text selection
                results_selection_events::handle_mouse_down(app, mouse);
                // Also handle focus
                mouse_click::handle_click(app, region, mouse);
            } else {
                // Clear any existing selection when clicking elsewhere
                app.results_selection.clear();
                mouse_click::handle_click(app, region, mouse);
            }
        }

        MouseEventKind::Drag(MouseButton::Left) => {
            if app.results_selection.is_dragging() {
                results_selection_events::handle_mouse_drag(app, mouse);
            } else {
                mouse_hover::handle_hover(app, region, mouse);
            }
        }

        MouseEventKind::Up(MouseButton::Left) => {
            if app.results_selection.is_dragging() {
                results_selection_events::handle_mouse_up(app, mouse);
            }
        }

        // ... existing scroll handlers
    }
}
```

### Test File: `src/app/results_selection_events_tests.rs`

Test cases:
1. Mouse down starts selection at correct position
2. Mouse drag updates selection bounds
3. Mouse up finishes selection
4. Click outside results pane clears selection
5. Drag outside viewport calculates extended positions
6. Selection persists after mouse up for copying

---

## Phase 4: Selection Rendering (Visual Highlight)

**Goal:** Render selected text with a distinct background color.

**Dependency:** Phase 1 (SelectionBounds), Phase 3 (selection events populate state)

### Approach

Follow the pattern used for search highlighting in `results_render.rs`. Selection highlighting should:
1. Only process visible lines (viewport optimization)
2. Apply background style to selected character ranges
3. Preserve existing text styles (colors, bold, etc.)
4. Layer on top of search highlights if both active

### Style Constants

```rust
// In src/results/results_render.rs or selection module
use ratatui::style::{Color, Modifier, Style};

const SELECTION_STYLE: Style = Style::new()
    .bg(Color::Rgb(68, 68, 120))  // Muted blue background
    .add_modifier(Modifier::REVERSED);  // Or use reversed for better terminal compat
```

### Implementation in `src/results/results_render.rs`

Add new function `apply_selection_highlighting()`:

```rust
fn apply_selection_highlighting<'a>(
    text: Text<'a>,
    selection: Option<&SelectionBounds>,
    scroll_offset: u16,
    viewport_height: u16,
) -> Text<'a> {
    let Some(sel) = selection else {
        return text;
    };

    let viewport_start = scroll_offset as u32;
    let viewport_end = viewport_start + viewport_height as u32;

    // Only process if selection overlaps viewport
    if sel.end_line < viewport_start || sel.start_line >= viewport_end {
        return text;
    }

    let mut new_lines = Vec::with_capacity(text.lines.len());

    for (line_idx, line) in text.lines.into_iter().enumerate() {
        let line_num = line_idx as u32;

        if let Some((start_col, end_col)) = sel.columns_for_line(line_num, line_width(&line)) {
            // This line has selection - apply highlighting
            let highlighted = apply_selection_to_line(line, start_col, end_col);
            new_lines.push(highlighted);
        } else {
            // No selection on this line
            new_lines.push(line);
        }
    }

    Text::from(new_lines)
}

fn apply_selection_to_line(line: Line<'_>, start_col: u16, end_col: u16) -> Line<'_> {
    let mut new_spans = Vec::new();
    let mut char_pos = 0u16;

    for span in line.spans {
        let span_start = char_pos;
        let span_len = span.content.chars().count() as u16;
        let span_end = span_start + span_len;

        if span_end <= start_col || span_start >= end_col {
            // Span entirely outside selection
            new_spans.push(span);
        } else if span_start >= start_col && span_end <= end_col {
            // Span entirely inside selection
            new_spans.push(Span::styled(
                span.content,
                span.style.patch(SELECTION_STYLE),
            ));
        } else {
            // Span partially overlaps - need to split
            let chars: Vec<char> = span.content.chars().collect();

            // Before selection
            if span_start < start_col {
                let before: String = chars[..(start_col - span_start) as usize].iter().collect();
                new_spans.push(Span::styled(before, span.style));
            }

            // Selection portion
            let sel_start = (start_col.saturating_sub(span_start)) as usize;
            let sel_end = ((end_col - span_start) as usize).min(chars.len());
            let selected: String = chars[sel_start..sel_end].iter().collect();
            new_spans.push(Span::styled(selected, span.style.patch(SELECTION_STYLE)));

            // After selection
            if span_end > end_col {
                let after: String = chars[(end_col - span_start) as usize..].iter().collect();
                new_spans.push(Span::styled(after, span.style));
            }
        }

        char_pos = span_end;
    }

    Line::from(new_spans)
}

fn line_width(line: &Line<'_>) -> u16 {
    line.spans.iter()
        .map(|s| s.content.chars().count() as u16)
        .sum()
}
```

### Integration into Render Pipeline

In `render_pane()`, apply selection highlighting after search highlighting:

```rust
let mut text = last_rendered.clone();

// Apply search highlights (existing)
if let Some(search_matches) = &app.search.matches {
    text = apply_search_highlights(text, search_matches, ...);
}

// Apply selection highlights (new)
if let Some(selection) = app.results_selection.selection() {
    text = apply_selection_highlighting(text, Some(selection), scroll.offset, viewport_height);
}
```

### Test File: `src/results/selection_render_tests.rs`

Test cases:
1. No selection - text unchanged
2. Single line selection - correct characters highlighted
3. Multi-line selection - all lines processed correctly
4. Selection at line start/end
5. Selection spanning entire line
6. Partial span overlap handled correctly
7. Selection outside viewport - no processing
8. Selection + search highlight coexistence

---

## Phase 5: Auto-Scroll During Drag

**Goal:** Automatically scroll the viewport when dragging selection to edges.

**Dependency:** Phase 3 (drag detection)

### Auto-Scroll Behavior

When user drags to viewport edges during selection:
- **Near top edge**: Scroll up to reveal more content above
- **Near bottom edge**: Scroll down to reveal more content below
- **Near left edge**: Scroll left (horizontal)
- **Near right edge**: Scroll right (horizontal)

### Configuration Constants

```rust
// Pixels/rows from edge to trigger auto-scroll
const SCROLL_EDGE_THRESHOLD: u16 = 2;

// Lines to scroll per tick when at edge
const SCROLL_SPEED: u16 = 1;
```

### Implementation in `src/app/results_selection_events.rs`

Add to `handle_mouse_drag()`:

```rust
pub fn handle_mouse_drag(app: &mut App, mouse: MouseEvent) {
    if !app.results_selection.is_dragging() {
        return;
    }

    let Some(results_area) = app.layout_regions.results_pane else {
        return;
    };

    // Check for auto-scroll first
    auto_scroll_if_needed(app, mouse, results_area);

    // Then update selection (after scroll so position is accurate)
    // ... existing selection update code
}

fn auto_scroll_if_needed(
    app: &mut App,
    mouse: MouseEvent,
    results_area: Rect,
) {
    let content_y_start = results_area.y + 1;
    let content_y_end = results_area.y + results_area.height.saturating_sub(1);
    let content_x_start = results_area.x + 1;
    let content_x_end = results_area.x + results_area.width.saturating_sub(2); // Account for scrollbar

    // Vertical auto-scroll
    if mouse.row < content_y_start + SCROLL_EDGE_THRESHOLD {
        // Near top or above - scroll up
        if app.results_scroll.offset > 0 {
            app.results_scroll.scroll_up(SCROLL_SPEED as usize);
        }
    } else if mouse.row >= content_y_end.saturating_sub(SCROLL_EDGE_THRESHOLD) {
        // Near bottom or below - scroll down
        let max_offset = app.results_scroll.max_offset;
        if app.results_scroll.offset < max_offset {
            app.results_scroll.scroll_down(SCROLL_SPEED as usize);
        }
    }

    // Horizontal auto-scroll
    if mouse.column < content_x_start + SCROLL_EDGE_THRESHOLD {
        // Near left - scroll left
        if app.results_scroll.h_offset > 0 {
            app.results_scroll.h_offset = app.results_scroll.h_offset.saturating_sub(SCROLL_SPEED);
        }
    } else if mouse.column >= content_x_end.saturating_sub(SCROLL_EDGE_THRESHOLD) {
        // Near right - scroll right
        let max_h = app.results_scroll.max_h_offset;
        if app.results_scroll.h_offset < max_h {
            app.results_scroll.h_offset = (app.results_scroll.h_offset + SCROLL_SPEED).min(max_h);
        }
    }
}
```

### Edge Cases

1. **Already at scroll boundary**: Don't scroll further
2. **Mouse leaves window entirely**: Continue scrolling in direction of exit
3. **Fast mouse movement**: Scroll speed is fixed, selection catches up

### Test Cases

1. Drag to top edge - scrolls up
2. Drag to bottom edge - scrolls down
3. Drag above viewport (row < area.y) - scrolls up
4. Drag below viewport - scrolls down
5. At scroll offset 0 - doesn't scroll up further
6. At max scroll offset - doesn't scroll down further
7. Horizontal scroll similar tests

---

## Phase 6: Copy Selected Text to Clipboard

**Goal:** Copy selected text with `Y` or `c` key.

**Dependency:** Phase 1 (SelectionBounds), Phase 3 (selection exists)

### Text Extraction

Extract selected text from the unformatted result (plain text without ANSI codes).

### New Function in `src/clipboard/clipboard_events.rs`

```rust
/// Copy selected text to clipboard
pub fn handle_copy_selection(app: &mut App) -> bool {
    let Some(selection) = app.results_selection.selection() else {
        return false;
    };

    let Some(query_state) = &app.query else {
        return false;
    };

    let Some(unformatted) = &query_state.last_successful_result_unformatted else {
        return false;
    };

    let selected_text = extract_selected_text(unformatted, selection);

    if selected_text.is_empty() {
        return false;
    }

    match copy_to_clipboard(&selected_text, app.clipboard_backend) {
        Ok(()) => {
            app.notification.show("Copied selection!");
            // Optionally clear selection after copy
            // app.results_selection.clear();
            true
        }
        Err(_) => {
            app.notification.show_error("Failed to copy selection");
            false
        }
    }
}

fn extract_selected_text(text: &str, selection: &SelectionBounds) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result = String::new();

    for line_idx in selection.start_line..=selection.end_line {
        let Some(line) = lines.get(line_idx as usize) else {
            continue;
        };

        let chars: Vec<char> = line.chars().collect();

        let start_col = if line_idx == selection.start_line {
            selection.start_col as usize
        } else {
            0
        };

        let end_col = if line_idx == selection.end_line {
            (selection.end_col as usize).min(chars.len())
        } else {
            chars.len()
        };

        if start_col < chars.len() {
            let selected: String = chars[start_col..end_col].iter().collect();
            result.push_str(&selected);
        }

        // Add newline between lines (but not after last)
        if line_idx < selection.end_line {
            result.push('\n');
        }
    }

    result
}
```

### Key Binding Integration

**File:** `src/results/results_events.rs`

```rust
pub fn handle_results_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // Existing 'y' handler for yanking entire result
        KeyCode::Char('y') if !app.search.is_visible() => {
            // If selection exists, copy selection; otherwise copy all
            if app.results_selection.has_selection() {
                clipboard_events::handle_copy_selection(app)
            } else {
                clipboard_events::handle_yank_key(app, app.clipboard_backend)
            }
        }

        // New 'c' handler for copy (selection only)
        KeyCode::Char('c') if !app.search.is_visible() => {
            if app.results_selection.has_selection() {
                clipboard_events::handle_copy_selection(app)
            } else {
                false  // 'c' does nothing without selection
            }
        }

        // Clear selection on Escape
        KeyCode::Esc => {
            if app.results_selection.has_selection() {
                app.results_selection.clear();
                true
            } else {
                // ... existing Esc handling
            }
        }

        // ... other handlers
    }
}
```

### Behavior Notes

1. **`Y` key**: If selection exists, copy selection. Otherwise, copy entire result (existing behavior).
2. **`c` key**: Only copies if selection exists. No-op otherwise.
3. **`Esc` key**: Clears selection (in addition to other Esc behaviors).
4. **Notification**: Shows "Copied selection!" on success.

### Test Cases

1. Copy single line selection
2. Copy multi-line selection
3. Copy with no selection - `c` does nothing, `Y` copies all
4. Selection at document boundaries
5. Empty selection returns empty string
6. Notification shown on success

---

## Phase 7: Integration & Edge Cases

**Goal:** Handle edge cases and ensure smooth integration.

**Dependency:** All previous phases

### Edge Cases to Address

#### 1. Empty Results

```rust
// In handle_mouse_down
if text.lines.is_empty() {
    return;  // Nothing to select
}
```

#### 2. Selection Across Scroll Boundaries

When selection spans more lines than viewport:
- Selection bounds stored in absolute coordinates
- Rendering only highlights visible portion
- Copy extracts full selection regardless of viewport

#### 3. Window Resize During Selection

```rust
// In resize handler
// Selection bounds remain valid (absolute coordinates)
// Just need to re-render with new viewport
```

#### 4. New Query Clears Selection

```rust
// When query changes
pub fn update_query_result(&mut self, ...) {
    // Clear selection when results change
    self.results_selection.clear();
    // ... rest of update
}
```

#### 5. Focus Change Behavior

```rust
// When focus leaves results pane
// Option A: Clear selection (simpler)
// Option B: Keep selection visible but not editable (chosen)
// We'll keep selection visible so user can switch focus and still copy
```

#### 6. Search Highlight + Selection Overlap

Both can be visible simultaneously:
- Search highlights use yellow/orange background
- Selection uses blue/inverse
- Selection applied after search, so it takes precedence visually

### Keyboard Navigation Clears Selection

When user uses `j`/`k`/`h`/`l` to navigate:
```rust
KeyCode::Char('j') | KeyCode::Char('k') | ... => {
    app.results_selection.clear();
    // ... handle scroll
}
```

### Help Text Update

Update help popup to include new keybindings:
- "Mouse drag: Select text"
- "Y/c: Copy selection (or entire result if no selection)"
- "Esc: Clear selection"

---

## Files Summary

### New Files

| File | Description | Lines (est.) |
|------|-------------|--------------|
| `src/results/selection.rs` | SelectionBounds, SelectionState | ~120 |
| `src/results/selection_tests.rs` | Unit tests for selection | ~150 |
| `src/results/selection_coords.rs` | Screen-to-text coordinate mapping | ~80 |
| `src/results/selection_coords_tests.rs` | Coordinate mapping tests | ~120 |
| `src/app/results_selection_events.rs` | Mouse event handlers for selection | ~100 |
| `src/app/results_selection_events_tests.rs` | Event handler tests | ~150 |
| `src/results/selection_render_tests.rs` | Rendering tests | ~100 |

### Modified Files

| File | Changes |
|------|---------|
| `src/app/app_state.rs` | Add `results_selection: SelectionState` field |
| `src/app/mouse_events.rs` | Route drag/click events to selection handlers |
| `src/results/results_render.rs` | Add selection highlighting in render pipeline |
| `src/results/results_events.rs` | Add `c` key binding, modify `Y` behavior |
| `src/clipboard/clipboard_events.rs` | Add `handle_copy_selection()` function |
| `src/results.rs` | Export new selection modules |
| `src/app.rs` | Export new selection events module |

---

## Manual Testing Checklist

### Phase 1-2: Foundation
- [ ] Build succeeds with new types
- [ ] Unit tests pass for coordinate conversion

### Phase 3: Mouse Events
- [ ] Click in results pane starts selection
- [ ] Drag updates selection in real-time
- [ ] Release finalizes selection
- [ ] Click outside clears selection

### Phase 4: Rendering
- [ ] Selected text has visible highlight
- [ ] Highlight respects line boundaries
- [ ] Multi-line selection renders correctly
- [ ] Selection visible with horizontal scroll

### Phase 5: Auto-Scroll
- [ ] Drag to bottom edge scrolls down
- [ ] Drag to top edge scrolls up
- [ ] Scroll stops at document boundaries
- [ ] Selection updates as viewport scrolls

### Phase 6: Copy
- [ ] `c` key copies selection to clipboard
- [ ] `Y` key copies selection when exists
- [ ] `Y` key copies all when no selection
- [ ] "Copied selection!" notification appears
- [ ] Pasted content matches selection exactly

### Phase 7: Edge Cases
- [ ] Selection works with empty results (no crash)
- [ ] Selection works at document boundaries
- [ ] Selection persists across focus changes
- [ ] New query clears selection
- [ ] Keyboard navigation clears selection
- [ ] Selection + search highlight both visible

---

## Open Questions

1. **Should selection persist after copy?**
   - Current plan: Yes, keep selection visible
   - Alternative: Clear after successful copy

2. **Should `Esc` clear selection or just exit to input?**
   - Current plan: Clear selection first, then normal Esc behavior
   - Could require two Esc presses

3. **Horizontal auto-scroll speed?**
   - Single character per tick vs multiple
   - May need tuning based on feel

4. **Visual style for selection?**
   - Blue background with reverse
   - Could use terminal's native selection style if detectable

5. **Double-click to select word?**
   - Not in initial scope
   - Could be Phase 8 enhancement

---

## References

- Results rendering: `src/results/results_render.rs:1-564`
- Search highlighting pattern: `src/results/results_render.rs` (apply_highlights functions)
- Mouse event router: `src/app/mouse_events.rs`
- Clipboard handling: `src/clipboard/clipboard_events.rs`
- Scroll state: `src/scroll/scroll_state.rs`
- Layout regions: `src/layout/layout_regions.rs`
