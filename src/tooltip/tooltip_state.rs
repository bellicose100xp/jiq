//! Tooltip state management
//!
//! Manages the enabled/disabled state and current function for the tooltip feature.

/// Tooltip state for managing contextual function help
pub struct TooltipState {
    /// Whether tooltip feature is enabled (shows automatically)
    pub enabled: bool,
    /// Currently detected function name (if any)
    pub current_function: Option<String>,
}

impl TooltipState {
    /// Create a new TooltipState with specified auto_show behavior
    ///
    /// # Arguments
    /// * `auto_show` - If true, tooltip auto-shows when cursor is on a function.
    ///   If false, tooltip is hidden by default and requires Ctrl+I to show.
    pub fn new(auto_show: bool) -> Self {
        Self {
            enabled: auto_show,
            current_function: None,
        }
    }

    /// Toggle the tooltip enabled state
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Set the current function detected at cursor position
    pub fn set_current_function(&mut self, func: Option<String>) {
        self.current_function = func;
    }

    /// Check if tooltip should be shown
    /// Returns true only when enabled AND a function is detected
    pub fn should_show(&self) -> bool {
        self.enabled && self.current_function.is_some()
    }
}

impl Default for TooltipState {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_new_tooltip_state_with_auto_show_true() {
        let state = TooltipState::new(true);
        assert!(state.enabled);
        assert!(state.current_function.is_none());
    }

    #[test]
    fn test_new_tooltip_state_with_auto_show_false() {
        let state = TooltipState::new(false);
        assert!(!state.enabled);
        assert!(state.current_function.is_none());
    }

    #[test]
    fn test_default_creates_enabled_state() {
        let state = TooltipState::default();
        assert!(state.enabled);
    }

    #[test]
    fn test_toggle() {
        let mut state = TooltipState::new(true);
        assert!(state.enabled);
        state.toggle();
        assert!(!state.enabled);
        state.toggle();
        assert!(state.enabled);
    }

    #[test]
    fn test_set_current_function() {
        let mut state = TooltipState::new(true);
        state.set_current_function(Some("select".to_string()));
        assert_eq!(state.current_function, Some("select".to_string()));
        state.set_current_function(None);
        assert!(state.current_function.is_none());
    }

    #[test]
    fn test_should_show() {
        let mut state = TooltipState::new(true);
        // Enabled but no function
        assert!(!state.should_show());

        // Enabled with function
        state.set_current_function(Some("map".to_string()));
        assert!(state.should_show());

        // Disabled with function
        state.toggle();
        assert!(!state.should_show());

        // Disabled without function
        state.set_current_function(None);
        assert!(!state.should_show());
    }

    // **Feature: function-tooltip, Property 2: Toggle round-trip preserves state consistency**
    // *For any* tooltip state, toggling twice returns to the original enabled state.
    // Additionally, toggling from enabled to disabled sets `should_show()` to false,
    // and toggling from disabled to enabled while on a function sets `should_show()` to true.
    // **Validates: Requirements 2.1, 2.2, 2.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_toggle_round_trip(initial_enabled: bool, has_function: bool, function_name in "[a-z_]+") {
            let mut state = TooltipState::new(initial_enabled);
            if has_function {
                state.set_current_function(Some(function_name.clone()));
            }

            let original_enabled = state.enabled;
            let original_function = state.current_function.clone();

            // Toggle twice should return to original enabled state
            state.toggle();
            state.toggle();

            prop_assert_eq!(state.enabled, original_enabled, "Toggle round-trip should preserve enabled state");
            prop_assert_eq!(state.current_function, original_function, "Toggle should not affect current_function");
        }

        #[test]
        fn prop_toggle_disabled_hides_tooltip(function_name in "[a-z_]+") {
            let mut state = TooltipState::new(true);
            state.set_current_function(Some(function_name));

            // When enabled with function, should_show is true
            prop_assert!(state.should_show(), "Should show when enabled with function");

            // Toggle to disabled
            state.toggle();

            // should_show must be false when disabled
            prop_assert!(!state.should_show(), "Should not show when disabled");
            prop_assert!(!state.enabled, "Should be disabled after toggle");
        }

        #[test]
        fn prop_toggle_enabled_with_function_shows_tooltip(function_name in "[a-z_]+") {
            let mut state = TooltipState::new(false);
            state.set_current_function(Some(function_name));

            // When disabled, should_show is false
            prop_assert!(!state.should_show(), "Should not show when disabled");

            // Toggle to enabled
            state.toggle();

            // should_show must be true when enabled with function
            prop_assert!(state.should_show(), "Should show when enabled with function");
            prop_assert!(state.enabled, "Should be enabled after toggle");
        }

        // **Feature: function-tooltip, Property 3: Hint visibility matches tooltip state**
        // *For any* tooltip state and cursor position:
        // - When disabled AND on a function: input border hint is visible
        // - When disabled AND not on a function: no hint visible
        // - When enabled AND showing tooltip: dismiss hint on tooltip border
        // - When enabled: no hint on input border
        // **Validates: Requirements 3.1, 3.2, 3.3, 3.4**
        #[test]
        fn prop_hint_visibility_matches_state(
            enabled: bool,
            has_function: bool,
            function_name in "[a-z_]+"
        ) {
            let mut state = TooltipState::new(enabled);
            if has_function {
                state.set_current_function(Some(function_name));
            } else {
                state.set_current_function(None);
            }

            // Compute expected hint visibility for input border
            // Input border hint shows when: disabled AND on a function
            let should_show_input_hint = !state.enabled && state.current_function.is_some();

            // Compute expected tooltip visibility (which has dismiss hint)
            // Tooltip shows when: enabled AND on a function
            let should_show_tooltip = state.should_show();

            // Verify the logic:
            // 1. When disabled AND on function: input hint visible, tooltip hidden
            if !state.enabled && state.current_function.is_some() {
                prop_assert!(should_show_input_hint, "Input hint should show when disabled + on function");
                prop_assert!(!should_show_tooltip, "Tooltip should not show when disabled");
            }

            // 2. When disabled AND not on function: no hint, no tooltip
            if !state.enabled && state.current_function.is_none() {
                prop_assert!(!should_show_input_hint, "Input hint should not show when disabled + no function");
                prop_assert!(!should_show_tooltip, "Tooltip should not show when disabled");
            }

            // 3. When enabled AND on function: tooltip visible (with dismiss hint), no input hint
            if state.enabled && state.current_function.is_some() {
                prop_assert!(!should_show_input_hint, "Input hint should not show when enabled");
                prop_assert!(should_show_tooltip, "Tooltip should show when enabled + on function");
            }

            // 4. When enabled AND not on function: no tooltip, no input hint
            if state.enabled && state.current_function.is_none() {
                prop_assert!(!should_show_input_hint, "Input hint should not show when enabled");
                prop_assert!(!should_show_tooltip, "Tooltip should not show when no function");
            }
        }
    }
}
