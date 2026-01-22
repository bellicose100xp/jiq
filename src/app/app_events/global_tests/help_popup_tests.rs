//! Help popup tests

use super::*;
use crate::help::HelpTab;

#[test]
fn test_help_popup_initializes_hidden() {
    let app = test_app(TEST_JSON);
    assert!(!app.help.visible);
}

#[test]
fn test_f1_toggles_help_popup() {
    let mut app = app_with_query(".");
    assert!(!app.help.visible);

    app.handle_key_event(key(KeyCode::F(1)));
    assert!(app.help.visible);

    app.handle_key_event(key(KeyCode::F(1)));
    assert!(!app.help.visible);
}

#[test]
fn test_question_mark_toggles_help_in_normal_mode() {
    let mut app = app_with_query(".");
    app.input.editor_mode = EditorMode::Normal;
    app.focus = Focus::InputField;

    app.handle_key_event(key(KeyCode::Char('?')));
    assert!(app.help.visible);

    app.handle_key_event(key(KeyCode::Char('?')));
    assert!(!app.help.visible);
}

#[test]
fn test_question_mark_does_not_toggle_help_in_insert_mode() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    app.handle_key_event(key(KeyCode::Char('?')));
    // Should type '?' not toggle help
    assert!(!app.help.visible);
    assert!(app.query().contains('?'));
}

#[test]
fn test_esc_closes_help_popup() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    app.handle_key_event(key(KeyCode::Esc));
    assert!(!app.help.visible);
}

#[test]
fn test_q_closes_help_popup() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    app.handle_key_event(key(KeyCode::Char('q')));
    assert!(!app.help.visible);
}

#[test]
fn test_help_popup_blocks_other_keys() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.input.editor_mode = EditorMode::Insert;

    // Try to type - should be blocked
    app.handle_key_event(key(KeyCode::Char('x')));
    assert!(!app.query().contains('x'));
    assert!(app.help.visible);
}

#[test]
fn test_f1_works_in_insert_mode() {
    let mut app = app_with_query(".");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    app.handle_key_event(key(KeyCode::F(1)));
    assert!(app.help.visible);
}

#[test]
fn test_help_popup_scroll_j_scrolls_down() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content (using current tab's scroll)
    app.help.current_scroll_mut().update_bounds(60, 20);

    app.handle_key_event(key(KeyCode::Char('j')));
    assert_eq!(app.help.current_scroll().offset, 1);
}

#[test]
fn test_help_popup_scroll_k_scrolls_up() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.current_scroll_mut().update_bounds(60, 20);
    app.help.current_scroll_mut().scroll_down(5);

    app.handle_key_event(key(KeyCode::Char('k')));
    assert_eq!(app.help.current_scroll().offset, 4);
}

#[test]
fn test_help_popup_scroll_down_arrow() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content
    app.help.current_scroll_mut().update_bounds(60, 20);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.help.current_scroll().offset, 1);
}

#[test]
fn test_help_popup_scroll_up_arrow() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.current_scroll_mut().update_bounds(60, 20);
    app.help.current_scroll_mut().scroll_down(5);

    app.handle_key_event(key(KeyCode::Up));
    assert_eq!(app.help.current_scroll().offset, 4);
}

#[test]
fn test_help_popup_scroll_capital_j_scrolls_10() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content
    app.help.current_scroll_mut().update_bounds(60, 20);

    app.handle_key_event(key(KeyCode::Char('J')));
    assert_eq!(app.help.current_scroll().offset, 10);
}

#[test]
fn test_help_popup_scroll_capital_k_scrolls_10() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.current_scroll_mut().update_bounds(60, 20);
    app.help.current_scroll_mut().scroll_down(15);

    app.handle_key_event(key(KeyCode::Char('K')));
    assert_eq!(app.help.current_scroll().offset, 5);
}

#[test]
fn test_help_popup_scroll_ctrl_d() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content
    app.help.current_scroll_mut().update_bounds(60, 20);

    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));
    assert_eq!(app.help.current_scroll().offset, 10);
}

#[test]
fn test_help_popup_scroll_ctrl_u() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.current_scroll_mut().update_bounds(60, 20);
    app.help.current_scroll_mut().scroll_down(15);

    app.handle_key_event(key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL));
    assert_eq!(app.help.current_scroll().offset, 5);
}

#[test]
fn test_help_popup_scroll_g_jumps_to_top() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.current_scroll_mut().update_bounds(60, 20);
    app.help.current_scroll_mut().scroll_down(20);

    app.handle_key_event(key(KeyCode::Char('g')));
    assert_eq!(app.help.current_scroll().offset, 0);
}

#[test]
fn test_help_popup_scroll_capital_g_jumps_to_bottom() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content
    app.help.current_scroll_mut().update_bounds(60, 20);

    app.handle_key_event(key(KeyCode::Char('G')));
    assert_eq!(
        app.help.current_scroll().offset,
        app.help.current_scroll().max_offset
    );
}

#[test]
fn test_help_popup_scroll_k_saturates_at_zero() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    // offset is 0 by default

    app.handle_key_event(key(KeyCode::Char('k')));
    assert_eq!(app.help.current_scroll().offset, 0);
}

#[test]
fn test_help_popup_close_resets_scroll() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.current_scroll_mut().update_bounds(60, 20);
    app.help.current_scroll_mut().scroll_down(10);

    app.handle_key_event(key(KeyCode::Esc));
    assert!(!app.help.visible);
    assert_eq!(app.help.current_scroll().offset, 0);
}

#[test]
fn test_help_popup_scroll_page_down() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content
    app.help.current_scroll_mut().update_bounds(60, 20);

    app.handle_key_event(key(KeyCode::PageDown));
    assert_eq!(app.help.current_scroll().offset, 10);
}

#[test]
fn test_help_popup_scroll_page_up() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.current_scroll_mut().update_bounds(60, 20);
    app.help.current_scroll_mut().scroll_down(15);

    app.handle_key_event(key(KeyCode::PageUp));
    assert_eq!(app.help.current_scroll().offset, 5);
}

#[test]
fn test_help_popup_scroll_home_jumps_to_top() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.current_scroll_mut().update_bounds(60, 20);
    app.help.current_scroll_mut().scroll_down(20);

    app.handle_key_event(key(KeyCode::Home));
    assert_eq!(app.help.current_scroll().offset, 0);
}

#[test]
fn test_help_popup_scroll_end_jumps_to_bottom() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content
    app.help.current_scroll_mut().update_bounds(60, 20);

    app.handle_key_event(key(KeyCode::End));
    assert_eq!(
        app.help.current_scroll().offset,
        app.help.current_scroll().max_offset
    );
}

// Tab navigation tests

#[test]
fn test_help_popup_h_navigates_to_previous_tab() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.active_tab = HelpTab::Input;

    app.handle_key_event(key(KeyCode::Char('h')));
    assert_eq!(app.help.active_tab, HelpTab::Global);
}

#[test]
fn test_help_popup_l_navigates_to_next_tab() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.active_tab = HelpTab::Global;

    app.handle_key_event(key(KeyCode::Char('l')));
    assert_eq!(app.help.active_tab, HelpTab::Input);
}

#[test]
fn test_help_popup_left_arrow_navigates_to_previous_tab() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.active_tab = HelpTab::Result;

    app.handle_key_event(key(KeyCode::Left));
    assert_eq!(app.help.active_tab, HelpTab::Input);
}

#[test]
fn test_help_popup_right_arrow_navigates_to_next_tab() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.active_tab = HelpTab::Input;

    app.handle_key_event(key(KeyCode::Right));
    assert_eq!(app.help.active_tab, HelpTab::Result);
}

#[test]
fn test_help_popup_tab_key_navigates_to_next_tab() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.active_tab = HelpTab::Global;

    app.handle_key_event(key(KeyCode::Tab));
    assert_eq!(app.help.active_tab, HelpTab::Input);
}

#[test]
fn test_help_popup_shift_tab_navigates_to_previous_tab() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.active_tab = HelpTab::Input;

    app.handle_key_event(key(KeyCode::BackTab));
    assert_eq!(app.help.active_tab, HelpTab::Global);
}

#[test]
fn test_help_popup_tab_wraps_at_end() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.active_tab = HelpTab::Snippet;

    app.handle_key_event(key(KeyCode::Char('l')));
    assert_eq!(app.help.active_tab, HelpTab::Global);
}

#[test]
fn test_help_popup_tab_wraps_at_beginning() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.active_tab = HelpTab::Global;

    app.handle_key_event(key(KeyCode::Char('h')));
    assert_eq!(app.help.active_tab, HelpTab::Snippet);
}

#[test]
fn test_help_popup_number_keys_jump_to_tab() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    app.handle_key_event(key(KeyCode::Char('1')));
    assert_eq!(app.help.active_tab, HelpTab::Global);

    app.handle_key_event(key(KeyCode::Char('2')));
    assert_eq!(app.help.active_tab, HelpTab::Input);

    app.handle_key_event(key(KeyCode::Char('3')));
    assert_eq!(app.help.active_tab, HelpTab::Result);

    app.handle_key_event(key(KeyCode::Char('4')));
    assert_eq!(app.help.active_tab, HelpTab::History);

    app.handle_key_event(key(KeyCode::Char('5')));
    assert_eq!(app.help.active_tab, HelpTab::AI);

    app.handle_key_event(key(KeyCode::Char('6')));
    assert_eq!(app.help.active_tab, HelpTab::Search);

    app.handle_key_event(key(KeyCode::Char('7')));
    assert_eq!(app.help.active_tab, HelpTab::Snippet);
}

// Context-aware tab selection tests

#[test]
fn test_help_opens_to_input_tab_when_input_focused() {
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key(KeyCode::F(1)));
    assert!(app.help.visible);
    assert_eq!(app.help.active_tab, HelpTab::Input);
}

#[test]
fn test_help_opens_to_result_tab_when_results_focused() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    app.handle_key_event(key(KeyCode::F(1)));
    assert!(app.help.visible);
    assert_eq!(app.help.active_tab, HelpTab::Result);
}

#[test]
fn test_help_opens_to_search_tab_when_search_active() {
    let mut app = app_with_query(".");
    crate::search::search_events::open_search(&mut app);

    app.handle_key_event(key(KeyCode::F(1)));
    assert!(app.help.visible);
    assert_eq!(app.help.active_tab, HelpTab::Search);
}

#[test]
fn test_help_opens_to_snippet_tab_when_snippets_visible() {
    let mut app = app_with_query(".");
    app.snippets.open();

    app.handle_key_event(key(KeyCode::F(1)));
    assert!(app.help.visible);
    assert_eq!(app.help.active_tab, HelpTab::Snippet);
}

#[test]
fn test_help_does_not_auto_focus_history_tab() {
    // History tab never auto-focuses - should use the underlying context (Input here)
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;
    app.history.open(None);

    app.handle_key_event(key(KeyCode::F(1)));
    assert!(app.help.visible);
    // Should NOT be History tab - falls back to Input since focus is InputField
    assert_eq!(app.help.active_tab, HelpTab::Input);
}

#[test]
fn test_help_does_not_auto_focus_ai_tab() {
    // AI tab never auto-focuses - should use the underlying context (Input here)
    let mut app = app_with_query(".");
    app.focus = Focus::InputField;
    app.ai.visible = true;

    app.handle_key_event(key(KeyCode::F(1)));
    assert!(app.help.visible);
    // Should NOT be AI tab - falls back to Input since focus is InputField
    assert_eq!(app.help.active_tab, HelpTab::Input);
}

#[test]
fn test_each_tab_has_independent_scroll() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.active_tab = HelpTab::Global;
    app.help.current_scroll_mut().update_bounds(60, 20);
    app.help.current_scroll_mut().scroll_down(5);

    // Switch to Input tab
    app.help.active_tab = HelpTab::Input;
    app.help.current_scroll_mut().update_bounds(60, 20);
    assert_eq!(app.help.current_scroll().offset, 0);

    // Scroll Input tab
    app.help.current_scroll_mut().scroll_down(10);
    assert_eq!(app.help.current_scroll().offset, 10);

    // Switch back to Global - should still be at 5
    app.help.active_tab = HelpTab::Global;
    assert_eq!(app.help.current_scroll().offset, 5);
}
