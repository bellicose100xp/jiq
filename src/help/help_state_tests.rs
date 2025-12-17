//! Tests for help_state

use super::*;

#[test]
fn test_new_help_state() {
    let state = HelpPopupState::new();
    assert!(!state.visible);
    assert_eq!(state.scroll.offset, 0);
}
