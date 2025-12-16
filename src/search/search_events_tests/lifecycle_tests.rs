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
