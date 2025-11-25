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

        // Check if path already has array access (ends with ])
        // This covers [], [0], [0:5], etc.
        let has_array_access = path.trim_end().ends_with(']');

        // Check if we're after a pipe - in this case, suggestions need dot prefix
        // e.g., ".items | ." should suggest ".[]" not "[]"
        let after_pipe = path.trim_end().ends_with('|');

        // Extract fields from the value at this path
        extract_fields_from_value(value_at_path, prefix, has_array_access, after_pipe)
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
            Value::Array(arr) => {
                // Root is an array - suggest .[] and .[].field for element fields
                if let Some(first) = arr.first() {
                    // Use after_pipe=true to get "." prefix (e.g., ".[]" not "[]")
                    extract_array_element_fields(first, prefix, true)
                } else {
                    Vec::new()
                }
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
        Value::Array(arr) => {
            // Peek at first element to determine array element type
            if let Some(first) = arr.first() {
                let element_type = detect_json_type(first);
                JsonFieldType::ArrayOf(Box::new(element_type))
            } else {
                // Empty array - can't determine element type
                JsonFieldType::Array
            }
        }
    }
}

/// Extract fields from a specific JSON value
/// When `already_has_array_access` is true, arrays show normal .field suggestions
/// When false, arrays show [].field suggestions to indicate [] is needed
/// When `after_pipe` is true, suggestions need dot prefix (e.g., ".[]" instead of "[]")
fn extract_fields_from_value(
    value: &Value,
    prefix: &str,
    already_has_array_access: bool,
    after_pipe: bool,
) -> Vec<Suggestion> {
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
                if already_has_array_access {
                    // User already typed [], [0], etc. - show normal .field suggestions
                    extract_fields_from_value(first, prefix, true, after_pipe)
                } else {
                    // User needs [] - show [].field suggestions
                    extract_array_element_fields(first, prefix, after_pipe)
                }
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(), // Primitives have no fields
    }
}

/// Extract fields from array elements with [] prefix
/// Used when the user is at an array but hasn't typed [] yet
/// When `after_pipe` is true, prefix with "." (e.g., ".[]" instead of "[]")
fn extract_array_element_fields(value: &Value, prefix: &str, after_pipe: bool) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    // Determine the prefix for suggestions
    // After a pipe, we need ".[]" and ".[].field"
    // Otherwise, we need "[]" and "[].field"
    let dot_prefix = if after_pipe { "." } else { "" };

    // Add standalone [] as first suggestion (only if no prefix typed)
    if prefix.is_empty() {
        suggestions.push(
            Suggestion::new(format!("{}[]", dot_prefix), SuggestionType::Pattern)
                .with_description("iterate all elements"),
        );
    }

    // Add [].field suggestions for object fields
    if let Value::Object(map) = value {
        let mut fields: Vec<_> = map
            .iter()
            .filter(|(k, _)| {
                prefix.is_empty()
                    || k.to_lowercase().starts_with(&prefix.to_lowercase())
            })
            .map(|(k, v)| {
                let field_type = detect_json_type(v);
                Suggestion::new_with_type(
                    format!("{}[].{}", dot_prefix, k),
                    SuggestionType::Field,
                    Some(field_type),
                )
            })
            .collect();
        fields.sort_by(|a, b| a.text.cmp(&b.text));
        suggestions.extend(fields);
    }

    suggestions
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

        // Arrays with elements should show element type
        let items_suggestion = suggestions.iter().find(|s| s.text == ".items").unwrap();
        assert_eq!(items_suggestion.field_type, Some(JsonFieldType::ArrayOf(Box::new(JsonFieldType::Number))));

        let tags_suggestion = suggestions.iter().find(|s| s.text == ".tags").unwrap();
        assert_eq!(tags_suggestion.field_type, Some(JsonFieldType::ArrayOf(Box::new(JsonFieldType::String))));

        // Empty array should show just Array (can't determine element type)
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
            Some(JsonFieldType::ArrayOf(Box::new(JsonFieldType::String)))
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
            Some(JsonFieldType::ArrayOf(Box::new(JsonFieldType::String)))
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

    #[test]
    fn test_prefix_filtering_preserves_types() {
        let json = r#"{"name": "Alice", "nickname": "Ally", "age": 30, "note": null}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        // Filter with prefix "n" - should get name, nickname, note
        let suggestions = analyzer.get_contextual_field_suggestions("", "n");
        assert_eq!(suggestions.len(), 3);

        // Verify types are preserved after filtering
        let name_sugg = suggestions.iter().find(|s| s.text == ".name").unwrap();
        assert_eq!(name_sugg.field_type, Some(JsonFieldType::String));

        let nickname_sugg = suggestions.iter().find(|s| s.text == ".nickname").unwrap();
        assert_eq!(nickname_sugg.field_type, Some(JsonFieldType::String));

        let note_sugg = suggestions.iter().find(|s| s.text == ".note").unwrap();
        assert_eq!(note_sugg.field_type, Some(JsonFieldType::Null));

        // age should be filtered out
        assert!(suggestions.iter().all(|s| s.text != ".age"));
    }

    #[test]
    fn test_case_insensitive_prefix_filtering_with_types() {
        let json = r#"{"Name": "Bob", "AGE": 25, "Active": true}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        // Filter with lowercase prefix should match mixed case fields
        let suggestions = analyzer.get_contextual_field_suggestions("", "a");
        assert_eq!(suggestions.len(), 2); // "AGE" and "Active"

        // Verify types are correct
        let age_sugg = suggestions.iter().find(|s| s.text == ".AGE").unwrap();
        assert_eq!(age_sugg.field_type, Some(JsonFieldType::Number));

        let active_sugg = suggestions.iter().find(|s| s.text == ".Active").unwrap();
        assert_eq!(active_sugg.field_type, Some(JsonFieldType::Boolean));
    }

    #[test]
    fn test_root_level_array_shows_suggestions() {
        let json = r#"[{"id": 1, "name": "Item1"}, {"id": 2, "name": "Item2"}]"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        // Top-level array should suggest .[] and .[].field for element fields
        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        assert!(suggestions.len() >= 3); // .[], .[].id, .[].name

        // Should have .[] as first suggestion
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
        // Should have field suggestions with .[]. prefix
        assert!(suggestions.iter().any(|s| s.text == ".[].id"));
        assert!(suggestions.iter().any(|s| s.text == ".[].name"));
    }

    // ===== Array Element Type Tests =====

    #[test]
    fn test_array_element_type_object() {
        let json = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        let users = suggestions.iter().find(|s| s.text == ".users").unwrap();

        assert_eq!(users.field_type, Some(JsonFieldType::ArrayOf(Box::new(JsonFieldType::Object))));
    }

    #[test]
    fn test_array_element_type_number() {
        let json = r#"{"scores": [100, 95, 87], "floats": [1.5, 2.3, 3.7]}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");

        let scores = suggestions.iter().find(|s| s.text == ".scores").unwrap();
        assert_eq!(scores.field_type, Some(JsonFieldType::ArrayOf(Box::new(JsonFieldType::Number))));

        let floats = suggestions.iter().find(|s| s.text == ".floats").unwrap();
        assert_eq!(floats.field_type, Some(JsonFieldType::ArrayOf(Box::new(JsonFieldType::Number))));
    }

    #[test]
    fn test_array_element_type_boolean() {
        let json = r#"{"flags": [true, false, true]}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        let flags = suggestions.iter().find(|s| s.text == ".flags").unwrap();

        assert_eq!(flags.field_type, Some(JsonFieldType::ArrayOf(Box::new(JsonFieldType::Boolean))));
    }

    #[test]
    fn test_array_element_type_null() {
        let json = r#"{"nulls": [null, null]}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        let nulls = suggestions.iter().find(|s| s.text == ".nulls").unwrap();

        assert_eq!(nulls.field_type, Some(JsonFieldType::ArrayOf(Box::new(JsonFieldType::Null))));
    }

    #[test]
    fn test_nested_arrays() {
        let json = r#"{"matrix": [[1, 2], [3, 4]]}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        let matrix = suggestions.iter().find(|s| s.text == ".matrix").unwrap();

        // Array of arrays (first element is [1, 2] which is Array[Number])
        assert_eq!(
            matrix.field_type,
            Some(JsonFieldType::ArrayOf(
                Box::new(JsonFieldType::ArrayOf(
                    Box::new(JsonFieldType::Number)
                ))
            ))
        );
    }

    #[test]
    fn test_mixed_type_array_shows_first_element_type() {
        let json = r#"{"mixed": [42, "string", true, null]}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        let mixed = suggestions.iter().find(|s| s.text == ".mixed").unwrap();

        // Should show type of first element (Number)
        assert_eq!(mixed.field_type, Some(JsonFieldType::ArrayOf(Box::new(JsonFieldType::Number))));
    }

    #[test]
    fn test_empty_array_shows_generic_array() {
        let json = r#"{"empty": []}"#;
        let mut analyzer = JsonAnalyzer::new();
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        let empty = suggestions.iter().find(|s| s.text == ".empty").unwrap();

        // Can't determine element type, should show generic Array
        assert_eq!(empty.field_type, Some(JsonFieldType::Array));
    }

    #[test]
    fn test_array_element_types_display_format() {
        // Test the Display implementation for ArrayOf
        assert_eq!(format!("{}", JsonFieldType::ArrayOf(Box::new(JsonFieldType::String))), "Array[String]");
        assert_eq!(format!("{}", JsonFieldType::ArrayOf(Box::new(JsonFieldType::Number))), "Array[Number]");
        assert_eq!(format!("{}", JsonFieldType::ArrayOf(Box::new(JsonFieldType::Object))), "Array[Object]");
        assert_eq!(
            format!("{}", JsonFieldType::ArrayOf(Box::new(JsonFieldType::ArrayOf(Box::new(JsonFieldType::Number))))),
            "Array[Array[Number]]"
        );
    }

    // ===== Array Bracket Prefix Tests =====

    #[test]
    fn test_array_without_brackets_shows_bracket_prefix() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"items": [{"id": 1, "name": "Item1"}]}"#;
        analyzer.analyze(json).unwrap();

        // .items. (no []) should suggest [], [].id, [].name
        let suggestions = analyzer.get_contextual_field_suggestions(".items", "");

        // First suggestion should be standalone []
        assert_eq!(suggestions[0].text, "[]");
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::Pattern);

        // Should have [].field suggestions
        assert!(suggestions.iter().any(|s| s.text == "[].id"));
        assert!(suggestions.iter().any(|s| s.text == "[].name"));

        // Should NOT have bare .id, .name
        assert!(!suggestions.iter().any(|s| s.text == ".id"));
    }

    #[test]
    fn test_array_with_brackets_shows_normal_fields() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"items": [{"id": 1, "name": "Item1"}]}"#;
        analyzer.analyze(json).unwrap();

        // .items[]. should suggest .id, .name (normal)
        let suggestions = analyzer.get_contextual_field_suggestions(".items[]", "");
        assert!(suggestions.iter().any(|s| s.text == ".id"));
        assert!(suggestions.iter().any(|s| s.text == ".name"));
        // Should NOT have [].id, [].name
        assert!(!suggestions.iter().any(|s| s.text == "[].id"));
        // Should NOT have standalone []
        assert!(!suggestions.iter().any(|s| s.text == "[]"));
    }

    #[test]
    fn test_array_with_index_shows_normal_fields() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"items": [{"id": 1, "name": "Item1"}]}"#;
        analyzer.analyze(json).unwrap();

        // .items[0]. should suggest .id, .name (normal)
        let suggestions = analyzer.get_contextual_field_suggestions(".items[0]", "");
        assert!(suggestions.iter().any(|s| s.text == ".id"));
        assert!(suggestions.iter().any(|s| s.text == ".name"));
        // Should NOT have [].field
        assert!(!suggestions.iter().any(|s| s.text == "[].id"));
    }

    #[test]
    fn test_array_with_slice_shows_normal_fields() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"items": [{"id": 1, "name": "Item1"}]}"#;
        analyzer.analyze(json).unwrap();

        // .items[0:5]. should suggest .id, .name (normal)
        let suggestions = analyzer.get_contextual_field_suggestions(".items[0:5]", "");
        assert!(suggestions.iter().any(|s| s.text == ".id"));
        assert!(suggestions.iter().any(|s| s.text == ".name"));
        // Should NOT have [].field
        assert!(!suggestions.iter().any(|s| s.text == "[].id"));
    }

    #[test]
    fn test_nested_array_without_brackets() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"data": {"users": [{"id": 1, "profile": {"email": "test"}}]}}"#;
        analyzer.analyze(json).unwrap();

        // .data.users. should suggest [], [].id, [].profile
        let suggestions = analyzer.get_contextual_field_suggestions(".data.users", "");
        assert!(suggestions.iter().any(|s| s.text == "[]"));
        assert!(suggestions.iter().any(|s| s.text == "[].id"));
        assert!(suggestions.iter().any(|s| s.text == "[].profile"));
    }

    #[test]
    fn test_array_with_pipe_without_brackets() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"items": [{"id": 1, "name": "Item1"}]}"#;
        analyzer.analyze(json).unwrap();

        // .items | . should suggest .[], .[].id, .[].name (with dot prefix after pipe)
        let suggestions = analyzer.get_contextual_field_suggestions(".items |", "");
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
        assert!(suggestions.iter().any(|s| s.text == ".[].id"));
        assert!(suggestions.iter().any(|s| s.text == ".[].name"));
    }

    #[test]
    fn test_array_with_pipe_with_brackets() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"items": [{"id": 1, "name": "Item1"}]}"#;
        analyzer.analyze(json).unwrap();

        // .items | .[]. should suggest .id, .name
        let suggestions = analyzer.get_contextual_field_suggestions(".items | .[]", "");
        assert!(suggestions.iter().any(|s| s.text == ".id"));
        assert!(suggestions.iter().any(|s| s.text == ".name"));
        // Should NOT have [].field
        assert!(!suggestions.iter().any(|s| s.text == "[].id"));
    }

    #[test]
    fn test_prefix_filtering_with_bracket_suggestions() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"items": [{"id": 1, "name": "Item1", "note": "test"}]}"#;
        analyzer.analyze(json).unwrap();

        // .items. with prefix "n" should suggest [].name, [].note (no standalone [])
        let suggestions = analyzer.get_contextual_field_suggestions(".items", "n");
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == "[].name"));
        assert!(suggestions.iter().any(|s| s.text == "[].note"));
        // Should NOT have [].id (doesn't match prefix)
        assert!(!suggestions.iter().any(|s| s.text == "[].id"));
        // Should NOT have standalone [] when prefix is typed
        assert!(!suggestions.iter().any(|s| s.text == "[]"));
    }

    #[test]
    fn test_prefix_filtering_with_pipe_bracket_suggestions() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"items": [{"id": 1, "name": "Item1", "note": "test"}]}"#;
        analyzer.analyze(json).unwrap();

        // .items | . with prefix "n" should suggest .[].name, .[].note (with dot prefix)
        let suggestions = analyzer.get_contextual_field_suggestions(".items |", "n");
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.text == ".[].name"));
        assert!(suggestions.iter().any(|s| s.text == ".[].note"));
        // Should NOT have .[].id (doesn't match prefix)
        assert!(!suggestions.iter().any(|s| s.text == ".[].id"));
        // Should NOT have standalone .[] when prefix is typed
        assert!(!suggestions.iter().any(|s| s.text == ".[]"));
    }

    #[test]
    fn test_root_level_array_without_brackets() {
        let mut analyzer = JsonAnalyzer::new();
        // JSON is a root-level array
        let json = r#"[{"id": 1, "name": "Item1"}, {"id": 2, "name": "Item2"}]"#;
        analyzer.analyze(json).unwrap();

        // At root level with array, typing "." should suggest .[], .[].id, .[].name
        // Because user needs to iterate the root array
        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        // Root array should show .[].field suggestions (with dot since we're at root)
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
        assert!(suggestions.iter().any(|s| s.text == ".[].id"));
        assert!(suggestions.iter().any(|s| s.text == ".[].name"));
    }

    #[test]
    fn test_root_level_array_with_brackets() {
        let mut analyzer = JsonAnalyzer::new();
        // JSON is a root-level array
        let json = r#"[{"id": 1, "name": "Item1"}, {"id": 2, "name": "Item2"}]"#;
        analyzer.analyze(json).unwrap();

        // After typing .[], should suggest .id, .name
        let suggestions = analyzer.get_contextual_field_suggestions(".[]", "");
        assert!(suggestions.iter().any(|s| s.text == ".id"));
        assert!(suggestions.iter().any(|s| s.text == ".name"));
    }

    #[test]
    fn test_root_level_array_with_index() {
        let mut analyzer = JsonAnalyzer::new();
        // JSON is a root-level array
        let json = r#"[{"id": 1, "name": "Item1"}, {"id": 2, "name": "Item2"}]"#;
        analyzer.analyze(json).unwrap();

        // After typing .[0], should suggest .id, .name
        let suggestions = analyzer.get_contextual_field_suggestions(".[0]", "");
        assert!(suggestions.iter().any(|s| s.text == ".id"));
        assert!(suggestions.iter().any(|s| s.text == ".name"));
    }

    #[test]
    fn test_root_level_array_with_prefix_filter() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"[{"id": 1, "name": "Item1", "notes": "test"}]"#;
        analyzer.analyze(json).unwrap();

        // Typing "n" as prefix should filter to fields starting with "n"
        let suggestions = analyzer.get_contextual_field_suggestions("", "n");
        assert!(suggestions.iter().any(|s| s.text == ".[].name"));
        assert!(suggestions.iter().any(|s| s.text == ".[].notes"));
        // Should NOT include .[].id (doesn't start with "n")
        assert!(!suggestions.iter().any(|s| s.text == ".[].id"));
        // Should NOT include standalone .[] when prefix is typed
        assert!(!suggestions.iter().any(|s| s.text == ".[]"));
    }

    #[test]
    fn test_root_level_empty_array() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"[]"#;
        analyzer.analyze(json).unwrap();

        // Empty array has no elements to analyze, should return empty
        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_root_level_array_of_primitives() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"[1, 2, 3, 4, 5]"#;
        analyzer.analyze(json).unwrap();

        // Array of primitives - should suggest .[] but no field suggestions
        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
    }

    #[test]
    fn test_root_level_array_of_strings() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"["apple", "banana", "cherry"]"#;
        analyzer.analyze(json).unwrap();

        // Array of strings - should suggest .[] but no field suggestions
        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
    }

    #[test]
    fn test_root_level_nested_array() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"[[1, 2], [3, 4], [5, 6]]"#;
        analyzer.analyze(json).unwrap();

        // Array of arrays - should suggest .[] for outer array
        let suggestions = analyzer.get_contextual_field_suggestions("", "");
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions.iter().any(|s| s.text == ".[]"));
    }
}
