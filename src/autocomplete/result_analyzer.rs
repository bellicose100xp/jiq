use crate::autocomplete::autocomplete_state::{JsonFieldType, Suggestion, SuggestionType};
use crate::query::ResultType;
use serde_json::Value;
use std::sync::Arc;

pub struct ResultAnalyzer;

#[inline]
fn dot_prefix(needs_leading_dot: bool) -> &'static str {
    if needs_leading_dot { "." } else { "" }
}

impl ResultAnalyzer {
    /// Check if a field name can use jq's simple dot syntax (e.g., .foo)
    /// According to jq manual: "The .foo syntax only works for simple, identifier-like keys,
    /// that is, keys that are all made of alphanumeric characters and underscore,
    /// and which do not start with a digit."
    /// (https://jqlang.org/manual/#object-identifier-index)
    /// Names that don't fit require quoted access: ."field-name"
    fn is_simple_jq_identifier(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let first_char = name.chars().next().unwrap();
        !first_char.is_numeric() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Format a field name for jq syntax, quoting if it doesn't fit simple dot syntax
    fn format_field_name(prefix: &str, name: &str) -> String {
        if Self::is_simple_jq_identifier(name) {
            format!("{}{}", prefix, name)
        } else {
            format!("{}\"{}\"", prefix, name)
        }
    }
    fn extract_object_fields(
        map: &serde_json::Map<String, Value>,
        prefix: &str,
        suggestions: &mut Vec<Suggestion>,
    ) {
        for (key, val) in map {
            let field_type = Self::detect_json_type(val);
            let field_text = Self::format_field_name(prefix, key);
            suggestions.push(Suggestion::new_with_type(
                field_text,
                SuggestionType::Field,
                Some(field_type),
            ));
        }
    }

    /// Analyze a JSON value for field suggestions, inferring type from the value itself.
    ///
    /// Unlike `analyze_parsed_result`, this method does not require an external `ResultType`.
    /// It infers the appropriate suggestion strategy directly from the JSON structure.
    /// This is essential for nested navigation where the original `ResultType` doesn't
    /// apply to navigated sub-values.
    ///
    /// # Parameters
    /// - `value`: The JSON value to analyze (can be a navigated nested value)
    /// - `needs_leading_dot`: Whether suggestions should include leading dot
    /// - `suppress_array_brackets`: Whether to suppress .[] suggestions
    pub fn analyze_value(
        value: &Value,
        needs_leading_dot: bool,
        suppress_array_brackets: bool,
    ) -> Vec<Suggestion> {
        let prefix = dot_prefix(needs_leading_dot);

        match value {
            Value::Object(map) => {
                let mut suggestions = Vec::new();
                Self::extract_object_fields(map, prefix, &mut suggestions);
                suggestions
            }
            Value::Array(arr) => {
                let mut suggestions = Vec::new();

                // Only suggest .[] when not suppressing array brackets
                if !suppress_array_brackets {
                    suggestions.push(Suggestion::new_with_type(
                        format!("{}[]", prefix),
                        SuggestionType::Pattern,
                        None,
                    ));
                }

                // If array contains objects, suggest their fields
                if let Some(Value::Object(map)) = arr.first() {
                    for (key, val) in map {
                        let field_type = Self::detect_json_type(val);
                        let field_text = if suppress_array_brackets {
                            Self::format_field_name(prefix, key)
                        } else {
                            // For array iteration syntax, quote non-simple identifiers
                            if Self::is_simple_jq_identifier(key) {
                                // Simple identifier: .[].fieldname
                                format!("{}[].{}", prefix, key)
                            } else {
                                // Non-simple identifier (starts with digit, contains special chars): .[]."fieldname"
                                format!("{}[].\"{}\"", prefix, key)
                            }
                        };
                        suggestions.push(Suggestion::new_with_type(
                            field_text,
                            SuggestionType::Field,
                            Some(field_type),
                        ));
                    }
                }

                suggestions
            }
            // Scalars (null, bool, number, string) have no field suggestions
            _ => Vec::new(),
        }
    }

    /// Analyze pre-parsed JSON value for field suggestions
    ///
    /// Optimized path that avoids re-parsing on every keystroke.
    /// Critical for large files where parsing takes 50-100ms.
    ///
    /// When `suppress_array_brackets` is true, field suggestions for arrays will
    /// omit the `[]` prefix. This applies when:
    /// - Inside element-context functions (map, select, etc.)
    /// - Inside object construction {.field}
    pub fn analyze_parsed_result(
        value: &Arc<Value>,
        result_type: ResultType,
        needs_leading_dot: bool,
        suppress_array_brackets: bool,
    ) -> Vec<Suggestion> {
        Self::extract_suggestions_for_type(
            value,
            result_type,
            needs_leading_dot,
            suppress_array_brackets,
        )
    }

    fn extract_suggestions_for_type(
        value: &Value,
        result_type: ResultType,
        needs_leading_dot: bool,
        suppress_array_brackets: bool,
    ) -> Vec<Suggestion> {
        match result_type {
            ResultType::ArrayOfObjects => {
                let prefix = dot_prefix(needs_leading_dot);
                let mut suggestions = Vec::new();

                // Only suggest .[] when not suppressing array brackets
                if !suppress_array_brackets {
                    suggestions.push(Suggestion::new_with_type(
                        format!("{}[]", prefix),
                        SuggestionType::Pattern,
                        None,
                    ));
                }

                if let Value::Array(arr) = value
                    && let Some(Value::Object(map)) = arr.first()
                {
                    for (key, val) in map {
                        let field_type = Self::detect_json_type(val);
                        // When suppressing brackets, suggest ".field"
                        // Otherwise, suggest ".[].field" with quoting if needed
                        let field_text = if suppress_array_brackets {
                            Self::format_field_name(prefix, key)
                        } else {
                            // Apply quoting logic for array iteration
                            if Self::is_simple_jq_identifier(key) {
                                format!("{}[].{}", prefix, key)
                            } else {
                                format!("{}[].\"{}\"", prefix, key)
                            }
                        };
                        suggestions.push(Suggestion::new_with_type(
                            field_text,
                            SuggestionType::Field,
                            Some(field_type),
                        ));
                    }
                }

                suggestions
            }
            ResultType::DestructuredObjects => {
                let prefix = dot_prefix(needs_leading_dot);
                let mut suggestions = Vec::new();

                if let Value::Object(map) = value {
                    Self::extract_object_fields(map, prefix, &mut suggestions);
                }

                suggestions
            }
            ResultType::Object => {
                let prefix = dot_prefix(needs_leading_dot);
                let mut suggestions = Vec::new();

                if let Value::Object(map) = value {
                    Self::extract_object_fields(map, prefix, &mut suggestions);
                }

                suggestions
            }
            ResultType::Array => {
                let prefix = dot_prefix(needs_leading_dot);
                vec![Suggestion::new_with_type(
                    format!("{}[]", prefix),
                    SuggestionType::Pattern,
                    None,
                )]
            }
            _ => Vec::new(),
        }
    }

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
#[path = "result_analyzer_tests.rs"]
mod result_analyzer_tests;
