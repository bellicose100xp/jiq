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
    state.result = Ok(content.clone());
    state.last_successful_result = Some(Arc::new(content.clone()));
    state.cached_line_count = content.lines().count() as u32;

    assert_eq!(state.line_count(), 50);
}

#[test]
fn test_line_count_uses_cached_on_error() {
    let json = r#"{"test": true}"#;
    let mut state = QueryState::new(json.to_string());

    let valid_result: String = (0..30).map(|i| format!("line{}\n", i)).collect();
    state.result = Ok(valid_result.clone());
    state.last_successful_result = Some(Arc::new(valid_result.clone()));
    state.cached_line_count = valid_result.lines().count() as u32;

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
    state.cached_line_count = 0;

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
// Parsed Result Caching Tests (Phase 3 - Performance Optimization)
// ============================================================================

#[test]
fn test_parsed_result_cached_for_autocomplete() {
    let json =
        r#"{"services": [{"name": "svc1", "arn": "arn1"}, {"name": "svc2", "arn": "arn2"}]}"#;
    let mut state = QueryState::new(json.to_string());

    // Initial query should have parsed result cached
    assert!(state.last_successful_result_parsed.is_some());

    // Execute query that returns array of objects
    state.execute(".services");
    assert!(state.last_successful_result_parsed.is_some());

    // Destructured query returns multiple objects (not valid JSON as whole)
    state.execute(".services[]");

    // Should still have parsed result (first object)
    assert!(state.last_successful_result_parsed.is_some());

    // Parsed result should be an object with fields from first array element
    let parsed = state.last_successful_result_parsed.as_ref().unwrap();
    assert!(parsed.is_object());
    assert!(parsed.get("name").is_some());
    assert!(parsed.get("arn").is_some());
}

#[test]
fn test_parsed_result_handles_destructured_output() {
    let json = r#"{"items": [{"id": 1, "value": "a"}, {"id": 2, "value": "b"}]}"#;
    let mut state = QueryState::new(json.to_string());

    // Query that destructures array - output is multiple objects separated by newlines
    state.execute(".items[]");

    // Verify result is destructured (multiple objects, not valid JSON as whole)
    let result = state.result.as_ref().unwrap();
    let line_count = result.lines().count();
    assert!(
        line_count >= 2,
        "Should have multiple lines for destructured output"
    );

    // Parsed result should still be available (first object)
    assert!(state.last_successful_result_parsed.is_some());

    // Should be first object with correct fields
    let parsed = state.last_successful_result_parsed.as_ref().unwrap();
    assert!(parsed.is_object());
    assert_eq!(parsed.get("id").and_then(|v| v.as_i64()), Some(1));
    assert_eq!(parsed.get("value").and_then(|v| v.as_str()), Some("a"));
}

#[test]
fn test_parse_first_value_handles_single_object() {
    let json = r#"{"name": "test", "value": 42}"#;
    let parsed = QueryState::parse_first_value(json);

    assert!(parsed.is_some());
    let value = parsed.unwrap();
    assert!(value.is_object());
    assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("test"));
}

#[test]
fn test_parse_first_value_handles_destructured_objects() {
    let json = "{\"name\": \"first\"}\n{\"name\": \"second\"}\n{\"name\": \"third\"}";
    let parsed = QueryState::parse_first_value(json);

    assert!(parsed.is_some());
    let value = parsed.unwrap();
    assert!(value.is_object());
    // Should only parse first object
    assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("first"));
}

#[test]
fn test_parse_first_value_handles_array() {
    let json = r#"[{"id": 1}, {"id": 2}]"#;
    let parsed = QueryState::parse_first_value(json);

    assert!(parsed.is_some());
    let value = parsed.unwrap();
    assert!(value.is_array());
}

#[test]
fn test_parse_first_value_returns_none_for_empty() {
    assert!(QueryState::parse_first_value("").is_none());
    assert!(QueryState::parse_first_value("   ").is_none());
}

#[test]
fn test_parse_first_value_returns_none_for_invalid_json() {
    assert!(QueryState::parse_first_value("invalid json {").is_none());
    assert!(QueryState::parse_first_value("not json at all").is_none());
}

// ============================================================================
// Async Execution Tests (Phase 3 - Async Query Worker)
// ============================================================================

#[test]
fn test_execute_async_basic_flow() {
    let json = r#"{"name": "test", "value": 42}"#;
    let mut state = QueryState::new(json.to_string());

    // Execute async query
    state.execute_async(".name");

    // Should be marked as pending
    assert!(state.is_pending());
    assert!(state.in_flight_request_id.is_some());

    // Poll for result (may need multiple attempts)
    let timeout = std::time::Instant::now();
    while state.is_pending() && timeout.elapsed() < std::time::Duration::from_secs(2) {
        let _ = state.poll_response();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Should complete successfully
    assert!(!state.is_pending());
    assert!(state.result.is_ok());
    assert!(state.result.as_ref().unwrap().contains("test"));
}

#[test]
fn test_execute_async_cancellation() {
    let json = r#"{"data": "value"}"#;
    let mut state = QueryState::new(json.to_string());

    // Start first query
    state.execute_async(".data");
    let first_request_id = state.in_flight_request_id;
    assert!(first_request_id.is_some());

    // Immediately start second query (should cancel first)
    state.execute_async(".data | length");
    let second_request_id = state.in_flight_request_id;

    // Request ID should have incremented
    assert!(second_request_id.is_some());
    assert_ne!(first_request_id, second_request_id);

    // Wait for completion
    let timeout = std::time::Instant::now();
    while state.is_pending() && timeout.elapsed() < std::time::Duration::from_secs(2) {
        let _ = state.poll_response();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Should complete with second query result
    assert!(!state.is_pending());
    assert!(state.result.is_ok());
}

#[test]
fn test_poll_response_filters_stale_responses() {
    let json = r#"{"test": true}"#;
    let mut state = QueryState::new(json.to_string());

    // Start query and get request ID
    state.execute_async(".");
    let first_id = state.in_flight_request_id.unwrap();

    // Cancel and start new query
    state.execute_async(".test");
    let second_id = state.in_flight_request_id.unwrap();
    assert_ne!(first_id, second_id);

    // Poll for results - should filter out stale response from first query
    let timeout = std::time::Instant::now();
    while state.is_pending() && timeout.elapsed() < std::time::Duration::from_secs(2) {
        let _ = state.poll_response();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Should have result from second query only
    assert!(!state.is_pending());
}

#[test]
fn test_request_id_increments_correctly() {
    let json = r#"{"test": true}"#;
    let mut state = QueryState::new(json.to_string());

    // Initial request ID should be 1
    state.execute_async(".");
    assert_eq!(state.in_flight_request_id, Some(1));
    state.cancel_in_flight();

    // Should increment
    state.execute_async(".test");
    assert_eq!(state.in_flight_request_id, Some(2));
    state.cancel_in_flight();

    // Should keep incrementing
    state.execute_async(".");
    assert_eq!(state.in_flight_request_id, Some(3));
}

#[test]
fn test_is_pending_tracks_state_correctly() {
    let json = r#"{"test": true}"#;
    let mut state = QueryState::new(json.to_string());

    // Initially not pending
    assert!(!state.is_pending());

    // After execute_async, should be pending
    state.execute_async(".");
    assert!(state.is_pending());

    // Wait for completion
    let timeout = std::time::Instant::now();
    while state.is_pending() && timeout.elapsed() < std::time::Duration::from_secs(2) {
        let _ = state.poll_response();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // After completion, should not be pending
    assert!(!state.is_pending());
}

#[test]
fn test_cancel_in_flight_clears_state() {
    let json = r#"{"test": true}"#;
    let mut state = QueryState::new(json.to_string());

    state.execute_async(".");
    assert!(state.is_pending());
    assert!(state.in_flight_request_id.is_some());
    assert!(state.current_cancel_token.is_some());

    state.cancel_in_flight();

    assert!(!state.is_pending());
    assert!(state.in_flight_request_id.is_none());
    assert!(state.current_cancel_token.is_none());
}

#[test]
fn test_async_execution_updates_base_query_for_suggestions() {
    let json = r#"{"services": [{"name": "svc1"}, {"name": "svc2"}]}"#;
    let mut state = QueryState::new(json.to_string());

    // Execute async query
    state.execute_async(".services");

    // Wait for completion
    let timeout = std::time::Instant::now();
    while state.is_pending() && timeout.elapsed() < std::time::Duration::from_secs(2) {
        let _ = state.poll_response();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // base_query_for_suggestions should be updated
    assert_eq!(
        state.base_query_for_suggestions,
        Some(".services".to_string()),
        "Async execution should update base_query_for_suggestions"
    );

    // Parsed result should be available for autocomplete
    assert!(
        state.last_successful_result_parsed.is_some(),
        "Async execution should cache parsed result"
    );
}

#[test]
fn test_async_execution_handles_errors_correctly() {
    let json = r#"{"test": true}"#;
    let mut state = QueryState::new(json.to_string());

    // Execute async query with invalid syntax
    state.execute_async(".invalid syntax [");

    // Wait for completion
    let timeout = std::time::Instant::now();
    while state.is_pending() && timeout.elapsed() < std::time::Duration::from_secs(2) {
        let _ = state.poll_response();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Should have error result
    assert!(state.result.is_err());
    assert!(!state.is_pending());
}

#[test]
fn test_poll_response_returns_completed_query() {
    let json = r#"{"name": "test", "value": 42}"#;
    let mut state = QueryState::new(json.to_string());

    // Execute async query
    state.execute_async(".name");

    // Poll for result
    let timeout = std::time::Instant::now();
    let mut completed_query = None;
    while timeout.elapsed() < std::time::Duration::from_secs(2) {
        if let Some(query) = state.poll_response() {
            completed_query = Some(query);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Should return the query that was executed
    assert_eq!(
        completed_query,
        Some(".name".to_string()),
        "poll_response should return the query that produced the result"
    );
}

#[test]
fn test_poll_response_returns_query_for_errors() {
    let json = r#"{"test": true}"#;
    let mut state = QueryState::new(json.to_string());

    let error_query = ".invalid syntax [";
    state.execute_async(error_query);

    // Poll for error result
    let timeout = std::time::Instant::now();
    let mut completed_query = None;
    while timeout.elapsed() < std::time::Duration::from_secs(2) {
        if let Some(query) = state.poll_response() {
            completed_query = Some(query);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Should return the query that produced the error
    assert_eq!(
        completed_query,
        Some(error_query.to_string()),
        "poll_response should return query even for errors (for AI context)"
    );
    assert!(state.result.is_err());
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

// ============================================================================
// Rendered Cache Tests
// ============================================================================

#[test]
fn test_new_populates_rendered_cache() {
    let json = r#"{"name": "test"}"#;
    let state = QueryState::new(json.to_string());

    assert!(
        state.last_successful_result_rendered.is_some(),
        "new() should populate rendered cache"
    );
}

#[test]
fn test_successful_query_updates_rendered_cache() {
    let json = r#"{"name": "test", "value": 42}"#;
    let mut state = QueryState::new(json.to_string());

    state.execute(".name");
    assert!(state.result.is_ok(), "Query should succeed");
    assert!(
        state.last_successful_result_rendered.is_some(),
        "Rendered cache should be populated after successful query"
    );
}

#[test]
fn test_error_query_preserves_rendered_cache() {
    let json = r#"{"value": 42}"#;
    let mut state = QueryState::new(json.to_string());

    state.execute(".value");
    let cached_rendered = state.last_successful_result_rendered.clone();
    assert!(cached_rendered.is_some());

    // Execute invalid query (syntax error)
    state.execute(".[invalid syntax");
    assert!(state.result.is_err(), "Query should fail");

    // Rendered cache should be unchanged
    assert_eq!(
        state.last_successful_result_rendered.is_some(),
        cached_rendered.is_some(),
        "Rendered cache should be preserved on error"
    );
}

#[test]
fn test_null_result_preserves_rendered_cache() {
    let json = r#"{"name": "test", "age": 30}"#;
    let mut state = QueryState::new(json.to_string());

    let initial_rendered = state.last_successful_result_rendered.clone();
    assert!(initial_rendered.is_some());

    // Execute a query that returns null (partial field that doesn't exist)
    state.execute(".nonexistent");
    assert!(state.result.is_ok());
    assert!(state.result.as_ref().unwrap().contains("null"));

    // Rendered cache should NOT be updated
    assert_eq!(
        state.last_successful_result_rendered.is_some(),
        initial_rendered.is_some(),
        "Rendered cache should be preserved for null-only results"
    );
}

#[test]
fn test_line_count_always_uses_last_successful_result() {
    let json = r#"{"test": true}"#;
    let mut state = QueryState::new(json.to_string());

    // Set up: successful result with known line count
    let multiline_result: String = (0..50).map(|i| format!("line{}\n", i)).collect();
    state.last_successful_result = Some(Arc::new(multiline_result.clone()));
    state.cached_line_count = multiline_result.lines().count() as u32;

    // Set current result to something different
    state.result = Ok("null\n".to_string());

    // line_count() should use last_successful_result (50 lines), not result (1 line)
    assert_eq!(state.line_count(), 50);
}

#[test]
fn test_max_line_width_always_uses_last_successful_result() {
    let json = r#"{"test": true}"#;
    let mut state = QueryState::new(json.to_string());

    // Set up: successful result with known max width
    let wide_result = "short\nthis_is_a_very_long_line_with_many_characters\nshort";
    state.last_successful_result = Some(Arc::new(wide_result.to_string()));
    state.cached_max_line_width = wide_result.lines().map(|l| l.len()).max().unwrap_or(0) as u16;

    // Set current result to something different (short)
    state.result = Ok("x".to_string());

    // max_line_width() should use last_successful_result (45 chars), not result (1 char)
    assert_eq!(state.max_line_width(), 45);
}
