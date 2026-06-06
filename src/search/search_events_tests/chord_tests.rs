//! Tests for the `*` (iterate-splat) and `}` (keep-kv) drill chords while
//! the search overlay is confirmed.
//!
//! Both chords are intercepted by the search-mode dispatcher before they
//! reach the textarea or the confirmed-mode delegate, and both resolve the
//! *match* row's path rather than the cursor row. They are the analogues of
//! the already-tested `>` drill: success closes the overlay and rewrites the
//! query; a soft failure surfaces a notification and leaves search visible
//! without touching the query.

use super::super::*;
use crate::test_utils::test_helpers::{key, test_app};
use ratatui::crossterm::event::KeyCode;

/// Build an app with parsed JSON in the result and a confirmed search whose
/// current match lands on the row matching `search_text`. Mirrors
/// `drill_tests::open_search_with_match` but confirms the search so the
/// drill chords are live.
fn confirmed_search_with_match(json: &str, search_text: &str) -> crate::app::App {
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
    app.search
        .current_match()
        .expect("test setup: at least one match");
    app.search.confirm();
    app
}

#[test]
fn iterate_chord_splats_array_match_and_closes_search() {
    // Match "20" lands on the array-element row `.items[1]`. `*` splats the
    // nearest index to `[]`, composing `.items[]`, then closes the overlay.
    let mut app = confirmed_search_with_match(r#"{"items": [10, 20, 30]}"#, "20");

    handle_search_key(&mut app, key(KeyCode::Char('*')));

    assert_eq!(app.input.query(), ".items[]");
    assert!(
        !app.search.is_visible(),
        "search closes on successful iterate-splat"
    );
}

#[test]
fn iterate_chord_with_no_array_in_path_notifies_and_keeps_search_open() {
    // Match "1" resolves to `.a`, a pure key path with no index to splat.
    // `*` must return NoArrayToIterate, notify, and not mutate the query.
    let mut app = confirmed_search_with_match(r#"{"a": 1}"#, "1");
    let query_before = app.input.query().to_string();

    handle_search_key(&mut app, key(KeyCode::Char('*')));

    assert!(
        app.search.is_visible(),
        "search stays open on no-array failure"
    );
    assert_eq!(
        app.notification.current_message(),
        Some("No array in path to iterate")
    );
    assert_eq!(
        app.input.query(),
        query_before,
        "query untouched when iterate fails"
    );
}

#[test]
fn keep_kv_chord_wraps_keyed_match_and_closes_search() {
    // Match "target" resolves to the keyed leaf `.target` (parent is root),
    // so `}` wraps it as `{target}` and closes the overlay.
    let mut app = confirmed_search_with_match(r#"{"target": 1}"#, "target");

    handle_search_key(&mut app, key(KeyCode::Char('}')));

    assert!(
        app.input.query().contains("{target}"),
        "query should wrap the key, got `{}`",
        app.input.query()
    );
    assert!(
        !app.search.is_visible(),
        "search closes on successful keep-kv"
    );
}

#[test]
fn keep_kv_chord_on_array_element_match_notifies_no_key() {
    // Match "42" lands on the array element `.items[0]`, an Index step with
    // no trailing Key. `}` must return NoKeyToWrap, notify, leave query alone.
    let mut app = confirmed_search_with_match(r#"{"items": [42]}"#, "42");
    let query_before = app.input.query().to_string();

    handle_search_key(&mut app, key(KeyCode::Char('}')));

    assert!(
        app.search.is_visible(),
        "search stays open on no-key failure"
    );
    assert_eq!(
        app.notification.current_message(),
        Some("No key at cursor to wrap")
    );
    assert_eq!(
        app.input.query(),
        query_before,
        "query untouched when keep-kv fails"
    );
}
