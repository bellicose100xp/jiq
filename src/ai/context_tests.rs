//! Tests for query context building

use super::*;
use proptest::prelude::*;

fn empty_params() -> ContextParams<'static> {
    ContextParams {
        input_schema: None,
        base_query: None,
        base_query_result: None,
        is_empty_result: false,
    }
}

#[test]
fn test_truncate_json_short() {
    let json = r#"{"short": true}"#;
    let truncated = truncate_json(json, 1000);
    assert_eq!(truncated, json);
}

#[test]
fn test_truncate_json_long() {
    let json = "x".repeat(2000);
    let truncated = truncate_json(&json, 1000);
    assert!(truncated.len() < json.len());
    assert!(truncated.ends_with("... [truncated]"));
}

#[test]
fn test_query_context_new() {
    let query = ".name".to_string();
    let ctx = QueryContext::new(
        query.clone(),
        5,
        None,
        Some("error".to_string()),
        empty_params(),
        MAX_JSON_SAMPLE_LENGTH,
    );

    assert_eq!(ctx.query, query);
    assert_eq!(ctx.cursor_pos, 5);
    assert_eq!(ctx.error, Some("error".to_string()));
}

// === try_minify_json Tests ===

#[test]
fn test_try_minify_json_removes_whitespace() {
    let pretty_json = r#"{
  "a": 1,
  "b": 2
}"#;
    let minified = try_minify_json(pretty_json);
    assert!(minified.is_some());
    let result = minified.unwrap();
    assert_eq!(result, r#"{"a":1,"b":2}"#);
}

#[test]
fn test_try_minify_json_preserves_data_integrity() {
    let pretty_json = r#"{
  "name": "test",
  "value": 123,
  "active": true
}"#;
    let minified = try_minify_json(pretty_json);
    assert!(minified.is_some());

    let original: serde_json::Value = serde_json::from_str(pretty_json).unwrap();
    let result: serde_json::Value = serde_json::from_str(&minified.unwrap()).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_try_minify_json_handles_nested_objects() {
    let pretty_json = r#"{
  "outer": {
    "inner": {
      "value": 42
    }
  }
}"#;
    let minified = try_minify_json(pretty_json);
    assert!(minified.is_some());
    let result = minified.unwrap();
    assert_eq!(result, r#"{"outer":{"inner":{"value":42}}}"#);
    assert!(result.len() < pretty_json.len());
}

#[test]
fn test_try_minify_json_handles_arrays() {
    let pretty_json = r#"[
  1,
  2,
  3
]"#;
    let minified = try_minify_json(pretty_json);
    assert!(minified.is_some());
    let result = minified.unwrap();
    assert_eq!(result, "[1,2,3]");
}

#[test]
fn test_try_minify_json_returns_none_for_invalid_json() {
    let invalid_json = "not valid json";
    let result = try_minify_json(invalid_json);
    assert!(result.is_none());
}

#[test]
fn test_try_minify_json_returns_none_for_empty_string() {
    let empty = "";
    let result = try_minify_json(empty);
    assert!(result.is_none());
}

#[test]
fn test_try_minify_json_handles_already_minified() {
    let minified_json = r#"{"a":1,"b":2}"#;
    let result = try_minify_json(minified_json);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), minified_json);
}

// === prepare_json_for_context Tests ===

#[test]
fn test_prepare_json_returns_unchanged_when_under_limit() {
    let json = r#"{"short": true}"#;
    let result = prepare_json_for_context(json, MAX_JSON_SAMPLE_LENGTH);
    assert_eq!(result, json);
}

#[test]
fn test_prepare_json_returns_unchanged_when_exactly_at_limit() {
    let json = "x".repeat(MAX_JSON_SAMPLE_LENGTH);
    let result = prepare_json_for_context(&json, MAX_JSON_SAMPLE_LENGTH);
    assert_eq!(result, json);
}

#[test]
fn test_prepare_json_minifies_moderate_sized_json() {
    let pretty_json = format!(
        r#"{{
  "data": "{}",
  "count": 100
}}"#,
        "x".repeat(30_000)
    );
    let original_len = pretty_json.len();
    assert!(original_len > MAX_JSON_SAMPLE_LENGTH);
    assert!(original_len <= MAX_JSON_SAMPLE_LENGTH * MINIFY_THRESHOLD_RATIO);

    let result = prepare_json_for_context(&pretty_json, MAX_JSON_SAMPLE_LENGTH);

    assert!(result.len() <= MAX_JSON_SAMPLE_LENGTH + 15);
    assert!(!result.starts_with(&pretty_json[..100]));
}

#[test]
fn test_prepare_json_skips_minify_above_threshold() {
    let large_json = format!(r#"{{"data": "{}"}}"#, "x".repeat(260_000));
    assert!(large_json.len() > MAX_JSON_SAMPLE_LENGTH * MINIFY_THRESHOLD_RATIO);

    let result = prepare_json_for_context(&large_json, MAX_JSON_SAMPLE_LENGTH);

    assert!(result.ends_with("... [truncated]"));
    assert!(result.starts_with(&large_json[..100]));
}

#[test]
fn test_prepare_json_minifies_at_threshold_boundary() {
    let threshold_size = MAX_JSON_SAMPLE_LENGTH * MINIFY_THRESHOLD_RATIO;
    let overhead = r#"{ "data": "" }"#.len();
    let padding = "x".repeat(threshold_size - overhead);
    let json = format!(r#"{{ "data": "{}" }}"#, padding);

    assert_eq!(json.len(), threshold_size);

    let result = prepare_json_for_context(&json, MAX_JSON_SAMPLE_LENGTH);

    assert!(result.len() <= MAX_JSON_SAMPLE_LENGTH + 15);
}

#[test]
fn test_prepare_json_skips_minify_just_over_threshold() {
    let threshold_size = MAX_JSON_SAMPLE_LENGTH * MINIFY_THRESHOLD_RATIO;
    let json = format!(r#"{{"data": "{}"}}"#, "x".repeat(threshold_size + 1));

    let result = prepare_json_for_context(&json, MAX_JSON_SAMPLE_LENGTH);

    assert!(result.ends_with("... [truncated]"));
    assert!(result.starts_with(r#"{"data":"#));
}

#[test]
fn test_prepare_json_truncates_after_minify_if_still_over() {
    let pretty_json = format!(
        r#"{{
  "large": "{}",
  "more": "data"
}}"#,
        "x".repeat(50_000)
    );
    assert!(pretty_json.len() > MAX_JSON_SAMPLE_LENGTH);
    assert!(pretty_json.len() <= MAX_JSON_SAMPLE_LENGTH * MINIFY_THRESHOLD_RATIO);

    let result = prepare_json_for_context(&pretty_json, MAX_JSON_SAMPLE_LENGTH);

    assert!(result.len() <= MAX_JSON_SAMPLE_LENGTH + 15);
    assert!(result.ends_with("... [truncated]"));
}

#[test]
fn test_prepare_json_no_truncate_if_minify_brings_under() {
    let json_template = r#"{
  "data": ""
}"#;
    let overhead = json_template.len() - 2;
    let padding_size = MAX_JSON_SAMPLE_LENGTH - overhead + 1000;
    let pretty_json = format!(
        r#"{{
  "data": "{}"
}}"#,
        "x".repeat(padding_size)
    );
    assert!(pretty_json.len() > MAX_JSON_SAMPLE_LENGTH);

    let result = prepare_json_for_context(&pretty_json, MAX_JSON_SAMPLE_LENGTH);

    let minified_directly = try_minify_json(&pretty_json).unwrap();
    if minified_directly.len() <= MAX_JSON_SAMPLE_LENGTH {
        assert!(!result.ends_with("... [truncated]"));
        assert_eq!(result, minified_directly);
    }
}

#[test]
fn test_prepare_json_falls_back_on_invalid_json() {
    let invalid_json = format!("not valid json {}", "x".repeat(30_000));
    assert!(invalid_json.len() > MAX_JSON_SAMPLE_LENGTH);

    let result = prepare_json_for_context(&invalid_json, MAX_JSON_SAMPLE_LENGTH);

    assert!(result.ends_with("... [truncated]"));
    assert!(result.starts_with("not valid json"));
}

#[test]
fn test_prepare_json_falls_back_on_empty_input() {
    let empty = "";
    let result = prepare_json_for_context(empty, MAX_JSON_SAMPLE_LENGTH);
    assert_eq!(result, "");
}

#[test]
fn test_prepare_json_fallback_truncates_to_correct_length() {
    let invalid_json = "invalid".repeat(10_000);
    assert!(invalid_json.len() > MAX_JSON_SAMPLE_LENGTH);

    let result = prepare_json_for_context(&invalid_json, MAX_JSON_SAMPLE_LENGTH);

    assert!(result.len() <= MAX_JSON_SAMPLE_LENGTH + 15);
    assert!(result.ends_with("... [truncated]"));
}

// === Integration Tests ===

#[test]
fn test_query_context_applies_prepare_to_output_sample() {
    let large_output = format!(r#"{{ "result": "{}" }}"#, "x".repeat(30_000));
    assert!(large_output.len() > MAX_JSON_SAMPLE_LENGTH);

    let ctx = QueryContext::new(
        ".".to_string(),
        1,
        Some(large_output.clone()),
        None,
        empty_params(),
        MAX_JSON_SAMPLE_LENGTH,
    );

    assert!(ctx.output_sample.is_some());
    let sample = ctx.output_sample.unwrap();
    assert!(sample.len() <= MAX_JSON_SAMPLE_LENGTH + 15);
}

#[test]
fn test_query_context_with_empty_output_skips_processing() {
    // Empty output should not be processed
    let ctx = QueryContext::new(
        ".".to_string(),
        1,
        Some("".to_string()),
        None,
        empty_params(),
        MAX_JSON_SAMPLE_LENGTH,
    );

    assert!(ctx.output_sample.is_none());
}

#[test]
fn test_query_context_with_null_output_skips_processing() {
    // Null output should not be processed
    let ctx = QueryContext::new(
        ".".to_string(),
        1,
        Some("null".to_string()),
        None,
        empty_params(),
        MAX_JSON_SAMPLE_LENGTH,
    );

    assert!(ctx.output_sample.is_none());
}

#[test]
fn test_query_context_passes_through_preprocessed_base_query_result() {
    let preprocessed = "already processed and truncated";

    let params = ContextParams {
        input_schema: None,
        base_query: Some(".base"),
        base_query_result: Some(preprocessed),
        is_empty_result: false,
    };

    let ctx = QueryContext::new(
        ".invalid".to_string(),
        8,
        None,
        Some("error".to_string()),
        params,
        MAX_JSON_SAMPLE_LENGTH,
    );

    assert!(ctx.base_query_result.is_some());
    let result = ctx.base_query_result.unwrap();
    assert_eq!(result, preprocessed);
}

// === Edge Cases ===

#[test]
fn test_prepare_json_handles_unicode_content() {
    let unicode_json = r#"{"emoji": "🎉🎊", "chinese": "你好", "arabic": "مرحبا"}"#;
    let result = prepare_json_for_context(unicode_json, MAX_JSON_SAMPLE_LENGTH);
    assert_eq!(result, unicode_json);
}

#[test]
fn test_prepare_json_handles_deeply_nested_json() {
    let mut nested = String::from(r#"{"a":1"#);
    for _ in 0..10 {
        nested = format!(r#"{{"nested":{}}}"#, nested);
    }
    nested.push('}');

    let result = prepare_json_for_context(&nested, MAX_JSON_SAMPLE_LENGTH);
    assert!(result.len() <= MAX_JSON_SAMPLE_LENGTH + 15);
}

#[test]
fn test_prepare_json_handles_special_json_values() {
    let special_json =
        r#"{"null":null,"true":true,"false":false,"number":123.45,"escaped":"\"quoted\""}"#;
    let result = prepare_json_for_context(special_json, MAX_JSON_SAMPLE_LENGTH);
    assert_eq!(result, special_json);
}

// === Property-Based Tests ===

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_prepare_json_output_never_exceeds_limit(json_size in 0usize..100_000) {
        let json = format!(r#"{{"data":"{}"}}"#, "x".repeat(json_size));
        let result = prepare_json_for_context(&json, MAX_JSON_SAMPLE_LENGTH);

        let max_with_suffix = MAX_JSON_SAMPLE_LENGTH + 15;
        prop_assert!(
            result.len() <= max_with_suffix,
            "Result length {} exceeds max {} for input size {}",
            result.len(),
            max_with_suffix,
            json.len()
        );
    }

    #[test]
    fn prop_prepare_json_preserves_content_prefix(json_size in 1usize..50_000) {
        let json = format!(r#"{{"data":"{}"}}"#, "x".repeat(json_size));
        let result = prepare_json_for_context(&json, MAX_JSON_SAMPLE_LENGTH);

        let prefix_len = result.len().min(10);
        prop_assert_eq!(&result[..prefix_len], &json[..prefix_len]);
    }

    #[test]
    fn prop_minify_preserves_json_semantics(
        depth in 1usize..5,
        keys in 1usize..10
    ) {
        let mut json = String::from("{");
        for i in 0..keys {
            if i > 0 {
                json.push_str(", ");
            }
            json.push_str(&format!(r#""key{}": {}"#, i, depth));
        }
        json.push('}');

        if let Some(minified) = try_minify_json(&json) {
            let original: Result<serde_json::Value, _> = serde_json::from_str(&json);
            let result: Result<serde_json::Value, _> = serde_json::from_str(&minified);

            if let (Ok(orig), Ok(res)) = (original, result) {
                prop_assert_eq!(orig, res, "Minification changed JSON semantics");
            }
        }
    }

    #[test]
    fn prop_invalid_json_fallback_uses_original_content(
        garbage_size in 1000usize..30_000
    ) {
        let garbage = "x".repeat(garbage_size);
        prop_assert!(garbage.len() >= 1000);

        let result = prepare_json_for_context(&garbage, MAX_JSON_SAMPLE_LENGTH);

        let prefix_len = 10.min(result.len());
        prop_assert_eq!(
            &result[..prefix_len],
            &garbage[..prefix_len],
            "Fallback should preserve original content prefix"
        );
    }

    #[test]
    fn prop_context_completeness(
        query in "[.a-zA-Z0-9_]{1,50}",
        cursor_pos in 0usize..50,
        has_error in proptest::bool::ANY,
        error_msg in "[a-zA-Z ]{0,50}"
    ) {
        let error = if has_error { Some(error_msg) } else { None };

        let ctx = QueryContext::new(
            query.clone(),
            cursor_pos.min(query.len()),
            None,
            error.clone(),
            empty_params(),
            MAX_JSON_SAMPLE_LENGTH,
        );

        // Verify required fields are present
        prop_assert_eq!(&ctx.query, &query, "Query text should match");
        prop_assert!(ctx.cursor_pos <= query.len(), "Cursor should be valid");
        prop_assert_eq!(&ctx.error, &error, "Error should match");
    }

    #[test]
    fn prop_error_context_includes_error_message(
        query in "[.a-zA-Z0-9_]{1,50}",
        cursor_pos in 0usize..50,
        error_msg in "[a-zA-Z ]{1,100}"
    ) {
        let ctx = QueryContext::new(
            query.clone(),
            cursor_pos.min(query.len()),
            None,
            Some(error_msg.clone()),
            empty_params(),
            MAX_JSON_SAMPLE_LENGTH,
        );

        // Verify error context is properly set
        prop_assert!(!ctx.is_success, "Context should indicate failure");
        prop_assert!(ctx.error.is_some(), "Error should be present");
        prop_assert_eq!(
            ctx.error.as_ref().unwrap(),
            &error_msg,
            "Error message should match"
        );
        prop_assert!(ctx.output_sample.is_none(), "Output sample should be None on error");
    }

    #[test]
    fn prop_success_context_includes_output(
        query in "[.a-zA-Z0-9_]{1,50}",
        cursor_pos in 0usize..50,
        output_content in "[a-zA-Z0-9 ]{1,200}"
    ) {
        let output = format!(r#""{}""#, output_content);

        let ctx = QueryContext::new(
            query.clone(),
            cursor_pos.min(query.len()),
            Some(output.clone()),
            None,
            empty_params(),
            MAX_JSON_SAMPLE_LENGTH,
        );

        // Verify success context is properly set
        prop_assert!(ctx.is_success, "Context should indicate success");
        prop_assert!(ctx.error.is_none(), "Error should be None on success");
        prop_assert!(ctx.output_sample.is_some(), "Output sample should be present");
    }

    #[test]
    fn prop_success_context_output_sample_bounded(
        query in "[.a-zA-Z0-9_]{1,50}",
        output_size in 1usize..3000
    ) {
        let output = "x".repeat(output_size);

        let ctx = QueryContext::new(
            query.clone(),
            0,
            Some(output.clone()),
            None,
            empty_params(),
            MAX_JSON_SAMPLE_LENGTH,
        );

        // Verify output_sample is bounded
        if let Some(ref sample) = ctx.output_sample {
            let max_with_suffix = MAX_JSON_SAMPLE_LENGTH + 15;
            prop_assert!(
                sample.len() <= max_with_suffix,
                "Output sample length {} exceeds max {} for output size {}",
                sample.len(),
                max_with_suffix,
                output.len()
            );
        }
    }
}

#[test]
fn test_prepare_schema_for_context_returns_unchanged_when_under_limit() {
    let schema = r#"{"name":"string","age":"number"}"#;
    let result = prepare_schema_for_context(schema, MAX_JSON_SAMPLE_LENGTH);
    assert_eq!(result, schema);
}

#[test]
fn test_prepare_schema_for_context_truncates_when_over_limit() {
    let schema = "x".repeat(MAX_JSON_SAMPLE_LENGTH + 100);
    let result = prepare_schema_for_context(&schema, MAX_JSON_SAMPLE_LENGTH);
    assert!(result.len() <= MAX_JSON_SAMPLE_LENGTH + 20);
    assert!(result.contains("... [truncated]"));
    assert_eq!(
        &result[..MAX_JSON_SAMPLE_LENGTH],
        &schema[..MAX_JSON_SAMPLE_LENGTH]
    );
}

#[test]
fn test_prepare_schema_for_context_exactly_at_limit() {
    let schema = "x".repeat(MAX_JSON_SAMPLE_LENGTH);
    let result = prepare_schema_for_context(&schema, MAX_JSON_SAMPLE_LENGTH);
    assert_eq!(result, schema);
    assert!(!result.contains("truncated"));
}
