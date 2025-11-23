use super::state::{JsonFieldType, Suggestion, SuggestionType};
use serde_json::Value;
use std::collections::HashSet;

/// Analyze JSON structure and extract field names
pub struct JsonAnalyzer {
    /// All unique field names found in the JSON
    field_names: HashSet<String>,
    /// Cached root JSON value for context-aware analysis
    root_value: Option<Value>,
}

impl JsonAnalyzer {
    pub fn new() -> Self {
        Self {
            field_names: HashSet::new(),
            root_value: None,
        }
    }

    /// Analyze JSON value and extract all field names
    pub fn analyze(&mut self, json: &str) -> Result<(), String> {
        self.field_names.clear();

        let value: Value = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        self.extract_fields(&value);
        self.root_value = Some(value);
        Ok(())
    }

    /// Recursively extract field names from JSON value
    fn extract_fields(&mut self, value: &Value) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    self.field_names.insert(key.clone());
                    self.extract_fields(val);
                }
            }
            Value::Array(arr) => {
                for val in arr {
                    self.extract_fields(val);
                }
            }
            _ => {}
        }
    }

    /// Get context-aware field suggestions based on a partial jq path
    /// Returns fields available at the specified path in the JSON
    pub fn get_contextual_field_suggestions(
        &self,
        path: &str,
        prefix: &str,
    ) -> Vec<Suggestion> {
        // If path is empty or just ".", return top-level fields
        if path.is_empty() || path == "." {
            return self.get_top_level_fields(prefix);
        }

        // Get the value at the specified path
        let value_at_path = match self.get_value_at_path(path) {
            Some(v) => v,
            None => return Vec::new(), // Path doesn't exist, no suggestions
        };

        // Extract fields from the value at this path
        extract_fields_from_value(value_at_path, prefix)
    }

    /// Get top-level fields only
    fn get_top_level_fields(&self, prefix: &str) -> Vec<Suggestion> {
        let root = match &self.root_value {
            Some(v) => v,
            None => return Vec::new(),
        };

        match root {
            Value::Object(map) => {
                let mut fields: Vec<_> = map
                    .iter()
                    .filter(|(k, _)| {
                        prefix.is_empty()
                            || k.to_lowercase().starts_with(&prefix.to_lowercase())
                    })
                    .map(|(k, v)| {
                        let field_type = detect_json_type(v);
                        Suggestion::new_with_type(
                            format!(".{}", k),
                            SuggestionType::Field,
                            Some(field_type),
                        )
                    })
                    .collect();
                fields.sort_by(|a, b| a.text.cmp(&b.text));
                fields
            }
            _ => Vec::new(),
        }
    }

    /// Navigate to a value at the specified jq path
    fn get_value_at_path(&self, path: &str) -> Option<&Value> {
        let root = self.root_value.as_ref()?;

        // Handle pipes by evaluating left side first, then right side
        if let Some(pipe_pos) = path.rfind('|') {
            let left_path = path[..pipe_pos].trim();
            let right_path = path[pipe_pos + 1..].trim();

            // Navigate left side from root (recursively handle multiple pipes)
            let left_value = if left_path.is_empty() {
                root
            } else {
                self.get_value_at_path(left_path)?
            };

            // Navigate right side from left value
            return self.navigate_path(left_value, right_path);
        }

        // No pipe, just navigate normally
        self.navigate_path(root, path)
    }

    /// Navigate a path from a starting value
    fn navigate_path<'a>(&self, start_value: &'a Value, path: &str) -> Option<&'a Value> {
        // Remove leading dot if present
        let path = path.strip_prefix('.').unwrap_or(path);

        // Split path by dots and navigate
        let mut current = start_value;
        for segment in path.split('.').filter(|s| !s.is_empty()) {
            // Handle array access like "items[]" or "items[0]"
            let (field_name, is_array) = if let Some(idx) = segment.find('[') {
                (&segment[..idx], true)
            } else {
                (segment, false)
            };

            // Navigate to the field (unless it's just array access on current value like "[]")
            if !field_name.is_empty() {
                current = match current {
                    Value::Object(map) => map.get(field_name)?,
                    _ => return None,
                };
            }

            // If it's array access, get first element for field discovery
            if is_array {
                current = match current {
                    Value::Array(arr) => arr.first()?,
                    _ => return None,
                };
            }
        }

        Some(current)
    }

    /// Get all field names (used in tests)
    #[cfg(test)]
    pub fn get_all_fields(&self) -> Vec<String> {
        let mut fields: Vec<_> = self.field_names.iter().cloned().collect();
        fields.sort();
        fields
    }
}

/// Detect the JSON type of a value
fn detect_json_type(value: &Value) -> JsonFieldType {
    match value {
        Value::String(_) => JsonFieldType::String,
        Value::Number(_) => JsonFieldType::Number,
        Value::Bool(_) => JsonFieldType::Boolean,
        Value::Null => JsonFieldType::Null,
        Value::Object(_) => JsonFieldType::Object,
        Value::Array(_) => JsonFieldType::Array,
    }
}

/// Extract fields from a specific JSON value
fn extract_fields_from_value(value: &Value, prefix: &str) -> Vec<Suggestion> {
    match value {
        Value::Object(map) => {
            let mut fields: Vec<_> = map
                .iter()
                .filter(|(k, _)| {
                    prefix.is_empty()
                        || k.to_lowercase().starts_with(&prefix.to_lowercase())
                })
                .map(|(k, v)| {
                    let field_type = detect_json_type(v);
                    Suggestion::new_with_type(
                        format!(".{}", k),
                        SuggestionType::Field,
                        Some(field_type),
                    )
                })
                .collect();
            fields.sort_by(|a, b| a.text.cmp(&b.text));
            fields
        }
        Value::Array(arr) => {
            // For arrays, analyze the first element to get available fields
            if let Some(first) = arr.first() {
                extract_fields_from_value(first, prefix)
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(), // Primitives have no fields
    }
}

impl Default for JsonAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_object() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"name": "John", "age": 30}"#;
        analyzer.analyze(json).unwrap();

        let fields = analyzer.get_all_fields();
        assert_eq!(fields, vec!["age", "name"]);
    }

    #[test]
    fn test_nested_object() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"user": {"name": "John", "email": "john@example.com"}, "posts": []}"#;
        analyzer.analyze(json).unwrap();

        let fields = analyzer.get_all_fields();
        assert_eq!(fields, vec!["email", "name", "posts", "user"]);
    }

    #[test]
    fn test_array_of_objects() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"[{"id": 1, "name": "Item 1"}, {"id": 2, "name": "Item 2", "extra": true}]"#;
        analyzer.analyze(json).unwrap();

        let fields = analyzer.get_all_fields();
        assert_eq!(fields, vec!["extra", "id", "name"]);
    }

    #[test]
    fn test_contextual_top_level_fields() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"services": {"items": []}, "products": {"type": "xyz", "sku": "123"}}"#;
        analyzer.analyze(json).unwrap();

        // Top level should only return "services" and "products"
        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".services"));
        assert!(suggestions.iter().any(|s| s.text == ".products"));
    }

    #[test]
    fn test_contextual_nested_fields() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"services": {"items": []}, "products": {"type": "xyz", "sku": "123"}}"#;
        analyzer.analyze(json).unwrap();

        // At .products level should only return "type" and "sku"
        let suggestions = analyzer.get_contextual_field_suggestions(".products", "");
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".type"));
        assert!(suggestions.iter().any(|s| s.text == ".sku"));

        // Should NOT include "items" (from services)
        assert!(!suggestions.iter().any(|s| s.text == ".items"));
    }

    #[test]
    fn test_contextual_array_fields() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"items": [{"id": 1, "name": "Item1"}]}"#;
        analyzer.analyze(json).unwrap();

        // At .items[] level should return fields from array elements
        let suggestions = analyzer.get_contextual_field_suggestions(".items[]", "");
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".id"));
        assert!(suggestions.iter().any(|s| s.text == ".name"));
    }

    #[test]
    fn test_pipe_with_array_expansion() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"data": {"users": [{"userId": "123", "name": "John"}]}}"#;
        analyzer.analyze(json).unwrap();

        // After pipe with array expansion: .data.users | .[]
        let suggestions = analyzer.get_contextual_field_suggestions(".data.users | .[]", "");
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".userId"));
        assert!(suggestions.iter().any(|s| s.text == ".name"));
    }

    #[test]
    fn test_pipe_with_array_expansion_and_field() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"data": {"users": [{"userId": "123", "profile": {"email": "test@example.com"}}]}}"#;
        analyzer.analyze(json).unwrap();

        // After pipe with array expansion and field: .data.users | .[].profile
        let suggestions = analyzer.get_contextual_field_suggestions(".data.users | .[].profile", "");
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions.iter().any(|s| s.text == ".email"));
    }

    #[test]
    fn test_multiple_pipes() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"outer": {"middle": {"inner": {"value": "test"}}}}"#;
        analyzer.analyze(json).unwrap();

        // Multiple pipes: .outer | .middle | .inner
        let suggestions = analyzer.get_contextual_field_suggestions(".outer | .middle | .inner", "");
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions.iter().any(|s| s.text == ".value"));
    }

    #[test]
    fn test_multiple_pipes_with_arrays() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"data": {"users": [{"posts": [{"title": "Hello", "body": "World"}]}]}}"#;
        analyzer.analyze(json).unwrap();

        // Complex: .data.users | .[].posts | .[].title
        let suggestions = analyzer.get_contextual_field_suggestions(".data.users | .[].posts | .[]", "");
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".title"));
        assert!(suggestions.iter().any(|s| s.text == ".body"));
    }

    #[test]
    fn test_nested_arrays_without_pipes() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"items": [{"subitems": [{"name": "test", "id": 1}]}]}"#;
        analyzer.analyze(json).unwrap();

        // Nested arrays: .items[].subitems[]
        let suggestions = analyzer.get_contextual_field_suggestions(".items[].subitems[]", "");
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".name"));
        assert!(suggestions.iter().any(|s| s.text == ".id"));
    }

    #[test]
    fn test_pipe_at_root() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"field1": "value1", "field2": "value2"}"#;
        analyzer.analyze(json).unwrap();

        // Pipe at root: . | (same as just .)
        let suggestions = analyzer.get_contextual_field_suggestions(". | ", "");
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".field1"));
        assert!(suggestions.iter().any(|s| s.text == ".field2"));
    }

    #[test]
    fn test_complex_real_world_scenario() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{
            "status": "success",
            "data": {
                "users": [
                    {
                        "userId": "usr-001",
                        "username": "johndoe",
                        "profile": {
                            "firstName": "John",
                            "lastName": "Doe",
                            "email": "john@example.com"
                        },
                        "posts": [
                            {
                                "postId": "post-1",
                                "title": "My First Post",
                                "tags": ["tech", "coding"]
                            }
                        ]
                    }
                ]
            }
        }"#;
        analyzer.analyze(json).unwrap();

        // Test the exact scenario from the bug report: .data.users | .[].userId
        let suggestions = analyzer.get_contextual_field_suggestions(".data.users | .[]", "u");
        assert!(suggestions.iter().any(|s| s.text == ".userId"));
        assert!(suggestions.iter().any(|s| s.text == ".username"));

        // Test nested: .data.users | .[].profile
        let suggestions = analyzer.get_contextual_field_suggestions(".data.users | .[].profile", "");
        assert_eq!(suggestions.len(), 3);
        assert!(suggestions.iter().any(|s| s.text == ".firstName"));
        assert!(suggestions.iter().any(|s| s.text == ".lastName"));
        assert!(suggestions.iter().any(|s| s.text == ".email"));

        // Test double array expansion: .data.users | .[].posts | .[]
        let suggestions = analyzer.get_contextual_field_suggestions(".data.users | .[].posts | .[]", "");
        assert_eq!(suggestions.len(), 3);
        assert!(suggestions.iter().any(|s| s.text == ".postId"));
        assert!(suggestions.iter().any(|s| s.text == ".title"));
        assert!(suggestions.iter().any(|s| s.text == ".tags"));
    }

    #[test]
    fn test_malformed_path_with_unmatched_paren() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"items": [{"name": "test"}]}"#;
        analyzer.analyze(json).unwrap();

        // Path with unmatched ')' should fail gracefully (return empty)
        // This can happen from context extraction with function calls like: map(.items | .name) | .f
        let suggestions = analyzer.get_contextual_field_suggestions(".items | .name) |", "");
        // Should return empty because .name) isn't a valid field
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_invalid_paths_fail_gracefully() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"data": {"items": [{"id": 1}]}}"#;
        analyzer.analyze(json).unwrap();

        // These should all return empty (nonexistent paths), not crash
        assert_eq!(analyzer.get_contextual_field_suggestions(".nonexistent | .foo", "").len(), 0);
        assert_eq!(analyzer.get_contextual_field_suggestions(".data.wrong | .[]", "").len(), 0);
        assert_eq!(analyzer.get_contextual_field_suggestions(".data.items.nothere", "").len(), 0);
    }

    // ===== JSON Type Detection Tests =====

    #[test]
    fn test_field_suggestions_include_string_type() {
        let json = r#"{"name": "Alice", "email": "alice@example.com"}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");

        let name_suggestion = suggestions.iter().find(|s| s.text == ".name").unwrap();
        assert_eq!(name_suggestion.field_type, Some(JsonFieldType::String));

        let email_suggestion = suggestions.iter().find(|s| s.text == ".email").unwrap();
        assert_eq!(email_suggestion.field_type, Some(JsonFieldType::String));
    }

    #[test]
    fn test_field_suggestions_include_number_type() {
        let json = r#"{"age": 30, "height": 5.9, "count": 0}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");

        let age_suggestion = suggestions.iter().find(|s| s.text == ".age").unwrap();
        assert_eq!(age_suggestion.field_type, Some(JsonFieldType::Number));

        let height_suggestion = suggestions.iter().find(|s| s.text == ".height").unwrap();
        assert_eq!(height_suggestion.field_type, Some(JsonFieldType::Number));

        let count_suggestion = suggestions.iter().find(|s| s.text == ".count").unwrap();
        assert_eq!(count_suggestion.field_type, Some(JsonFieldType::Number));
    }

    #[test]
    fn test_field_suggestions_include_boolean_type() {
        let json = r#"{"active": true, "verified": false}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");

        let active_suggestion = suggestions.iter().find(|s| s.text == ".active").unwrap();
        assert_eq!(active_suggestion.field_type, Some(JsonFieldType::Boolean));

        let verified_suggestion = suggestions.iter().find(|s| s.text == ".verified").unwrap();
        assert_eq!(verified_suggestion.field_type, Some(JsonFieldType::Boolean));
    }

    #[test]
    fn test_field_suggestions_include_null_type() {
        let json = r#"{"nothing": null, "empty": null}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");

        let nothing_suggestion = suggestions.iter().find(|s| s.text == ".nothing").unwrap();
        assert_eq!(nothing_suggestion.field_type, Some(JsonFieldType::Null));

        let empty_suggestion = suggestions.iter().find(|s| s.text == ".empty").unwrap();
        assert_eq!(empty_suggestion.field_type, Some(JsonFieldType::Null));
    }

    #[test]
    fn test_field_suggestions_include_object_type() {
        let json = r#"{"user": {"name": "Bob"}, "config": {"debug": true}}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");

        let user_suggestion = suggestions.iter().find(|s| s.text == ".user").unwrap();
        assert_eq!(user_suggestion.field_type, Some(JsonFieldType::Object));

        let config_suggestion = suggestions.iter().find(|s| s.text == ".config").unwrap();
        assert_eq!(config_suggestion.field_type, Some(JsonFieldType::Object));
    }

    #[test]
    fn test_field_suggestions_include_array_type() {
        let json = r#"{"items": [1, 2, 3], "tags": ["a", "b"], "empty": []}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");

        let items_suggestion = suggestions.iter().find(|s| s.text == ".items").unwrap();
        assert_eq!(items_suggestion.field_type, Some(JsonFieldType::Array));

        let tags_suggestion = suggestions.iter().find(|s| s.text == ".tags").unwrap();
        assert_eq!(tags_suggestion.field_type, Some(JsonFieldType::Array));

        let empty_suggestion = suggestions.iter().find(|s| s.text == ".empty").unwrap();
        assert_eq!(empty_suggestion.field_type, Some(JsonFieldType::Array));
    }

    #[test]
    fn test_all_json_types_together() {
        let json = r#"{
            "name": "Alice",
            "age": 30,
            "active": true,
            "nothing": null,
            "address": {"city": "NYC"},
            "hobbies": ["reading", "coding"]
        }"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        assert_eq!(suggestions.len(), 6);

        assert_eq!(
            suggestions.iter().find(|s| s.text == ".name").unwrap().field_type,
            Some(JsonFieldType::String)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".age").unwrap().field_type,
            Some(JsonFieldType::Number)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".active").unwrap().field_type,
            Some(JsonFieldType::Boolean)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".nothing").unwrap().field_type,
            Some(JsonFieldType::Null)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".address").unwrap().field_type,
            Some(JsonFieldType::Object)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".hobbies").unwrap().field_type,
            Some(JsonFieldType::Array)
        );
    }

    #[test]
    fn test_nested_field_types() {
        let json = r#"{"user": {"name": "Bob", "age": 25, "verified": true}}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions(".user", "");
        assert_eq!(suggestions.len(), 3);

        assert_eq!(
            suggestions.iter().find(|s| s.text == ".name").unwrap().field_type,
            Some(JsonFieldType::String)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".age").unwrap().field_type,
            Some(JsonFieldType::Number)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".verified").unwrap().field_type,
            Some(JsonFieldType::Boolean)
        );
    }

    #[test]
    fn test_array_element_field_types() {
        let json = r#"{"items": [{"id": 1, "name": "Item1", "active": true}]}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions(".items[]", "");
        assert_eq!(suggestions.len(), 3);

        assert_eq!(
            suggestions.iter().find(|s| s.text == ".id").unwrap().field_type,
            Some(JsonFieldType::Number)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".name").unwrap().field_type,
            Some(JsonFieldType::String)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".active").unwrap().field_type,
            Some(JsonFieldType::Boolean)
        );
    }

    #[test]
    fn test_field_types_with_pipe() {
        let json = r#"{"data": {"users": [{"userId": "123", "score": 100, "admin": false}]}}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions(".data.users | .[]", "");
        assert_eq!(suggestions.len(), 3);

        assert_eq!(
            suggestions.iter().find(|s| s.text == ".userId").unwrap().field_type,
            Some(JsonFieldType::String)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".score").unwrap().field_type,
            Some(JsonFieldType::Number)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".admin").unwrap().field_type,
            Some(JsonFieldType::Boolean)
        );
    }

    #[test]
    fn test_mixed_types_in_nested_structure() {
        let json = r#"{
            "status": "success",
            "data": {
                "users": [
                    {
                        "userId": "usr-001",
                        "profile": {
                            "firstName": "John",
                            "age": 30,
                            "verified": true,
                            "metadata": null,
                            "settings": {"theme": "dark"},
                            "tags": ["admin", "user"]
                        }
                    }
                ]
            }
        }"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions(".data.users | .[].profile", "");
        assert_eq!(suggestions.len(), 6);

        assert_eq!(
            suggestions.iter().find(|s| s.text == ".firstName").unwrap().field_type,
            Some(JsonFieldType::String)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".age").unwrap().field_type,
            Some(JsonFieldType::Number)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".verified").unwrap().field_type,
            Some(JsonFieldType::Boolean)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".metadata").unwrap().field_type,
            Some(JsonFieldType::Null)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".settings").unwrap().field_type,
            Some(JsonFieldType::Object)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".tags").unwrap().field_type,
            Some(JsonFieldType::Array)
        );
    }

    #[test]
    fn test_empty_object_field_type() {
        let json = r#"{"emptyObj": {}}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");

        let obj_suggestion = suggestions.iter().find(|s| s.text == ".emptyObj").unwrap();
        assert_eq!(obj_suggestion.field_type, Some(JsonFieldType::Object));
    }

    #[test]
    fn test_negative_and_float_numbers() {
        let json = r#"{"negative": -42, "float": 3.14159, "scientific": 1.5e10}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");

        // All should be detected as Number type
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".negative").unwrap().field_type,
            Some(JsonFieldType::Number)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".float").unwrap().field_type,
            Some(JsonFieldType::Number)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".scientific").unwrap().field_type,
            Some(JsonFieldType::Number)
        );
    }

    #[test]
    fn test_unicode_strings_type() {
        let json = r#"{"emoji": "🎉", "chinese": "你好", "arabic": "مرحبا"}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");

        // All should be detected as String type
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".emoji").unwrap().field_type,
            Some(JsonFieldType::String)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".chinese").unwrap().field_type,
            Some(JsonFieldType::String)
        );
        assert_eq!(
            suggestions.iter().find(|s| s.text == ".arabic").unwrap().field_type,
            Some(JsonFieldType::String)
        );
    }
}
