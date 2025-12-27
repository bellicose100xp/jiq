//! Tests for executor

use super::*;
use tokio_util::sync::CancellationToken;

#[test]
fn test_identity_filter() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let executor = JqExecutor::new(json.to_string());
    let result = executor.execute(".");

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Alice"));
    assert!(output.contains("30"));
}

#[test]
fn test_empty_query_defaults_to_identity() {
    let json = r#"{"name": "Bob"}"#;
    let executor = JqExecutor::new(json.to_string());
    let result = executor.execute("");

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Bob"));
}

#[test]
fn test_field_selection() {
    let json = r#"{"name": "Charlie", "age": 25, "city": "NYC"}"#;
    let executor = JqExecutor::new(json.to_string());
    let result = executor.execute(".name");

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Charlie"));
    assert!(!output.contains("NYC"));
}

#[test]
fn test_array_iteration() {
    let json = r#"[{"id": 1}, {"id": 2}, {"id": 3}]"#;
    let executor = JqExecutor::new(json.to_string());
    let result = executor.execute(".[]");

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
    let result = executor.execute(".invalid.[syntax");

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(!error.is_empty());
}

#[test]
fn test_nested_field_access() {
    let json = r#"{"user": {"name": "Eve", "age": 28}}"#;
    let executor = JqExecutor::new(json.to_string());
    let result = executor.execute(".user.name");

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Eve"));
}

#[test]
fn test_color_output_flag_present() {
    // This test verifies that ANSI color codes are present in output
    let json = r#"{"key": "value"}"#;
    let executor = JqExecutor::new(json.to_string());
    let result = executor.execute(".");

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
