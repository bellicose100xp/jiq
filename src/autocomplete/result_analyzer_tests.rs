//! Tests for result analysis and suggestion generation

use super::*;
use serde_json::Value;
use std::sync::Arc;

/// Helper function to parse JSON string and wrap in Arc for testing
///
/// For multi-line JSON (destructured objects), parses only the first complete object
fn parse_json(json_str: &str) -> Arc<Value> {
    // Try parsing the whole string first
    if let Ok(value) = serde_json::from_str(json_str) {
        return Arc::new(value);
    }

    // If that fails, try to parse the first complete JSON object by accumulating lines
    let mut accumulated = String::new();
    for line in json_str.lines() {
        if line.trim().is_empty() {
            continue;
        }
        accumulated.push_str(line);
        accumulated.push('\n');

        // Try parsing after each line
        if let Ok(value) = serde_json::from_str::<Value>(&accumulated) {
            return Arc::new(value);
        }
    }

    // If all else fails, return Null
    Arc::new(Value::Null)
}

#[test]
fn test_analyze_simple_object() {
    let result = r#"{"name": "test", "age": 30, "active": true}"#;
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::Object,
        true,  // After operator like | or at start
        false, // Not in element context
    );

    assert_eq!(suggestions.len(), 3);
    assert!(suggestions.iter().any(|s| s.text == ".name"));
    assert!(suggestions.iter().any(|s| s.text == ".age"));
    assert!(suggestions.iter().any(|s| s.text == ".active"));
}

#[test]
fn test_analyze_nested_object() {
    // Nested object after operator
    let result = r#"{"user": {"name": "Alice", "profile": {"city": "NYC"}}}"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Object, true, false);

    // Should only return top-level fields
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions.iter().any(|s| s.text == ".user"));
}

#[test]
fn test_analyze_array_of_objects_after_operator() {
    // Array of objects after operator (needs leading dot)
    let result = r#"[{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::ArrayOfObjects, true, false);

    // Should return .[] and element field suggestions with leading dot
    assert!(suggestions.iter().any(|s| s.text == ".[]"));
    assert!(suggestions.iter().any(|s| s.text == ".[].id"));
    assert!(suggestions.iter().any(|s| s.text == ".[].name"));
}

#[test]
fn test_analyze_array_of_objects_after_continuation() {
    // Array of objects after continuation like .services. (no leading dot)
    let result = r#"[{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::ArrayOfObjects, false, false);

    // Should return [] and element field suggestions without leading dot
    assert!(suggestions.iter().any(|s| s.text == "[]"));
    assert!(suggestions.iter().any(|s| s.text == "[].id"));
    assert!(suggestions.iter().any(|s| s.text == "[].name"));
}

#[test]
fn test_analyze_empty_array() {
    // Empty array after operator
    let result = "[]";
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Array, true, false);

    // Should only return .[] for empty arrays
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].text, ".[]");
}

#[test]
fn test_analyze_empty_object() {
    // Empty object
    let result = "{}";
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Object, true, false);

    assert_eq!(suggestions.len(), 0);
}

// ============================================================================
// Pretty-Printed JSON Handling Tests
// ============================================================================

#[test]
fn test_analyze_pretty_printed_object() {
    // Pretty-printed object after operator
    let result = r#"{
  "name": "Alice",
  "age": 30
}"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Object, true, false);

    assert_eq!(suggestions.len(), 2);
    assert!(suggestions.iter().any(|s| s.text == ".name"));
    assert!(suggestions.iter().any(|s| s.text == ".age"));
}

// ============================================================================
// Multi-value Output Tests
// ============================================================================

#[test]
fn test_multivalue_destructured_objects_after_bracket() {
    // Destructured objects after [] (no leading dot needed)
    let result = r#"{"name": "Alice", "age": 30}
{"name": "Bob", "age": 25}
{"name": "Charlie", "age": 35}"#;
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::DestructuredObjects,
        false,
        false,
    );

    // Should parse first object only, no leading dot
    assert_eq!(suggestions.len(), 2);
    assert!(suggestions.iter().any(|s| s.text == "name"));
    assert!(suggestions.iter().any(|s| s.text == "age"));
}

#[test]
fn test_multivalue_destructured_objects_after_operator() {
    // Destructured objects after operator (with leading dot)
    let result = r#"{"clusterArn": "arn1", "name": "svc1"}
{"clusterArn": "arn2", "name": "svc2"}"#;
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::DestructuredObjects,
        true,
        false,
    );

    // Should parse first object with leading dot
    assert_eq!(suggestions.len(), 2);
    assert!(suggestions.iter().any(|s| s.text == ".clusterArn"));
    assert!(suggestions.iter().any(|s| s.text == ".name"));
}

#[test]
fn test_multivalue_pretty_printed_destructured_after_bracket() {
    // Pretty-printed destructured objects after [] (no leading dot)
    let result = r#"{
  "clusterArn": "arn1",
  "name": "svc1"
}
{
  "clusterArn": "arn2",
  "name": "svc2"
}"#;
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::DestructuredObjects,
        false,
        false,
    );

    // Should parse first object, no leading dot
    assert_eq!(suggestions.len(), 2);
    assert!(suggestions.iter().any(|s| s.text == "clusterArn"));
    assert!(suggestions.iter().any(|s| s.text == "name"));
}

#[test]
fn test_multivalue_mixed_types() {
    // Multiple primitives - first is number
    let result = r#"42
"hello"
{"field": "value"}"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Number, true, false);

    // Primitives have no field suggestions
    assert_eq!(suggestions.len(), 0);
}

#[test]
fn test_multivalue_with_whitespace() {
    // Object with whitespace, after operator
    let result = r#"

{"key1": "val1"}

{"key2": "val2"}
"#;
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::DestructuredObjects,
        true,
        false,
    );

    // Should skip empty lines and parse first object with leading dot
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].text, ".key1");
}

// ============================================================================
// The Main Fix: Object Constructor Tests
// ============================================================================

#[test]
fn test_object_constructor_suggestions_after_operator() {
    // THE main fix: after `.services[] | {name: .serviceName, cap: .base}`
    // Result is object with ONLY "name" and "cap" fields
    let result = r#"{"name": "MyService", "cap": 10}"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Object, true, false);

    assert_eq!(suggestions.len(), 2);
    assert!(suggestions.iter().any(|s| s.text == ".name"));
    assert!(suggestions.iter().any(|s| s.text == ".cap"));

    // Should NOT suggest original fields like .serviceName or .base
    assert!(!suggestions.iter().any(|s| s.text == ".serviceName"));
    assert!(!suggestions.iter().any(|s| s.text == ".base"));
}

#[test]
fn test_array_constructor_suggestions() {
    // After `[.field1, .field2]` the result is a primitive array
    let result = r#"["value1", "value2"]"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Array, true, false);

    // Should suggest .[] for array access
    assert!(suggestions.iter().any(|s| s.text == ".[]"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_primitive_results() {
    // Primitives have no field suggestions
    assert_eq!(
        ResultAnalyzer::analyze_parsed_result(&parse_json("42"), ResultType::Number, true, false)
            .len(),
        0
    );
    assert_eq!(
        ResultAnalyzer::analyze_parsed_result(
            &parse_json(r#""hello""#),
            ResultType::String,
            true,
            false
        )
        .len(),
        0
    );
    assert_eq!(
        ResultAnalyzer::analyze_parsed_result(
            &parse_json("true"),
            ResultType::Boolean,
            true,
            false
        )
        .len(),
        0
    );
}

#[test]
fn test_null_result() {
    let parsed = parse_json("null");
    let suggestions = ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Null, true, false);
    assert_eq!(suggestions.len(), 0);
}

#[test]
fn test_empty_string_result() {
    // Empty result treated as null
    let parsed = parse_json("");
    let suggestions = ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Null, true, false);
    assert_eq!(suggestions.len(), 0);
}

#[test]
fn test_invalid_json_result() {
    // Invalid JSON treated as null - returns empty gracefully
    let result = "not valid json {]";
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Null, true, false);
    assert_eq!(suggestions.len(), 0);
}

#[test]
fn test_very_large_result() {
    // Test with 1000+ objects to ensure performance
    let mut result = String::from("[");
    for i in 0..1000 {
        if i > 0 {
            result.push(',');
        }
        result.push_str(&format!(
            r#"{{"id": {}, "name": "item{}", "value": {}}}"#,
            i,
            i,
            i * 2
        ));
    }
    result.push(']');

    let parsed = parse_json(&result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::ArrayOfObjects, true, false);

    // Should extract fields from first array element with leading dot
    assert!(suggestions.iter().any(|s| s.text == ".[]"));
    assert!(suggestions.iter().any(|s| s.text == ".[].id"));
    assert!(suggestions.iter().any(|s| s.text == ".[].name"));
    assert!(suggestions.iter().any(|s| s.text == ".[].value"));
}

// ============================================================================
// Optional Chaining Tests
// ============================================================================

#[test]
fn test_array_with_nulls_in_result() {
    // Array with nulls from optional chaining, after operator
    let result = r#"[null, null, {"field": "value"}]"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::ArrayOfObjects, true, false);

    // Should suggest based on first element (null has no fields)
    assert!(suggestions.iter().any(|s| s.text == ".[]"));
    assert_eq!(suggestions.len(), 1); // Only .[] since first element is null
}

#[test]
fn test_bounded_scan_in_results() {
    // Test that we only look at the first element, not all elements
    let result = r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::ArrayOfObjects, true, false);

    // Should only have fields from first element with leading dot
    assert!(suggestions.iter().any(|s| s.text == ".[]"));
    assert!(suggestions.iter().any(|s| s.text == ".[].a"));
    assert!(!suggestions.iter().any(|s| s.text == ".[].b"));
    assert!(!suggestions.iter().any(|s| s.text == ".[].c"));
}

// ============================================================================
// Type Detection Tests
// ============================================================================

// ============================================================================
// Type-Aware Suggestion Generation Tests
// ============================================================================

#[test]
fn test_destructured_objects_after_bracket_no_prefix() {
    // After .services[]. → destructured objects, no prefix
    let result = r#"{"serviceArn": "arn1", "config": {}}
{"serviceArn": "arn2", "config": {}}"#;
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::DestructuredObjects,
        false,
        false,
    );

    // Should suggest fields without any prefix
    assert!(suggestions.iter().any(|s| s.text == "serviceArn"));
    assert!(suggestions.iter().any(|s| s.text == "config"));
    // Should NOT have leading dot
    assert!(!suggestions.iter().any(|s| s.text.starts_with('.')));
}

#[test]
fn test_destructured_objects_after_pipe_with_prefix() {
    // After .services[] | . → destructured objects, needs leading dot
    let result = r#"{"serviceArn": "arn1"}
{"serviceArn": "arn2"}"#;
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::DestructuredObjects,
        true,
        false,
    );

    // Should suggest fields WITH leading dot
    assert!(suggestions.iter().any(|s| s.text == ".serviceArn"));
    // All should start with dot
    assert!(suggestions.iter().all(|s| s.text.starts_with('.')));
}

#[test]
fn test_array_of_objects_after_dot_no_prefix() {
    // After .services. → array of objects, no leading dot
    let result = r#"[{"id": 1}, {"id": 2}]"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::ArrayOfObjects, false, false);

    // Should suggest [] and [].field without leading dot
    assert!(suggestions.iter().any(|s| s.text == "[]"));
    assert!(suggestions.iter().any(|s| s.text == "[].id"));
    // Should NOT start with dot
    assert!(!suggestions.iter().any(|s| s.text.starts_with('.')));
}

#[test]
fn test_array_of_objects_after_pipe_with_prefix() {
    // After .services | . → array of objects, needs leading dot
    let result = r#"[{"id": 1}, {"id": 2}]"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::ArrayOfObjects, true, false);

    // Should suggest .[] and .[].field with leading dot
    assert!(suggestions.iter().any(|s| s.text == ".[]"));
    assert!(suggestions.iter().any(|s| s.text == ".[].id"));
    // All should start with dot
    assert!(suggestions.iter().all(|s| s.text.starts_with('.')));
}

#[test]
fn test_single_object_after_bracket_no_prefix() {
    // After .user[0]. → single object, no leading dot
    let result = r#"{"name": "Alice", "age": 30}"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Object, false, false);

    // Should suggest fields without leading dot
    assert!(suggestions.iter().any(|s| s.text == "name"));
    assert!(suggestions.iter().any(|s| s.text == "age"));
    assert!(!suggestions.iter().any(|s| s.text.starts_with('.')));
}

#[test]
fn test_single_object_after_operator_with_prefix() {
    // After .user | . → single object, needs leading dot
    let result = r#"{"name": "Alice", "age": 30}"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Object, true, false);

    // Should suggest fields WITH leading dot
    assert!(suggestions.iter().any(|s| s.text == ".name"));
    assert!(suggestions.iter().any(|s| s.text == ".age"));
    assert!(suggestions.iter().all(|s| s.text.starts_with('.')));
}

#[test]
fn test_primitive_array_after_operator() {
    // Array of primitives after operator
    let result = "[1, 2, 3]";
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Array, true, false);

    // Should only suggest .[]
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].text, ".[]");
}

#[test]
fn test_primitive_array_after_continuation() {
    // Array of primitives after continuation
    let result = "[1, 2, 3]";
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Array, false, false);

    // Should only suggest [] (no leading dot)
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].text, "[]");
}

#[test]
fn test_field_type_detection() {
    // Object with various field types, after operator
    let result = r#"{
            "str": "hello",
            "num": 42,
            "bool": true,
            "null": null,
            "obj": {"nested": "value"},
            "arr": [1, 2, 3]
        }"#;
    let parsed = parse_json(result);
    let suggestions =
        ResultAnalyzer::analyze_parsed_result(&parsed, ResultType::Object, true, false);

    // Verify types are correctly detected
    let str_field = suggestions.iter().find(|s| s.text == ".str").unwrap();
    assert!(matches!(str_field.field_type, Some(JsonFieldType::String)));

    let num_field = suggestions.iter().find(|s| s.text == ".num").unwrap();
    assert!(matches!(num_field.field_type, Some(JsonFieldType::Number)));

    let bool_field = suggestions.iter().find(|s| s.text == ".bool").unwrap();
    assert!(matches!(
        bool_field.field_type,
        Some(JsonFieldType::Boolean)
    ));

    let null_field = suggestions.iter().find(|s| s.text == ".null").unwrap();
    assert!(matches!(null_field.field_type, Some(JsonFieldType::Null)));

    let obj_field = suggestions.iter().find(|s| s.text == ".obj").unwrap();
    assert!(matches!(obj_field.field_type, Some(JsonFieldType::Object)));

    let arr_field = suggestions.iter().find(|s| s.text == ".arr").unwrap();
    assert!(matches!(
        arr_field.field_type,
        Some(JsonFieldType::ArrayOf(_))
    ));
}

// ============================================================================
// Element Context Tests
// ============================================================================

#[test]
fn test_array_suggestions_in_element_context() {
    // Inside map(), should suggest .field not .[].field
    let result = r#"[{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]"#;
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::ArrayOfObjects,
        true, // needs_leading_dot
        true, // in_element_context
    );

    // Should NOT have .[] pattern (iteration already provided by map/select/etc.)
    assert!(
        !suggestions.iter().any(|s| s.text == ".[]"),
        "Should not suggest .[] in element context"
    );
    // Should have .field (NOT .[].field)
    assert!(suggestions.iter().any(|s| s.text == ".id"));
    assert!(suggestions.iter().any(|s| s.text == ".name"));
    // Should NOT have .[].field
    assert!(!suggestions.iter().any(|s| s.text == ".[].id"));
    assert!(!suggestions.iter().any(|s| s.text == ".[].name"));
}

#[test]
fn test_array_suggestions_outside_element_context() {
    // Outside map(), should suggest .[].field
    let result = r#"[{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]"#;
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::ArrayOfObjects,
        true,  // needs_leading_dot
        false, // NOT in_element_context
    );

    // Should have .[].field (original behavior)
    assert!(suggestions.iter().any(|s| s.text == ".[].id"));
    assert!(suggestions.iter().any(|s| s.text == ".[].name"));
    // Should NOT have .field without []
    assert!(!suggestions.iter().any(|s| s.text == ".id"));
    assert!(!suggestions.iter().any(|s| s.text == ".name"));
}

#[test]
fn test_element_context_preserves_field_types() {
    // Type info should still be present in element context
    let result = r#"[{"str": "hello", "num": 42, "obj": {}}]"#;
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::ArrayOfObjects,
        true,
        true, // in_element_context
    );

    let str_field = suggestions.iter().find(|s| s.text == ".str").unwrap();
    assert!(matches!(str_field.field_type, Some(JsonFieldType::String)));

    let num_field = suggestions.iter().find(|s| s.text == ".num").unwrap();
    assert!(matches!(num_field.field_type, Some(JsonFieldType::Number)));

    let obj_field = suggestions.iter().find(|s| s.text == ".obj").unwrap();
    assert!(matches!(obj_field.field_type, Some(JsonFieldType::Object)));
}

#[test]
fn test_element_context_with_needs_leading_dot_false() {
    // In element context without leading dot (e.g., after .[]. inside map)
    let result = r#"[{"id": 1}]"#;
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::ArrayOfObjects,
        false, // no leading dot
        true,  // in_element_context
    );

    // Should have field without leading dot
    assert!(suggestions.iter().any(|s| s.text == "id"));
    // Should NOT have .id or [].id
    assert!(!suggestions.iter().any(|s| s.text == ".id"));
    assert!(!suggestions.iter().any(|s| s.text == "[].id"));
}

#[test]
fn test_element_context_does_not_suggest_iterator() {
    // In element context, .[] should NOT be suggested since iteration is already provided
    let result = r#"[{"id": 1}]"#;
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::ArrayOfObjects,
        true,
        true, // in_element_context
    );

    assert!(
        !suggestions.iter().any(|s| s.text == ".[]"),
        "Should not suggest .[] in element context"
    );
    // But field suggestions should still be present
    assert!(suggestions.iter().any(|s| s.text == ".id"));
}

#[test]
fn test_element_context_does_not_affect_other_types() {
    // Object type should be unaffected by element context
    let result = r#"{"name": "test"}"#;
    let parsed = parse_json(result);

    let suggestions_in = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::Object,
        true,
        true, // in_element_context
    );

    let suggestions_out = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::Object,
        true,
        false, // NOT in_element_context
    );

    // Both should produce the same result for Object type
    assert_eq!(suggestions_in.len(), suggestions_out.len());
    assert!(suggestions_in.iter().any(|s| s.text == ".name"));
    assert!(suggestions_out.iter().any(|s| s.text == ".name"));
}

#[test]
fn test_element_context_empty_array() {
    // Empty array in element context
    let result = "[]";
    let parsed = parse_json(result);
    let suggestions = ResultAnalyzer::analyze_parsed_result(
        &parsed,
        ResultType::Array,
        true,
        true, // in_element_context
    );

    // Should only have .[]
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].text, ".[]");
}

// ============================================================================
// Tests for analyze_value() - type inference from Value
// ============================================================================

mod analyze_value_tests {
    use super::*;

    #[test]
    fn test_analyze_value_object() {
        let json: Value = serde_json::from_str(r#"{"name": "test", "age": 30}"#).unwrap();
        let suggestions = ResultAnalyzer::analyze_value(&json, true, false);

        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".name"));
        assert!(suggestions.iter().any(|s| s.text == ".age"));
    }

    #[test]
    fn test_analyze_value_array_of_objects() {
        let json: Value =
            serde_json::from_str(r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#)
                .unwrap();
        let suggestions = ResultAnalyzer::analyze_value(&json, true, false);

        // Should have .[] and .[].id, .[].name
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
        assert!(suggestions.iter().any(|s| s.text == ".[].id"));
        assert!(suggestions.iter().any(|s| s.text == ".[].name"));
    }

    #[test]
    fn test_analyze_value_array_of_objects_suppressed() {
        let json: Value = serde_json::from_str(r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        let suggestions = ResultAnalyzer::analyze_value(&json, true, true);

        // When suppressed, should have .id instead of .[].id, no .[]
        assert!(suggestions.iter().any(|s| s.text == ".id"));
        assert!(!suggestions.iter().any(|s| s.text == ".[]"));
        assert!(!suggestions.iter().any(|s| s.text == ".[].id"));
    }

    #[test]
    fn test_analyze_value_empty_array() {
        let json: Value = serde_json::from_str("[]").unwrap();
        let suggestions = ResultAnalyzer::analyze_value(&json, true, false);

        // Only .[] for empty array
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, ".[]");
    }

    #[test]
    fn test_analyze_value_scalar_returns_empty() {
        let number: Value = serde_json::from_str("42").unwrap();
        let string: Value = serde_json::from_str(r#""hello""#).unwrap();
        let boolean: Value = serde_json::from_str("true").unwrap();
        let null: Value = serde_json::from_str("null").unwrap();

        assert!(ResultAnalyzer::analyze_value(&number, true, false).is_empty());
        assert!(ResultAnalyzer::analyze_value(&string, true, false).is_empty());
        assert!(ResultAnalyzer::analyze_value(&boolean, true, false).is_empty());
        assert!(ResultAnalyzer::analyze_value(&null, true, false).is_empty());
    }

    #[test]
    fn test_analyze_value_nested_object() {
        let json: Value = serde_json::from_str(
            r#"{"user": {"profile": {"name": "Alice"}}, "settings": {"theme": "dark"}}"#,
        )
        .unwrap();
        let suggestions = ResultAnalyzer::analyze_value(&json, true, false);

        // Should suggest top-level fields only
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".user"));
        assert!(suggestions.iter().any(|s| s.text == ".settings"));
    }

    #[test]
    fn test_analyze_value_without_leading_dot() {
        let json: Value = serde_json::from_str(r#"{"name": "test"}"#).unwrap();
        let suggestions = ResultAnalyzer::analyze_value(&json, false, false);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, "name");
    }

    #[test]
    fn test_analyze_value_array_of_primitives() {
        let json: Value = serde_json::from_str("[1, 2, 3]").unwrap();
        let suggestions = ResultAnalyzer::analyze_value(&json, true, false);

        // Only .[] for array of primitives
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, ".[]");
    }

    #[test]
    fn test_analyze_value_accepts_reference() {
        // This test verifies the API accepts &Value (not Arc<Value>)
        let json: Value = serde_json::from_str(r#"{"field": "value"}"#).unwrap();
        let suggestions = ResultAnalyzer::analyze_value(&json, true, false);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, ".field");
    }
}

// ============================================================================
// Nonsimple Field Name Quoting Tests
// ============================================================================

#[test]
fn test_field_starting_with_digit_gets_quoted() {
    let json: Value = serde_json::from_str(r#"{"1numeric_key": "value"}"#).unwrap();
    let suggestions = ResultAnalyzer::analyze_value(&json, true, false);

    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].text, r#"."1numeric_key""#);
}

#[test]
fn test_field_with_hyphen_gets_quoted() {
    let json: Value = serde_json::from_str(r#"{"my-field": "value"}"#).unwrap();
    let suggestions = ResultAnalyzer::analyze_value(&json, true, false);

    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].text, r#"."my-field""#);
}

#[test]
fn test_valid_field_name_not_quoted() {
    let json: Value = serde_json::from_str(r#"{"simple_key": "value"}"#).unwrap();
    let suggestions = ResultAnalyzer::analyze_value(&json, true, false);

    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].text, ".simple_key");
}

#[test]
fn test_multiple_fields_with_mixed_identifier_types() {
    let json: Value =
        serde_json::from_str(r#"{"simple_key": 1, "1numeric_key": 2, "hyphen-key": 3}"#).unwrap();
    let suggestions = ResultAnalyzer::analyze_value(&json, true, false);

    assert_eq!(suggestions.len(), 3);
    let suggestion_texts: Vec<_> = suggestions.iter().map(|s| s.text.as_str()).collect();
    assert!(suggestion_texts.contains(&".simple_key"));
    assert!(suggestion_texts.contains(&r#"."1numeric_key""#));
    assert!(suggestion_texts.contains(&r#"."hyphen-key""#));
}

#[test]
fn test_array_of_objects_with_nonsimple_field_names() {
    let json: Value = serde_json::from_str(
        r#"[{"1numeric_key": "value1", "simple_key": "value2"}, {"1numeric_key": "value3", "simple_key": "value4"}]"#,
    )
    .unwrap();
    let suggestions = ResultAnalyzer::analyze_value(&json, true, false);

    assert_eq!(suggestions.len(), 3); // .[], .[].1numeric_key, .[].simple_key
    let suggestion_texts: Vec<_> = suggestions.iter().map(|s| s.text.as_str()).collect();
    assert!(suggestion_texts.contains(&".[]"));
    assert!(suggestion_texts.contains(&r#".[]."1numeric_key""#));
    assert!(suggestion_texts.contains(&".[].simple_key"));
}

#[test]
fn test_no_leading_dot_with_nonsimple_field() {
    let json: Value = serde_json::from_str(r#"{"1numeric_key": "value"}"#).unwrap();
    let suggestions = ResultAnalyzer::analyze_value(&json, false, false);

    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].text, r#""1numeric_key""#);
}

// ============================================================================
// Complex Nested Scenarios Tests
// ============================================================================

#[test]
fn test_nested_array_name_quoting_across_levels() {
    // Scenario: ."hyphen-array"[]."nested-items"[] with nested array names
    let json: Value = serde_json::from_str(
        r#"{
            "hyphen-array": [
                {
                    "nested-items": [
                        {"simple_key": "value"}
                    ]
                }
            ]
        }"#,
    )
    .unwrap();

    let top_level = ResultAnalyzer::analyze_value(&json, true, false);
    assert!(top_level.iter().any(|s| s.text == r#"."hyphen-array""#));
    assert!(!top_level.iter().any(|s| s.text == ".hyphen-array"));

    let outer_array = json
        .get("hyphen-array")
        .and_then(Value::as_array)
        .expect("outer array should exist");
    let outer_obj = outer_array
        .first()
        .and_then(Value::as_object)
        .expect("outer array should contain object");
    let outer_obj_value = Value::Object(outer_obj.clone());
    let nested_level = ResultAnalyzer::analyze_value(&outer_obj_value, true, false);
    assert!(nested_level.iter().any(|s| s.text == r#"."nested-items""#));
    assert!(!nested_level.iter().any(|s| s.text == ".nested-items"));
}

#[test]
fn test_nested_field_quoting_with_iteration() {
    // Scenario: .outer[]."inner-array"[]."hyphen-key" and .[]."1numeric_key"
    let json: Value = serde_json::from_str(
        r#"{
            "outer": [
                {
                    "inner-array": [
                        {"hyphen-key": "v1", "1numeric_key": "v2", "simple_key": "v3"}
                    ]
                }
            ]
        }"#,
    )
    .unwrap();

    let outer_array = json
        .get("outer")
        .and_then(Value::as_array)
        .expect("outer array should exist");
    let outer_obj = outer_array
        .first()
        .and_then(Value::as_object)
        .expect("outer array should contain object");
    let outer_obj_value = Value::Object(outer_obj.clone());
    let outer_suggestions = ResultAnalyzer::analyze_value(&outer_obj_value, true, false);
    assert!(
        outer_suggestions
            .iter()
            .any(|s| s.text == r#"."inner-array""#)
    );

    let inner_array = outer_obj_value
        .get("inner-array")
        .and_then(Value::as_array)
        .expect("inner array should exist");
    let inner_array_value = Value::Array(inner_array.clone());
    let inner_suggestions = ResultAnalyzer::analyze_value(&inner_array_value, true, false);

    assert!(inner_suggestions.iter().any(|s| s.text == ".[]"));
    assert!(
        inner_suggestions
            .iter()
            .any(|s| s.text == r#".[]."hyphen-key""#)
    );
    assert!(
        inner_suggestions
            .iter()
            .any(|s| s.text == r#".[]."1numeric_key""#)
    );
    assert!(inner_suggestions.iter().any(|s| s.text == ".[].simple_key"));
    assert!(!inner_suggestions.iter().any(|s| s.text == ".[].hyphen-key"));
    assert!(
        !inner_suggestions
            .iter()
            .any(|s| s.text == ".[].1numeric_key")
    );
}
