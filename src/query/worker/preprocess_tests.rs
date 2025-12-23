//! Tests for preprocessing functions

use crate::query::query_state::ResultType;
use crate::query::worker::preprocess::preprocess_result;
use crate::query::worker::types::QueryError;
use tokio_util::sync::CancellationToken;

#[test]
fn test_preprocess_result_basic() {
    let output = r#"{"name": "Alice"}"#.to_string();
    let query = ".";
    let cancel_token = CancellationToken::new();

    let result = preprocess_result(output.clone(), query, &cancel_token);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.output.as_ref(), &output);
    assert_eq!(processed.query, ".");
    assert_eq!(processed.result_type, ResultType::Object);
    assert!(processed.parsed.is_some());
    assert!(!processed.rendered_lines.is_empty());
}

#[test]
fn test_preprocess_result_strips_ansi() {
    let output_with_ansi = "\x1b[0;32m\"test\"\x1b[0m".to_string();
    let cancel_token = CancellationToken::new();

    let result = preprocess_result(output_with_ansi, ".", &cancel_token);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(
        processed.unformatted.as_ref(),
        "\"test\"",
        "Should strip ANSI codes"
    );
}

#[test]
fn test_preprocess_result_computes_line_metrics() {
    let output = "line1\nline2\nline3".to_string();
    let cancel_token = CancellationToken::new();

    let result = preprocess_result(output, ".", &cancel_token);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.line_count, 3);
    assert_eq!(processed.max_width, 5); // "line1".len()
}

#[test]
fn test_preprocess_result_handles_cancellation() {
    let output = "test".to_string();
    let cancel_token = CancellationToken::new();

    // Cancel before preprocessing
    cancel_token.cancel();

    let result = preprocess_result(output, ".", &cancel_token);
    assert!(result.is_err());

    match result {
        Err(QueryError::Cancelled) => {}
        _ => panic!("Expected Cancelled error"),
    }
}

#[test]
fn test_preprocess_result_normalizes_query() {
    let output = "null".to_string();
    let cancel_token = CancellationToken::new();

    // Query with trailing " | ." should be normalized to ".services"
    let result = preprocess_result(output, ".services | .", &cancel_token);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(
        processed.query, ".services",
        "Should normalize trailing ' | .'"
    );
}

#[test]
fn test_preprocess_result_detects_result_types() {
    let cancel_token = CancellationToken::new();

    // Test various result types
    let cases = vec![
        (r#"{"a": 1}"#, ResultType::Object),
        (r#"[1, 2, 3]"#, ResultType::Array),
        (r#"[{"a": 1}]"#, ResultType::ArrayOfObjects),
        (r#"{"a": 1}\n{"b": 2}"#, ResultType::DestructuredObjects),
        (r#""hello""#, ResultType::String),
        ("42", ResultType::Number),
        ("true", ResultType::Boolean),
        ("null", ResultType::Null),
    ];

    for (output, expected_type) in cases {
        let result = preprocess_result(output.to_string(), ".", &cancel_token);
        assert!(result.is_ok(), "Failed for output: {}", output);

        let processed = result.unwrap();
        assert_eq!(
            processed.result_type, expected_type,
            "Wrong type for output: {}",
            output
        );
    }
}

#[test]
fn test_preprocess_result_parses_json() {
    let output = r#"{"name": "Alice", "age": 30}"#.to_string();
    let cancel_token = CancellationToken::new();

    let result = preprocess_result(output, ".", &cancel_token);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert!(processed.parsed.is_some(), "Should parse valid JSON");

    let parsed = processed.parsed.unwrap();
    assert!(parsed.is_object());
    assert_eq!(parsed["name"], "Alice");
}

#[test]
fn test_preprocess_result_handles_invalid_json() {
    let output = "not valid json".to_string();
    let cancel_token = CancellationToken::new();

    let result = preprocess_result(output, ".", &cancel_token);
    assert!(result.is_ok(), "Should not error on invalid JSON");

    let processed = result.unwrap();
    assert!(
        processed.parsed.is_none(),
        "Should have None for invalid JSON"
    );
}

#[test]
fn test_rendered_lines_conversion() {
    // Test that rendered lines are created correctly
    let output_with_colors = "\x1b[0;32mtest\x1b[0m".to_string();
    let cancel_token = CancellationToken::new();

    let result = preprocess_result(output_with_colors, ".", &cancel_token);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert!(
        !processed.rendered_lines.is_empty(),
        "Should have rendered lines"
    );

    // Verify line structure
    let first_line = &processed.rendered_lines[0];
    assert!(!first_line.spans.is_empty(), "Should have spans");

    // Content should be unformatted
    let content: String = first_line
        .spans
        .iter()
        .map(|s| s.content.as_str())
        .collect();
    assert!(content.contains("test"), "Should contain unformatted text");
}

#[test]
fn test_preprocess_large_file_computes_correct_width() {
    // Test max_width calculation with very long lines
    let long_line = "a".repeat(500);
    let output = format!("short\n{}\nshort", long_line);
    let cancel_token = CancellationToken::new();

    let result = preprocess_result(output, ".", &cancel_token);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.line_count, 3);
    assert_eq!(processed.max_width, 500);
}

#[test]
fn test_preprocess_max_width_clamped_to_u16_max() {
    // Test that max_width doesn't overflow u16
    let very_long_line = "a".repeat(100_000);
    let cancel_token = CancellationToken::new();

    let result = preprocess_result(very_long_line, ".", &cancel_token);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(
        processed.max_width,
        u16::MAX,
        "max_width should be clamped to u16::MAX"
    );
}
