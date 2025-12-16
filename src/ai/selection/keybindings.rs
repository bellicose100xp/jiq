//! Keybinding handlers for AI suggestion selection
//!
//! Handles Alt+1-5 for direct selection, Alt+Up/Down/j/k for navigation,
//! and Enter for applying navigated selection.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::state::SelectionState;

/// Handle direct selection keybindings (Alt+1-5)
///
/// Parses Alt+1 through Alt+5 keybindings and validates the selection
/// index against the available suggestion count.
///
/// # Arguments
/// * `key` - The key event to handle
/// * `suggestion_count` - Number of available suggestions
///
/// # Returns
/// * `Some(index)` - The 0-based index of the selected suggestion if valid
/// * `None` - If the key is not a selection key or the index is invalid
///
/// # Requirements
/// - 1.1-1.5: Alt+1-5 selects corresponding suggestion
/// - 2.1-2.4: Invalid selections are ignored
pub fn handle_direct_selection(key: KeyEvent, suggestion_count: usize) -> Option<usize> {
    // Log key event for troubleshooting
    #[cfg(debug_assertions)]
    log::debug!(
        "handle_direct_selection: key={:?}, modifiers={:?}, suggestion_count={}",
        key.code,
        key.modifiers,
        suggestion_count
    );

    // Only handle Alt+digit keys
    if !key.modifiers.contains(KeyModifiers::ALT) {
        return None;
    }

    // Parse digit from key code
    let digit = match key.code {
        KeyCode::Char('1') => 1,
        KeyCode::Char('2') => 2,
        KeyCode::Char('3') => 3,
        KeyCode::Char('4') => 4,
        KeyCode::Char('5') => 5,
        _ => return None,
    };

    #[cfg(debug_assertions)]
    log::debug!("Parsed digit: {}, index: {}", digit, digit - 1);

    // Convert to 0-based index
    let index = digit - 1;

    // Validate against suggestion count
    // Requirements 2.3, 2.4: Ignore if index >= suggestion_count
    if index < suggestion_count {
        #[cfg(debug_assertions)]
        log::debug!("Valid selection: index={}", index);
        Some(index)
    } else {
        #[cfg(debug_assertions)]
        log::debug!(
            "Invalid selection: index={} >= count={}",
            index,
            suggestion_count
        );
        None
    }
}

/// Handle navigation keybindings (Alt+Up/Down and Alt+j/k)
///
/// Parses Alt+Up/Down and Alt+j/k keybindings and updates the selection state
/// with wrapping behavior at boundaries.
///
/// # Arguments
/// * `key` - The key event to handle
/// * `selection_state` - The selection state to update
/// * `suggestion_count` - Number of available suggestions
///
/// # Returns
/// * `true` - If the key was handled (Alt+Up/Down or Alt+j/k)
/// * `false` - If the key was not a navigation key
///
/// # Requirements
/// - 8.1: Alt+Down/j moves selection to next suggestion
/// - 8.2: Alt+Up/k moves selection to previous suggestion
/// - 8.3: Wraps to first suggestion when at the end
/// - 8.4: Wraps to last suggestion when at the beginning
pub fn handle_navigation(
    key: KeyEvent,
    selection_state: &mut SelectionState,
    suggestion_count: usize,
) -> bool {
    // Only handle Alt+arrow keys or Alt+j/k
    if !key.modifiers.contains(KeyModifiers::ALT) {
        return false;
    }

    // No suggestions to navigate
    if suggestion_count == 0 {
        return false;
    }

    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            selection_state.navigate_next(suggestion_count);
            true
        }
        KeyCode::Up | KeyCode::Char('k') => {
            selection_state.navigate_previous(suggestion_count);
            true
        }
        _ => false,
    }
}

/// Handle Enter key for applying navigated selection
///
/// Checks if navigation mode is active (user has used Alt+Up/Down/j/k) and
/// returns the selected index if so. This allows Enter to apply the
/// currently highlighted suggestion.
///
/// # Arguments
/// * `key` - The key event to handle
/// * `selection_state` - The selection state to check
///
/// # Returns
/// * `Some(index)` - The 0-based index of the selected suggestion if navigation is active
/// * `None` - If Enter was not pressed or no suggestion is selected via navigation
///
/// # Requirements
/// - 9.1: Enter applies the highlighted suggestion when navigation is active
/// - 9.2: Does not interfere with normal Enter behavior when no navigation
/// - 9.3: Clears selection highlight after application (caller responsibility)
/// - 9.4: Does not interfere when popup has no suggestions
pub fn handle_apply_selection(key: KeyEvent, selection_state: &SelectionState) -> Option<usize> {
    // Only handle Enter key
    if key.code != KeyCode::Enter {
        return None;
    }

    // Only apply if navigation mode is active (user has used Alt+Up/Down/j/k)
    // Requirements 9.2, 9.4: Don't interfere with normal Enter behavior
    if !selection_state.is_navigation_active() {
        return None;
    }

    // Return the selected index
    selection_state.get_selected()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

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
        let result =
            handle_direct_selection(key_with_mods(KeyCode::Char('1'), KeyModifiers::ALT), 3);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_alt_2_selects_second_suggestion() {
        let result =
            handle_direct_selection(key_with_mods(KeyCode::Char('2'), KeyModifiers::ALT), 3);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_alt_5_selects_fifth_suggestion() {
        let result =
            handle_direct_selection(key_with_mods(KeyCode::Char('5'), KeyModifiers::ALT), 5);
        assert_eq!(result, Some(4));
    }

    #[test]
    fn test_alt_3_invalid_when_only_2_suggestions() {
        let result =
            handle_direct_selection(key_with_mods(KeyCode::Char('3'), KeyModifiers::ALT), 2);
        assert_eq!(result, None);
    }

    #[test]
    fn test_alt_1_invalid_when_no_suggestions() {
        let result =
            handle_direct_selection(key_with_mods(KeyCode::Char('1'), KeyModifiers::ALT), 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_plain_digit_not_handled() {
        let result = handle_direct_selection(key(KeyCode::Char('1')), 5);
        assert_eq!(result, None);
    }

    #[test]
    fn test_alt_6_not_handled() {
        let result =
            handle_direct_selection(key_with_mods(KeyCode::Char('6'), KeyModifiers::ALT), 10);
        assert_eq!(result, None);
    }

    #[test]
    fn test_alt_a_not_handled() {
        let result =
            handle_direct_selection(key_with_mods(KeyCode::Char('a'), KeyModifiers::ALT), 5);
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
        let handled =
            handle_navigation(key_with_mods(KeyCode::Up, KeyModifiers::ALT), &mut state, 5);
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
}
