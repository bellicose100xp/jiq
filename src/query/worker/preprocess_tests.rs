//! Tests for preprocessing functions

use crate::query::query_state::ResultType;
use crate::query::worker::preprocess::{
    parse_and_detect_type, preprocess_result, strip_ansi_codes,
};
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
    assert_eq!(processed.line_widths.len(), 3);
    assert_eq!(processed.line_widths[0], 5);
    assert_eq!(processed.line_widths[1], 5);
    assert_eq!(processed.line_widths[2], 5);
}

#[test]
fn test_preprocess_result_computes_line_widths_varying_lengths() {
    let output = "a\nbb\nccc\ndddd".to_string();
    let cancel_token = CancellationToken::new();

    let result = preprocess_result(output, ".", &cancel_token);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.line_count, 4);
    assert_eq!(processed.max_width, 4); // "dddd".len()
    assert_eq!(processed.line_widths.len(), 4);
    assert_eq!(processed.line_widths[0], 1);
    assert_eq!(processed.line_widths[1], 2);
    assert_eq!(processed.line_widths[2], 3);
    assert_eq!(processed.line_widths[3], 4);
}

#[test]
fn test_preprocess_result_line_widths_empty_lines() {
    let output = "abc\n\nxyz".to_string();
    let cancel_token = CancellationToken::new();

    let result = preprocess_result(output, ".", &cancel_token);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.line_count, 3);
    assert_eq!(processed.line_widths.len(), 3);
    assert_eq!(processed.line_widths[0], 3);
    assert_eq!(processed.line_widths[1], 0); // empty line
    assert_eq!(processed.line_widths[2], 3);
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
    // Test that max_width and line_widths don't overflow u16
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
    assert_eq!(processed.line_widths.len(), 1);
    assert_eq!(
        processed.line_widths[0],
        u16::MAX,
        "line_widths should be clamped to u16::MAX"
    );
}

// Unit tests for strip_ansi_codes function

#[test]
fn test_strip_ansi_codes_basic_sgr() {
    let input = "\x1b[0;32mgreen text\x1b[0m";
    let result = strip_ansi_codes(input);
    assert_eq!(result, "green text", "Should strip SGR sequences");
}

#[test]
fn test_strip_ansi_codes_multiple_sequences() {
    let input = "\x1b[1;39mbold\x1b[0m normal \x1b[0;32mgreen\x1b[0m";
    let result = strip_ansi_codes(input);
    assert_eq!(result, "bold normal green", "Should strip all sequences");
}

#[test]
fn test_strip_ansi_codes_typical_jq_output() {
    let input = "\x1b[1;39m{\x1b[0m\n  \x1b[0;34m\"name\"\x1b[0m: \x1b[0;32m\"Alice\"\x1b[0m,\n  \x1b[0;34m\"age\"\x1b[0m: \x1b[0;33m30\x1b[0m\n\x1b[1;39m}\x1b[0m";
    let result = strip_ansi_codes(input);
    assert_eq!(
        result, "{\n  \"name\": \"Alice\",\n  \"age\": 30\n}",
        "Should strip typical jq colored output"
    );
}

#[test]
fn test_strip_ansi_codes_empty_string() {
    let input = "";
    let result = strip_ansi_codes(input);
    assert_eq!(result, "", "Should handle empty string");
}

#[test]
fn test_strip_ansi_codes_no_escapes() {
    let input = "plain text without escapes";
    let result = strip_ansi_codes(input);
    assert_eq!(
        result, "plain text without escapes",
        "Should return identical content for plain text"
    );
}

#[test]
fn test_strip_ansi_codes_only_escape_sequences() {
    let input = "\x1b[0m\x1b[1;39m\x1b[0;32m";
    let result = strip_ansi_codes(input);
    assert_eq!(result, "", "Should result in empty string");
}

#[test]
fn test_strip_ansi_codes_malformed_no_bracket() {
    let input = "\x1bX some text";
    let result = strip_ansi_codes(input);
    assert_eq!(
        result, "X some text",
        "Should handle escape without bracket"
    );
}

#[test]
fn test_strip_ansi_codes_malformed_no_terminator() {
    let input = "\x1b[0;32 no closing bracket";
    let result = strip_ansi_codes(input);
    assert_eq!(
        result, "",
        "Should consume everything after unclosed escape sequence"
    );
}

#[test]
fn test_strip_ansi_codes_preserves_utf8() {
    let input = "\x1b[0;32m你好世界\x1b[0m emoji: 🎉";
    let result = strip_ansi_codes(input);
    assert_eq!(
        result, "你好世界 emoji: 🎉",
        "Should preserve UTF-8 characters"
    );
}

#[test]
fn test_strip_ansi_codes_consecutive_escapes() {
    let input = "text\x1b[0m\x1b[1;39m\x1b[0;32mmore text";
    let result = strip_ansi_codes(input);
    assert_eq!(result, "textmore text", "Should handle consecutive escapes");
}

#[test]
fn test_strip_ansi_codes_escape_at_boundaries() {
    let input = "\x1b[0;32mstart\x1b[0m middle \x1b[1;39mend\x1b[0m";
    let result = strip_ansi_codes(input);
    assert_eq!(result, "start middle end", "Should handle boundary escapes");
}

// Unit tests for parse_and_detect_type function

#[test]
fn test_parse_and_detect_type_single_object() {
    let (parsed, result_type) = parse_and_detect_type(r#"{"name": "test"}"#);
    assert!(parsed.is_some());
    assert!(parsed.unwrap().is_object());
    assert_eq!(result_type, ResultType::Object);
}

#[test]
fn test_parse_and_detect_type_destructured_objects() {
    let input = "{\"a\": 1}\n{\"b\": 2}";
    let (parsed, result_type) = parse_and_detect_type(input);
    assert!(parsed.is_some());
    assert!(parsed.unwrap().is_object());
    assert_eq!(result_type, ResultType::DestructuredObjects);
}

#[test]
fn test_parse_and_detect_type_array_of_objects() {
    let (parsed, result_type) = parse_and_detect_type(r#"[{"id": 1}, {"id": 2}]"#);
    assert!(parsed.is_some());
    assert!(parsed.unwrap().is_array());
    assert_eq!(result_type, ResultType::ArrayOfObjects);
}

#[test]
fn test_parse_and_detect_type_empty_array() {
    let (parsed, result_type) = parse_and_detect_type("[]");
    assert!(parsed.is_some());
    assert_eq!(result_type, ResultType::Array);
}

#[test]
fn test_parse_and_detect_type_array_of_primitives() {
    let (parsed, result_type) = parse_and_detect_type("[1, 2, 3]");
    assert!(parsed.is_some());
    assert_eq!(result_type, ResultType::Array);
}

#[test]
fn test_parse_and_detect_type_string() {
    let (parsed, result_type) = parse_and_detect_type(r#""hello""#);
    assert!(parsed.is_some());
    assert_eq!(result_type, ResultType::String);
}

#[test]
fn test_parse_and_detect_type_number() {
    let (parsed, result_type) = parse_and_detect_type("42");
    assert!(parsed.is_some());
    assert_eq!(result_type, ResultType::Number);

    let (parsed, result_type) = parse_and_detect_type("3.14");
    assert!(parsed.is_some());
    assert_eq!(result_type, ResultType::Number);
}

#[test]
fn test_parse_and_detect_type_boolean() {
    let (parsed, result_type) = parse_and_detect_type("true");
    assert!(parsed.is_some());
    assert_eq!(result_type, ResultType::Boolean);

    let (parsed, result_type) = parse_and_detect_type("false");
    assert!(parsed.is_some());
    assert_eq!(result_type, ResultType::Boolean);
}

#[test]
fn test_parse_and_detect_type_null() {
    let (parsed, result_type) = parse_and_detect_type("null");
    assert!(parsed.is_some());
    assert_eq!(result_type, ResultType::Null);
}

#[test]
fn test_parse_and_detect_type_empty_string() {
    let (parsed, result_type) = parse_and_detect_type("");
    assert!(parsed.is_none());
    assert_eq!(result_type, ResultType::Null);
}

#[test]
fn test_parse_and_detect_type_whitespace_only() {
    let (parsed, result_type) = parse_and_detect_type("   \n\t  ");
    assert!(parsed.is_none());
    assert_eq!(result_type, ResultType::Null);
}

#[test]
fn test_parse_and_detect_type_invalid_json() {
    let (parsed, result_type) = parse_and_detect_type("not valid json");
    assert!(parsed.is_none());
    assert_eq!(result_type, ResultType::Null);
}

#[test]
fn test_parse_and_detect_type_trims_whitespace() {
    let (parsed, result_type) = parse_and_detect_type("  {\"a\": 1}  ");
    assert!(parsed.is_some());
    assert_eq!(result_type, ResultType::Object);
}

#[test]
fn test_parse_and_detect_type_pretty_printed_object() {
    let input = r#"{
  "name": "test",
  "value": 42
}"#;
    let (parsed, result_type) = parse_and_detect_type(input);
    assert!(parsed.is_some());
    assert_eq!(result_type, ResultType::Object);
}

#[test]
fn test_parse_and_detect_type_pretty_printed_destructured() {
    let input = r#"{
  "id": 1
}
{
  "id": 2
}"#;
    let (parsed, result_type) = parse_and_detect_type(input);
    assert!(parsed.is_some());
    assert_eq!(result_type, ResultType::DestructuredObjects);
}
