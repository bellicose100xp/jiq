//! Tests for keybindings.rs

use super::*;
use crate::ai::selection::state::SelectionState;
use proptest::prelude::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// Helper to create key events
fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn key_with_mods(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

// =========================================================================
// Unit Tests for handle_direct_selection
// =========================================================================

#[test]
fn test_alt_1_selects_first_suggestion() {
    let result = handle_direct_selection(key_with_mods(KeyCode::Char('1'), KeyModifiers::ALT), 3);
    assert_eq!(result, Some(0));
}

#[test]
fn test_alt_2_selects_second_suggestion() {
    let result = handle_direct_selection(key_with_mods(KeyCode::Char('2'), KeyModifiers::ALT), 3);
    assert_eq!(result, Some(1));
}

#[test]
fn test_alt_5_selects_fifth_suggestion() {
    let result = handle_direct_selection(key_with_mods(KeyCode::Char('5'), KeyModifiers::ALT), 5);
    assert_eq!(result, Some(4));
}

#[test]
fn test_alt_3_invalid_when_only_2_suggestions() {
    let result = handle_direct_selection(key_with_mods(KeyCode::Char('3'), KeyModifiers::ALT), 2);
    assert_eq!(result, None);
}

#[test]
fn test_alt_1_invalid_when_no_suggestions() {
    let result = handle_direct_selection(key_with_mods(KeyCode::Char('1'), KeyModifiers::ALT), 0);
    assert_eq!(result, None);
}

#[test]
fn test_plain_digit_not_handled() {
    let result = handle_direct_selection(key(KeyCode::Char('1')), 5);
    assert_eq!(result, None);
}

#[test]
fn test_alt_6_not_handled() {
    let result = handle_direct_selection(key_with_mods(KeyCode::Char('6'), KeyModifiers::ALT), 10);
    assert_eq!(result, None);
}

#[test]
fn test_alt_a_not_handled() {
    let result = handle_direct_selection(key_with_mods(KeyCode::Char('a'), KeyModifiers::ALT), 5);
    assert_eq!(result, None);
}

// =========================================================================
// Unit Tests for handle_navigation
// =========================================================================

#[test]
fn test_alt_down_navigates_next() {
    let mut state = SelectionState::new();
    let handled = handle_navigation(
        key_with_mods(KeyCode::Down, KeyModifiers::ALT),
        &mut state,
        5,
    );
    assert!(handled);
    assert_eq!(state.get_selected(), Some(0));
    assert!(state.is_navigation_active());
}

#[test]
fn test_alt_j_navigates_next() {
    let mut state = SelectionState::new();
    let handled = handle_navigation(
        key_with_mods(KeyCode::Char('j'), KeyModifiers::ALT),
        &mut state,
        5,
    );
    assert!(handled);
    assert_eq!(state.get_selected(), Some(0));
    assert!(state.is_navigation_active());
}

#[test]
fn test_alt_up_navigates_previous() {
    let mut state = SelectionState::new();
    let handled = handle_navigation(key_with_mods(KeyCode::Up, KeyModifiers::ALT), &mut state, 5);
    assert!(handled);
    assert_eq!(state.get_selected(), Some(4)); // Wraps to last
    assert!(state.is_navigation_active());
}

#[test]
fn test_alt_k_navigates_previous() {
    let mut state = SelectionState::new();
    let handled = handle_navigation(
        key_with_mods(KeyCode::Char('k'), KeyModifiers::ALT),
        &mut state,
        5,
    );
    assert!(handled);
    assert_eq!(state.get_selected(), Some(4)); // Wraps to last
    assert!(state.is_navigation_active());
}

#[test]
fn test_plain_down_not_handled() {
    let mut state = SelectionState::new();
    let handled = handle_navigation(key(KeyCode::Down), &mut state, 5);
    assert!(!handled);
    assert!(state.get_selected().is_none());
}

#[test]
fn test_alt_down_no_suggestions() {
    let mut state = SelectionState::new();
    let handled = handle_navigation(
        key_with_mods(KeyCode::Down, KeyModifiers::ALT),
        &mut state,
        0,
    );
    assert!(!handled);
    assert!(state.get_selected().is_none());
}

#[test]
fn test_alt_left_not_handled() {
    let mut state = SelectionState::new();
    let handled = handle_navigation(
        key_with_mods(KeyCode::Left, KeyModifiers::ALT),
        &mut state,
        5,
    );
    assert!(!handled);
}

// =========================================================================
// Unit Tests for handle_apply_selection
// =========================================================================

#[test]
fn test_enter_applies_when_navigation_active() {
    let mut state = SelectionState::new();
    state.navigate_next(5); // Activates navigation, selects index 0

    let result = handle_apply_selection(key(KeyCode::Enter), &state);
    assert_eq!(result, Some(0));
}

#[test]
fn test_enter_not_handled_when_no_navigation() {
    let state = SelectionState::new();
    let result = handle_apply_selection(key(KeyCode::Enter), &state);
    assert_eq!(result, None);
}

#[test]
fn test_enter_not_handled_after_direct_selection() {
    let mut state = SelectionState::new();
    state.select_index(2); // Direct selection doesn't activate navigation

    let result = handle_apply_selection(key(KeyCode::Enter), &state);
    assert_eq!(result, None);
}

#[test]
fn test_other_key_not_handled_for_apply() {
    let mut state = SelectionState::new();
    state.navigate_next(5);

    let result = handle_apply_selection(key(KeyCode::Tab), &state);
    assert_eq!(result, None);
}

// =========================================================================
// Property-Based Tests
// =========================================================================

// **Feature: ai-assistant-phase3, Property 1: Direct selection applies correct suggestion**
// *For any* AI popup with N suggestions (1 ≤ N ≤ 5), pressing Alt+M where 1 ≤ M ≤ N
// should return the (M-1)th index (0-based).
// **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_direct_selection_applies_correct_suggestion(
        suggestion_count in 1usize..=5,
        selection in 1usize..=5
    ) {
        let key = key_with_mods(
            KeyCode::Char(char::from_digit(selection as u32, 10).unwrap()),
            KeyModifiers::ALT,
        );

        let result = handle_direct_selection(key, suggestion_count);

        if selection <= suggestion_count {
            // Valid selection should return the 0-based index
            prop_assert_eq!(
                result,
                Some(selection - 1),
                "Alt+{} with {} suggestions should select index {}",
                selection, suggestion_count, selection - 1
            );
        } else {
            // Invalid selection should return None
            prop_assert_eq!(
                result,
                None,
                "Alt+{} with {} suggestions should be ignored",
                selection, suggestion_count
            );
        }
    }
}

// **Feature: ai-assistant-phase3, Property 2: Invalid selection has no effect**
// *For any* AI popup state (hidden, visible with no suggestions, or visible with N suggestions),
// pressing Alt+M where M > N or when popup is hidden should return None.
// **Validates: Requirements 2.1, 2.2, 2.3, 2.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_invalid_selection_has_no_effect(
        suggestion_count in 0usize..10,
        selection in 1usize..=5
    ) {
        let key = key_with_mods(
            KeyCode::Char(char::from_digit(selection as u32, 10).unwrap()),
            KeyModifiers::ALT,
        );

        let result = handle_direct_selection(key, suggestion_count);

        if selection > suggestion_count {
            // Selection beyond available suggestions should return None
            prop_assert_eq!(
                result,
                None,
                "Alt+{} with {} suggestions should be ignored (index out of bounds)",
                selection, suggestion_count
            );
        }
        // Note: Valid selections are tested in prop_direct_selection_applies_correct_suggestion
    }

    #[test]
    fn prop_non_alt_keys_ignored(
        suggestion_count in 1usize..10,
        digit in 1u32..=5
    ) {
        // Plain digit without Alt should be ignored
        let key = key(KeyCode::Char(char::from_digit(digit, 10).unwrap()));
        let result = handle_direct_selection(key, suggestion_count);

        prop_assert_eq!(
            result,
            None,
            "Plain digit {} should be ignored",
            digit
        );
    }

    #[test]
    fn prop_alt_non_digit_ignored(
        suggestion_count in 1usize..10,
        c_idx in 0usize..26
    ) {
        // Alt+letter should be ignored
        let c = (b'a' + c_idx as u8) as char;
        let key = key_with_mods(KeyCode::Char(c), KeyModifiers::ALT);
        let result = handle_direct_selection(key, suggestion_count);

        prop_assert_eq!(
            result,
            None,
            "Alt+{} should be ignored",
            c
        );
    }
}

// **Feature: ai-assistant-phase3, Property 10: Enter applies only when navigated**
// *For any* AI popup state, pressing Enter should apply a suggestion only when
// a suggestion has been explicitly selected via Alt+Up/Down navigation.
// **Validates: Requirements 9.1, 9.2, 9.3, 9.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_enter_applies_only_when_navigated(
        suggestion_count in 1usize..10,
        nav_steps in 0usize..20
    ) {
        let mut state = SelectionState::new();

        // Navigate some number of times
        for _ in 0..nav_steps {
            state.navigate_next(suggestion_count);
        }

        let result = handle_apply_selection(key(KeyCode::Enter), &state);

        if nav_steps > 0 {
            // After navigation, Enter should return the selected index
            prop_assert!(
                result.is_some(),
                "Enter should apply selection after {} navigation steps",
                nav_steps
            );
            prop_assert!(
                result.unwrap() < suggestion_count,
                "Selected index {} should be < suggestion count {}",
                result.unwrap(), suggestion_count
            );
        } else {
            // Without navigation, Enter should return None
            prop_assert_eq!(
                result,
                None,
                "Enter should not apply without navigation"
            );
        }
    }

    #[test]
    fn prop_enter_not_handled_after_direct_selection(
        suggestion_count in 1usize..5,
        direct_index in 0usize..5
    ) {
        prop_assume!(direct_index < suggestion_count);

        let mut state = SelectionState::new();
        state.select_index(direct_index); // Direct selection doesn't activate navigation

        let result = handle_apply_selection(key(KeyCode::Enter), &state);

        prop_assert_eq!(
            result,
            None,
            "Enter should not apply after direct selection (navigation not active)"
        );
    }

    #[test]
    fn prop_enter_clears_after_clear_selection(
        suggestion_count in 1usize..10,
        nav_steps in 1usize..10
    ) {
        let mut state = SelectionState::new();

        // Navigate to activate selection
        for _ in 0..nav_steps {
            state.navigate_next(suggestion_count);
        }

        // Clear selection
        state.clear_selection();

        let result = handle_apply_selection(key(KeyCode::Enter), &state);

        prop_assert_eq!(
            result,
            None,
            "Enter should not apply after selection is cleared"
        );
    }
}
