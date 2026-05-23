//! Tests for `>` and `<` key handling while the search overlay is active.
//!
//! The search-mode dispatcher intercepts both chords before they reach the
//! search textarea or the confirmed-mode delegate, so these tests must be
//! kept distinct from the results-pane drill tests in
//! `results_events_tests.rs`. Coverage:
//! - `>` resolves the *match* row (not the cursor row) and closes search
//!   on success.
//! - `>` with no current match notifies and leaves search visible.
//! - `>` with a match on the root row notifies and leaves search visible.
//! - `<` works identically to results-pane mode and does not close search.
//! - Both chords work in editing mode and confirmed mode.

use super::super::*;
use crate::test_utils::test_helpers::{key, test_app};
use ratatui::crossterm::event::KeyCode;

/// Build an app with parsed JSON in the result and a single-line search
/// match on a non-root row. Returns the app with search open and the
/// match-row index for assertions.
fn open_search_with_match(json: &str, search_text: &str) -> (crate::app::App, u32) {
    let mut app = test_app(json);
    if let Some(qs) = app.query.as_mut() {
        qs.execute(".");
    }
    let unformatted = app
        .query
        .as_ref()
        .and_then(|qs| qs.last_successful_result_unformatted.clone())
        .map(|s| s.as_ref().clone())
        .unwrap_or_default();

    open_search(&mut app);
    app.search.search_textarea_mut().insert_str(search_text);
    app.search.update_matches(&unformatted);
    let match_row = app
        .search
        .current_match()
        .expect("test setup: at least one match")
        .line;
    (app, match_row)
}

#[test]
fn drill_in_uses_match_row_not_cursor_row() {
    // Cursor sits at row 0 (the root `{` brace). The match is on a
    // non-root row. `>` must drill into the match, not the cursor.
    let (mut app, match_row) =
        open_search_with_match(r#"{"alpha": 1, "beta": 2, "gamma": 3}"#, "gamma");
    let total = app.results_line_count_u32();
    app.results_cursor.update_total_lines(total);
    app.results_cursor.move_to_line(0);
    assert_ne!(match_row, 0, "test guard: match must not be the root row");

    handle_search_key(&mut app, key(KeyCode::Char('>')));

    assert_eq!(app.input.query(), ".gamma");
    assert!(
        !app.search.is_visible(),
        "search closes on successful drill"
    );
}

#[test]
fn drill_in_closes_search_on_success() {
    let (mut app, _) = open_search_with_match(r#"{"target": 1}"#, "target");

    handle_search_key(&mut app, key(KeyCode::Char('>')));

    assert!(!app.search.is_visible());
    assert_eq!(app.input.query(), ".target");
}

#[test]
fn drill_in_with_no_matches_notifies_and_keeps_search_open() {
    let mut app = test_app(r#"{"a": 1}"#);
    if let Some(qs) = app.query.as_mut() {
        qs.execute(".");
    }
    open_search(&mut app);
    app.search.search_textarea_mut().insert_str("missing-token");
    app.search.update_matches("{\"a\": 1}");
    assert!(app.search.matches().is_empty());

    handle_search_key(&mut app, key(KeyCode::Char('>')));

    assert!(app.search.is_visible(), "search stays open with no match");
    assert_eq!(
        app.notification.current_message(),
        Some("No match to navigate to")
    );
    assert_eq!(app.input.query(), "");
}

#[test]
fn drill_in_when_match_resolves_to_root_keeps_search_open() {
    // Root scalar JSON: every line maps to the root path `.`. The handler
    // must notify and *not* close search so the user can pick another match.
    let mut app = test_app(r#"42"#);
    if let Some(qs) = app.query.as_mut() {
        qs.execute(".");
    }
    open_search(&mut app);
    let unformatted = app
        .query
        .as_ref()
        .unwrap()
        .last_successful_result_unformatted
        .as_ref()
        .map(|s| s.as_ref().clone())
        .unwrap_or_default();
    app.search.search_textarea_mut().insert_str("42");
    app.search.update_matches(&unformatted);
    assert!(app.search.current_match().is_some());

    handle_search_key(&mut app, key(KeyCode::Char('>')));

    assert!(app.search.is_visible(), "search stays open at root");
    assert_eq!(app.notification.current_message(), Some("Already at root"));
}

#[test]
fn drill_in_works_in_search_editing_mode() {
    // The handler dispatches `>` ahead of the textarea-input fall-through
    // even when search is unconfirmed. Confirms `>` is not buffered into
    // the search query.
    let (mut app, _) = open_search_with_match(r#"{"target": 1}"#, "target");
    assert!(!app.search.is_confirmed(), "test guard: still editing");

    handle_search_key(&mut app, key(KeyCode::Char('>')));

    assert_eq!(app.input.query(), ".target");
}

#[test]
fn drill_in_works_in_search_confirmed_mode() {
    let (mut app, _) = open_search_with_match(r#"{"target": 1}"#, "target");
    handle_search_key(&mut app, key(KeyCode::Enter));
    assert!(app.search.is_confirmed());

    handle_search_key(&mut app, key(KeyCode::Char('>')));

    assert_eq!(app.input.query(), ".target");
    assert!(!app.search.is_visible());
}

#[test]
fn drill_back_in_search_does_not_close_search() {
    // First drill in (which closes search), then re-open search and drill
    // back. `<` should pop the snapshot and leave the freshly-reopened
    // search overlay alone.
    let (mut app, _) = open_search_with_match(r#"{"target": 1, "other": 2}"#, "target");
    handle_search_key(&mut app, key(KeyCode::Char('>')));
    assert_eq!(app.input.query(), ".target");

    open_search(&mut app);

    handle_search_key(&mut app, key(KeyCode::Char('<')));

    assert_eq!(app.input.query(), "");
    assert!(app.search.is_visible(), "search still visible after back");
}

#[test]
fn drill_back_on_empty_ring_notifies_in_search() {
    let mut app = test_app(r#"{"a": 1}"#);
    if let Some(qs) = app.query.as_mut() {
        qs.execute(".");
    }
    open_search(&mut app);

    handle_search_key(&mut app, key(KeyCode::Char('<')));

    assert_eq!(
        app.notification.current_message(),
        Some("Nothing to go back to")
    );
    assert!(app.search.is_visible());
}

#[test]
fn drill_in_does_not_appear_in_search_query_text() {
    // Regression: confirm `>` is intercepted and never reaches the search
    // textarea, even after the user has typed a few chars.
    let mut app = test_app(r#"{"a": 1}"#);
    if let Some(qs) = app.query.as_mut() {
        qs.execute(".");
    }
    open_search(&mut app);
    app.search.search_textarea_mut().insert_str("a");
    app.search.update_matches("{\"a\": 1}");

    handle_search_key(&mut app, key(KeyCode::Char('>')));

    assert!(!app.search.query().contains('>'));
}

#[test]
fn drill_back_does_not_appear_in_search_query_text() {
    let mut app = test_app(r#"{"a": 1}"#);
    if let Some(qs) = app.query.as_mut() {
        qs.execute(".");
    }
    open_search(&mut app);
    app.search.search_textarea_mut().insert_str("a");

    handle_search_key(&mut app, key(KeyCode::Char('<')));

    assert!(!app.search.query().contains('<'));
}

#[test]
fn search_match_path_resolves_to_match_row_not_cursor() {
    // Sanity check on the renderer-side accessor: with search visible and a
    // match at row N, `App::path_at_row(N)` returns the match's path even
    // when the cursor is on row 0.
    let (mut app, match_row) = open_search_with_match(r#"{"alpha": 1, "beta": 2}"#, "beta");
    let total = app.results_line_count_u32();
    app.results_cursor.update_total_lines(total);
    app.results_cursor.move_to_line(0);

    let match_path = app.path_at_row(match_row).unwrap().to_jq();
    let cursor_path = app.current_cursor_path().unwrap().to_jq();

    assert_eq!(match_path, ".beta");
    assert_eq!(cursor_path, ".");
}
