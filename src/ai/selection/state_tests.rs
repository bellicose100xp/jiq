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
