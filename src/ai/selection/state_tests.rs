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
fn test_navigate_next_stops_at_last() {
    let mut state = SelectionState::new();
    state.selected_index = Some(4);
    state.navigate_next(5);
    assert_eq!(state.get_selected(), Some(4)); // Stays at last
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
fn test_navigate_previous_stops_at_first() {
    let mut state = SelectionState::new();
    state.selected_index = Some(0);
    state.navigate_previous(5);
    assert_eq!(state.get_selected(), Some(0)); // Stays at first
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
    // Y positions: 0, 3, 8, 10 (heights already include spacing)
    assert_eq!(state.suggestion_y_positions, vec![0, 3, 8, 10]);
}

#[test]
fn test_scroll_offset_getter() {
    let mut state = SelectionState::new();
    assert_eq!(state.scroll_offset_u16(), 0);

    state.scroll_offset = 5;
    assert_eq!(state.scroll_offset_u16(), 5);
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
    // Y positions: 0, 5, 10, 15 (heights already include spacing)
    state.update_layout(vec![5, 5, 5, 5], 10);
    state.scroll_offset = 0;
    state.selected_index = Some(2); // Select suggestion at Y=10, ends at Y=15

    state.ensure_selected_visible();

    // Viewport is 0-10, suggestion is 10-15, should scroll to show it
    // scroll_offset should be 15 - 10 = 5
    assert_eq!(state.scroll_offset, 5);
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
    // Y positions: 0, 5, 10 (heights already include spacing)
    state.update_layout(vec![5, 5, 5], 10);
    state.scroll_offset = 0; // Viewing Y=0-10
    state.selected_index = Some(1); // Select suggestion at Y=5, ends at Y=10

    state.ensure_selected_visible();

    // Suggestion at Y=5, height=5, ends at Y=10
    // Viewport is 0-10, so suggestion is fully visible
    // No scroll needed
    assert_eq!(state.scroll_offset, 0);
}

#[test]
fn test_navigate_next_scrolls_to_selection() {
    let mut state = SelectionState::new();
    // 5 tall suggestions, viewport = 10, so can only see ~1.5 suggestions at a time
    // Y positions: 0, 8, 16, 24, 32 (heights already include spacing)
    state.update_layout(vec![8, 8, 8, 8, 8], 10);
    state.scroll_offset = 0;
    state.selected_index = Some(0);

    // Navigate to next suggestion (index 1, Y=8, ends at Y=16)
    state.navigate_next(5);

    assert_eq!(state.selected_index, Some(1));
    // Viewport 0-10, suggestion 8-16, should scroll to show it
    // scroll_offset should be 16 - 10 = 6
    assert_eq!(state.scroll_offset, 6);
}

#[test]
fn test_navigate_previous_scrolls_to_selection() {
    let mut state = SelectionState::new();
    // 5 suggestions with heights [5, 5, 5, 5, 5]
    // Y positions: 0, 5, 10, 15, 20 (heights already include spacing)
    state.update_layout(vec![5, 5, 5, 5, 5], 10);
    state.scroll_offset = 15; // Viewing Y=15-25
    state.selected_index = Some(4); // At last suggestion, Y=20

    // Navigate to previous (index 3, Y=15, ends at Y=20)
    state.navigate_previous(5);

    assert_eq!(state.selected_index, Some(3));
    // Suggestion 15-20 is within viewport 15-25, no scroll needed
    assert_eq!(state.scroll_offset, 15);
}

#[test]
fn test_navigate_next_at_last_stays_in_place() {
    let mut state = SelectionState::new();
    state.update_layout(vec![5, 5, 5, 5], 10);
    state.scroll_offset = 10;
    state.selected_index = Some(3);

    // Try to navigate next from last - should stay at last
    state.navigate_next(4);

    assert_eq!(state.selected_index, Some(3));
    // Scroll offset unchanged since we stayed at same position
    assert_eq!(state.scroll_offset, 10);
}

#[test]
fn test_navigate_previous_at_first_stays_in_place() {
    let mut state = SelectionState::new();
    // 4 suggestions, Y positions: 0, 5, 10, 15 (heights already include spacing)
    state.update_layout(vec![5, 5, 5, 5], 10);
    state.scroll_offset = 0;
    state.selected_index = Some(0);

    // Try to navigate previous from first - should stay at first
    state.navigate_previous(4);

    assert_eq!(state.selected_index, Some(0));
    // Scroll offset unchanged since we stayed at same position
    assert_eq!(state.scroll_offset, 0);
}

// Tests for Scrollable trait implementation

use crate::scroll::Scrollable;

#[test]
fn test_scrollable_scroll_view_down() {
    let mut state = SelectionState::new();
    state.update_layout(vec![5, 5, 5, 5, 5], 10);

    state.scroll_view_down(3);
    assert_eq!(Scrollable::scroll_offset(&state), 3);

    state.scroll_view_down(5);
    assert_eq!(Scrollable::scroll_offset(&state), 8);
}

#[test]
fn test_scrollable_scroll_view_down_clamped() {
    let mut state = SelectionState::new();
    // Total height: 25, viewport: 10, max_scroll: 15
    state.update_layout(vec![5, 5, 5, 5, 5], 10);

    state.scroll_view_down(100);
    assert_eq!(Scrollable::scroll_offset(&state), 15);
}

#[test]
fn test_scrollable_scroll_view_up() {
    let mut state = SelectionState::new();
    state.update_layout(vec![5, 5, 5, 5, 5], 10);
    state.scroll_offset = 10;

    state.scroll_view_up(3);
    assert_eq!(Scrollable::scroll_offset(&state), 7);

    state.scroll_view_up(4);
    assert_eq!(Scrollable::scroll_offset(&state), 3);
}

#[test]
fn test_scrollable_scroll_view_up_clamped() {
    let mut state = SelectionState::new();
    state.update_layout(vec![5, 5, 5, 5, 5], 10);
    state.scroll_offset = 5;

    state.scroll_view_up(10);
    assert_eq!(Scrollable::scroll_offset(&state), 0);
}

#[test]
fn test_scrollable_max_scroll() {
    let mut state = SelectionState::new();
    // Total height: 25, viewport: 10, max_scroll: 15
    state.update_layout(vec![5, 5, 5, 5, 5], 10);
    assert_eq!(state.max_scroll(), 15);

    // Content fits in viewport: total height: 8, viewport: 10, max_scroll: 0
    state.update_layout(vec![3, 5], 10);
    assert_eq!(state.max_scroll(), 0);
}

#[test]
fn test_scrollable_viewport_size() {
    let mut state = SelectionState::new();
    state.update_layout(vec![5, 5, 5], 15);
    assert_eq!(state.viewport_size(), 15);
}

#[test]
fn test_scrollable_content_fits_in_viewport() {
    let mut state = SelectionState::new();
    // Total height: 6, viewport: 10, max_scroll: 0
    state.update_layout(vec![3, 3], 10);
    assert_eq!(state.max_scroll(), 0);

    state.scroll_view_down(5);
    assert_eq!(Scrollable::scroll_offset(&state), 0); // Can't scroll when content fits
}

#[test]
fn test_last_selection_maintains_correct_spacing() {
    // Regression test for spacing bug: selecting the last option should not
    // cause spacing lines between earlier options to disappear
    let mut state = SelectionState::new();

    // Setup: 3 suggestions, each 2 content lines + 1 spacing (except last)
    // Heights: [3, 3, 2] (last has no spacing line)
    // Expected Y positions: [0, 3, 6]
    state.update_layout(vec![3, 3, 2], 5);

    // Verify Y positions match actual rendering positions
    assert_eq!(state.suggestion_y_positions, vec![0, 3, 6]);

    // Select last option (index 2)
    state.selected_index = Some(2);
    state.ensure_selected_visible();

    // Suggestion at Y=6, height=2, ends at Y=8
    // Viewport height=5, so viewport should be [3, 8)
    // This should show suggestion 1 (Y=3-6) and suggestion 2 (Y=6-8)
    // The spacing line between suggestion 0 and 1 is at Y=2, which is
    // outside the viewport but that's expected - we're scrolled to show the last item
    assert_eq!(state.scroll_offset, 3);

    // Now select middle option (index 1)
    state.selected_index = Some(1);
    state.ensure_selected_visible();

    // Suggestion at Y=3, height=3, ends at Y=6
    // Current viewport [3, 8) (scroll_offset still 3 from previous), suggestion [3, 6)
    // Suggestion is fully visible, no scroll needed
    assert_eq!(state.scroll_offset, 3);

    // Now select first option (index 0)
    state.selected_index = Some(0);
    state.ensure_selected_visible();

    // Suggestion at Y=0, which is above current scroll offset (3), so scroll up to 0
    assert_eq!(state.scroll_offset, 0);

    // Now select middle option again from the top
    state.selected_index = Some(1);
    state.ensure_selected_visible();

    // Viewport [0, 5), suggestion [3, 6), suggestion_end (6) > viewport_end (5)
    // Should scroll to 6-5=1
    assert_eq!(state.scroll_offset, 1);

    // Now select last option to verify correct scroll
    state.selected_index = Some(2);
    state.ensure_selected_visible();

    // Suggestion at Y=6, height=2, ends at Y=8
    // Viewport [1, 6), suggestion [6, 8), so suggestion_end (8) > viewport_end (6)
    // Should scroll to 8-5=3
    assert_eq!(state.scroll_offset, 3);
}

// =========================================================================
// Property-Based Tests
// =========================================================================

// **Feature: ai-assistant-phase3, Property 4: Navigation boundary behavior**
// *For any* AI popup with N suggestions, navigating down from suggestion N-1
// should stay at N-1 (no wrap), and navigating up from suggestion 0 should
// stay at 0 (no wrap).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_navigation_stops_at_boundaries(suggestion_count in 1usize..20) {
        // Test boundary at last suggestion (navigate_next)
        let mut state = SelectionState::new();
        state.selected_index = Some(suggestion_count - 1);
        state.navigate_next(suggestion_count);

        prop_assert_eq!(
            state.get_selected(),
            Some(suggestion_count - 1),
            "Navigating next from last suggestion ({}) should stay at last",
            suggestion_count - 1
        );

        // Test boundary at first suggestion (navigate_previous)
        let mut state = SelectionState::new();
        state.selected_index = Some(0);
        state.navigate_previous(suggestion_count);

        prop_assert_eq!(
            state.get_selected(),
            Some(0),
            "Navigating previous from suggestion 0 should stay at 0"
        );
    }
}

// **Feature: ai-assistant-phase3, Property 5: Navigation movement**
// *For any* AI popup with N suggestions and current selection at index I,
// pressing Alt+Down should move selection to min(I+1, N-1), and pressing
// Alt+Up should move selection to max(I-1, 0).
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

        // Test navigate_next: should move to min(current + 1, count - 1)
        let mut state = SelectionState::new();
        state.selected_index = Some(current_index);
        state.navigate_next(suggestion_count);

        let expected_next = std::cmp::min(current_index + 1, suggestion_count - 1);
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

        // Test navigate_previous: should move to max(current - 1, 0)
        let mut state = SelectionState::new();
        state.selected_index = Some(current_index);
        state.navigate_previous(suggestion_count);

        let expected_prev = current_index.saturating_sub(1);
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
