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

    /// Analyze pre-parsed JSON value for field suggestions
    ///
    /// This is the optimized path that avoids re-parsing on every keystroke.
    /// Critical for large files (127MB+) where parsing takes 50-100ms.
    pub fn analyze_parsed_result(
        value: &Arc<Value>,
        result_type: ResultType,
        needs_leading_dot: bool,
    ) -> Vec<Suggestion> {
        Self::extract_suggestions_for_type(value, result_type, needs_leading_dot)
    }

    /// Analyze result string by parsing it first (legacy path for compatibility)
    ///
    /// WARNING: This parses JSON on every call. Prefer analyze_parsed_result() for performance.
    pub fn analyze_result(
        result: &str,
        result_type: ResultType,
        needs_leading_dot: bool,
    ) -> Vec<Suggestion> {
        if result.trim().is_empty() {
            return Vec::new();
        }

        let value = match Self::parse_first_json_value(result) {
            Some(v) => v,
            None => return Vec::new(),
        };

        Self::extract_suggestions_for_type(&value, result_type, needs_leading_dot)
    }

    fn parse_first_json_value(text: &str) -> Option<Value> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }

        // Try to parse the entire text first (common case: single value)
        if let Ok(value) = serde_json::from_str(text) {
            return Some(value);
        }

        let mut deserializer = serde_json::Deserializer::from_str(text).into_iter();
        if let Some(Ok(value)) = deserializer.next() {
            return Some(value);
        }

        None
    }

    fn extract_suggestions_for_type(
        value: &Value,
        result_type: ResultType,
        needs_leading_dot: bool,
    ) -> Vec<Suggestion> {
        match result_type {
            ResultType::ArrayOfObjects => {
                let prefix = dot_prefix(needs_leading_dot);
                let mut suggestions = vec![Suggestion::new_with_type(
                    format!("{}[]", prefix),
                    SuggestionType::Pattern,
                    None,
                )];

                if let Value::Array(arr) = value
                    && let Some(Value::Object(map)) = arr.first()
                {
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
