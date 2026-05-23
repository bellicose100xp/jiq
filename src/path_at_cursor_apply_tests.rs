use super::*;
use crate::test_utils::test_helpers::test_app;

fn place_cursor(app: &mut App, row: u32) {
    let total = app.results_line_count_u32();
    app.results_cursor.update_total_lines(total);
    app.results_cursor.move_to_line(row);
}

#[test]
fn apply_schedules_debouncer_does_not_run_query_synchronously() {
    let mut app = test_app(r#"{"a": 1, "b": 2}"#);
    app.input.textarea.insert_str(".");
    place_cursor(&mut app, 1);
    assert!(!app.debouncer.has_pending());

    apply_path(&mut app, PathSource::CursorRow);

    assert!(
        app.debouncer.has_pending(),
        "drill-in must schedule debounced async execution"
    );
}

#[test]
fn undo_schedules_debouncer_does_not_run_query_synchronously() {
    let mut app = test_app(r#"{"a": 1}"#);
    place_cursor(&mut app, 1);
    apply_path(&mut app, PathSource::CursorRow);
    // Drain the prior schedule (in real life the main loop would have
    // executed by now; in the test we just clear the flag).
    app.debouncer = crate::query::Debouncer::new();
    assert!(!app.debouncer.has_pending());

    pop_undo(&mut app);

    assert!(
        app.debouncer.has_pending(),
        "back must schedule debounced async execution"
    );
}

#[test]
fn apply_pipe_composes_onto_existing_query() {
    let mut app = test_app(r#"{"users": [{"name": "alice"}, {"name": "bob"}]}"#);
    app.input.textarea.insert_str(".users");
    if let Some(qs) = app.query.as_mut() {
        qs.execute(".users");
    }
    place_cursor(&mut app, 2);

    let outcome = apply_path(&mut app, PathSource::CursorRow);

    assert_eq!(outcome, ApplyOutcome::Applied(".users | .[0].name".into()));
    assert_eq!(app.input.query(), ".users | .[0].name");
}

#[test]
fn apply_replaces_when_current_query_is_root() {
    let mut app = test_app(r#"{"a": 1, "b": 2}"#);
    app.input.textarea.insert_str(".");
    place_cursor(&mut app, 1);

    apply_path(&mut app, PathSource::CursorRow);

    assert_eq!(app.input.query(), ".a");
}

#[test]
fn apply_replaces_when_current_query_is_empty() {
    let mut app = test_app(r#"{"a": 1, "b": 2}"#);
    place_cursor(&mut app, 2);

    apply_path(&mut app, PathSource::CursorRow);

    assert_eq!(app.input.query(), ".b");
}

#[test]
fn apply_at_root_is_no_op() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.input.textarea.insert_str(".existing");
    place_cursor(&mut app, 0);

    let outcome = apply_path(&mut app, PathSource::CursorRow);

    assert_eq!(outcome, ApplyOutcome::AtRoot);
    assert_eq!(app.input.query(), ".existing");
    assert!(app.query_undo.is_empty(), "no snapshot pushed at root");
}

#[test]
fn apply_returns_no_path_when_resolution_fails() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.input.textarea.insert_str(".existing");
    if let Some(qs) = app.query.as_mut() {
        qs.last_successful_result_parsed = None;
    }

    let outcome = apply_path(&mut app, PathSource::CursorRow);

    assert_eq!(outcome, ApplyOutcome::NoPath);
    assert_eq!(app.input.query(), ".existing");
}

#[test]
fn apply_then_undo_round_trips() {
    let mut app = test_app(r#"{"a": 1, "b": 2}"#);
    app.input.textarea.insert_str(".");
    place_cursor(&mut app, 1);

    apply_path(&mut app, PathSource::CursorRow);
    assert_eq!(app.input.query(), ".a");

    let outcome = pop_undo(&mut app);
    assert_eq!(outcome, UndoOutcome::Restored(".".into()));
    assert_eq!(app.input.query(), ".");
    assert!(app.query_undo.is_empty());
}

#[test]
fn undo_on_empty_ring() {
    let mut app = test_app(r#"{"a": 1}"#);
    let outcome = pop_undo(&mut app);
    assert_eq!(outcome, UndoOutcome::Empty);
}

#[test]
fn undo_after_manual_edit_is_invalidated() {
    let mut app = test_app(r#"{"a": 1, "b": 2}"#);
    place_cursor(&mut app, 1);

    apply_path(&mut app, PathSource::CursorRow);
    // Simulate the user typing extra characters into the textarea.
    app.input.textarea.insert_str(" | .extra");

    let outcome = pop_undo(&mut app);
    assert_eq!(outcome, UndoOutcome::Invalidated);
    assert!(app.query_undo.is_empty());
}

#[test]
fn apply_with_match_row_uses_explicit_row() {
    let mut app = test_app(r#"{"a": 1, "b": 2}"#);
    place_cursor(&mut app, 0);

    let outcome = apply_path(&mut app, PathSource::Row(2));

    assert_eq!(outcome, ApplyOutcome::Applied(".b".into()));
    assert_eq!(app.input.query(), ".b");
}

#[test]
fn deep_chain_apply_then_undo_to_root() {
    let mut app = test_app(r#"{"users": [{"name": "alice"}]}"#);
    app.input.textarea.insert_str(".");
    if let Some(qs) = app.query.as_mut() {
        qs.execute(".");
    }
    place_cursor(&mut app, 1);

    apply_path(&mut app, PathSource::CursorRow);
    assert_eq!(app.input.query(), ".users");

    if let Some(qs) = app.query.as_mut() {
        qs.execute(".users");
    }
    // After a real drill-in the worker delivers a new result, which
    // invalidates the path-at-cursor cache via `update_stats`. Mirror
    // that here so the next `apply_path` resolves against the new parsed
    // result.
    app.path_at_cursor.invalidate();
    place_cursor(&mut app, 1);
    apply_path(&mut app, PathSource::CursorRow);
    assert_eq!(app.input.query(), ".users | .[0]");

    pop_undo(&mut app);
    assert_eq!(app.input.query(), ".users");
    pop_undo(&mut app);
    assert_eq!(app.input.query(), ".");
}
