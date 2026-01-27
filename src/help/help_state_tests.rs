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

    // Hovered tab should also be reset
    assert_eq!(state.get_hovered_tab(), None);
}

#[test]
fn test_help_popup_hovered_tab() {
    let mut state = HelpPopupState::new();

    assert_eq!(state.get_hovered_tab(), None);

    state.set_hovered_tab(Some(HelpTab::Input));
    assert_eq!(state.get_hovered_tab(), Some(HelpTab::Input));

    state.set_hovered_tab(Some(HelpTab::AI));
    assert_eq!(state.get_hovered_tab(), Some(HelpTab::AI));

    state.clear_hovered_tab();
    assert_eq!(state.get_hovered_tab(), None);
}

#[test]
fn test_tab_at_x_global_active() {
    let state = HelpPopupState::new(); // Global is active
    // Use container width equal to tab bar width (no centering)
    let width = state.tab_bar_width();

    // Active Global has [1:Global] = 10 chars
    // Tab positions with Global active (3-char divider):
    // [1:Global] = 10 chars, divider = 3
    // 2:Input = 7 chars, divider = 3
    // 3:Result = 8 chars, ...

    // Position 0-9: [1:Global]
    assert_eq!(state.tab_at_x(0, width), Some(HelpTab::Global));
    assert_eq!(state.tab_at_x(9, width), Some(HelpTab::Global));

    // Position 10-12 is divider (3 chars)
    assert_eq!(state.tab_at_x(10, width), None);
    assert_eq!(state.tab_at_x(12, width), None);

    // Position 13-19: 2:Input (7 chars)
    assert_eq!(state.tab_at_x(13, width), Some(HelpTab::Input));
    assert_eq!(state.tab_at_x(19, width), Some(HelpTab::Input));

    // Position 20-22 is divider
    assert_eq!(state.tab_at_x(20, width), None);

    // Position 23-30: 3:Result (8 chars)
    assert_eq!(state.tab_at_x(23, width), Some(HelpTab::Result));
}

#[test]
fn test_tab_at_x_input_active() {
    let mut state = HelpPopupState::new();
    state.active_tab = HelpTab::Input;
    // Use container width equal to tab bar width (no centering)
    let width = state.tab_bar_width();

    // With Input active (3-char divider):
    // 1:Global = 8 chars, divider = 3
    // [2:Input] = 9 chars, divider = 3
    // 3:Result = 8 chars, ...

    // Position 0-7: 1:Global (8 chars)
    assert_eq!(state.tab_at_x(0, width), Some(HelpTab::Global));
    assert_eq!(state.tab_at_x(7, width), Some(HelpTab::Global));

    // Position 8-10 is divider (3 chars)
    assert_eq!(state.tab_at_x(8, width), None);
    assert_eq!(state.tab_at_x(10, width), None);

    // Position 11-19: [2:Input] (9 chars)
    assert_eq!(state.tab_at_x(11, width), Some(HelpTab::Input));
    assert_eq!(state.tab_at_x(19, width), Some(HelpTab::Input));
}

#[test]
fn test_tab_at_x_out_of_bounds() {
    let state = HelpPopupState::new();
    let width = state.tab_bar_width();

    // Way past the end
    assert_eq!(state.tab_at_x(200, width), None);
}

#[test]
fn test_tab_at_x_with_centering() {
    let state = HelpPopupState::new();
    let tab_bar_width = state.tab_bar_width();
    // Use a wider container to test centering
    let container_width = tab_bar_width + 20; // 10 chars padding on each side

    // Clicking before the centered content should return None
    assert_eq!(state.tab_at_x(0, container_width), None);
    assert_eq!(state.tab_at_x(9, container_width), None);

    // Position 10 is now the start of [1:Global]
    assert_eq!(state.tab_at_x(10, container_width), Some(HelpTab::Global));
    assert_eq!(state.tab_at_x(19, container_width), Some(HelpTab::Global));
}
