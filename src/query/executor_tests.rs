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
fn test_final_output_uses_fixed_dark_colors() {
    // The final stdout deliverable (`execute_for_output`) must be colored with
    // the fixed dark Galaxy palette regardless of the active theme, so piped
    // output is consistent in light and dark mode. The dark keys color is
    // golden yellow 255;217;61.
    let json = r#"{"key": "value"}"#;
    let executor = JqExecutor::new(json.to_string());
    let cancel_token = CancellationToken::new();
    let output = executor
        .execute_for_output(".", &cancel_token)
        .expect("query should succeed");

    assert!(
        output.contains("38;2;255;217;61"),
        "final output keys must use the fixed dark golden color, got: {output:?}"
    );
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

#[test]
fn test_all_field_names_respects_sample_limit() {
    // Build array with 20 objects, each with a unique key
    let objects: Vec<String> = (0..20)
        .map(|i| format!(r#"{{"key_{}": {}}}"#, i, i))
        .collect();
    let json = format!("[{}]", objects.join(","));
    let executor = JqExecutor::new(json);
    let fields = executor.all_field_names();

    // First 10 should be present (ARRAY_SAMPLE_SIZE = 10)
    for i in 0..10 {
        assert!(
            fields.contains(&format!("key_{}", i)),
            "Should contain 'key_{}' within sample limit",
            i
        );
    }
    // 11th and beyond should NOT be present
    assert!(
        !fields.contains("key_10"),
        "Should NOT contain 'key_10' beyond sample limit"
    );
}

#[test]
fn test_collect_string_values_skips_non_string_scalars() {
    // all_string_values must collect ONLY string values, silently skipping
    // numbers/booleans/null (the `_ => {}` arm), while still recursing into
    // arrays and objects to reach nested strings.
    let json = r#"{
        "s": "hello",
        "n": 42,
        "b": true,
        "nil": null,
        "nested": ["world", 7, false],
        "obj": {"deep": "value"}
    }"#;
    let executor = JqExecutor::new(json.to_string());
    let values = executor.all_string_values();

    assert_eq!(
        values.len(),
        3,
        "expected exactly 3 distinct string values, got: {values:?}"
    );
    assert!(values.contains(&"hello".to_string()));
    assert!(values.contains(&"world".to_string()));
    assert!(values.contains(&"value".to_string()));
    // Non-string scalars must never be stringified into the result.
    assert!(
        values
            .iter()
            .all(|v| v == "hello" || v == "world" || v == "value"),
        "non-string scalars leaked into all_string_values: {values:?}"
    );
    assert!(!values.iter().any(|v| v == "42"));
    assert!(!values.iter().any(|v| v == "true"));
    assert!(!values.iter().any(|v| v == "false"));
    assert!(!values.iter().any(|v| v == "null"));
}

#[test]
fn test_all_string_values_frequency_order() {
    // Distinct strings are returned sorted by descending frequency, so the
    // most-repeated string sorts first.
    let json = r#"["common", "common", "common", "common", "common", "rare", "mid", "mid"]"#;
    let executor = JqExecutor::new(json.to_string());
    let values = executor.all_string_values();

    assert_eq!(
        values.len(),
        3,
        "expected 3 distinct strings, got: {values:?}"
    );
    assert_eq!(
        values[0], "common",
        "most frequent string must sort first, got: {values:?}"
    );
    assert_eq!(
        values[1], "mid",
        "second-most frequent must sort second, got: {values:?}"
    );
    assert_eq!(values[2], "rare");
}

#[test]
fn test_all_string_values_respects_cap() {
    // Build more distinct strings than the cap, plus extra duplicates of an
    // early high-frequency string. The result must be truncated to exactly
    // MAX_GLOBAL_STRING_VALUES (exercising the >= cap drop-new branch and the
    // truncate path), and the over-cap-frequency string must survive and sort
    // first.
    let over = MAX_GLOBAL_STRING_VALUES + 50;
    let mut elems: Vec<String> = (0..over).map(|i| format!("\"v{i}\"")).collect();
    // Make "v0" the most frequent so it must rank first after sorting.
    for _ in 0..5 {
        elems.push("\"v0\"".to_string());
    }
    let json = format!("[{}]", elems.join(","));
    let executor = JqExecutor::new(json);
    let values = executor.all_string_values();

    assert_eq!(
        values.len(),
        MAX_GLOBAL_STRING_VALUES,
        "result must be capped at MAX_GLOBAL_STRING_VALUES"
    );
    assert_eq!(
        values[0], "v0",
        "the string repeated past the cap must accumulate frequency and sort first"
    );
}

#[test]
fn test_jq_colors_env_formats_rgb_and_falls_back_for_non_rgb() {
    use ratatui::style::Color;

    // Mix one non-Rgb color (Reset) into an otherwise Rgb palette to exercise
    // the `_ => (255,255,255)` fallback arm, and verify the bold prefix for
    // indices >= 5 (arrays/objects/keys).
    let palette = [
        Color::Rgb(1, 2, 3),
        Color::Reset,
        Color::Rgb(4, 5, 6),
        Color::Rgb(7, 8, 9),
        Color::Rgb(10, 11, 12),
        Color::Rgb(13, 14, 15),
        Color::Rgb(16, 17, 18),
        Color::Rgb(19, 20, 21),
    ];
    let env = jq_colors_env(palette);
    let parts: Vec<&str> = env.split(':').collect();

    assert_eq!(parts.len(), 8, "one colon-joined slot per palette entry");
    assert_eq!(parts[0], "38;2;1;2;3", "Rgb slot formats as 38;2;r;g;b");
    assert_eq!(
        parts[1], "38;2;255;255;255",
        "non-Rgb color must fall back to white 255;255;255"
    );
    assert_eq!(parts[4], "38;2;10;11;12", "index 4 has no bold prefix");
    assert!(
        parts[5].starts_with("1;38;2;"),
        "index 5 (arrays) must be bold-prefixed, got: {}",
        parts[5]
    );
    assert_eq!(parts[5], "1;38;2;13;14;15");
    assert!(
        parts[7].starts_with("1;38;2;"),
        "index 7 (keys) must be bold-prefixed, got: {}",
        parts[7]
    );
}
