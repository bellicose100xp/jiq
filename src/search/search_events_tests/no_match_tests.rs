use super::super::*;
use crate::test_utils::test_helpers::{key, test_app};
use std::sync::Arc;

fn setup_app_with_matchable_content(query_state_content: &str) -> crate::app::App {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.query.as_mut().unwrap().last_successful_result =
        Some(Arc::new(query_state_content.to_string()));
    app.query
        .as_mut()
        .unwrap()
        .last_successful_result_unformatted = Some(Arc::new(query_state_content.to_string()));

    app.results_scroll.viewport_height = 10;
    app.results_scroll.max_offset = 100;
    app.results_scroll.max_h_offset = 100;
    open_search(&mut app);
    app
}

#[test]
fn test_typing_no_match_resets_scroll_to_top() {
    let content: String = (0..50)
        .map(|i| {
            if i == 30 {
                "match line\n".to_string()
            } else {
                format!("other {}\n", i)
            }
        })
        .collect();
    let mut app = setup_app_with_matchable_content(&content);

    handle_search_key(&mut app, key(KeyCode::Char('m')));
    handle_search_key(&mut app, key(KeyCode::Char('a')));
    handle_search_key(&mut app, key(KeyCode::Char('t')));
    handle_search_key(&mut app, key(KeyCode::Char('c')));
    handle_search_key(&mut app, key(KeyCode::Char('h')));

    assert_eq!(app.search.matches().len(), 1);
    assert!(
        app.results_scroll.offset > 0,
        "viewport should have scrolled to the match"
    );

    handle_search_key(&mut app, key(KeyCode::Char('z')));

    assert_eq!(app.search.query(), "matchz");
    assert!(app.search.matches().is_empty());
    assert_eq!(
        app.results_scroll.offset, 0,
        "offset should reset to 0 on no-match"
    );
    assert_eq!(
        app.results_scroll.h_offset, 0,
        "h_offset should reset to 0 on no-match"
    );
}

#[test]
fn test_typing_no_match_resets_horizontal_scroll() {
    let content = format!("{}match\n", " ".repeat(150));
    let mut app = setup_app_with_matchable_content(&content);

    handle_search_key(&mut app, key(KeyCode::Char('m')));
    handle_search_key(&mut app, key(KeyCode::Char('a')));
    handle_search_key(&mut app, key(KeyCode::Char('t')));
    handle_search_key(&mut app, key(KeyCode::Char('c')));
    handle_search_key(&mut app, key(KeyCode::Char('h')));

    assert!(
        app.results_scroll.h_offset > 0,
        "should have scrolled horizontally to the match"
    );

    handle_search_key(&mut app, key(KeyCode::Char('z')));

    assert!(app.search.matches().is_empty());
    assert_eq!(app.results_scroll.h_offset, 0);
    assert_eq!(app.results_scroll.offset, 0);
}

#[test]
fn test_backspace_to_match_restores_scroll_to_match() {
    let content: String = (0..50)
        .map(|i| {
            if i == 30 {
                "match line\n".to_string()
            } else {
                format!("other {}\n", i)
            }
        })
        .collect();
    let mut app = setup_app_with_matchable_content(&content);

    handle_search_key(&mut app, key(KeyCode::Char('m')));
    handle_search_key(&mut app, key(KeyCode::Char('a')));
    handle_search_key(&mut app, key(KeyCode::Char('t')));
    handle_search_key(&mut app, key(KeyCode::Char('c')));
    handle_search_key(&mut app, key(KeyCode::Char('h')));
    handle_search_key(&mut app, key(KeyCode::Char('z')));

    assert!(app.search.matches().is_empty());
    assert_eq!(app.results_scroll.offset, 0);

    handle_search_key(&mut app, key(KeyCode::Backspace));

    assert_eq!(app.search.query(), "match");
    assert_eq!(app.search.matches().len(), 1);
    assert!(
        app.results_scroll.offset > 0,
        "scroll should re-target the match after backspace"
    );
}

#[test]
fn test_backspace_to_empty_query_does_not_reset_scroll() {
    let content: String = (0..50)
        .map(|i| {
            if i == 30 {
                "match line\n".to_string()
            } else {
                format!("other {}\n", i)
            }
        })
        .collect();
    let mut app = setup_app_with_matchable_content(&content);

    app.results_scroll.offset = 25;
    app.results_scroll.h_offset = 7;

    handle_search_key(&mut app, key(KeyCode::Char('z')));
    assert!(app.search.matches().is_empty());
    assert_eq!(
        app.results_scroll.offset, 0,
        "no-match while typing should reset"
    );

    app.results_scroll.offset = 25;
    app.results_scroll.h_offset = 7;

    handle_search_key(&mut app, key(KeyCode::Backspace));

    assert!(app.search.query().is_empty());
    assert_eq!(
        app.results_scroll.offset, 25,
        "empty-query path must not reset offset"
    );
    assert_eq!(
        app.results_scroll.h_offset, 7,
        "empty-query path must not reset h_offset"
    );
}
