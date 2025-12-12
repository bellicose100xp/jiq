use crate::query::ResultType;
use crate::autocomplete::autocomplete_state::{JsonFieldType, Suggestion, SuggestionType};
use serde_json::Value;

/// Analyze query execution results to extract field suggestions
///
/// This module provides the core functionality for generating autocompletion suggestions
/// based on the actual query results rather than parsing jq syntax.
pub struct ResultAnalyzer;

/// Helper to determine if a leading dot is needed
#[inline]
fn dot_prefix(needs_leading_dot: bool) -> &'static str {
    if needs_leading_dot { "." } else { "" }
}

impl ResultAnalyzer {
    /// Extract field suggestions from object map
    fn extract_object_fields(
        map: &serde_json::Map<String, Value>,
        prefix: &str,
        suggestions: &mut Vec<Suggestion>,
    ) {
        for (key, val) in map {
            let field_type = Self::detect_json_type(val);
            suggestions.push(Suggestion::new_with_type(
                format!("{}{}", prefix, key),
                SuggestionType::Field,
                Some(field_type),
            ));
        }
    }
    /// Main entry point: analyze a query result and extract suggestions
    ///
    /// # Arguments
    /// * `result` - The query execution result WITHOUT ANSI codes (pre-stripped)
    /// * `result_type` - The type of result (ArrayOfObjects, DestructuredObjects, etc.)
    /// * `needs_leading_dot` - Whether suggestions should include leading dot (based on trigger context)
    ///
    /// # Returns
    /// Vector of field suggestions in context-appropriate format
    pub fn analyze_result(
        result: &str,
        result_type: ResultType,
        needs_leading_dot: bool,
    ) -> Vec<Suggestion> {
        if result.trim().is_empty() {
            return Vec::new();
        }

        // Parse the first JSON value from the result
        let value = match Self::parse_first_json_value(result) {
            Some(v) => v,
            None => return Vec::new(),
        };

        // Extract field suggestions based on result type and context
        Self::extract_suggestions_for_type(&value, result_type, needs_leading_dot)
    }

    /// Parse the first JSON value from text (handles multi-value output)
    ///
    /// jq can output multiple JSON values when using `.[]` or similar:
    /// ```text
    /// {"a": 1}
    /// {"a": 2}
    /// {"a": 3}
    /// ```
    ///
    /// This function parses the first valid JSON value and ignores the rest.
    /// Handles both compact and pretty-printed JSON output.
    fn parse_first_json_value(text: &str) -> Option<Value> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }

        // Try to parse the entire text first (common case: single value)
        if let Ok(value) = serde_json::from_str(text) {
            return Some(value);
        }

        // Handle multi-value output - could be pretty-printed or compact
        // Strategy: Use serde_json's streaming parser to read first value
        let mut deserializer = serde_json::Deserializer::from_str(text).into_iter();
        if let Some(Ok(value)) = deserializer.next() {
            return Some(value);
        }

        None
    }

    /// Extract suggestions based on result type and context
    ///
    /// Generates suggestions in format appropriate for the result type and trigger context:
    /// - ArrayOfObjects: `[]`, `[].field` (with optional leading `.`)
    /// - DestructuredObjects: `field` (with optional leading `.`)
    /// - Object: `.field` (with optional leading `.`)
    /// - Primitives: No suggestions
    fn extract_suggestions_for_type(
        value: &Value,
        result_type: ResultType,
        needs_leading_dot: bool,
    ) -> Vec<Suggestion> {
        match result_type {
            ResultType::ArrayOfObjects => {
                // Suggestions for array containing objects
                let prefix = dot_prefix(needs_leading_dot);
                let mut suggestions = vec![
                    Suggestion::new_with_type(
                        format!("{}[]", prefix),
                        SuggestionType::Pattern,
                        None,
                    )
                ];

                // Add field suggestions from first object
                if let Value::Array(arr) = value
                    && let Some(Value::Object(map)) = arr.first() {
                        for (key, val) in map {
                            let field_type = Self::detect_json_type(val);
                            suggestions.push(Suggestion::new_with_type(
                                format!("{}[].{}", prefix, key),
                                SuggestionType::Field,
                                Some(field_type),
                            ));
                        }
                    }

                suggestions
            }
            ResultType::DestructuredObjects => {
                // Suggestions for destructured objects (from .[])
                let prefix = dot_prefix(needs_leading_dot);
                let mut suggestions = Vec::new();

                if let Value::Object(map) = value {
                    Self::extract_object_fields(map, prefix, &mut suggestions);
                }

                suggestions
            }
            ResultType::Object => {
                // Suggestions for single object
                let prefix = dot_prefix(needs_leading_dot);
                let mut suggestions = Vec::new();

                if let Value::Object(map) = value {
                    Self::extract_object_fields(map, prefix, &mut suggestions);
                }

                suggestions
            }
            ResultType::Array => {
                // Array of primitives - just suggest []
                let prefix = dot_prefix(needs_leading_dot);
                vec![Suggestion::new_with_type(
                    format!("{}[]", prefix),
                    SuggestionType::Pattern,
                    None,
                )]
            }
            _ => Vec::new(), // Primitives (String, Number, Boolean, Null): no field suggestions
        }
    }

    /// Detect the JSON field type for a value
    ///
    /// This function is copied from the original json_analyzer.rs
    /// to maintain type detection logic.
    fn detect_json_type(value: &Value) -> JsonFieldType {
        match value {
            Value::Null => JsonFieldType::Null,
            Value::Bool(_) => JsonFieldType::Boolean,
            Value::Number(_) => JsonFieldType::Number,
            Value::String(_) => JsonFieldType::String,
            Value::Array(arr) => {
                if arr.is_empty() {
                    JsonFieldType::Array
                } else {
                    let inner_type = Self::detect_json_type(&arr[0]);
                    JsonFieldType::ArrayOf(Box::new(inner_type))
                }
            }
            Value::Object(_) => JsonFieldType::Object,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Basic Functionality Tests
    // ============================================================================

    #[test]
    fn test_analyze_simple_object() {
        // Single object result after operator (needs leading dot)
        let result = r#"{"name": "test", "age": 30, "active": true}"#;
        let suggestions = ResultAnalyzer::analyze_result(
            result,
            ResultType::Object,
            true, // After operator like | or at start
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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Object, true);

        // Should only return top-level fields
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions.iter().any(|s| s.text == ".user"));
    }

    #[test]
    fn test_analyze_array_of_objects_after_operator() {
        // Array of objects after operator (needs leading dot)
        let result = r#"[{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]"#;
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::ArrayOfObjects, true);

        // Should return .[] and element field suggestions with leading dot
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
        assert!(suggestions.iter().any(|s| s.text == ".[].id"));
        assert!(suggestions.iter().any(|s| s.text == ".[].name"));
    }

    #[test]
    fn test_analyze_array_of_objects_after_continuation() {
        // Array of objects after continuation like .services. (no leading dot)
        let result = r#"[{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]"#;
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::ArrayOfObjects, false);

        // Should return [] and element field suggestions without leading dot
        assert!(suggestions.iter().any(|s| s.text == "[]"));
        assert!(suggestions.iter().any(|s| s.text == "[].id"));
        assert!(suggestions.iter().any(|s| s.text == "[].name"));
    }

    #[test]
    fn test_analyze_empty_array() {
        // Empty array after operator
        let result = "[]";
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Array, true);

        // Should only return .[] for empty arrays
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, ".[]");
    }

    #[test]
    fn test_analyze_empty_object() {
        // Empty object
        let result = "{}";
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Object, true);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Object, true);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::DestructuredObjects, false);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::DestructuredObjects, true);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::DestructuredObjects, false);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Number, true);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::DestructuredObjects, true);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Object, true);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Array, true);

        // Should suggest .[] for array access
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_primitive_results() {
        // Primitives have no field suggestions
        assert_eq!(ResultAnalyzer::analyze_result("42", ResultType::Number, true).len(), 0);
        assert_eq!(ResultAnalyzer::analyze_result(r#""hello""#, ResultType::String, true).len(), 0);
        assert_eq!(ResultAnalyzer::analyze_result("true", ResultType::Boolean, true).len(), 0);
    }

    #[test]
    fn test_null_result() {
        let suggestions = ResultAnalyzer::analyze_result("null", ResultType::Null, true);
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_empty_string_result() {
        // Empty result treated as null
        let suggestions = ResultAnalyzer::analyze_result("", ResultType::Null, true);
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_invalid_json_result() {
        // Invalid JSON treated as null - returns empty gracefully
        let result = "not valid json {]";
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Null, true);
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
            result.push_str(&format!(r#"{{"id": {}, "name": "item{}", "value": {}}}"#, i, i, i * 2));
        }
        result.push(']');

        let suggestions = ResultAnalyzer::analyze_result(&result, ResultType::ArrayOfObjects, true);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::ArrayOfObjects, true);

        // Should suggest based on first element (null has no fields)
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
        assert_eq!(suggestions.len(), 1); // Only .[] since first element is null
    }

    #[test]
    fn test_bounded_scan_in_results() {
        // Test that we only look at the first element, not all elements
        let result = r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#;
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::ArrayOfObjects, true);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::DestructuredObjects, false);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::DestructuredObjects, true);

        // Should suggest fields WITH leading dot
        assert!(suggestions.iter().any(|s| s.text == ".serviceArn"));
        // All should start with dot
        assert!(suggestions.iter().all(|s| s.text.starts_with('.')));
    }

    #[test]
    fn test_array_of_objects_after_dot_no_prefix() {
        // After .services. → array of objects, no leading dot
        let result = r#"[{"id": 1}, {"id": 2}]"#;
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::ArrayOfObjects, false);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::ArrayOfObjects, true);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Object, false);

        // Should suggest fields without leading dot
        assert!(suggestions.iter().any(|s| s.text == "name"));
        assert!(suggestions.iter().any(|s| s.text == "age"));
        assert!(!suggestions.iter().any(|s| s.text.starts_with('.')));
    }

    #[test]
    fn test_single_object_after_operator_with_prefix() {
        // After .user | . → single object, needs leading dot
        let result = r#"{"name": "Alice", "age": 30}"#;
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Object, true);

        // Should suggest fields WITH leading dot
        assert!(suggestions.iter().any(|s| s.text == ".name"));
        assert!(suggestions.iter().any(|s| s.text == ".age"));
        assert!(suggestions.iter().all(|s| s.text.starts_with('.')));
    }

    #[test]
    fn test_primitive_array_after_operator() {
        // Array of primitives after operator
        let result = "[1, 2, 3]";
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Array, true);

        // Should only suggest .[]
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, ".[]");
    }

    #[test]
    fn test_primitive_array_after_continuation() {
        // Array of primitives after continuation
        let result = "[1, 2, 3]";
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Array, false);

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
        let suggestions = ResultAnalyzer::analyze_result(result, ResultType::Object, true);

        // Verify types are correctly detected
        let str_field = suggestions.iter().find(|s| s.text == ".str").unwrap();
        assert!(matches!(str_field.field_type, Some(JsonFieldType::String)));

        let num_field = suggestions.iter().find(|s| s.text == ".num").unwrap();
        assert!(matches!(num_field.field_type, Some(JsonFieldType::Number)));

        let bool_field = suggestions.iter().find(|s| s.text == ".bool").unwrap();
        assert!(matches!(bool_field.field_type, Some(JsonFieldType::Boolean)));

        let null_field = suggestions.iter().find(|s| s.text == ".null").unwrap();
        assert!(matches!(null_field.field_type, Some(JsonFieldType::Null)));

        let obj_field = suggestions.iter().find(|s| s.text == ".obj").unwrap();
        assert!(matches!(obj_field.field_type, Some(JsonFieldType::Object)));

        let arr_field = suggestions.iter().find(|s| s.text == ".arr").unwrap();
        assert!(matches!(arr_field.field_type, Some(JsonFieldType::ArrayOf(_))));
    }
}
