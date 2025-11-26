use crate::autocomplete::state::{JsonFieldType, Suggestion, SuggestionType};
use serde_json::Value;

/// Analyze query execution results to extract field suggestions
///
/// This module provides the core functionality for generating autocompletion suggestions
/// based on the actual query results rather than parsing jq syntax.
pub struct ResultAnalyzer;

impl ResultAnalyzer {
    /// Main entry point: analyze a query result and extract suggestions
    ///
    /// # Arguments
    /// * `result` - The query execution result (may contain ANSI codes, multiple values)
    ///
    /// # Returns
    /// Vector of field suggestions extracted from the result
    pub fn analyze_result(result: &str) -> Vec<Suggestion> {
        if result.trim().is_empty() {
            return Vec::new();
        }

        // Strip ANSI color codes from jq output
        let clean_result = Self::strip_ansi_codes(result);

        // Parse the first JSON value from the result
        let value = match Self::parse_first_json_value(&clean_result) {
            Some(v) => v,
            None => return Vec::new(),
        };

        // Extract field suggestions from the parsed value
        Self::extract_top_level_suggestions(&value)
    }

    /// Strip ANSI escape codes from a string
    ///
    /// jq outputs colored results with ANSI codes like:
    /// - `\x1b[0m` (reset)
    /// - `\x1b[1;39m` (bold)
    /// - `\x1b[0;32m` (green)
    ///
    /// This function removes all ANSI codes efficiently
    fn strip_ansi_codes(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Found escape character, skip until 'm' (end of ANSI sequence)
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    while let Some(c) = chars.next() {
                        if c == 'm' {
                            break;
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
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
    fn parse_first_json_value(text: &str) -> Option<Value> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }

        // Try to parse the entire text first (common case: single value)
        if let Ok(value) = serde_json::from_str(text) {
            return Some(value);
        }

        // Handle multi-value output: parse line by line until we find valid JSON
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(value) = serde_json::from_str(line) {
                return Some(value);
            }
        }

        None
    }

    /// Extract field suggestions from a JSON value
    ///
    /// # Behavior based on value type:
    /// - **Object**: Returns field names (`.fieldName`)
    /// - **Array**: Returns array accessor (`.[]`) + suggestions for array elements
    /// - **Primitives** (string/number/bool/null): Returns empty (no suggestions)
    fn extract_top_level_suggestions(value: &Value) -> Vec<Suggestion> {
        match value {
            Value::Object(map) => {
                let mut suggestions = Vec::new();
                for (key, val) in map {
                    let field_type = Self::detect_json_type(val);
                    suggestions.push(Suggestion::new_with_type(
                        format!(".{}", key),
                        SuggestionType::Field,
                        Some(field_type),
                    ));
                }
                suggestions
            }
            Value::Array(arr) => {
                let mut suggestions = vec![
                    Suggestion::new_with_type(".[]", SuggestionType::Pattern, None)
                ];

                // Add suggestions for array element fields if first element is an object
                if let Some(Value::Object(map)) = arr.first() {
                    for (key, val) in map {
                        let field_type = Self::detect_json_type(val);
                        suggestions.push(Suggestion::new_with_type(
                            format!(".[]?.{}", key),
                            SuggestionType::Field,
                            Some(field_type),
                        ));
                    }
                }

                suggestions
            }
            _ => Vec::new(), // Primitives: no field suggestions
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
        let result = r#"{"name": "test", "age": 30, "active": true}"#;
        let suggestions = ResultAnalyzer::analyze_result(result);

        assert_eq!(suggestions.len(), 3);
        assert!(suggestions.iter().any(|s| s.text == ".name"));
        assert!(suggestions.iter().any(|s| s.text == ".age"));
        assert!(suggestions.iter().any(|s| s.text == ".active"));
    }

    #[test]
    fn test_analyze_nested_object() {
        let result = r#"{"user": {"name": "Alice", "profile": {"city": "NYC"}}}"#;
        let suggestions = ResultAnalyzer::analyze_result(result);

        // Should only return top-level fields
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions.iter().any(|s| s.text == ".user"));
    }

    #[test]
    fn test_analyze_array_result() {
        let result = r#"[{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]"#;
        let suggestions = ResultAnalyzer::analyze_result(result);

        // Should return .[] and element field suggestions
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
        assert!(suggestions.iter().any(|s| s.text == ".[]?.id"));
        assert!(suggestions.iter().any(|s| s.text == ".[]?.name"));
    }

    #[test]
    fn test_analyze_empty_array() {
        let result = "[]";
        let suggestions = ResultAnalyzer::analyze_result(result);

        // Should only return .[] for empty arrays
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, ".[]");
    }

    #[test]
    fn test_analyze_empty_object() {
        let result = "{}";
        let suggestions = ResultAnalyzer::analyze_result(result);

        assert_eq!(suggestions.len(), 0);
    }

    // ============================================================================
    // ANSI Handling Tests
    // ============================================================================

    #[test]
    fn test_strip_ansi_simple() {
        let input = "\x1b[0m{\x1b[1;39m\"name\"\x1b[0m: \x1b[0;32m\"test\"\x1b[0m}";
        let output = ResultAnalyzer::strip_ansi_codes(input);
        assert_eq!(output, r#"{"name": "test"}"#);
    }

    #[test]
    fn test_strip_ansi_complex() {
        let input = "\x1b[1;39m{\x1b[0m\n  \x1b[0;34m\"key\"\x1b[0m: \x1b[0;32m\"value\"\x1b[0m\n\x1b[1;39m}\x1b[0m";
        let output = ResultAnalyzer::strip_ansi_codes(input);
        assert!(output.contains(r#""key""#));
        assert!(output.contains(r#""value""#));
        assert!(!output.contains("\x1b"));
    }

    #[test]
    fn test_analyze_colored_output() {
        let result = "\x1b[0m{\x1b[1;39m\"name\"\x1b[0m: \x1b[0;32m\"Alice\"\x1b[0m, \x1b[1;39m\"age\"\x1b[0m: \x1b[0;33m30\x1b[0m}";
        let suggestions = ResultAnalyzer::analyze_result(result);

        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".name"));
        assert!(suggestions.iter().any(|s| s.text == ".age"));
    }

    // ============================================================================
    // Multi-value Output Tests
    // ============================================================================

    #[test]
    fn test_multivalue_objects() {
        let result = r#"{"name": "Alice", "age": 30}
{"name": "Bob", "age": 25}
{"name": "Charlie", "age": 35}"#;
        let suggestions = ResultAnalyzer::analyze_result(result);

        // Should parse first object only
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".name"));
        assert!(suggestions.iter().any(|s| s.text == ".age"));
    }

    #[test]
    fn test_multivalue_mixed_types() {
        let result = r#"42
"hello"
{"field": "value"}"#;
        let suggestions = ResultAnalyzer::analyze_result(result);

        // Should parse first value (42), which is a primitive -> no suggestions
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_multivalue_with_whitespace() {
        let result = r#"

{"key1": "val1"}

{"key2": "val2"}
"#;
        let suggestions = ResultAnalyzer::analyze_result(result);

        // Should skip empty lines and parse first object
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, ".key1");
    }

    // ============================================================================
    // The Main Fix: Object Constructor Tests
    // ============================================================================

    #[test]
    fn test_object_constructor_suggestions() {
        // This is THE main fix: after `.services[] | {name: .serviceName, cap: .base}`
        // the result is objects with ONLY "name" and "cap" fields
        let result = r#"{"name": "MyService", "cap": 10}"#;
        let suggestions = ResultAnalyzer::analyze_result(result);

        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".name"));
        assert!(suggestions.iter().any(|s| s.text == ".cap"));

        // Should NOT suggest original fields like .serviceName or .base
        assert!(!suggestions.iter().any(|s| s.text == ".serviceName"));
        assert!(!suggestions.iter().any(|s| s.text == ".base"));
    }

    #[test]
    fn test_array_constructor_suggestions() {
        // After `[.field1, .field2]` the result is an array
        let result = r#"["value1", "value2"]"#;
        let suggestions = ResultAnalyzer::analyze_result(result);

        // Should suggest .[] for array access
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_primitive_results() {
        // Number
        assert_eq!(ResultAnalyzer::analyze_result("42").len(), 0);
        // String
        assert_eq!(ResultAnalyzer::analyze_result(r#""hello""#).len(), 0);
        // Boolean
        assert_eq!(ResultAnalyzer::analyze_result("true").len(), 0);
    }

    #[test]
    fn test_null_result() {
        let suggestions = ResultAnalyzer::analyze_result("null");
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_empty_string_result() {
        let suggestions = ResultAnalyzer::analyze_result("");
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_invalid_json_result() {
        let result = "not valid json {]";
        let suggestions = ResultAnalyzer::analyze_result(result);
        // Should return empty gracefully
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

        let suggestions = ResultAnalyzer::analyze_result(&result);

        // Should extract fields from first array element
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
        assert!(suggestions.iter().any(|s| s.text == ".[]?.id"));
        assert!(suggestions.iter().any(|s| s.text == ".[]?.name"));
        assert!(suggestions.iter().any(|s| s.text == ".[]?.value"));
    }

    // ============================================================================
    // Optional Chaining Tests
    // ============================================================================

    #[test]
    fn test_array_with_nulls_in_result() {
        // Result contains nulls from optional chaining
        let result = r#"[null, null, {"field": "value"}]"#;
        let suggestions = ResultAnalyzer::analyze_result(result);

        // Should suggest based on first element (null has no fields)
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
        assert_eq!(suggestions.len(), 1); // Only .[] since first element is null
    }

    #[test]
    fn test_bounded_scan_in_results() {
        // Test that we only look at the first element, not all elements
        let result = r#"[{"a": 1}, {"b": 2}, {"c": 3}]"#;
        let suggestions = ResultAnalyzer::analyze_result(result);

        // Should only have fields from first element
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
        assert!(suggestions.iter().any(|s| s.text == ".[]?.a"));
        assert!(!suggestions.iter().any(|s| s.text == ".[]?.b"));
        assert!(!suggestions.iter().any(|s| s.text == ".[]?.c"));
    }

    // ============================================================================
    // Type Detection Tests
    // ============================================================================

    #[test]
    fn test_field_type_detection() {
        let result = r#"{
            "str": "hello",
            "num": 42,
            "bool": true,
            "null": null,
            "obj": {"nested": "value"},
            "arr": [1, 2, 3]
        }"#;
        let suggestions = ResultAnalyzer::analyze_result(result);

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
