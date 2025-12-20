//! Tests for query_state

use super::*;

#[test]
fn test_new_query_state() {
    let json = r#"{"name": "test"}"#;
    let state = QueryState::new(json.to_string());

    assert!(state.result.is_ok());
    assert!(state.last_successful_result.is_some());
}

#[test]
fn test_execute_updates_result() {
    let json = r#"{"name": "test", "age": 30}"#;
    let mut state = QueryState::new(json.to_string());

    state.execute(".name");
    assert!(state.result.is_ok());
    assert!(state.last_successful_result.is_some());
}

#[test]
fn test_execute_caches_successful_result() {
    let json = r#"{"value": 42}"#;
    let mut state = QueryState::new(json.to_string());

    state.execute(".value");
    let cached = state.last_successful_result.clone();
    assert!(cached.is_some());

    // Execute invalid query (syntax error)
    state.execute(".[invalid syntax");
    assert!(state.result.is_err());

    // Last successful result should still be cached
    assert_eq!(state.last_successful_result, cached);
}

#[test]
fn test_line_count_with_ok_result() {
    let json = r#"{"test": true}"#;
    let mut state = QueryState::new(json.to_string());

    let content: String = (0..50).map(|i| format!("line{}\n", i)).collect();
    state.result = Ok(content);

    assert_eq!(state.line_count(), 50);
}

#[test]
fn test_line_count_uses_cached_on_error() {
    let json = r#"{"test": true}"#;
    let mut state = QueryState::new(json.to_string());

    let valid_result: String = (0..30).map(|i| format!("line{}\n", i)).collect();
    state.result = Ok(valid_result.clone());
    state.last_successful_result = Some(Arc::new(valid_result));

    // Now set an error
    state.result = Err("syntax error".to_string());

    // Should use cached result line count
    assert_eq!(state.line_count(), 30);
}

#[test]
fn test_line_count_zero_on_error_without_cache() {
    let json = r#"{"test": true}"#;
    let mut state = QueryState::new(json.to_string());

    state.result = Err("error".to_string());
    state.last_successful_result = None;

    assert_eq!(state.line_count(), 0);
}

#[test]
fn test_null_results_dont_overwrite_cache() {
    let json = r#"{"name": "test", "age": 30}"#;
    let mut state = QueryState::new(json.to_string());

    // Initial state: should have cached the root object
    let initial_cache = state.last_successful_result.clone();
    let initial_unformatted = state.last_successful_result_unformatted.clone();
    assert!(initial_cache.is_some());
    assert!(initial_unformatted.is_some());

    // Execute a query that returns null (like typing partial field ".s")
    state.execute(".nonexistent");
    assert!(state.result.is_ok());
    // jq returns "null" with ANSI codes, just check it contains "null"
    assert!(state.result.as_ref().unwrap().contains("null"));

    // Both caches should NOT be updated - should still have the root object
    assert_eq!(state.last_successful_result, initial_cache);
    assert_eq!(
        state.last_successful_result_unformatted,
        initial_unformatted
    );

    // Execute a valid query that returns data
    state.execute(".name");
    // jq returns with ANSI codes, just check it contains "test"
    assert!(state.result.as_ref().unwrap().contains("test"));

    // Both caches should now be updated
    assert_ne!(state.last_successful_result, initial_cache);
    assert_ne!(
        state.last_successful_result_unformatted,
        initial_unformatted
    );
    assert!(
        state
            .last_successful_result
            .as_ref()
            .unwrap()
            .contains("test")
    );

    // Unformatted should not have ANSI codes
    let unformatted = state.last_successful_result_unformatted.as_ref().unwrap();
    assert!(!unformatted.contains("\x1b"));
    assert!(unformatted.contains("test"));
}

#[test]
fn test_strip_ansi_codes_simple() {
    let input = "\x1b[0m{\x1b[1;39m\"name\"\x1b[0m: \x1b[0;32m\"test\"\x1b[0m}";
    let output = QueryState::strip_ansi_codes(input);
    assert_eq!(output, r#"{"name": "test"}"#);
    assert!(!output.contains("\x1b"));
}

#[test]
fn test_strip_ansi_codes_complex() {
    let input = "\x1b[1;39m{\x1b[0m\n  \x1b[0;34m\"key\"\x1b[0m: \x1b[0;32m\"value\"\x1b[0m\n\x1b[1;39m}\x1b[0m";
    let output = QueryState::strip_ansi_codes(input);
    assert!(output.contains(r#""key""#));
    assert!(output.contains(r#""value""#));
    assert!(!output.contains("\x1b"));
}

#[test]
fn test_strip_ansi_codes_no_codes() {
    let input = r#"{"name": "plain"}"#;
    let output = QueryState::strip_ansi_codes(input);
    assert_eq!(output, input);
}

#[test]
fn test_strip_ansi_codes_null_with_color() {
    let input = "\x1b[0;90mnull\x1b[0m";
    let output = QueryState::strip_ansi_codes(input);
    assert_eq!(output, "null");
}

#[test]
fn test_unformatted_result_stored_on_execute() {
    let json = r#"{"name": "test"}"#;
    let mut state = QueryState::new(json.to_string());

    // Execute a query
    state.execute(".name");

    // Both formatted and unformatted should be cached
    assert!(state.last_successful_result.is_some());
    assert!(state.last_successful_result_unformatted.is_some());

    // Unformatted should not contain ANSI codes
    let unformatted = state.last_successful_result_unformatted.as_ref().unwrap();
    assert!(!unformatted.contains("\x1b"));
}

#[test]
fn test_unformatted_result_handles_multiline_objects() {
    let json = r#"{"items": [{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]}"#;
    let mut state = QueryState::new(json.to_string());

    // Execute query that returns multiple objects (pretty-printed)
    state.execute(".items[]");

    // Unformatted result should be parseable
    let unformatted = state.last_successful_result_unformatted.as_ref().unwrap();
    assert!(!unformatted.contains("\x1b"));

    // Should contain the field names
    assert!(unformatted.contains("id"));
    assert!(unformatted.contains("name"));
}

#[test]
fn test_multiple_nulls_dont_overwrite_cache() {
    let json = r#"{"items": [{"id": 1}, {"id": 2}, {"id": 3}]}"#;
    let mut state = QueryState::new(json.to_string());

    // Execute query that returns multiple objects
    state.execute(".items[]");
    let cached_after_items = state.last_successful_result_unformatted.clone();
    assert!(cached_after_items.is_some());
    assert!(cached_after_items.as_ref().unwrap().contains("id"));

    // Execute query that returns multiple nulls (nonexistent field)
    state.execute(".items[].nonexistent");

    // Result should contain "null\n" repeated
    assert!(state.result.as_ref().unwrap().contains("null"));

    // Cache should NOT be updated - should still have the objects from .items[]
    assert_eq!(state.last_successful_result_unformatted, cached_after_items);
}

// ============================================================================
// ResultType Detection Tests
// ============================================================================

#[test]
fn test_detect_array_of_objects() {
    let result = r#"[{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]"#;
    assert_eq!(
        QueryState::detect_result_type(result),
        ResultType::ArrayOfObjects
    );
}

#[test]
fn test_detect_empty_array() {
    let result = "[]";
    assert_eq!(QueryState::detect_result_type(result), ResultType::Array);
}

#[test]
fn test_detect_array_of_primitives() {
    let result = "[1, 2, 3, 4, 5]";
    assert_eq!(QueryState::detect_result_type(result), ResultType::Array);

    let result = r#"["a", "b", "c"]"#;
    assert_eq!(QueryState::detect_result_type(result), ResultType::Array);

    let result = "[true, false, true]";
    assert_eq!(QueryState::detect_result_type(result), ResultType::Array);
}

#[test]
fn test_detect_destructured_objects() {
    // Multiple objects on separate lines (from .[] iteration)
    let result = r#"{"id": 1, "name": "a"}
{"id": 2, "name": "b"}
{"id": 3, "name": "c"}"#;
    assert_eq!(
        QueryState::detect_result_type(result),
        ResultType::DestructuredObjects
    );
}

#[test]
fn test_detect_destructured_objects_pretty_printed() {
    // Pretty-printed multi-value output
    let result = r#"{
  "id": 1,
  "name": "a"
}
{
  "id": 2,
  "name": "b"
}"#;
    assert_eq!(
        QueryState::detect_result_type(result),
        ResultType::DestructuredObjects
    );
}

#[test]
fn test_detect_single_object() {
    let result = r#"{"name": "test", "age": 30}"#;
    assert_eq!(QueryState::detect_result_type(result), ResultType::Object);
}

#[test]
fn test_detect_single_object_pretty_printed() {
    let result = r#"{
  "name": "test",
  "age": 30
}"#;
    assert_eq!(QueryState::detect_result_type(result), ResultType::Object);
}

#[test]
fn test_detect_string() {
    let result = r#""hello world""#;
    assert_eq!(QueryState::detect_result_type(result), ResultType::String);
}

#[test]
fn test_detect_number() {
    let result = "42";
    assert_eq!(QueryState::detect_result_type(result), ResultType::Number);

    let result = "3.14159";
    assert_eq!(QueryState::detect_result_type(result), ResultType::Number);

    let result = "-100";
    assert_eq!(QueryState::detect_result_type(result), ResultType::Number);
}

#[test]
fn test_detect_boolean() {
    let result = "true";
    assert_eq!(QueryState::detect_result_type(result), ResultType::Boolean);

    let result = "false";
    assert_eq!(QueryState::detect_result_type(result), ResultType::Boolean);
}

#[test]
fn test_detect_null() {
    let result = "null";
    assert_eq!(QueryState::detect_result_type(result), ResultType::Null);
}

#[test]
fn test_detect_invalid_json_returns_null() {
    let result = "not valid json";
    assert_eq!(QueryState::detect_result_type(result), ResultType::Null);
}

#[test]
fn test_detect_multiple_primitives() {
    // Multiple strings (destructured)
    let result = r#""value1"
"value2"
"value3""#;
    // First value is string, has multiple values, but not objects
    // So it's just String (we don't have "DestructuredStrings")
    assert_eq!(QueryState::detect_result_type(result), ResultType::String);
}

// ============================================================================
// CharType Classification Tests
// ============================================================================

#[test]
fn test_classify_pipe_operator() {
    assert_eq!(QueryState::classify_char(Some('|')), CharType::PipeOperator);
}

#[test]
fn test_classify_semicolon() {
    assert_eq!(QueryState::classify_char(Some(';')), CharType::Semicolon);
}

#[test]
fn test_classify_comma() {
    assert_eq!(QueryState::classify_char(Some(',')), CharType::Comma);
}

#[test]
fn test_classify_colon() {
    assert_eq!(QueryState::classify_char(Some(':')), CharType::Colon);
}

#[test]
fn test_classify_open_paren() {
    assert_eq!(QueryState::classify_char(Some('(')), CharType::OpenParen);
}

#[test]
fn test_classify_close_paren() {
    assert_eq!(QueryState::classify_char(Some(')')), CharType::CloseParen);
}

#[test]
fn test_classify_open_bracket() {
    assert_eq!(QueryState::classify_char(Some('[')), CharType::OpenBracket);
}

#[test]
fn test_classify_close_bracket() {
    assert_eq!(QueryState::classify_char(Some(']')), CharType::CloseBracket);
}

#[test]
fn test_classify_open_brace() {
    assert_eq!(QueryState::classify_char(Some('{')), CharType::OpenBrace);
}

#[test]
fn test_classify_close_brace() {
    assert_eq!(QueryState::classify_char(Some('}')), CharType::CloseBrace);
}

#[test]
fn test_classify_question_mark() {
    assert_eq!(QueryState::classify_char(Some('?')), CharType::QuestionMark);
}

#[test]
fn test_classify_dot() {
    assert_eq!(QueryState::classify_char(Some('.')), CharType::Dot);
}

#[test]
fn test_classify_no_op_characters() {
    // Regular identifier characters
    assert_eq!(QueryState::classify_char(Some('a')), CharType::NoOp);
    assert_eq!(QueryState::classify_char(Some('Z')), CharType::NoOp);
    assert_eq!(QueryState::classify_char(Some('5')), CharType::NoOp);
    assert_eq!(QueryState::classify_char(Some('_')), CharType::NoOp);
}

#[test]
fn test_classify_none() {
    // None (at start of query)
    assert_eq!(QueryState::classify_char(None), CharType::NoOp);
}

// ============================================================================
// Base Query Normalization Tests
// ============================================================================

#[test]
fn test_normalize_strips_pipe_with_identity() {
    // ".services | ." should strip " | ."
    assert_eq!(
        QueryState::normalize_base_query(".services | ."),
        ".services"
    );
    assert_eq!(QueryState::normalize_base_query(".items[] | ."), ".items[]");
}

#[test]
fn test_normalize_strips_incomplete_pipe() {
    // Trailing " | " without operand
    assert_eq!(QueryState::normalize_base_query(".services |"), ".services");
    assert_eq!(QueryState::normalize_base_query(".items[] | "), ".items[]");
}

#[test]
fn test_normalize_strips_trailing_dot() {
    // Trailing dot (incomplete field access)
    assert_eq!(QueryState::normalize_base_query(".services."), ".services");
    assert_eq!(
        QueryState::normalize_base_query(".services[]."),
        ".services[]"
    );
    assert_eq!(
        QueryState::normalize_base_query(".user.profile."),
        ".user.profile"
    );
}

#[test]
fn test_normalize_strips_trailing_whitespace() {
    // Trailing whitespace
    assert_eq!(QueryState::normalize_base_query(".services "), ".services");
    assert_eq!(QueryState::normalize_base_query(".services  "), ".services");
    assert_eq!(QueryState::normalize_base_query(".services\t"), ".services");
}

#[test]
fn test_normalize_preserves_root() {
    // Root query "." should be preserved
    assert_eq!(QueryState::normalize_base_query("."), ".");
}

#[test]
fn test_normalize_preserves_complete_queries() {
    // Complete queries should not be modified
    assert_eq!(QueryState::normalize_base_query(".services"), ".services");
    assert_eq!(
        QueryState::normalize_base_query(".services[]"),
        ".services[]"
    );
    assert_eq!(QueryState::normalize_base_query(".user.name"), ".user.name");
}

#[test]
fn test_normalize_handles_complex_patterns() {
    // Complex incomplete patterns
    assert_eq!(QueryState::normalize_base_query(".a | .b | ."), ".a | .b");
    assert_eq!(
        QueryState::normalize_base_query(".services[].config | ."),
        ".services[].config"
    );
}
