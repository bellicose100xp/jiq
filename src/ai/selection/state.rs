//! Selection state management for AI suggestions
//!
//! Tracks the currently selected suggestion index and navigation state.

/// Selection state for AI suggestion navigation
///
/// Tracks which suggestion is currently selected (if any) and whether
/// the user is actively navigating through suggestions.
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    /// Currently selected suggestion index (None = no selection)
    selected_index: Option<usize>,
    /// Whether navigation mode is active (user has used Alt+Up/Down/j/k)
    navigation_active: bool,
}

impl SelectionState {
    /// Create a new SelectionState with no selection
    pub fn new() -> Self {
        Self {
            selected_index: None,
            navigation_active: false,
        }
    }

    /// Select a specific suggestion index (for direct Alt+1-5 selection)
    ///
    /// This does NOT activate navigation mode since it's a direct selection.
    #[allow(dead_code)]
    pub fn select_index(&mut self, index: usize) {
        self.selected_index = Some(index);
        // Direct selection doesn't activate navigation mode
        self.navigation_active = false;
    }

    /// Clear the current selection
    pub fn clear_selection(&mut self) {
        self.selected_index = None;
        self.navigation_active = false;
    }

    /// Get the currently selected suggestion index
    pub fn get_selected(&self) -> Option<usize> {
        self.selected_index
    }

    /// Check if navigation mode is active
    ///
    /// Navigation mode is active when the user has used Alt+Up/Down/j/k
    /// to navigate through suggestions. In this mode, Enter applies
    /// the selected suggestion.
    pub fn is_navigation_active(&self) -> bool {
        self.navigation_active
    }

    /// Navigate to the next suggestion (Alt+Down or Alt+j)
    ///
    /// Wraps around to the first suggestion when at the end.
    /// Activates navigation mode.
    ///
    /// # Arguments
    /// * `suggestion_count` - Total number of available suggestions
    ///
    /// # Requirements
    /// - 8.1: Alt+Down/j moves selection to next suggestion
    /// - 8.3: Wraps to first suggestion when at the end
    pub fn navigate_next(&mut self, suggestion_count: usize) {
        if suggestion_count == 0 {
            return;
        }

        self.navigation_active = true;

        match self.selected_index {
            Some(current) => {
                // Wrap around to first suggestion
                self.selected_index = Some((current + 1) % suggestion_count);
            }
            None => {
                // Start at first suggestion
                self.selected_index = Some(0);
            }
        }
    }

    /// Navigate to the previous suggestion (Alt+Up or Alt+k)
    ///
    /// Wraps around to the last suggestion when at the beginning.
    /// Activates navigation mode.
    ///
    /// # Arguments
    /// * `suggestion_count` - Total number of available suggestions
    ///
    /// # Requirements
    /// - 8.2: Alt+Up/k moves selection to previous suggestion
    /// - 8.4: Wraps to last suggestion when at the beginning
    pub fn navigate_previous(&mut self, suggestion_count: usize) {
        if suggestion_count == 0 {
            return;
        }

        self.navigation_active = true;

        match self.selected_index {
            Some(current) => {
                if current == 0 {
                    // Wrap around to last suggestion
                    self.selected_index = Some(suggestion_count - 1);
                } else {
                    self.selected_index = Some(current - 1);
                }
            }
            None => {
                // Start at last suggestion
                self.selected_index = Some(suggestion_count - 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
}
