use super::state::{Suggestion, SuggestionType};
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
        self.extract_fields_from_value(value_at_path, prefix)
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
                    .keys()
                    .filter(|k| {
                        prefix.is_empty()
                            || k.to_lowercase().starts_with(&prefix.to_lowercase())
                    })
                    .map(|k| Suggestion::new(format!(".{}", k), SuggestionType::Field))
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

        // Remove leading dot if present
        let path = path.strip_prefix('.').unwrap_or(path);

        // Split path by dots and navigate
        let mut current = root;
        for segment in path.split('.').filter(|s| !s.is_empty()) {
            // Handle array access like "items[]" or "items[0]"
            let (field_name, is_array) = if let Some(idx) = segment.find('[') {
                (&segment[..idx], true)
            } else {
                (segment, false)
            };

            // Navigate to the field
            current = match current {
                Value::Object(map) => map.get(field_name)?,
                _ => return None,
            };

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

    /// Extract fields from a specific JSON value
    fn extract_fields_from_value(&self, value: &Value, prefix: &str) -> Vec<Suggestion> {
        match value {
            Value::Object(map) => {
                let mut fields: Vec<_> = map
                    .keys()
                    .filter(|k| {
                        prefix.is_empty()
                            || k.to_lowercase().starts_with(&prefix.to_lowercase())
                    })
                    .map(|k| Suggestion::new(format!(".{}", k), SuggestionType::Field))
                    .collect();
                fields.sort_by(|a, b| a.text.cmp(&b.text));
                fields
            }
            Value::Array(arr) => {
                // For arrays, analyze the first element to get available fields
                if let Some(first) = arr.first() {
                    self.extract_fields_from_value(first, prefix)
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(), // Primitives have no fields
        }
    }

    /// Get all field names (used in tests)
    #[cfg(test)]
    pub fn get_all_fields(&self) -> Vec<String> {
        let mut fields: Vec<_> = self.field_names.iter().cloned().collect();
        fields.sort();
        fields
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
}
