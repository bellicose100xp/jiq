#[path = "search_events_tests/chord_tests.rs"]
mod chord_tests;
#[path = "search_events_tests/drill_tests.rs"]
mod drill_tests;
#[path = "search_events_tests/lifecycle_tests.rs"]
mod lifecycle_tests;
#[path = "search_events_tests/navigation_tests.rs"]
mod navigation_tests;
#[path = "search_events_tests/no_match_tests.rs"]
mod no_match_tests;
#[path = "search_events_tests/scroll_tests.rs"]
mod scroll_tests;

// Round-2 coverage tests for the search-event dispatcher's remaining
// reachable branches: the lowercase-`n`+SHIFT prev-match arm, the iterate
// (`*`) and keep-kv (`}`) no-match bails, and the NoPath chord paths that
// fire when a match exists but the underlying result is un-drillable.
use super::*;
use crate::test_utils::test_helpers::{key, key_with_mods, test_app};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use std::sync::Arc;

/// Confirmed search over `content` with `token` matched. Sets both the
/// formatted and unformatted result so the dispatcher's match-update path
/// (which reads `last_successful_result_unformatted`) works, then confirms.
fn confirmed_search(content: &str, token: &str) -> crate::app::App {
    let mut app = test_app(r#"{"name": "test"}"#);
    let qs = app.query.as_mut().unwrap();
    qs.last_successful_result = Some(Arc::new(content.to_string()));
    qs.last_successful_result_unformatted = Some(Arc::new(content.to_string()));
    open_search(&mut app);
    app.search.search_textarea_mut().insert_str(token);
    app.search.update_matches(content);
    app.search.confirm();
    app
}

/// Confirmed search over real parsed JSON whose current match lands on the
/// `token` row. Mirrors `chord_tests::confirmed_search_with_match` but is a
/// local copy so this module owns its setup without reaching across modules.
fn confirmed_search_over_json(json: &str, token: &str) -> crate::app::App {
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
    app.search.search_textarea_mut().insert_str(token);
    app.search.update_matches(&unformatted);
    app.search
        .current_match()
        .expect("test setup: at least one match");
    app.search.confirm();
    app
}

#[test]
fn shift_n_navigates_to_prev_match_when_confirmed() {
    // Terminals that deliver Shift+n as lowercase `n` WITH the SHIFT modifier
    // must hit the prev-match arm (lines 67-73), mirroring the `N` arm. From
    // index 0 this wraps backward to the last match (index 2).
    let mut app = confirmed_search("test\ntest\ntest", "test");
    assert_eq!(app.search.current_index(), 0);

    handle_search_key(
        &mut app,
        key_with_mods(KeyCode::Char('n'), KeyModifiers::SHIFT),
    );

    assert_eq!(
        app.search.current_index(),
        2,
        "Shift+n (lowercase n + SHIFT) navigates backward, wrapping to last match"
    );
}

#[test]
fn iterate_chord_with_no_match_notifies_and_keeps_search_open() {
    // `*` with zero current matches must bail via resolve_match_row's None
    // arm (line 219): notify, stay open, leave the query untouched.
    let mut app = confirmed_search("alpha\nbeta", "no-such-token");
    assert!(app.search.matches().is_empty(), "test guard: no matches");
    let query_before = app.input.query().to_string();

    handle_search_key(&mut app, key(KeyCode::Char('*')));

    assert!(
        app.search.is_visible(),
        "search stays open when iterate finds no match"
    );
    assert_eq!(
        app.notification.current_message(),
        Some("No match to navigate to")
    );
    assert_eq!(
        app.input.query(),
        query_before,
        "query untouched on iterate no-match bail"
    );
}

#[test]
fn keep_kv_chord_with_no_match_notifies_and_keeps_search_open() {
    // `}` with zero current matches must bail via resolve_match_row's None
    // arm (line 236): notify, stay open, leave the query untouched.
    let mut app = confirmed_search("alpha\nbeta", "no-such-token");
    assert!(app.search.matches().is_empty(), "test guard: no matches");
    let query_before = app.input.query().to_string();

    handle_search_key(&mut app, key(KeyCode::Char('}')));

    assert!(
        app.search.is_visible(),
        "search stays open when keep-kv finds no match"
    );
    assert_eq!(
        app.notification.current_message(),
        Some("No match to navigate to")
    );
    assert_eq!(
        app.input.query(),
        query_before,
        "query untouched on keep-kv no-match bail"
    );
}

#[test]
fn drill_chord_notifies_no_path_when_result_is_undrillable() {
    // A search match exists, but the parsed result is a synthetic merge, so
    // resolve_path bails and apply_path returns NoPath. `>` must surface
    // "No path at cursor" (line 207), stay open, and leave the query alone.
    let mut app = confirmed_search_over_json(r#"{"target": 1}"#, "target");
    app.query.as_mut().unwrap().is_synthetic_merge = true;
    assert!(
        app.search.current_match().is_some(),
        "test guard: match independent of parsed state"
    );
    let query_before = app.input.query().to_string();

    handle_search_key(&mut app, key(KeyCode::Char('>')));

    assert_eq!(
        app.notification.current_message(),
        Some("No path at cursor")
    );
    assert!(app.search.is_visible(), "search stays open on NoPath");
    assert_eq!(app.input.query(), query_before, "query untouched on NoPath");
}

#[test]
fn iterate_chord_notifies_no_path_when_result_is_undrillable() {
    // Same un-drillable setup; `*` routes through apply_iterate -> NoPath,
    // surfacing "No path at cursor" (line 224) without touching the query.
    let mut app = confirmed_search_over_json(r#"{"target": 1}"#, "target");
    app.query.as_mut().unwrap().is_synthetic_merge = true;
    let query_before = app.input.query().to_string();

    handle_search_key(&mut app, key(KeyCode::Char('*')));

    assert_eq!(
        app.notification.current_message(),
        Some("No path at cursor")
    );
    assert!(app.search.is_visible(), "search stays open on NoPath");
    assert_eq!(app.input.query(), query_before, "query untouched on NoPath");
}

#[test]
fn keep_kv_chord_notifies_no_path_when_result_is_undrillable() {
    // Same un-drillable setup; `}` routes through apply_keep_kv -> NoPath,
    // surfacing "No path at cursor" (line 241) without touching the query.
    let mut app = confirmed_search_over_json(r#"{"target": 1}"#, "target");
    app.query.as_mut().unwrap().is_synthetic_merge = true;
    let query_before = app.input.query().to_string();

    handle_search_key(&mut app, key(KeyCode::Char('}')));

    assert_eq!(
        app.notification.current_message(),
        Some("No path at cursor")
    );
    assert!(app.search.is_visible(), "search stays open on NoPath");
    assert_eq!(app.input.query(), query_before, "query untouched on NoPath");
}

#[test]
fn sibling_chord_notifies_no_path_when_result_is_undrillable() {
    // Same un-drillable setup; `]` routes through apply_sibling_cursor ->
    // SiblingCursorOutcome::NoPath, surfacing "No path at cursor" (line 264).
    // Pure cursor chord, so it leaves the query untouched and stays open.
    let mut app = confirmed_search_over_json(r#"{"target": 1, "other": 2}"#, "target");
    app.query.as_mut().unwrap().is_synthetic_merge = true;
    let query_before = app.input.query().to_string();

    handle_search_key(&mut app, key(KeyCode::Char(']')));

    assert_eq!(
        app.notification.current_message(),
        Some("No path at cursor")
    );
    assert!(app.search.is_visible(), "search stays open on NoPath");
    assert_eq!(app.input.query(), query_before, "query untouched on NoPath");
}
