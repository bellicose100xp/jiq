//! Help popup tests

use super::*;

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

    // Set up bounds for help content (48 lines + padding, viewport 20)
    app.help.scroll.update_bounds(60, 20);
    app.help.scroll.offset = 0;

    app.handle_key_event(key(KeyCode::Char('j')));
    assert_eq!(app.help.scroll.offset, 1);
}

#[test]
fn test_help_popup_scroll_k_scrolls_up() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.scroll.offset = 5;

    app.handle_key_event(key(KeyCode::Char('k')));
    assert_eq!(app.help.scroll.offset, 4);
}

#[test]
fn test_help_popup_scroll_down_arrow() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content
    app.help.scroll.update_bounds(60, 20);
    app.help.scroll.offset = 0;

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.help.scroll.offset, 1);
}

#[test]
fn test_help_popup_scroll_up_arrow() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.scroll.offset = 5;

    app.handle_key_event(key(KeyCode::Up));
    assert_eq!(app.help.scroll.offset, 4);
}

#[test]
fn test_help_popup_scroll_capital_j_scrolls_10() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content
    app.help.scroll.update_bounds(60, 20);
    app.help.scroll.offset = 0;

    app.handle_key_event(key(KeyCode::Char('J')));
    assert_eq!(app.help.scroll.offset, 10);
}

#[test]
fn test_help_popup_scroll_capital_k_scrolls_10() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.scroll.offset = 15;

    app.handle_key_event(key(KeyCode::Char('K')));
    assert_eq!(app.help.scroll.offset, 5);
}

#[test]
fn test_help_popup_scroll_ctrl_d() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content
    app.help.scroll.update_bounds(60, 20);
    app.help.scroll.offset = 0;

    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));
    assert_eq!(app.help.scroll.offset, 10);
}

#[test]
fn test_help_popup_scroll_ctrl_u() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.scroll.offset = 15;

    app.handle_key_event(key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL));
    assert_eq!(app.help.scroll.offset, 5);
}

#[test]
fn test_help_popup_scroll_g_jumps_to_top() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.scroll.offset = 20;

    app.handle_key_event(key(KeyCode::Char('g')));
    assert_eq!(app.help.scroll.offset, 0);
}

#[test]
fn test_help_popup_scroll_capital_g_jumps_to_bottom() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content
    app.help.scroll.update_bounds(60, 20);
    app.help.scroll.offset = 0;

    app.handle_key_event(key(KeyCode::Char('G')));
    assert_eq!(app.help.scroll.offset, app.help.scroll.max_offset);
}

#[test]
fn test_help_popup_scroll_k_saturates_at_zero() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.scroll.offset = 0;

    app.handle_key_event(key(KeyCode::Char('k')));
    assert_eq!(app.help.scroll.offset, 0);
}

#[test]
fn test_help_popup_close_resets_scroll() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.scroll.offset = 10;

    app.handle_key_event(key(KeyCode::Esc));
    assert!(!app.help.visible);
    assert_eq!(app.help.scroll.offset, 0);
}

#[test]
fn test_help_popup_scroll_page_down() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content
    app.help.scroll.update_bounds(60, 20);
    app.help.scroll.offset = 0;

    app.handle_key_event(key(KeyCode::PageDown));
    assert_eq!(app.help.scroll.offset, 10);
}

#[test]
fn test_help_popup_scroll_page_up() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.scroll.offset = 15;

    app.handle_key_event(key(KeyCode::PageUp));
    assert_eq!(app.help.scroll.offset, 5);
}

#[test]
fn test_help_popup_scroll_home_jumps_to_top() {
    let mut app = app_with_query(".");
    app.help.visible = true;
    app.help.scroll.offset = 20;

    app.handle_key_event(key(KeyCode::Home));
    assert_eq!(app.help.scroll.offset, 0);
}

#[test]
fn test_help_popup_scroll_end_jumps_to_bottom() {
    let mut app = app_with_query(".");
    app.help.visible = true;

    // Set up bounds for help content
    app.help.scroll.update_bounds(60, 20);
    app.help.scroll.offset = 0;

    app.handle_key_event(key(KeyCode::End));
    assert_eq!(app.help.scroll.offset, app.help.scroll.max_offset);
}
