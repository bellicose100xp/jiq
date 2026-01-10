//! Mid-query insertion tests
//!
//! Tests for verifying that autocomplete suggestions work correctly
//! when the cursor is positioned in the middle of a query (not at the end).
//! These tests ensure text after the cursor is preserved.

use super::*;
use tui_textarea::CursorMove;

/// Position cursor at specific column in textarea for mid-query testing
fn position_cursor_at(textarea: &mut TextArea<'_>, col: usize) {
    textarea.move_cursor(CursorMove::Head);
    for _ in 0..col {
        textarea.move_cursor(CursorMove::Forward);
    }
}

#[test]
fn test_field_insertion_mid_query_basic() {
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.insert_str(".a | .b");
    query_state.base_query_for_suggestions = Some(".".to_string());

    position_cursor_at(&mut textarea, 2);

    let suggestion = test_suggestion(".alpha");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert!(
        result.contains(" | .b"),
        "Should preserve text after cursor: got '{}'",
        result
    );
}

#[test]
fn test_no_duplication_bug_scenario() {
    // This is the exact bug scenario from the original report
    let (mut textarea, mut query_state) = setup_insertion_test("");

    // User executed ".services | map(.name)" successfully
    textarea.insert_str(".services | map(.name)");
    query_state.base_query_for_suggestions = Some(".services | map(.name)".to_string());

    // User moves cursor to after ".services" and types ".f"
    position_cursor_at(&mut textarea, 9);
    textarea.insert_char('.');
    textarea.insert_char('f');

    // Now cursor is at position 11, query is ".services.f | map(.name)"
    position_cursor_at(&mut textarea, 11);

    let suggestion = test_suggestion("foo");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert_eq!(
        result, ".services.foo | map(.name)",
        "Should NOT duplicate ' | map(.name)'"
    );
    assert_eq!(
        result.matches("| map(.name)").count(),
        1,
        "Should have exactly 1 occurrence of '| map(.name)', not 2"
    );
}

#[test]
fn test_field_insertion_mid_query_no_duplication() {
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.insert_str(".services.u | map(.name)");
    query_state.base_query_for_suggestions = Some(".services".to_string());

    position_cursor_at(&mut textarea, 11);

    let suggestion = test_suggestion("users");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert_eq!(result, ".services.users | map(.name)");
    assert!(
        !result.contains("| map(.name) | map(.name)"),
        "Should not duplicate"
    );
}

#[test]
fn test_field_insertion_mid_query_with_complex_pipe() {
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.insert_str(".data.i | select(.active) | sort");
    query_state.base_query_for_suggestions = Some(".data".to_string());

    position_cursor_at(&mut textarea, 7);

    let suggestion = test_suggestion("items");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert_eq!(result, ".data.items | select(.active) | sort");
}

#[test]
fn test_array_syntax_insertion_preserves_after() {
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.insert_str(".services.i | map(.name)");
    query_state.base_query_for_suggestions = Some(".services".to_string());

    position_cursor_at(&mut textarea, 11);

    let suggestion = test_suggestion("[].id");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert_eq!(result, ".services[].id | map(.name)");
}

#[test]
fn test_function_insertion_mid_query_preserves_after() {
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.insert_str(".items | sel | sort");
    query_state.base_query_for_suggestions = Some(".items".to_string());

    position_cursor_at(&mut textarea, 12);

    let suggestion = Suggestion::new("select", SuggestionType::Function);
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert!(
        result.contains(" | sort"),
        "Should preserve ' | sort' after function insertion"
    );
}

#[test]
fn test_object_key_insertion_mid_query_preserves_after() {
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.insert_str("{n:.name, a | .foo");
    query_state.base_query_for_suggestions = Some(".".to_string());

    position_cursor_at(&mut textarea, 11);

    let suggestion = test_suggestion("age");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert!(
        result.contains("| .foo"),
        "Should preserve ' | .foo' after object key"
    );
}

#[test]
fn test_cursor_position_after_mid_query_insertion() {
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.insert_str(".services.i | map(.name)");
    query_state.base_query_for_suggestions = Some(".services".to_string());

    position_cursor_at(&mut textarea, 11);

    let suggestion = test_suggestion("items");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert_eq!(result, ".services.items | map(.name)");

    let cursor_pos = textarea.cursor().1;
    let expected_pos = 15;
    assert_eq!(
        cursor_pos, expected_pos,
        "Cursor should be after '.items', not at end"
    );
}

#[test]
fn test_suggestion_at_query_start() {
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.insert_str("");
    query_state.base_query_for_suggestions = Some(".".to_string());

    position_cursor_at(&mut textarea, 0);

    let suggestion = test_suggestion(".services");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert_eq!(result, ".services");
}

#[test]
fn test_array_suggestion_at_partial_start() {
    let (mut textarea, mut query_state) = setup_insertion_test("");

    textarea.insert_str("x");
    query_state.base_query_for_suggestions = Some(".".to_string());

    position_cursor_at(&mut textarea, 1);

    let suggestion = test_suggestion("[]");
    insert_suggestion(&mut textarea, &mut query_state, &suggestion);

    let result: &str = textarea.lines()[0].as_ref();
    assert_eq!(result, "[]");
}

#[test]
fn test_no_suggestions_after_bare_question_mark() {
    let query = ".services[]?";
    let cursor_pos = query.len();
    let before_cursor = &query[..cursor_pos];

    let mut brace_tracker = crate::autocomplete::BraceTracker::new();
    brace_tracker.rebuild(before_cursor);
    let (context, _partial) = crate::autocomplete::analyze_context(before_cursor, &brace_tracker);

    assert_eq!(
        context,
        crate::autocomplete::SuggestionContext::FunctionContext
    );
}

#[test]
fn test_suggestions_after_question_mark_with_dot() {
    let query = ".services[]?.";
    let cursor_pos = query.len();
    let before_cursor = &query[..cursor_pos];

    let mut brace_tracker = crate::autocomplete::BraceTracker::new();
    brace_tracker.rebuild(before_cursor);
    let (context, _partial) = crate::autocomplete::analyze_context(before_cursor, &brace_tracker);

    assert_eq!(
        context,
        crate::autocomplete::SuggestionContext::FieldContext
    );
}
