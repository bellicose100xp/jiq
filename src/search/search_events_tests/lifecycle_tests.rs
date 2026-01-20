use super::super::*;
use crate::app::Focus;
use crate::test_utils::test_helpers::{key, test_app};

#[test]
fn test_open_search_sets_visible_and_focus() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.focus = Focus::InputField;

    open_search(&mut app);

    assert!(app.search.is_visible());
    assert_eq!(app.focus, Focus::ResultsPane);
}

#[test]
fn test_close_search_clears_state() {
    let mut app = test_app(r#"{"name": "test"}"#);
    open_search(&mut app);
    app.search.search_textarea_mut().insert_str("test");

    close_search(&mut app);

    assert!(!app.search.is_visible());
    assert!(app.search.query().is_empty());
}

#[test]
fn test_handle_search_key_returns_false_when_not_visible() {
    let mut app = test_app(r#"{"name": "test"}"#);
    assert!(!app.search.is_visible());

    let handled = handle_search_key(&mut app, key(KeyCode::Char('n')));

    assert!(!handled);
}

#[test]
fn test_escape_closes_search() {
    let mut app = test_app(r#"{"name": "test"}"#);
    open_search(&mut app);

    let handled = handle_search_key(&mut app, key(KeyCode::Esc));

    assert!(handled);
    assert!(!app.search.is_visible());
}

#[test]
fn test_text_input_updates_query() {
    let mut app = test_app(r#"{"name": "test"}"#);
    open_search(&mut app);

    handle_search_key(&mut app, key(KeyCode::Char('t')));
    handle_search_key(&mut app, key(KeyCode::Char('e')));
    handle_search_key(&mut app, key(KeyCode::Char('s')));
    handle_search_key(&mut app, key(KeyCode::Char('t')));

    assert_eq!(app.search.query(), "test");
}

#[test]
fn test_open_search_hides_ai_and_saves_state() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.ai.visible = true;

    open_search(&mut app);

    assert!(!app.ai.visible);
    assert!(app.saved_ai_visibility_for_search);
}

#[test]
fn test_open_search_hides_tooltip_and_saves_state() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.tooltip.enabled = true;

    open_search(&mut app);

    assert!(!app.tooltip.enabled);
    assert!(app.saved_tooltip_visibility_for_search);
}

#[test]
fn test_close_search_restores_ai_visibility() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.ai.visible = true;

    open_search(&mut app);
    assert!(!app.ai.visible);

    close_search(&mut app);
    assert!(app.ai.visible);
}

#[test]
fn test_close_search_restores_tooltip_visibility() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.tooltip.enabled = true;

    open_search(&mut app);
    assert!(!app.tooltip.enabled);

    close_search(&mut app);
    assert!(app.tooltip.enabled);
}

#[test]
fn test_open_search_preserves_hidden_ai_state() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.ai.visible = false;

    open_search(&mut app);
    close_search(&mut app);

    assert!(!app.ai.visible);
}

#[test]
fn test_open_search_preserves_disabled_tooltip_state() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.tooltip.enabled = false;

    open_search(&mut app);
    close_search(&mut app);

    assert!(!app.tooltip.enabled);
}

#[test]
fn test_open_close_search_with_both_ai_and_tooltip_active() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.ai.visible = true;
    app.tooltip.enabled = true;

    open_search(&mut app);
    assert!(!app.ai.visible);
    assert!(!app.tooltip.enabled);

    close_search(&mut app);
    assert!(app.ai.visible);
    assert!(app.tooltip.enabled);
}
