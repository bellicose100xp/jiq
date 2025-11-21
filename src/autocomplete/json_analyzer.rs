use super::state::{Suggestion, SuggestionType};
use serde_json::Value;
use std::collections::HashSet;

/// Analyze JSON structure and extract field names
pub struct JsonAnalyzer {
    /// All unique field names found in the JSON
    field_names: HashSet<String>,
}

impl JsonAnalyzer {
    pub fn new() -> Self {
        Self {
            field_names: HashSet::new(),
        }
    }

    /// Analyze JSON value and extract all field names
    pub fn analyze(&mut self, json: &str) -> Result<(), String> {
        self.field_names.clear();

        let value: Value = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        self.extract_fields(&value);
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

    /// Get field suggestions matching the given prefix
    pub fn get_field_suggestions(&self, prefix: &str) -> Vec<Suggestion> {
        if prefix.is_empty() {
            // Return all fields if no prefix
            let mut fields: Vec<_> = self
                .field_names
                .iter()
                .map(|f| Suggestion::new(format!(".{}", f), SuggestionType::Field))
                .collect();
            fields.sort_by(|a, b| a.text.cmp(&b.text));
            return fields;
        }

        // Filter by prefix (case-insensitive)
        let prefix_lower = prefix.to_lowercase();
        let mut matching: Vec<_> = self
            .field_names
            .iter()
            .filter(|f| f.to_lowercase().starts_with(&prefix_lower))
            .map(|f| Suggestion::new(format!(".{}", f), SuggestionType::Field))
            .collect();

        matching.sort_by(|a, b| a.text.cmp(&b.text));
        matching
    }

    /// Get all field names
    pub fn get_all_fields(&self) -> Vec<String> {
        let mut fields: Vec<_> = self.field_names.iter().cloned().collect();
        fields.sort();
        fields
    }

    /// Check if a field exists
    pub fn has_field(&self, field: &str) -> bool {
        self.field_names.contains(field)
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
    fn test_field_suggestions() {
        let mut analyzer = JsonAnalyzer::new();
        let json = r#"{"name": "John", "nickname": "Johnny", "age": 30}"#;
        analyzer.analyze(json).unwrap();

        let suggestions = analyzer.get_field_suggestions("na");
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, ".name");

        let suggestions = analyzer.get_field_suggestions("n");
        assert_eq!(suggestions.len(), 2);
    }
}
