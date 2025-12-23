//! Tests for JSON utilities

use super::*;
use proptest::prelude::*;

#[test]
fn test_extract_json_schema_simple_object() {
    let json = r#"{"name": "John", "age": 30}"#;
    let schema = extract_json_schema(json, 5).unwrap();
    assert!(schema.contains(r#""name":"string""#));
    assert!(schema.contains(r#""age":"number""#));
}

#[test]
fn test_extract_json_schema_nested_object() {
    let json = r#"{"user": {"address": {"city": "NYC"}}}"#;
    let schema = extract_json_schema(json, 5).unwrap();
    assert!(schema.contains(r#""city":"string""#));
    assert!(schema.contains(r#""user""#));
    assert!(schema.contains(r#""address""#));
}

#[test]
fn test_extract_json_schema_array_of_objects() {
    let json = r#"[{"id": 1, "name": "test"}]"#;
    let schema = extract_json_schema(json, 5).unwrap();
    assert!(schema.contains(r#""id":"number""#));
    assert!(schema.contains(r#""name":"string""#));
}

#[test]
fn test_extract_json_schema_empty_array() {
    let json = "[]";
    let schema = extract_json_schema(json, 5).unwrap();
    assert_eq!(schema, "[]");
}

#[test]
fn test_extract_json_schema_empty_object() {
    let json = "{}";
    let schema = extract_json_schema(json, 5).unwrap();
    assert_eq!(schema, "{}");
}

#[test]
fn test_extract_json_schema_primitives() {
    assert_eq!(extract_json_schema("42", 5).unwrap(), r#""number""#);
    assert_eq!(extract_json_schema(r#""hello""#, 5).unwrap(), r#""string""#);
    assert_eq!(extract_json_schema("true", 5).unwrap(), r#""boolean""#);
    assert_eq!(extract_json_schema("null", 5).unwrap(), r#""null""#);
}

#[test]
fn test_extract_json_schema_max_depth() {
    let json = r#"{"a": {"b": {"c": {"d": {"e": "deep"}}}}}"#;
    let schema = extract_json_schema(json, 3).unwrap();
    // At depth 3, we should hit "..." for the deepest levels
    assert!(schema.contains(r#""...""#));
}

#[test]
fn test_extract_json_schema_invalid_json() {
    let result = extract_json_schema("not json", 5);
    assert!(result.is_none());
}

#[test]
fn test_extract_json_schema_array_of_primitives() {
    let json = r#"[1, 2, 3]"#;
    let schema = extract_json_schema(json, 5).unwrap();
    assert_eq!(schema, r#"["number"]"#);
}

#[test]
fn test_extract_json_schema_array_of_strings() {
    let json = r#"["a", "b", "c"]"#;
    let schema = extract_json_schema(json, 5).unwrap();
    assert_eq!(schema, r#"["string"]"#);
}

#[test]
fn test_extract_json_schema_mixed_types() {
    let json = r#"{"str": "text", "num": 42, "bool": true, "nil": null, "arr": [1, 2], "obj": {"nested": "value"}}"#;
    let schema = extract_json_schema(json, 5).unwrap();
    assert!(schema.contains(r#""str":"string""#));
    assert!(schema.contains(r#""num":"number""#));
    assert!(schema.contains(r#""bool":"boolean""#));
    assert!(schema.contains(r#""nil":"null""#));
    assert!(schema.contains(r#""arr":["number"]"#));
    assert!(schema.contains(r#""nested":"string""#));
}

// **Feature: json-utils, Property 1: Schema extraction produces valid JSON**
// *For any* valid JSON input, extract_json_schema should produce valid JSON output
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_schema_extraction_produces_valid_json(
        json_content in "[a-zA-Z0-9]{0,100}"
    ) {
        let json = format!(r#"{{"data": "{}"}}"#, json_content);
        if let Some(schema) = extract_json_schema(&json, 5) {
            // Schema should be valid JSON
            prop_assert!(serde_json::from_str::<Value>(&schema).is_ok());
        }
    }

    #[test]
    fn prop_schema_extraction_handles_nested_objects(
        depth in 1usize..10,
        key_name in "[a-z]{1,10}"
    ) {
        // Build nested object: {"a": {"a": {"a": ... }}}
        let mut json = String::from(r#"{"value": "leaf"}"#);
        for _ in 0..depth {
            json = format!(r#"{{"{}":{}}}"#, key_name, json);
        }

        if let Some(schema) = extract_json_schema(&json, depth + 1) {
            prop_assert!(serde_json::from_str::<Value>(&schema).is_ok());
        }
    }

    #[test]
    fn prop_schema_respects_depth_limit(
        depth_limit in 1usize..8
    ) {
        // Create deeply nested JSON beyond the limit
        let deep_depth = depth_limit + 5;
        let mut json = String::from(r#""leaf""#);
        for _ in 0..deep_depth {
            json = format!(r#"{{"key":{}}}"#, json);
        }

        if let Some(schema) = extract_json_schema(&json, depth_limit) {
            // Schema should contain the depth limit marker
            prop_assert!(schema.contains(r#""...""#) || depth_limit >= deep_depth);
        }
    }
}

#[test]
fn snapshot_schema_extraction_complex() {
    let json = r#"{"users": [{"name": "John", "age": 30, "address": {"city": "NYC", "zip": "10001"}}], "count": 1}"#;
    let schema = extract_json_schema(json, 5).unwrap();
    insta::assert_snapshot!(schema);
}

#[test]
fn snapshot_schema_extraction_nested_arrays() {
    let json = r#"{"matrix": [[1, 2], [3, 4]], "labels": ["a", "b"]}"#;
    let schema = extract_json_schema(json, 5).unwrap();
    insta::assert_snapshot!(schema);
}

#[test]
fn test_calculate_schema_depth() {
    // Edge case: zero size
    assert_eq!(calculate_schema_depth(0), SMALL_DEPTH);

    // Small: < 1MB -> depth 30
    assert_eq!(calculate_schema_depth(1000), SMALL_DEPTH);
    assert_eq!(calculate_schema_depth(500 * 1024), SMALL_DEPTH);
    assert_eq!(calculate_schema_depth(1024 * 1024 - 1), SMALL_DEPTH); // Just below 1MB

    // Medium: 1MB - 10MB -> depth 20
    assert_eq!(calculate_schema_depth(1024 * 1024), MEDIUM_DEPTH); // Exact 1MB
    assert_eq!(calculate_schema_depth(5 * 1024 * 1024), MEDIUM_DEPTH);
    assert_eq!(calculate_schema_depth(10 * 1024 * 1024 - 1), MEDIUM_DEPTH); // Just below 10MB

    // Large: 10MB - 100MB -> depth 10
    assert_eq!(calculate_schema_depth(10 * 1024 * 1024), LARGE_DEPTH); // Exact 10MB
    assert_eq!(calculate_schema_depth(50 * 1024 * 1024), LARGE_DEPTH);
    assert_eq!(calculate_schema_depth(100 * 1024 * 1024 - 1), LARGE_DEPTH); // Just below 100MB

    // Very Large: > 100MB -> depth 5
    assert_eq!(calculate_schema_depth(100 * 1024 * 1024), VERY_LARGE_DEPTH); // Exact 100MB
    assert_eq!(calculate_schema_depth(500 * 1024 * 1024), VERY_LARGE_DEPTH);
}

#[test]
fn test_extract_json_schema_dynamic() {
    let small_json = r#"{"a": {"b": {"c": 1}}}"#;
    let schema = extract_json_schema_dynamic(small_json);
    assert!(schema.is_some());
    let schema_value = schema.unwrap();
    assert!(schema_value.contains(r#""a""#));
    assert!(schema_value.contains(r#""b""#));
    assert!(schema_value.contains(r#""c""#));
}

#[test]
fn test_extract_json_schema_dynamic_invalid_json() {
    // Invalid JSON should return None
    let result = extract_json_schema_dynamic("not valid json");
    assert!(result.is_none());

    let result = extract_json_schema_dynamic("{broken: json}");
    assert!(result.is_none());
}

#[test]
fn test_extract_json_schema_dynamic_empty_string() {
    // Empty string is invalid JSON
    let result = extract_json_schema_dynamic("");
    assert!(result.is_none());
}

#[test]
fn test_extract_json_schema_dynamic_empty_containers() {
    // Empty object
    let schema = extract_json_schema_dynamic("{}").unwrap();
    assert_eq!(schema, "{}");

    // Empty array
    let schema = extract_json_schema_dynamic("[]").unwrap();
    assert_eq!(schema, "[]");
}

#[test]
fn test_extract_json_schema_dynamic_depth_scaling() {
    // Create a deeply nested JSON (35 levels deep)
    let mut json = String::from(r#""leaf""#);
    for i in 0..35 {
        json = format!(r#"{{"level{}": {}}}"#, i, json);
    }

    // For small JSON (< 1MB), should use depth 30
    let schema = extract_json_schema_dynamic(&json).unwrap();
    // Should have truncation at depth 30
    assert!(schema.contains(r#""...""#));

    // Verify it doesn't go too deep (would have more levels if depth was higher)
    let depth_check = extract_json_schema(&json, 35).unwrap();
    assert_ne!(schema, depth_check);
}
