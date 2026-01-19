# Result Navigation Improvement Planning Document

---

## Implementation Status

| ID | Improvement | Priority | Complexity | Status |
|----|-------------|----------|------------|--------|
| **N1** | Position indicator in title bar | High | Easy | Pending |
| **N2** | Mouse wheel scrolling | High | Easy | Pending |
| **N3** | Search wrap-around feedback | Medium | Easy | Pending |
| **N4** | Smart scroll-to-match | Medium | Easy | Pending |
| **N5** | Go-to-line command | Medium | Medium | Pending |
| **N6** | Percentage/middle jump | Low | Easy | Pending |
| **N7** | Scroll acceleration | Low | Medium | Pending |
| **N8** | Visual scrollbar | Medium | Medium | Pending |
| **N9** | Boundary feedback | Low | Easy | Pending |

---

## Current Implementation Analysis

### Architecture Overview

**Core Files:**
| File | Purpose | Lines |
|------|---------|-------|
| `src/scroll.rs` | ScrollState struct and scroll operations | ~97 |
| `src/scroll_tests.rs` | Scroll unit tests | ~161 |
| `src/results/results_events.rs` | Key binding handlers | ~80 |
| `src/results/results_render.rs` | Rendering with viewport slicing | ~436 |
| `src/search/search_state.rs` | Match tracking and navigation | ~195 |
| `src/search/search_events/scroll.rs` | Auto-scroll to match | ~69 |

### Data Structures

```rust
// src/scroll.rs
pub struct ScrollState {
    pub offset: u16,           // Current vertical scroll position
    pub max_offset: u16,       // Maximum scrollable distance
    pub viewport_height: u16,  // Visible area height
    pub h_offset: u16,         // Horizontal scroll position
    pub max_h_offset: u16,     // Maximum horizontal scroll
    pub viewport_width: u16,   // Visible area width
}
```

**Limitation**: All values are `u16`, limiting addressable range to 65,535 lines/columns.

### Current Key Bindings

| Keys | Action | Scroll Amount |
|------|--------|---------------|
| `j` / `Down` | Scroll down | 1 line |
| `k` / `Up` | Scroll up | 1 line |
| `J` | Fast scroll down | 10 lines |
| `K` | Fast scroll up | 10 lines |
| `PageDown` / `Ctrl+D` | Page down | Half viewport |
| `PageUp` / `Ctrl+U` | Page up | Half viewport |
| `g` / `Home` | Jump to top | Start |
| `G` / `End` | Jump to bottom | End |
| `h` / `Left` | Scroll left | 1 column |
| `l` / `Right` | Scroll right | 1 column |
| `H` | Fast scroll left | 10 columns |
| `L` | Fast scroll right | 10 columns |
| `0` / `^` | Jump to left edge | Column 0 |
| `$` | Jump to right edge | Max width |
| `/` | Open search | - |
| `n` | Next match | To match |
| `N` | Previous match | To match |

### Performance Optimizations (Already Implemented)

1. **Pre-rendered caching**: Results rendered once in worker thread, cached in `QueryState`
2. **Viewport slicing**: Only visible lines cloned per frame (`results_render.rs:164-177`)
3. **Line count caching**: `cached_line_count` computed once per result
4. **O(1) match lookup**: `matches_by_line: HashMap<u32, Vec<usize>>`

**Memory pattern for 100K line file:**
- Full cache: ~5-10MB (one-time)
- Per-frame allocation: ~1-3KB (visible viewport only)

---

## Issues Identified

### Issue 1: No Position Indicator

**Severity**: High

**Problem**: Users have no way to know where they are in large files. No line numbers, no percentage, no scrollbar.

**User impact**: Disorienting when navigating large JSON outputs. Common question: "Where am I?"

**Current state**: Title bar shows only `Results` or `Results | Objects (5)` for stats.

### Issue 2: No Mouse Support

**Severity**: Medium

**Problem**: No mouse wheel scrolling despite terminal support via crossterm.

**Evidence**: `grep -r "mouse|Mouse"` returns only changelog/fixture references.

**User impact**: Terminal users expect mouse wheel to work. Forces keyboard-only navigation.

### Issue 3: Search Wrap-Around Confusion

**Severity**: Medium

**Problem**: When pressing `n` at the last match, navigation silently wraps to the first match. Same for `N` at first match.

**Location**: `search_state.rs:138` and `search_state.rs:147-150`

```rust
// Current implementation - silent wrap
self.current_index = (self.current_index + 1) % self.matches.len();
```

**User impact**: Sudden jump to document start/end without indication.

### Issue 4: Aggressive Scroll-to-Match Centering

**Severity**: Low-Medium

**Problem**: `scroll_to_match()` always centers the target, even when already nearby.

**Location**: `search/search_events/scroll.rs:22-29`

```rust
if target_line < visible_start || target_line >= visible_end {
    // Line not visible, scroll to center it
    let half_viewport = viewport_height / 2;
    let new_offset = target_line.saturating_sub(half_viewport);
    // ...
}
```

**User impact**: Pressing `n` repeatedly causes unnecessary viewport jumps even when next match is just a few lines away.

### Issue 5: No Go-to-Line Feature

**Severity**: Medium

**Problem**: No way to jump directly to a specific line number.

**User impact**: When error messages reference line numbers, users must scroll manually.

### Issue 6: Fixed Scroll Increments

**Severity**: Low

**Problem**: Scroll amounts are hardcoded (1, 10, half-viewport). No acceleration for held keys.

**Location**: `results_events.rs` - all scroll amounts are literals.

**User impact**: Traversing medium-sized files (1000-5000 lines) is tedious.

### Issue 7: u16 Limitation for Large Files

**Severity**: Low (rare edge case)

**Problem**: Files exceeding 65,535 lines cannot be fully addressed.

**Location**: `scroll.rs:27-29` - offset saturates to `u16::MAX`

**User impact**: Cannot jump to arbitrary positions in very large files.

---

## Improvement Proposals

### N1: Position Indicator in Title Bar

**Priority**: High | **Complexity**: Easy

**Description**: Display scroll position in the results pane border/title.

**Proposed format options**:
```
Results | L45-95/1234 (4%)      # Compact
Results | Lines 45-95 of 1,234  # Verbose
Results (Top)                   # At boundaries
Results (Bottom)                # At boundaries
```

**Implementation approach**:
```rust
// In results_render.rs, when rendering the block title
fn format_position_indicator(scroll: &ScrollState, line_count: u32) -> String {
    let start = scroll.offset as u32 + 1;
    let end = (scroll.offset as u32 + scroll.viewport_height as u32).min(line_count);
    let percentage = if line_count > 0 {
        (scroll.offset as u32 * 100) / line_count
    } else {
        0
    };

    if scroll.offset == 0 {
        "Top".to_string()
    } else if scroll.offset >= scroll.max_offset {
        "Bottom".to_string()
    } else {
        format!("L{}-{}/{} ({}%)", start, end, line_count, percentage)
    }
}
```

**Performance**: O(1) - uses existing cached values.

**Files to modify**:
- `src/results/results_render.rs` - Add indicator to block title

---

### N2: Mouse Wheel Scrolling

**Priority**: High | **Complexity**: Easy

**Description**: Enable mouse wheel events for vertical scrolling.

**Implementation approach**:
```rust
// In results_events.rs or app event handler
use crossterm::event::{MouseEvent, MouseEventKind};

fn handle_mouse_event(app: &mut App, event: MouseEvent) {
    match event.kind {
        MouseEventKind::ScrollDown => {
            app.results_scroll.scroll_down(3); // 3 lines per tick
        }
        MouseEventKind::ScrollUp => {
            app.results_scroll.scroll_up(3);
        }
        _ => {}
    }
}
```

**Prerequisites**:
- Enable mouse capture in terminal setup (crossterm)
- Route mouse events to results pane when focused

**Configuration consideration**: Scroll amount (3 lines) could be configurable.

**Files to modify**:
- `src/app.rs` or main event loop - Enable mouse capture
- `src/results/results_events.rs` - Handle mouse events

---

### N3: Search Wrap-Around Feedback

**Priority**: Medium | **Complexity**: Easy

**Description**: Provide visual feedback when search wraps from last to first match (or vice versa).

**Proposed approaches**:

**Option A: Transient message in search bar**
```
(5/5) → (1/5) ↻ Wrapped
```
Display for 500ms, then revert to normal `(1/5)`.

**Option B: Flash/highlight effect**
Briefly change search bar border color when wrapping.

**Option C: Status message**
Show "Search wrapped to beginning" in a status area.

**Implementation (Option A)**:
```rust
// In search_state.rs
pub struct SearchState {
    // ... existing fields
    wrap_indicator_until: Option<Instant>,
}

pub fn next_match(&mut self) -> Option<u32> {
    if self.matches.is_empty() {
        return None;
    }
    let was_last = self.current_index == self.matches.len() - 1;
    self.current_index = (self.current_index + 1) % self.matches.len();

    if was_last {
        self.wrap_indicator_until = Some(Instant::now() + Duration::from_millis(500));
    }

    self.matches.get(self.current_index).map(|m| m.line)
}

pub fn match_count_display(&self) -> String {
    if self.wrap_indicator_until.map(|t| Instant::now() < t).unwrap_or(false) {
        format!("({}/{}) ↻", self.current_index + 1, self.matches.len())
    } else {
        format!("({}/{})", self.current_index + 1, self.matches.len())
    }
}
```

**Files to modify**:
- `src/search/search_state.rs` - Add wrap tracking
- `src/search/search_render.rs` - Display indicator

---

### N4: Smart Scroll-to-Match (Minimal Scroll)

**Priority**: Medium | **Complexity**: Easy

**Description**: Only scroll when necessary; prefer minimal movement over centering.

**Current behavior**: Always center the match in viewport.

**Proposed behavior**:
1. If match is already visible, don't scroll at all
2. If match is just outside viewport, scroll minimally to show it
3. If match is far away, center it (current behavior)

**Implementation**:
```rust
// In search/search_events/scroll.rs
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

    // Calculate distance from viewport
    let distance_from_top = visible_start.saturating_sub(target_line);
    let distance_from_bottom = target_line.saturating_sub(visible_end);

    // Threshold for "minimal scroll" vs "center"
    let scroll_threshold = viewport_height / 2;

    let new_offset = if distance_from_top > 0 && distance_from_top <= scroll_threshold {
        // Just above viewport - scroll up minimally (with small margin)
        target_line.saturating_sub(2)
    } else if distance_from_bottom > 0 && distance_from_bottom <= scroll_threshold {
        // Just below viewport - scroll down minimally (with small margin)
        target_line.saturating_sub(viewport_height.saturating_sub(3))
    } else {
        // Far away - center the match
        let half_viewport = viewport_height / 2;
        target_line.saturating_sub(half_viewport)
    };

    app.results_scroll.offset = new_offset.min(max_offset);

    // ... horizontal scrolling unchanged
}
```

**Files to modify**:
- `src/search/search_events/scroll.rs`

---

### N5: Go-to-Line Command

**Priority**: Medium | **Complexity**: Medium

**Description**: Add ability to jump to a specific line number.

**Proposed key bindings**:
- `:` (vim-style) - Opens line input prompt
- `Ctrl+G` (alternative) - Opens line input prompt

**UI approach**: Reuse search bar pattern with different mode.

**Implementation sketch**:
```rust
// New enum variant in input mode
pub enum ResultsMode {
    Normal,
    Search,
    GoToLine,  // NEW
}

// In results_events.rs
KeyCode::Char(':') if !app.search.is_visible() => {
    app.results_mode = ResultsMode::GoToLine;
    app.goto_line_input.clear();
}

// Handle input
KeyCode::Enter if app.results_mode == ResultsMode::GoToLine => {
    if let Ok(line_num) = app.goto_line_input.parse::<u32>() {
        let target = (line_num.saturating_sub(1)).min(app.results_scroll.max_offset as u32);
        app.results_scroll.offset = target as u16;
    }
    app.results_mode = ResultsMode::Normal;
}
```

**Display**: Show `Go to line: 123_` at bottom of results pane.

**Files to modify**:
- `src/results/results_events.rs` - Key handling
- `src/results/results_render.rs` - Render input prompt
- `src/app.rs` - Add GoToLine state

---

### N6: Percentage/Middle Jump

**Priority**: Low | **Complexity**: Easy

**Description**: Add shortcuts to jump to percentage positions.

**Proposed key bindings**:
- `M` - Jump to middle of document (vim-style)
- `50%` pattern - Jump to percentage (if feasible)

**Implementation (M for middle)**:
```rust
// In results_events.rs
KeyCode::Char('M') => {
    let middle = app.results_scroll.max_offset / 2;
    app.results_scroll.offset = middle;
}
```

**Alternative - numeric prefix**: Would require tracking numeric input state, more complex.

**Files to modify**:
- `src/results/results_events.rs`

---

### N7: Scroll Acceleration

**Priority**: Low | **Complexity**: Medium

**Description**: Increase scroll speed when keys are held down.

**Implementation approach**:
```rust
// Track key repeat state
pub struct ScrollAcceleration {
    last_key: Option<KeyCode>,
    last_time: Instant,
    repeat_count: u32,
}

impl ScrollAcceleration {
    pub fn get_scroll_amount(&mut self, key: KeyCode, base_amount: u16) -> u16 {
        let now = Instant::now();

        if self.last_key == Some(key) && now.duration_since(self.last_time) < Duration::from_millis(150) {
            self.repeat_count += 1;
        } else {
            self.repeat_count = 0;
        }

        self.last_key = Some(key);
        self.last_time = now;

        // Accelerate: 1x -> 2x -> 5x -> 10x
        let multiplier = match self.repeat_count {
            0..=3 => 1,
            4..=8 => 2,
            9..=15 => 5,
            _ => 10,
        };

        base_amount.saturating_mul(multiplier)
    }
}
```

**Consideration**: Terminal key repeat behavior varies. May need tuning.

**Files to modify**:
- `src/scroll.rs` - Add acceleration logic
- `src/results/results_events.rs` - Apply acceleration

---

### N8: Visual Scrollbar

**Priority**: Medium | **Complexity**: Medium

**Description**: Render a minimal scrollbar on the right edge of results pane.

**Ratatui support**: `Scrollbar` widget available.

**Implementation sketch**:
```rust
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

// In results_render.rs
fn render_scrollbar(f: &mut Frame, area: Rect, scroll: &ScrollState, line_count: u32) {
    if line_count <= scroll.viewport_height as u32 {
        return; // No scrollbar needed
    }

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(Some("│"))
        .thumb_symbol("█");

    let mut scrollbar_state = ScrollbarState::new(line_count as usize)
        .position(scroll.offset as usize)
        .viewport_content_length(scroll.viewport_height as usize);

    f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}
```

**Consideration**: Takes 1 column of width from results area.

**Files to modify**:
- `src/results/results_render.rs`

---

### N9: Boundary Feedback

**Priority**: Low | **Complexity**: Easy

**Description**: Visual indication when hitting top/bottom boundaries.

**Proposed approaches**:

**Option A: Flash border**
Briefly change border color when at boundary.

**Option B: Text indicator**
Show `── TOP ──` or `── END ──` in title.

**Option C: Prevent over-scroll with feedback**
When pressing `j` at bottom, briefly show indicator.

**Implementation (Option B - simple)**:
```rust
// In results_render.rs title formatting
fn format_title(scroll: &ScrollState, line_count: u32) -> String {
    if scroll.offset == 0 && scroll.max_offset > 0 {
        "Results ── TOP".to_string()
    } else if scroll.offset >= scroll.max_offset && scroll.max_offset > 0 {
        "Results ── END".to_string()
    } else {
        "Results".to_string()
    }
}
```

**Files to modify**:
- `src/results/results_render.rs`

---

## Performance Considerations

All proposed improvements maintain O(viewport) complexity:

| Improvement | Computation | Memory | Notes |
|-------------|-------------|--------|-------|
| N1: Position indicator | O(1) | 0 | Uses existing cached values |
| N2: Mouse scroll | O(1) | 0 | Same as keyboard scroll |
| N3: Wrap feedback | O(1) | ~16 bytes | Instant timestamp |
| N4: Smart centering | O(1) | 0 | Comparison logic only |
| N5: Go-to-line | O(1) | ~32 bytes | Input string buffer |
| N6: Percentage jump | O(1) | 0 | Division only |
| N7: Scroll acceleration | O(1) | ~24 bytes | State tracking |
| N8: Visual scrollbar | O(1) | ~50 bytes | Ratatui widget |
| N9: Boundary feedback | O(1) | 0 | Comparison only |

**Critical constraint**: None of these require parsing, iterating, or re-rendering the full content.

---

## Testing Strategy

### Unit Tests

```rust
// scroll_tests.rs additions

#[test]
fn test_position_indicator_at_top() {
    let scroll = ScrollState::new(0, 100, 20, 0, 50, 80);
    assert_eq!(format_position_indicator(&scroll, 100), "Top");
}

#[test]
fn test_position_indicator_at_bottom() {
    let scroll = ScrollState::new(80, 80, 20, 0, 50, 80);
    assert_eq!(format_position_indicator(&scroll, 100), "Bottom");
}

#[test]
fn test_position_indicator_middle() {
    let scroll = ScrollState::new(40, 80, 20, 0, 50, 80);
    let indicator = format_position_indicator(&scroll, 100);
    assert!(indicator.contains("41-60"));
    assert!(indicator.contains("40%"));
}

#[test]
fn test_smart_scroll_already_visible() {
    // Match at line 50, viewport showing 40-60
    // Should not scroll
}

#[test]
fn test_smart_scroll_minimal_up() {
    // Match at line 38, viewport showing 40-60
    // Should scroll up minimally, not center
}

#[test]
fn test_smart_scroll_far_away_centers() {
    // Match at line 200, viewport showing 40-60
    // Should center the match
}

#[test]
fn test_scroll_acceleration_increases() {
    let mut accel = ScrollAcceleration::default();
    let key = KeyCode::Char('j');

    assert_eq!(accel.get_scroll_amount(key, 1), 1);
    // Simulate rapid presses
    for _ in 0..10 {
        std::thread::sleep(Duration::from_millis(50));
        accel.get_scroll_amount(key, 1);
    }
    assert!(accel.get_scroll_amount(key, 1) > 1);
}
```

### Integration Tests

```rust
#[test]
fn test_mouse_wheel_scrolls_results() {
    let mut app = create_app_with_json(large_json());

    // Simulate mouse wheel down
    handle_mouse_event(&mut app, MouseEvent {
        kind: MouseEventKind::ScrollDown,
        // ...
    });

    assert!(app.results_scroll.offset > 0);
}

#[test]
fn test_goto_line_jumps_correctly() {
    let mut app = create_app_with_json(large_json());

    // Enter go-to-line mode
    handle_key(&mut app, KeyCode::Char(':'));
    type_string(&mut app, "500");
    handle_key(&mut app, KeyCode::Enter);

    assert_eq!(app.results_scroll.offset, 499); // 0-indexed
}

#[test]
fn test_wrap_indicator_shows_and_clears() {
    let mut app = create_app_with_search_matches(5);

    // Navigate to last match
    for _ in 0..4 {
        app.search.next_match();
    }
    assert_eq!(app.search.current_index(), 4);

    // Wrap to first
    app.search.next_match();
    assert!(app.search.match_count_display().contains("↻"));

    // After timeout, indicator clears
    std::thread::sleep(Duration::from_millis(600));
    assert!(!app.search.match_count_display().contains("↻"));
}
```

### Manual Testing Checklist

Before release, verify:

- [ ] **N1**: Position indicator shows correct line range and percentage
- [ ] **N1**: Shows "Top" at start, "Bottom" at end
- [ ] **N2**: Mouse wheel scrolls results up and down
- [ ] **N2**: Scroll direction matches expectation
- [ ] **N3**: Wrap indicator appears when search wraps
- [ ] **N3**: Indicator clears after timeout
- [ ] **N4**: Pressing `n` doesn't jump when next match is visible
- [ ] **N4**: Pressing `n` scrolls minimally for nearby matches
- [ ] **N5**: `:` opens go-to-line prompt
- [ ] **N5**: Valid line numbers jump correctly
- [ ] **N5**: Invalid input is handled gracefully
- [ ] **N6**: `M` jumps to middle of document
- [ ] **N7**: Holding `j` accelerates scrolling
- [ ] **N8**: Scrollbar appears for long content
- [ ] **N8**: Scrollbar position reflects scroll state
- [ ] **N9**: Boundary indicator shows at top/bottom

---

## Implementation Order Recommendation

**Phase 1: Quick Wins (High impact, easy)**
1. N1: Position indicator
2. N2: Mouse wheel scrolling
3. N9: Boundary feedback

**Phase 2: Search UX (Medium complexity)**
4. N3: Search wrap feedback
5. N4: Smart scroll-to-match

**Phase 3: Navigation Features (Medium complexity)**
6. N5: Go-to-line command
7. N6: Percentage/middle jump

**Phase 4: Polish (Lower priority)**
8. N8: Visual scrollbar
9. N7: Scroll acceleration

---

## Open Questions

1. **Position indicator format**: Compact (`L45-95/1234`) vs verbose (`Lines 45-95 of 1,234`)?

2. **Mouse scroll amount**: 3 lines per tick? Should it be configurable?

3. **Go-to-line key**: `:` (vim) or `Ctrl+G` (standard)? Or both?

4. **Scrollbar visibility**: Always visible when needed, or only on hover/scroll?

5. **Acceleration curve**: Linear or exponential? What thresholds?

---

## References

- `src/scroll.rs` - Core scroll state
- `src/results/results_events.rs` - Current key bindings
- `src/search/search_events/scroll.rs` - Match scroll logic
- Ratatui `Scrollbar` widget documentation
- Crossterm mouse event handling
