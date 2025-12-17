//! Tests for tooltip/tooltip_events

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
