//! Tooltip event handlers
//!
//! Handles keyboard events for the tooltip feature.

use super::tooltip_state::TooltipState;

/// Handle Ctrl+I keypress to toggle tooltip visibility
///
/// Returns `true` to indicate the event was handled.
///
/// # Requirements
/// - 2.1: When pressed while enabled, sets tooltip state to disabled
/// - 2.2: When pressed while disabled, sets tooltip state to enabled
pub fn handle_tooltip_toggle(state: &mut TooltipState) -> bool {
    state.toggle();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_tooltip_toggle_from_enabled() {
        let mut state = TooltipState::new(true);
        assert!(state.enabled);

        let handled = handle_tooltip_toggle(&mut state);

        assert!(handled);
        assert!(!state.enabled);
    }

    #[test]
    fn test_handle_tooltip_toggle_from_disabled() {
        let mut state = TooltipState::new(false);
        assert!(!state.enabled);

        let handled = handle_tooltip_toggle(&mut state);

        assert!(handled);
        assert!(state.enabled);
    }

    #[test]
    fn test_handle_tooltip_toggle_preserves_current_function() {
        let mut state = TooltipState::new(true);
        state.set_current_function(Some("select".to_string()));

        handle_tooltip_toggle(&mut state);

        assert_eq!(state.current_function, Some("select".to_string()));
    }
}
