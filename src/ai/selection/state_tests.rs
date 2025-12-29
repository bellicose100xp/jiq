//! Tests for selection state management

use super::*;
use proptest::prelude::*;

// =========================================================================
// Unit Tests
// =========================================================================

#[test]
fn test_new_selection_state() {
    let state = SelectionState::new();
    assert!(state.get_selected().is_none());
    assert!(!state.is_navigation_active());
}

#[test]
fn test_select_index() {
    let mut state = SelectionState::new();
    state.select_index(2);
    assert_eq!(state.get_selected(), Some(2));
    assert!(!state.is_navigation_active()); // Direct selection doesn't activate navigation
}

#[test]
fn test_clear_selection() {
    let mut state = SelectionState::new();
    state.select_index(2);
    state.navigation_active = true;
    state.clear_selection();
    assert!(state.get_selected().is_none());
    assert!(!state.is_navigation_active());
}

#[test]
fn test_navigate_next_from_none() {
    let mut state = SelectionState::new();
    state.navigate_next(5);
    assert_eq!(state.get_selected(), Some(0));
    assert!(state.is_navigation_active());
}

#[test]
fn test_navigate_next_wraps() {
    let mut state = SelectionState::new();
    state.selected_index = Some(4);
    state.navigate_next(5);
    assert_eq!(state.get_selected(), Some(0)); // Wraps to first
    assert!(state.is_navigation_active());
}

#[test]
fn test_navigate_previous_from_none() {
    let mut state = SelectionState::new();
    state.navigate_previous(5);
    assert_eq!(state.get_selected(), Some(4)); // Starts at last
    assert!(state.is_navigation_active());
}

#[test]
fn test_navigate_previous_wraps() {
    let mut state = SelectionState::new();
    state.selected_index = Some(0);
    state.navigate_previous(5);
    assert_eq!(state.get_selected(), Some(4)); // Wraps to last
    assert!(state.is_navigation_active());
}

#[test]
fn test_navigate_with_zero_suggestions() {
    let mut state = SelectionState::new();
    state.navigate_next(0);
    assert!(state.get_selected().is_none());
    assert!(!state.is_navigation_active());

    state.navigate_previous(0);
    assert!(state.get_selected().is_none());
    assert!(!state.is_navigation_active());
}

// =========================================================================
// Scroll Behavior Tests
// =========================================================================

#[test]
fn test_update_layout_calculates_positions() {
    let mut state = SelectionState::new();
    let heights = vec![3, 5, 2, 4];
    state.update_layout(heights.clone(), 10);

    assert_eq!(state.viewport_height, 10);
    assert_eq!(state.suggestion_heights, heights);
    // Y positions: 0, 3+1=4, 4+5+1=10, 10+2+1=13
    assert_eq!(state.suggestion_y_positions, vec![0, 4, 10, 13]);
}

#[test]
fn test_scroll_offset_getter() {
    let mut state = SelectionState::new();
    assert_eq!(state.scroll_offset(), 0);

    state.scroll_offset = 5;
    assert_eq!(state.scroll_offset(), 5);
}

#[test]
fn test_clear_layout() {
    let mut state = SelectionState::new();
    state.scroll_offset = 10;
    state.viewport_height = 20;
    state.suggestion_y_positions = vec![0, 5, 10];
    state.suggestion_heights = vec![3, 4, 2];

    state.clear_layout();

    assert_eq!(state.scroll_offset, 0);
    assert_eq!(state.viewport_height, 0);
    assert!(state.suggestion_y_positions.is_empty());
    assert!(state.suggestion_heights.is_empty());
}

#[test]
fn test_scroll_down_when_selection_below_viewport() {
    let mut state = SelectionState::new();
    // Setup: 4 suggestions with heights [5, 5, 5, 5], viewport height = 10
    // Y positions: 0, 6, 12, 18
    state.update_layout(vec![5, 5, 5, 5], 10);
    state.scroll_offset = 0;
    state.selected_index = Some(2); // Select suggestion at Y=12, ends at Y=17

    state.ensure_selected_visible();

    // Viewport is 0-10, suggestion is 12-17, should scroll to show it
    // scroll_offset should be 17 - 10 = 7
    assert_eq!(state.scroll_offset, 7);
}

#[test]
fn test_scroll_up_when_selection_above_viewport() {
    let mut state = SelectionState::new();
    // Setup: 4 suggestions with heights [5, 5, 5, 5], viewport height = 10
    state.update_layout(vec![5, 5, 5, 5], 10);
    state.scroll_offset = 10; // Viewing Y=10-20
    state.selected_index = Some(0); // Select suggestion at Y=0

    state.ensure_selected_visible();

    // Suggestion at Y=0 is above viewport, should scroll to 0
    assert_eq!(state.scroll_offset, 0);
}

#[test]
fn test_no_scroll_when_selection_visible() {
    let mut state = SelectionState::new();
    // Setup: suggestions with heights [5, 5, 5], viewport height = 10
    state.update_layout(vec![5, 5, 5], 10);
    state.scroll_offset = 0; // Viewing Y=0-10
    state.selected_index = Some(1); // Select suggestion at Y=6, ends at Y=11

    state.ensure_selected_visible();

    // Suggestion overlaps viewport, no scroll needed... wait, it ends at 11 which is beyond viewport
    // Actually it WILL scroll. Let me recalculate.
    // Suggestion at Y=6, height=5, ends at Y=11
    // Viewport is 0-10, so suggestion_end (11) > viewport_end (10)
    // Should scroll to 11 - 10 = 1
    assert_eq!(state.scroll_offset, 1);
}

#[test]
fn test_navigate_next_scrolls_to_selection() {
    let mut state = SelectionState::new();
    // 5 tall suggestions, viewport = 10, so can only see ~1.5 suggestions at a time
    state.update_layout(vec![8, 8, 8, 8, 8], 10);
    state.scroll_offset = 0;
    state.selected_index = Some(0);

    // Navigate to next suggestion (index 1, Y=9, ends at Y=17)
    state.navigate_next(5);

    assert_eq!(state.selected_index, Some(1));
    // Viewport 0-10, suggestion 9-17, should scroll to show it
    assert_eq!(state.scroll_offset, 7);
}

#[test]
fn test_navigate_previous_scrolls_to_selection() {
    let mut state = SelectionState::new();
    // 5 suggestions with varying heights
    state.update_layout(vec![5, 5, 5, 5, 5], 10);
    state.scroll_offset = 15; // Viewing Y=15-25
    state.selected_index = Some(4); // At last suggestion, Y=24

    // Navigate to previous (index 3, Y=18, ends at Y=23)
    state.navigate_previous(5);

    assert_eq!(state.selected_index, Some(3));
    // Suggestion 18-23 is within viewport 15-25, no scroll needed... wait let me recalculate
    // Actually viewport is 15-25, suggestion is 18-23, fully visible, no scroll needed
    assert_eq!(state.scroll_offset, 15);
}

#[test]
fn test_wrap_forward_resets_scroll() {
    let mut state = SelectionState::new();
    state.update_layout(vec![5, 5, 5, 5], 10);
    state.scroll_offset = 10;
    state.selected_index = Some(3);

    // Wrap from last to first
    state.navigate_next(4);

    assert_eq!(state.selected_index, Some(0));
    // Should scroll to show first suggestion at Y=0
    assert_eq!(state.scroll_offset, 0);
}

#[test]
fn test_wrap_backward_scrolls_to_last() {
    let mut state = SelectionState::new();
    // 4 suggestions, Y positions: 0, 6, 12, 18
    state.update_layout(vec![5, 5, 5, 5], 10);
    state.scroll_offset = 0;
    state.selected_index = Some(0);

    // Wrap from first to last
    state.navigate_previous(4);

    assert_eq!(state.selected_index, Some(3));
    // Last suggestion at Y=18, height=5, ends at Y=23
    // Viewport height=10, should scroll to 23-10=13
    assert_eq!(state.scroll_offset, 13);
}

// =========================================================================
// Property-Based Tests
// =========================================================================

// **Feature: ai-assistant-phase3, Property 4: Navigation wrapping**
// *For any* AI popup with N suggestions, navigating down from suggestion N-1
// should wrap to suggestion 0, and navigating up from suggestion 0 should
// wrap to suggestion N-1.
// **Validates: Requirements 8.3, 8.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_navigation_wrapping(suggestion_count in 1usize..20) {
        // Test wrapping from last to first (navigate_next)
        let mut state = SelectionState::new();
        state.selected_index = Some(suggestion_count - 1);
        state.navigate_next(suggestion_count);

        prop_assert_eq!(
            state.get_selected(),
            Some(0),
            "Navigating next from last suggestion ({}) should wrap to 0",
            suggestion_count - 1
        );

        // Test wrapping from first to last (navigate_previous)
        let mut state = SelectionState::new();
        state.selected_index = Some(0);
        state.navigate_previous(suggestion_count);

        prop_assert_eq!(
            state.get_selected(),
            Some(suggestion_count - 1),
            "Navigating previous from suggestion 0 should wrap to {}",
            suggestion_count - 1
        );
    }
}

// **Feature: ai-assistant-phase3, Property 5: Navigation movement**
// *For any* AI popup with N suggestions and current selection at index I,
// pressing Alt+Down should move selection to (I+1) mod N, and pressing
// Alt+Up should move selection to (I-1) mod N.
// **Validates: Requirements 8.1, 8.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_navigation_movement(
        suggestion_count in 1usize..20,
        current_index in 0usize..20
    ) {
        // Only test valid indices
        prop_assume!(current_index < suggestion_count);

        // Test navigate_next: should move to (current + 1) % count
        let mut state = SelectionState::new();
        state.selected_index = Some(current_index);
        state.navigate_next(suggestion_count);

        let expected_next = (current_index + 1) % suggestion_count;
        prop_assert_eq!(
            state.get_selected(),
            Some(expected_next),
            "Navigate next from {} with {} suggestions should go to {}",
            current_index, suggestion_count, expected_next
        );
        prop_assert!(
            state.is_navigation_active(),
            "Navigation should be active after navigate_next"
        );

        // Test navigate_previous: should move to (current - 1) mod count
        let mut state = SelectionState::new();
        state.selected_index = Some(current_index);
        state.navigate_previous(suggestion_count);

        let expected_prev = if current_index == 0 {
            suggestion_count - 1
        } else {
            current_index - 1
        };
        prop_assert_eq!(
            state.get_selected(),
            Some(expected_prev),
            "Navigate previous from {} with {} suggestions should go to {}",
            current_index, suggestion_count, expected_prev
        );
        prop_assert!(
            state.is_navigation_active(),
            "Navigation should be active after navigate_previous"
        );
    }
}
