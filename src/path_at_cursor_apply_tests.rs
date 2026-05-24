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
fn undo_after_manual_edit_pops_immediately_discarding_edits() {
    // `<` always pops, even when the user typed something between drills.
    // The discarded edits are the cost of the simpler "always undoes a
    // `>`" mental model.
    let mut app = test_app(r#"{"a": 1, "b": 2}"#);
    place_cursor(&mut app, 1);

    apply_path(&mut app, PathSource::CursorRow);
    app.input.textarea.insert_str(" | .extra");

    let outcome = pop_undo(&mut app);
    assert_eq!(outcome, UndoOutcome::Restored("".into()));
    assert_eq!(app.input.query(), "");
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

// ----- iterate (`*`) -----

#[test]
fn iterate_replaces_rightmost_index_with_splat() {
    let mut app = test_app(r#"{"users": [{"name": "alice"}, {"name": "bob"}]}"#);
    app.input.textarea.insert_str(".users");
    if let Some(qs) = app.query.as_mut() {
        qs.execute(".users");
    }
    place_cursor(&mut app, 2); // resolves to .[0].name within array result

    let outcome = apply_iterate(&mut app, PathSource::CursorRow);

    assert_eq!(outcome, ApplyOutcome::Applied(".users | .[].name".into()));
    assert_eq!(app.input.query(), ".users | .[].name");
}

#[test]
fn iterate_no_op_when_no_array_in_path() {
    let mut app = test_app(r#"{"a": {"b": 1}}"#);
    place_cursor(&mut app, 2); // .a.b — no array index

    let outcome = apply_iterate(&mut app, PathSource::CursorRow);

    assert_eq!(outcome, ApplyOutcome::NoArrayToIterate);
    assert_eq!(app.input.query(), "");
    assert!(
        app.query_undo.is_empty(),
        "no snapshot when nothing applied"
    );
}

#[test]
fn iterate_pushes_snapshot_so_back_undoes_it() {
    let mut app = test_app(r#"{"users": [{"x": 1}]}"#);
    if let Some(qs) = app.query.as_mut() {
        qs.execute(".");
    }
    place_cursor(&mut app, 2); // resolves to .users[0]

    apply_iterate(&mut app, PathSource::CursorRow);
    assert_eq!(app.input.query(), ".users[]");
    assert!(!app.query_undo.is_empty());

    pop_undo(&mut app);
    assert_eq!(app.input.query(), "");
}

// ----- step out (`^`) -----

#[test]
fn step_out_drops_last_key() {
    let mut app = test_app(r#"{"users": [{"name": "alice"}]}"#);
    app.input.textarea.insert_str(".users[0].name");

    let outcome = apply_step_out(&mut app);

    assert_eq!(outcome, StepOutOutcome::Stepped(".users[0]".into()));
    assert_eq!(app.input.query(), ".users[0]");
}

#[test]
fn step_out_drops_last_index() {
    let mut app = test_app(r#"{"users": [{"x": 1}]}"#);
    app.input.textarea.insert_str(".users[0]");

    apply_step_out(&mut app);

    assert_eq!(app.input.query(), ".users");
}

#[test]
fn step_out_at_root_returns_at_root_outcome() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.input.textarea.insert_str(".");

    let outcome = apply_step_out(&mut app);

    assert_eq!(outcome, StepOutOutcome::AtRoot);
    assert_eq!(app.input.query(), ".");
}

#[test]
fn step_out_on_empty_query_returns_at_root() {
    let mut app = test_app(r#"{"a": 1}"#);
    let outcome = apply_step_out(&mut app);
    assert_eq!(outcome, StepOutOutcome::AtRoot);
}

#[test]
fn step_out_walks_into_pipe_prefix_when_tail_exhausted() {
    let mut app = test_app(r#"{"users": [{"x": 1}]}"#);
    app.input.textarea.insert_str(".users | .[0]");

    apply_step_out(&mut app);
    assert_eq!(app.input.query(), ".users");
}

#[test]
fn step_out_drops_trailing_pipe_dot() {
    let mut app = test_app(r#"{"a": {"b": 1}}"#);
    // After `>` produces a chain, .a | .b -> step out tail to root, drop pipe.
    app.input.textarea.insert_str(".a | .b");

    apply_step_out(&mut app);
    assert_eq!(app.input.query(), ".a");
}

#[test]
fn step_out_unparseable_query_no_op() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.input.textarea.insert_str("map(.x)");

    let outcome = apply_step_out(&mut app);

    assert_eq!(outcome, StepOutOutcome::Unparseable);
    assert_eq!(app.input.query(), "map(.x)", "unparseable input untouched");
}

#[test]
fn step_out_does_not_push_to_undo_ring() {
    let mut app = test_app(r#"{"users": [{"x": 1}]}"#);
    app.input.textarea.insert_str(".users[0].x");
    assert!(app.query_undo.is_empty());

    apply_step_out(&mut app);

    assert!(
        app.query_undo.is_empty(),
        "[ must not pollute the undo ring"
    );
}

// ----- keep key+value (`}`) -----

#[test]
fn keep_kv_wraps_simple_key() {
    // Pretty layout of `{"users": [{"name": "alice"}]}`:
    //   row 3: "name": "alice"   path .users[0].name
    let mut app = test_app(r#"{"users": [{"name": "alice"}]}"#);
    place_cursor(&mut app, 3);

    let outcome = apply_keep_kv(&mut app, PathSource::CursorRow);

    assert_eq!(outcome, ApplyOutcome::Applied(".users[0] | {name}".into()));
    assert_eq!(app.input.query(), ".users[0] | {name}");
    assert!(!app.query_undo.is_empty(), "keep-kv pushes a snapshot");
}

#[test]
fn keep_kv_wraps_quoted_key_with_long_form() {
    let mut app = test_app(r#"{"users": [{"foo-bar": 1}]}"#);
    place_cursor(&mut app, 3); // .users[0]["foo-bar"]

    let outcome = apply_keep_kv(&mut app, PathSource::CursorRow);

    assert_eq!(
        outcome,
        ApplyOutcome::Applied(".users[0] | {\"foo-bar\": .[\"foo-bar\"]}".into())
    );
}

#[test]
fn keep_kv_top_level_key_drops_pipe_prefix() {
    // .a is at top level, parent is root → composes as just `{a}`.
    let mut app = test_app(r#"{"a": 1, "b": 2}"#);
    place_cursor(&mut app, 1);

    apply_keep_kv(&mut app, PathSource::CursorRow);

    assert_eq!(app.input.query(), "{a}");
}

#[test]
fn keep_kv_no_key_at_array_element_row() {
    let mut app = test_app(r#"{"users": [{"name": "alice"}]}"#);
    place_cursor(&mut app, 2); // .users[0] — last step is Index, not Key

    let outcome = apply_keep_kv(&mut app, PathSource::CursorRow);

    assert_eq!(outcome, ApplyOutcome::NoKeyToWrap);
    assert_eq!(app.input.query(), "");
    assert!(app.query_undo.is_empty());
}

#[test]
fn keep_kv_no_key_at_root() {
    let mut app = test_app(r#"{"a": 1}"#);
    place_cursor(&mut app, 0); // root row

    let outcome = apply_keep_kv(&mut app, PathSource::CursorRow);

    assert_eq!(outcome, ApplyOutcome::NoKeyToWrap);
}

#[test]
fn keep_kv_pushes_snapshot_so_back_undoes_it() {
    let mut app = test_app(r#"{"users": [{"name": "alice"}]}"#);
    place_cursor(&mut app, 3);

    apply_keep_kv(&mut app, PathSource::CursorRow);
    assert_eq!(app.input.query(), ".users[0] | {name}");

    pop_undo(&mut app);
    assert_eq!(app.input.query(), "");
}

#[test]
fn step_out_chain_walks_back_to_root() {
    let mut app = test_app(r#"{"a": {"b": {"c": 1}}}"#);
    app.input.textarea.insert_str(".a.b.c");

    apply_step_out(&mut app);
    assert_eq!(app.input.query(), ".a.b");
    apply_step_out(&mut app);
    assert_eq!(app.input.query(), ".a");
    apply_step_out(&mut app);
    assert_eq!(app.input.query(), ".");
    let outcome = apply_step_out(&mut app);
    assert_eq!(outcome, StepOutOutcome::AtRoot);
}

// ----- sibling (`[` / `]`) — pure cursor movement -----

#[test]
fn sibling_next_returns_next_object_key_line() {
    let mut app = test_app(r#"{"a": 1, "b": 2, "c": 3}"#);
    place_cursor(&mut app, 1); // .a

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next);

    assert_eq!(outcome, SiblingCursorOutcome::Moved(2));
}

#[test]
fn sibling_prev_returns_prev_object_key_line() {
    let mut app = test_app(r#"{"a": 1, "b": 2, "c": 3}"#);
    place_cursor(&mut app, 2); // .b

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Prev);

    assert_eq!(outcome, SiblingCursorOutcome::Moved(1));
}

#[test]
fn sibling_next_returns_array_index_line() {
    let mut app = test_app(r#"[10, 20, 30]"#);
    place_cursor(&mut app, 1); // .[0]

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next);

    assert_eq!(outcome, SiblingCursorOutcome::Moved(2));
}

#[test]
fn sibling_next_wraps_at_last_object_key() {
    let mut app = test_app(r#"{"a": 1, "b": 2, "c": 3}"#);
    place_cursor(&mut app, 3); // .c

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next);

    assert_eq!(outcome, SiblingCursorOutcome::Moved(1));
}

#[test]
fn sibling_prev_wraps_at_first_array_index() {
    let mut app = test_app(r#"[10, 20, 30]"#);
    place_cursor(&mut app, 1); // .[0]

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Prev);

    assert_eq!(outcome, SiblingCursorOutcome::Moved(3));
}

#[test]
fn sibling_does_not_modify_query_or_undo_ring() {
    let mut app = test_app(r#"{"a": 1, "b": 2}"#);
    app.input.textarea.insert_str(".untouched");
    place_cursor(&mut app, 1);

    apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next);

    assert_eq!(
        app.input.query(),
        ".untouched",
        "sibling must not rewrite the textarea"
    );
    assert!(
        app.query_undo.is_empty(),
        "sibling must not push to the undo ring"
    );
    assert!(
        !app.debouncer.has_pending(),
        "sibling must not schedule a query re-run"
    );
}

#[test]
fn sibling_at_root_returns_at_root() {
    let mut app = test_app(r#"{"a": 1}"#);
    place_cursor(&mut app, 0); // root row

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next);

    assert_eq!(outcome, SiblingCursorOutcome::AtRoot);
}

#[test]
fn sibling_with_single_child_returns_no_sibling() {
    let mut app = test_app(r#"{"only": 1}"#);
    place_cursor(&mut app, 1); // .only

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next);

    assert_eq!(outcome, SiblingCursorOutcome::NoSibling);
}

#[test]
fn sibling_no_path_when_no_parsed_result() {
    let mut app = test_app(r#"{"a": 1, "b": 2}"#);
    place_cursor(&mut app, 1);
    if let Some(qs) = app.query.as_mut() {
        qs.last_successful_result_parsed = None;
    }

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next);

    assert_eq!(outcome, SiblingCursorOutcome::NoPath);
}

#[test]
fn sibling_resolves_nested_array_element_lines() {
    let mut app = test_app(r#"{"users": [{"name": "alice"}, {"name": "bob"}]}"#);
    // The pretty layout:
    // 0: {
    // 1:   "users": [
    // 2:     {
    // 3:       "name": "alice"
    // 4:     },
    // 5:     {
    // 6:       "name": "bob"
    // 7:     }
    // ...
    place_cursor(&mut app, 2); // .users[0]

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next);

    assert_eq!(outcome, SiblingCursorOutcome::Moved(5));
}

#[test]
fn sibling_round_trips_via_next_then_prev() {
    let mut app = test_app(r#"{"a": 1, "b": 2, "c": 3}"#);
    place_cursor(&mut app, 2); // .b

    let next_line = match apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next) {
        SiblingCursorOutcome::Moved(line) => line,
        other => panic!("{:?}", other),
    };
    assert_eq!(next_line, 3); // .c
    place_cursor(&mut app, next_line);

    let back_line = match apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Prev) {
        SiblingCursorOutcome::Moved(line) => line,
        other => panic!("{:?}", other),
    };
    assert_eq!(back_line, 2); // .b
}

#[test]
fn sibling_with_explicit_match_row_uses_that_row() {
    let mut app = test_app(r#"{"a": 1, "b": 2, "c": 3}"#);
    place_cursor(&mut app, 0); // cursor at root, but...

    let outcome = apply_sibling_cursor(&mut app, PathSource::Row(2), SiblingDir::Next);

    // Row 2 is .b; sibling next is .c at line 3.
    assert_eq!(outcome, SiblingCursorOutcome::Moved(3));
}

#[test]
fn sibling_in_array_of_objects_cursor_on_open_brace_walks_to_next_open_brace() {
    let mut app = test_app(r#"[{"a": 1}, {"b": 2}]"#);
    place_cursor(&mut app, 1); // first `{` row → .[0]

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next);

    assert_eq!(
        outcome,
        SiblingCursorOutcome::Moved(4),
        "cursor on .[0]'s `{{` should walk to .[1]'s `{{` row"
    );
}

#[test]
fn sibling_in_array_of_objects_cursor_on_close_brace_walks_to_next_open_brace() {
    let mut app = test_app(r#"[{"a": 1}, {"b": 2}]"#);
    place_cursor(&mut app, 3); // `},` row → also .[0] per path_at_line semantics

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next);

    assert_eq!(
        outcome,
        SiblingCursorOutcome::Moved(4),
        "closing brace of .[0] still walks to .[1]'s open brace"
    );
}

#[test]
fn sibling_in_array_of_objects_cursor_on_inner_key_walks_inner_object_keys() {
    let mut app = test_app(r#"[{"a": 1, "b": 2}, {"c": 3, "d": 4}]"#);
    place_cursor(&mut app, 2); // .[0].a

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next);

    // Pretty layout:
    // 0: [
    // 1:   {
    // 2:     "a": 1,   ← cursor here
    // 3:     "b": 2    ← sibling next of .[0].a is .[0].b
    // ...
    assert_eq!(outcome, SiblingCursorOutcome::Moved(3));
}

#[test]
fn sibling_walk_inside_object_with_special_keys() {
    let mut app = test_app(r#"{"normal": 1, "foo-bar": 2}"#);
    place_cursor(&mut app, 1); // .normal

    let outcome = apply_sibling_cursor(&mut app, PathSource::CursorRow, SiblingDir::Next);

    assert_eq!(outcome, SiblingCursorOutcome::Moved(2));
}
