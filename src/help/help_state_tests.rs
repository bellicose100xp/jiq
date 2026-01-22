//! Tests for help_state

use super::*;

#[test]
fn test_new_help_state() {
    let state = HelpPopupState::new();
    assert!(!state.visible);
    assert_eq!(state.active_tab, HelpTab::Global);
    assert_eq!(state.current_scroll().offset, 0);
}

#[test]
fn test_help_tab_all() {
    let tabs = HelpTab::all();
    assert_eq!(tabs.len(), 7);
    assert_eq!(tabs[0], HelpTab::Global);
    assert_eq!(tabs[6], HelpTab::Snippet);
}

#[test]
fn test_help_tab_index() {
    assert_eq!(HelpTab::Global.index(), 0);
    assert_eq!(HelpTab::Input.index(), 1);
    assert_eq!(HelpTab::Result.index(), 2);
    assert_eq!(HelpTab::History.index(), 3);
    assert_eq!(HelpTab::AI.index(), 4);
    assert_eq!(HelpTab::Search.index(), 5);
    assert_eq!(HelpTab::Snippet.index(), 6);
}

#[test]
fn test_help_tab_from_index() {
    assert_eq!(HelpTab::from_index(0), HelpTab::Global);
    assert_eq!(HelpTab::from_index(1), HelpTab::Input);
    assert_eq!(HelpTab::from_index(2), HelpTab::Result);
    assert_eq!(HelpTab::from_index(3), HelpTab::History);
    assert_eq!(HelpTab::from_index(4), HelpTab::AI);
    assert_eq!(HelpTab::from_index(5), HelpTab::Search);
    assert_eq!(HelpTab::from_index(6), HelpTab::Snippet);
    // Out of bounds returns Global
    assert_eq!(HelpTab::from_index(100), HelpTab::Global);
}

#[test]
fn test_help_tab_name() {
    assert_eq!(HelpTab::Global.name(), "Global");
    assert_eq!(HelpTab::Input.name(), "Input");
    assert_eq!(HelpTab::Result.name(), "Result");
    assert_eq!(HelpTab::History.name(), "History");
    assert_eq!(HelpTab::AI.name(), "AI");
    assert_eq!(HelpTab::Search.name(), "Search");
    assert_eq!(HelpTab::Snippet.name(), "Snippet");
}

#[test]
fn test_help_tab_next() {
    assert_eq!(HelpTab::Global.next(), HelpTab::Input);
    assert_eq!(HelpTab::Input.next(), HelpTab::Result);
    assert_eq!(HelpTab::Snippet.next(), HelpTab::Global); // Wraps around
}

#[test]
fn test_help_tab_prev() {
    assert_eq!(HelpTab::Input.prev(), HelpTab::Global);
    assert_eq!(HelpTab::Result.prev(), HelpTab::Input);
    assert_eq!(HelpTab::Global.prev(), HelpTab::Snippet); // Wraps around
}

#[test]
fn test_help_popup_state_current_scroll() {
    let mut state = HelpPopupState::new();

    // Default tab is Global, check its scroll
    state.current_scroll_mut().update_bounds(50, 20);
    state.current_scroll_mut().scroll_down(5);
    assert_eq!(state.current_scroll().offset, 5);

    // Switch tab, should have separate scroll
    state.active_tab = HelpTab::Input;
    assert_eq!(state.current_scroll().offset, 0);

    // Modify Input's scroll
    state.current_scroll_mut().update_bounds(30, 15);
    state.current_scroll_mut().scroll_down(3);
    assert_eq!(state.current_scroll().offset, 3);

    // Switch back to Global, should still be at 5
    state.active_tab = HelpTab::Global;
    assert_eq!(state.current_scroll().offset, 5);
}

#[test]
fn test_help_popup_state_reset() {
    let mut state = HelpPopupState::new();

    state.visible = true;
    state.active_tab = HelpTab::Result;
    state.current_scroll_mut().update_bounds(50, 20);
    state.current_scroll_mut().scroll_down(10);

    state.reset();

    assert!(!state.visible);
    assert_eq!(state.active_tab, HelpTab::Global);
    // All tab scrolls should be reset
    for tab in HelpTab::all() {
        state.active_tab = *tab;
        assert_eq!(state.current_scroll().offset, 0);
    }
}
