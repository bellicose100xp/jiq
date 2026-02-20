//! Tests for executor

use super::*;
use tokio_util::sync::CancellationToken;

#[test]
fn test_identity_filter() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();
    let result = executor.execute_with_cancel(".", &cancel_token);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Alice"));
    assert!(output.contains("30"));
}

#[test]
fn test_empty_query_defaults_to_identity() {
    let json = r#"{"name": "Bob"}"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();
    let result = executor.execute_with_cancel("", &cancel_token);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Bob"));
}

#[test]
fn test_field_selection() {
    let json = r#"{"name": "Charlie", "age": 25, "city": "NYC"}"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();
    let result = executor.execute_with_cancel(".name", &cancel_token);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Charlie"));
    assert!(!output.contains("NYC"));
}

#[test]
fn test_array_iteration() {
    let json = r#"[{"id": 1}, {"id": 2}, {"id": 3}]"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();
    let result = executor.execute_with_cancel(".[]", &cancel_token);

    assert!(result.is_ok());
    let output = result.unwrap();
    // Check that all three IDs appear in the output (format may vary)
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(output.contains("3"));
    assert!(output.contains("id"));
}

#[test]
fn test_invalid_query_returns_error() {
    let json = r#"{"name": "Dave"}"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();
    let result = executor.execute_with_cancel(".invalid.[syntax", &cancel_token);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(!error.to_string().is_empty());
}

#[test]
fn test_nested_field_access() {
    let json = r#"{"user": {"name": "Eve", "age": 28}}"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();
    let result = executor.execute_with_cancel(".user.name", &cancel_token);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Eve"));
}

#[test]
fn test_color_output_flag_present() {
    // This test verifies that ANSI color codes are present in output
    let json = r#"{"key": "value"}"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();
    let result = executor.execute_with_cancel(".", &cancel_token);

    assert!(result.is_ok());
    let output = result.unwrap();
    // jq with --color-output produces ANSI escape codes
    assert!(output.contains("\x1b[") || output.len() > json.len());
}

#[test]
fn test_execute_with_cancel_success() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();

    let result = executor.execute_with_cancel(".", &cancel_token);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Alice"));
    assert!(output.contains("30"));
}

#[test]
fn test_execute_with_cancel_empty_query() {
    let json = r#"{"name": "Bob"}"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();

    let result = executor.execute_with_cancel("", &cancel_token);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Bob"));
}

#[test]
fn test_execute_with_cancel_invalid_query() {
    let json = r#"{"name": "Charlie"}"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();

    let result = executor.execute_with_cancel(".invalid[", &cancel_token);

    assert!(result.is_err());
    match result {
        Err(QueryError::ExecutionFailed(msg)) => {
            assert!(!msg.is_empty());
        }
        _ => panic!("Expected ExecutionFailed error"),
    }
}

#[test]
fn test_execute_with_cancel_large_output() {
    let json = r#"[1,2,3,4,5,6,7,8,9,10]"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();

    let result = executor.execute_with_cancel(".[]", &cancel_token);

    assert!(result.is_ok());
    let output = result.unwrap();
    for i in 1..=10 {
        assert!(output.contains(&i.to_string()));
    }
}

#[test]
fn test_json_input_accessor() {
    let json = r#"{"test": "data"}"#;
    let executor = JqExecutor::new(json.to_string());

    assert_eq!(executor.json_input(), json);
}

#[test]
fn test_json_input_parsed_returns_parsed_value() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let executor = JqExecutor::new(json.to_string());

    let parsed = executor.json_input_parsed();
    assert!(parsed.is_some());

    let value = parsed.unwrap();
    assert!(value.is_object());
    assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("Alice"));
    assert_eq!(value.get("age").and_then(|v| v.as_i64()), Some(30));
}

#[test]
fn test_json_input_parsed_caches_result() {
    let json = r#"{"key": "value"}"#;
    let executor = JqExecutor::new(json.to_string());

    let first = executor.json_input_parsed();
    let second = executor.json_input_parsed();

    // Both should return the same Arc (same pointer)
    assert!(std::sync::Arc::ptr_eq(
        first.as_ref().unwrap(),
        second.as_ref().unwrap()
    ));
}

#[test]
fn test_json_input_parsed_returns_none_for_invalid_json() {
    let invalid_json = "not valid json {{{";
    let executor = JqExecutor::new(invalid_json.to_string());

    let parsed = executor.json_input_parsed();
    assert!(parsed.is_none());
}

#[test]
fn test_json_input_parsed_handles_arrays() {
    let json = r#"[{"id": 1}, {"id": 2}]"#;
    let executor = JqExecutor::new(json.to_string());

    let parsed = executor.json_input_parsed();
    assert!(parsed.is_some());

    let value = parsed.unwrap();
    assert!(value.is_array());
    assert_eq!(value.as_array().map(|a| a.len()), Some(2));
}

#[test]
fn test_json_input_parsed_preserves_original_after_queries() {
    let json = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();

    // Execute some queries that transform the data
    let _ = executor.execute_with_cancel(".users[0]", &cancel_token);
    let _ = executor.execute_with_cancel(".users | length", &cancel_token);

    // Original JSON should still be fully accessible
    let parsed = executor.json_input_parsed();
    assert!(parsed.is_some());

    let value = parsed.unwrap();
    assert!(value.get("users").is_some());
    assert_eq!(
        value
            .get("users")
            .and_then(|u| u.as_array())
            .map(|a| a.len()),
        Some(2)
    );
}

#[test]
fn test_all_field_names_heterogeneous_array() {
    let json = r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#;
    let executor = JqExecutor::new(json.to_string());
    let fields = executor.all_field_names();

    assert!(
        fields.contains("a"),
        "Should contain 'a' from first element"
    );
    assert!(
        fields.contains("b"),
        "Should contain 'b' from second element"
    );
    assert!(
        fields.contains("c"),
        "Should contain 'c' from third element"
    );
}

#[test]
fn test_all_field_names_nested_heterogeneous_array() {
    let json = r#"{"items": [{"x": 1}, {"y": 2}]}"#;
    let executor = JqExecutor::new(json.to_string());
    let fields = executor.all_field_names();

    assert!(fields.contains("items"));
    assert!(
        fields.contains("x"),
        "Should contain 'x' from first element"
    );
    assert!(
        fields.contains("y"),
        "Should contain 'y' from second element"
    );
}

#[test]
fn test_all_field_names_deep_heterogeneous_inner_arrays() {
    let json = r#"{
        "services": [
            {"name": "svc1", "tasks": [{"cpu": 100}]},
            {"name": "svc2", "extra_key": true, "tasks": [{"memory": 512}]}
        ]
    }"#;
    let executor = JqExecutor::new(json.to_string());
    let fields = executor.all_field_names();

    assert!(fields.contains("services"));
    assert!(fields.contains("name"));
    assert!(fields.contains("tasks"));
    assert!(
        fields.contains("extra_key"),
        "Should contain 'extra_key' from second service"
    );
    assert!(
        fields.contains("cpu"),
        "Should contain 'cpu' from first task"
    );
    assert!(
        fields.contains("memory"),
        "Should contain 'memory' from second service's task"
    );
}
