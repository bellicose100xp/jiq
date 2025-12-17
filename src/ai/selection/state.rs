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
#[path = "state_tests.rs"]
mod state_tests;
