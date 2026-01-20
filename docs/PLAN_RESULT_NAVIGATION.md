# Result Navigation Improvements - Implementation Plan

## Implementation Guidelines

1. **Commit after each phase** - Each phase should be committed separately with a descriptive commit message
2. **100% test coverage** - All new code must have complete test coverage before committing
3. **Manual TUI testing** - Verify functionality manually before marking phase complete
4. **Update docs for deviations** - Any changes made during implementation that differ from the original plan must be documented

## Phase Checklist

- [ ] Phase 1: Position Indicator in Title Bar
- [ ] Phase 2: Neovim-Style Search Navigation
- [ ] Phase 3: Visual Scrollbar

---

## Overview

Improve result pane navigation by adding visual orientation features and better search navigation behavior.

## Architecture Decisions

### Position Indicator Format
- **Format**: `Results | L45-95/1234 (4%)`
- `L{start}-{end}` = visible line range (1-indexed)
- `/{total}` = total line count
- `({percent}%)` = scroll percentage

### Search Navigation (Neovim-Style)
- **Match visible**: Don't scroll at all
- **Match off-screen**: Scroll minimally so match appears 3 lines from viewport edge
- **No centering**: Unlike current aggressive centering behavior
- Margin constant: `SCROLL_MARGIN = 3`

### Scrollbar Behavior
- Only show when `line_count > viewport_height`
- Use Ratatui's `Scrollbar` widget with minimal style (no arrows)
- Renders on right edge of results pane

---

## Phased Implementation

Each phase delivers the smallest testable feature. Manual TUI testing after each phase.

### Phase 1: Position Indicator in Title Bar

**Goal**: Show line range and percentage in results pane title.

**Files to modify**:
- `src/results/results_render.rs` - Add indicator to block title

**Implementation**:
```rust
fn format_position_indicator(scroll: &ScrollState, line_count: u32) -> String {
    if line_count == 0 {
        return String::new();
    }
    let start = scroll.offset as u32 + 1;
    let end = (scroll.offset as u32 + scroll.viewport_height as u32).min(line_count);
    let percentage = (scroll.offset as u32 * 100) / line_count.max(1);
    format!("L{}-{}/{} ({}%)", start, end, line_count, percentage)
}
```

**Manual test**: Open large JSON, scroll around, verify title shows correct line range and percentage.

**Tests**: Unit tests for position indicator formatting (edge cases: empty, single line, at top, at bottom, middle).

---

### Phase 2: Neovim-Style Search Navigation

**Goal**: Minimal scroll with 3-line margin instead of aggressive centering.

**Files to modify**:
- `src/search/search_events/scroll.rs` - Replace centering logic with minimal scroll

**Implementation**:
```rust
const SCROLL_MARGIN: u16 = 3;

pub(super) fn scroll_to_match(app: &mut App) {
    let Some(current_match) = app.search.current_match() else {
        return;
    };

    let target_line = current_match.line.min(u16::MAX as u32) as u16;
    let viewport_height = app.results_scroll.viewport_height;
    let current_offset = app.results_scroll.offset;
    let max_offset = app.results_scroll.max_offset;

    if viewport_height == 0 || max_offset == 0 {
        return;
    }

    let visible_start = current_offset;
    let visible_end = current_offset.saturating_add(viewport_height);

    // Already visible - don't scroll
    if target_line >= visible_start && target_line < visible_end {
        return;
    }

    let new_offset = if target_line < visible_start {
        // Match above viewport - place 3 lines from top
        target_line.saturating_sub(SCROLL_MARGIN)
    } else {
        // Match below viewport - place 3 lines from bottom
        target_line.saturating_sub(viewport_height.saturating_sub(SCROLL_MARGIN + 1))
    };

    app.results_scroll.offset = new_offset.min(max_offset);
}
```

**Manual test**:
1. Search for pattern with multiple matches spread across document
2. Press `n`/`N` to navigate between matches
3. Verify: no scroll when next match is already visible
4. Verify: minimal scroll (match appears ~3 lines from edge) when match is off-screen

**Tests**: Unit tests for scroll behavior (match visible - no scroll, match above - scroll up with margin, match below - scroll down with margin).

---

### Phase 3: Visual Scrollbar

**Goal**: Render scrollbar on right edge when content exceeds viewport.

**Files to modify**:
- `src/results/results_render.rs` - Add scrollbar rendering

**Implementation**:
```rust
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

fn render_scrollbar(f: &mut Frame, area: Rect, scroll: &ScrollState, line_count: u32) {
    if line_count <= scroll.viewport_height as u32 {
        return;
    }

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None);

    let mut scrollbar_state = ScrollbarState::new(line_count as usize)
        .position(scroll.offset as usize)
        .viewport_content_length(scroll.viewport_height as usize);

    f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}
```

**Manual test**: Open JSON with content longer than viewport, verify scrollbar appears and tracks position correctly when scrolling.

**Tests**: Snapshot tests for scrollbar rendering (visible vs hidden based on content size).

---

## Critical Files Reference

| Purpose | File Path |
|---------|-----------|
| Results rendering | `src/results/results_render.rs` |
| Scroll state | `src/scroll.rs` |
| Search scroll logic | `src/search/search_events/scroll.rs` |
| Scroll tests | `src/scroll_tests.rs` |

## Verification Plan

After each phase:
1. Run `cargo build --release` - must pass
2. Run `cargo clippy --all-targets --all-features` - zero warnings
3. Run `cargo fmt --all --check` - zero formatting issues
4. Run `cargo test` - all tests pass
5. Manual TUI testing with explicit test steps
6. Verify 100% test coverage for new code
