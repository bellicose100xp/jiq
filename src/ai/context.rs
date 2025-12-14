//! Query context building
//!
//! Builds context from app state for AI requests including query, cursor position,
//! error messages, and JSON structure information.

use crate::stats::parser::StatsParser;
use crate::stats::types::ResultStats;

/// Maximum length for JSON sample in context (characters)
pub const MAX_JSON_SAMPLE_LENGTH: usize = 1000;

/// Information about the JSON structure for AI context
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonTypeInfo {
    /// Root type of the JSON (e.g., "Object", "Array", "String")
    pub root_type: String,
    /// Element type for arrays (e.g., "objects", "strings")
    pub element_type: Option<String>,
    /// Number of elements for arrays/streams
    pub element_count: Option<usize>,
    /// Top-level keys for objects
    pub top_level_keys: Vec<String>,
    /// Human-readable schema hint
    pub schema_hint: String,
}

impl Default for JsonTypeInfo {
    fn default() -> Self {
        Self {
            root_type: "Unknown".to_string(),
            element_type: None,
            element_count: None,
            top_level_keys: Vec::new(),
            schema_hint: String::new(),
        }
    }
}

impl JsonTypeInfo {
    /// Build JsonTypeInfo from a JSON string using the stats parser
    pub fn from_json(json: &str) -> Self {
        let stats = StatsParser::parse(json);
        let mut info = Self::default();

        match &stats {
            ResultStats::Array {
                count,
                element_type,
            } => {
                info.root_type = "Array".to_string();
                info.element_count = Some(*count);
                info.element_type = Some(element_type.to_string());
                info.schema_hint = format!("Array of {} {}", count, element_type);
            }
            ResultStats::Object => {
                info.root_type = "Object".to_string();
                info.top_level_keys = Self::extract_top_level_keys(json);
                let key_preview = if info.top_level_keys.len() > 5 {
                    format!(
                        "{}, ... ({} more)",
                        info.top_level_keys[..5].join(", "),
                        info.top_level_keys.len() - 5
                    )
                } else {
                    info.top_level_keys.join(", ")
                };
                info.schema_hint = format!("Object with keys: {}", key_preview);
            }
            ResultStats::String => {
                info.root_type = "String".to_string();
                info.schema_hint = "String value".to_string();
            }
            ResultStats::Number => {
                info.root_type = "Number".to_string();
                info.schema_hint = "Number value".to_string();
            }
            ResultStats::Boolean => {
                info.root_type = "Boolean".to_string();
                info.schema_hint = "Boolean value".to_string();
            }
            ResultStats::Null => {
                info.root_type = "Null".to_string();
                info.schema_hint = "Null value".to_string();
            }
            ResultStats::Stream { count } => {
                info.root_type = "Stream".to_string();
                info.element_count = Some(*count);
                info.schema_hint = format!("Stream of {} values", count);
            }
        }

        info
    }

    /// Extract top-level keys from a JSON object string
    fn extract_top_level_keys(json: &str) -> Vec<String> {
        let mut keys = Vec::new();
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut key_start: Option<usize> = None;
        let chars: Vec<char> = json.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' && in_string {
                escape_next = true;
                continue;
            }

            if ch == '"' {
                if !in_string && depth == 1 && key_start.is_none() {
                    // Starting a potential key at depth 1
                    key_start = Some(i + 1);
                } else if in_string && key_start.is_some() {
                    // Ending a string - check if it's followed by ':'
                    let key_end = i;
                    // Look ahead for ':'
                    let rest = &chars[i + 1..];
                    let has_colon = rest.iter().take_while(|c| c.is_whitespace()).count()
                        < rest.len()
                        && rest.iter().skip_while(|c| c.is_whitespace()).next() == Some(&':');
                    if has_colon {
                        if let Some(start) = key_start {
                            let key: String = chars[start..key_end].iter().collect();
                            keys.push(key);
                        }
                    }
                    key_start = None;
                }
                in_string = !in_string;
                continue;
            }

            if in_string {
                continue;
            }

            match ch {
                '{' | '[' => depth += 1,
                '}' | ']' => depth -= 1,
                _ => {}
            }
        }

        keys
    }
}

/// Context for AI queries built from app state
#[derive(Debug, Clone)]
pub struct QueryContext {
    /// Current query text
    pub query: String,
    /// Cursor position in the query
    pub cursor_pos: usize,
    /// Truncated sample of input JSON (max 1000 chars)
    pub input_sample: String,
    /// Current output (if successful)
    pub output: Option<String>,
    /// Truncated sample of output JSON (max 1000 chars) for successful queries
    pub output_sample: Option<String>,
    /// Error message (if query failed)
    pub error: Option<String>,
    /// JSON structure information
    pub json_type_info: JsonTypeInfo,
    /// Whether the query executed successfully
    pub is_success: bool,
}

impl QueryContext {
    /// Create a new QueryContext
    pub fn new(
        query: String,
        cursor_pos: usize,
        json_input: &str,
        output: Option<String>,
        error: Option<String>,
    ) -> Self {
        let input_sample = truncate_json(json_input, MAX_JSON_SAMPLE_LENGTH);
        let json_type_info = JsonTypeInfo::from_json(json_input);
        let is_success = error.is_none();
        let output_sample = output
            .as_ref()
            .map(|o| truncate_json(o, MAX_JSON_SAMPLE_LENGTH));

        Self {
            query,
            cursor_pos,
            input_sample,
            output,
            output_sample,
            error,
            json_type_info,
            is_success,
        }
    }

    /// Check if context has all required fields populated
    // TODO: Remove #[allow(dead_code)] when is_complete is used
    #[allow(dead_code)] // Phase 1: Reserved for future validation
    pub fn is_complete(&self) -> bool {
        !self.query.is_empty() && !self.input_sample.is_empty()
    }
}

/// Truncate JSON to a maximum length, trying to preserve valid structure
pub fn truncate_json(json: &str, max_len: usize) -> String {
    if json.len() <= max_len {
        return json.to_string();
    }

    // Simple truncation with ellipsis indicator
    let truncated = &json[..max_len];
    format!("{}... [truncated]", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // =========================================================================
    // Unit Tests
    // =========================================================================

    #[test]
    fn test_json_type_info_from_object() {
        let json = r#"{"name": "test", "age": 30, "active": true}"#;
        let info = JsonTypeInfo::from_json(json);

        assert_eq!(info.root_type, "Object");
        assert!(info.top_level_keys.contains(&"name".to_string()));
        assert!(info.top_level_keys.contains(&"age".to_string()));
        assert!(info.top_level_keys.contains(&"active".to_string()));
        assert!(info.schema_hint.contains("Object with keys"));
    }

    #[test]
    fn test_json_type_info_from_array_of_objects() {
        let json = r#"[{"id": 1}, {"id": 2}]"#;
        let info = JsonTypeInfo::from_json(json);

        assert_eq!(info.root_type, "Array");
        assert_eq!(info.element_count, Some(2));
        assert_eq!(info.element_type, Some("objects".to_string()));
    }

    #[test]
    fn test_json_type_info_from_array_of_numbers() {
        let json = "[1, 2, 3, 4, 5]";
        let info = JsonTypeInfo::from_json(json);

        assert_eq!(info.root_type, "Array");
        assert_eq!(info.element_count, Some(5));
        assert_eq!(info.element_type, Some("numbers".to_string()));
    }

    #[test]
    fn test_json_type_info_from_string() {
        let json = r#""hello world""#;
        let info = JsonTypeInfo::from_json(json);

        assert_eq!(info.root_type, "String");
        assert_eq!(info.schema_hint, "String value");
    }

    #[test]
    fn test_json_type_info_from_number() {
        let json = "42";
        let info = JsonTypeInfo::from_json(json);

        assert_eq!(info.root_type, "Number");
    }

    #[test]
    fn test_json_type_info_from_boolean() {
        let json = "true";
        let info = JsonTypeInfo::from_json(json);

        assert_eq!(info.root_type, "Boolean");
    }

    #[test]
    fn test_json_type_info_from_null() {
        let json = "null";
        let info = JsonTypeInfo::from_json(json);

        assert_eq!(info.root_type, "Null");
    }

    #[test]
    fn test_truncate_json_short() {
        let json = r#"{"short": true}"#;
        let truncated = truncate_json(json, 1000);
        assert_eq!(truncated, json);
    }

    #[test]
    fn test_truncate_json_long() {
        let json = "x".repeat(2000);
        let truncated = truncate_json(&json, 1000);
        assert!(truncated.len() < json.len());
        assert!(truncated.ends_with("... [truncated]"));
    }

    #[test]
    fn test_query_context_new() {
        let query = ".name".to_string();
        let json = r#"{"name": "test"}"#;
        let ctx = QueryContext::new(query.clone(), 5, json, None, Some("error".to_string()));

        assert_eq!(ctx.query, query);
        assert_eq!(ctx.cursor_pos, 5);
        assert_eq!(ctx.error, Some("error".to_string()));
        assert!(ctx.is_complete());
    }

    #[test]
    fn test_query_context_is_complete() {
        let ctx = QueryContext::new(".".to_string(), 1, "{}", None, None);
        assert!(ctx.is_complete());

        let ctx_empty_query = QueryContext {
            query: String::new(),
            cursor_pos: 0,
            input_sample: "{}".to_string(),
            output: None,
            output_sample: None,
            error: None,
            json_type_info: JsonTypeInfo::default(),
            is_success: true,
        };
        assert!(!ctx_empty_query.is_complete());
    }

    #[test]
    fn test_extract_top_level_keys_simple() {
        let json = r#"{"a": 1, "b": 2, "c": 3}"#;
        let keys = JsonTypeInfo::extract_top_level_keys(json);
        assert_eq!(keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_extract_top_level_keys_nested() {
        let json = r#"{"outer": {"inner": 1}, "other": 2}"#;
        let keys = JsonTypeInfo::extract_top_level_keys(json);
        assert_eq!(keys, vec!["outer", "other"]);
    }

    #[test]
    fn test_extract_top_level_keys_with_array() {
        let json = r#"{"items": [1, 2, 3], "count": 3}"#;
        let keys = JsonTypeInfo::extract_top_level_keys(json);
        assert_eq!(keys, vec!["items", "count"]);
    }

    // =========================================================================
    // Property-Based Tests
    // =========================================================================

    // **Feature: ai-assistant, Property 15: Context completeness**
    // *For any* app state, the built QueryContext should include: query text,
    // cursor position, error (if any), truncated JSON sample (≤1000 chars),
    // and JSON type info.
    // **Validates: Requirements 7.1, 7.2, 7.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_context_completeness(
            query in "[.a-zA-Z0-9_]{1,50}",
            cursor_pos in 0usize..50,
            json_content in "[a-zA-Z0-9]{0,100}",
            has_error in proptest::bool::ANY,
            error_msg in "[a-zA-Z ]{0,50}"
        ) {
            let json = format!(r#"{{"data": "{}"}}"#, json_content);
            let error = if has_error { Some(error_msg) } else { None };

            let ctx = QueryContext::new(
                query.clone(),
                cursor_pos.min(query.len()),
                &json,
                None,
                error.clone(),
            );

            // Verify all required fields are present
            prop_assert_eq!(&ctx.query, &query, "Query text should match");
            prop_assert!(ctx.cursor_pos <= query.len(), "Cursor should be valid");
            prop_assert!(!ctx.input_sample.is_empty(), "Input sample should exist");
            prop_assert_eq!(&ctx.error, &error, "Error should match");
            prop_assert!(!ctx.json_type_info.root_type.is_empty(), "Type info should exist");
        }
    }

    // **Feature: ai-assistant, Property 16: JSON truncation bound**
    // *For any* JSON input string, the truncated sample in QueryContext should
    // have length ≤ 1000 characters.
    // **Validates: Requirements 7.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_json_truncation_bound(
            json_size in 0usize..5000
        ) {
            // Generate JSON of specified size
            let json = format!(r#"{{"data": "{}"}}"#, "x".repeat(json_size));

            let truncated = truncate_json(&json, MAX_JSON_SAMPLE_LENGTH);

            // Truncated length should be bounded
            // Note: truncated string includes "... [truncated]" suffix (15 chars)
            let max_with_suffix = MAX_JSON_SAMPLE_LENGTH + 15;
            prop_assert!(
                truncated.len() <= max_with_suffix,
                "Truncated JSON length {} exceeds max {} for input size {}",
                truncated.len(),
                max_with_suffix,
                json.len()
            );

            // If original was short enough, should be unchanged
            if json.len() <= MAX_JSON_SAMPLE_LENGTH {
                prop_assert_eq!(
                    truncated, json,
                    "Short JSON should not be truncated"
                );
            } else {
                prop_assert!(
                    truncated.ends_with("... [truncated]"),
                    "Long JSON should have truncation marker"
                );
            }
        }
    }

    // **Feature: ai-assistant, Property 19: AI request on error includes error context**
    // *For any* query execution that produces an error, the AI request context
    // should include the error message.
    // **Validates: Requirements 3.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_error_context_includes_error_message(
            query in "[.a-zA-Z0-9_]{1,50}",
            cursor_pos in 0usize..50,
            json_content in "[a-zA-Z0-9]{0,100}",
            error_msg in "[a-zA-Z ]{1,100}"  // Non-empty error message
        ) {
            let json = format!(r#"{{"data": "{}"}}"#, json_content);

            // Create context with error (simulating failed query)
            let ctx = QueryContext::new(
                query.clone(),
                cursor_pos.min(query.len()),
                &json,
                None,  // No output on error
                Some(error_msg.clone()),
            );

            // Verify error context is properly set
            prop_assert!(!ctx.is_success, "Context should indicate failure");
            prop_assert!(ctx.error.is_some(), "Error should be present");
            prop_assert_eq!(
                ctx.error.as_ref().unwrap(),
                &error_msg,
                "Error message should match"
            );
            prop_assert!(ctx.output.is_none(), "Output should be None on error");
            prop_assert!(ctx.output_sample.is_none(), "Output sample should be None on error");
        }
    }

    // **Feature: ai-assistant, Property 20: AI request on success includes output context**
    // *For any* query execution that succeeds, the AI request context should
    // include the query output (truncated if needed).
    // **Validates: Requirements 3.4**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_success_context_includes_output(
            query in "[.a-zA-Z0-9_]{1,50}",
            cursor_pos in 0usize..50,
            json_content in "[a-zA-Z0-9]{0,100}",
            output_content in "[a-zA-Z0-9 ]{1,200}"  // Non-empty output
        ) {
            let json = format!(r#"{{"data": "{}"}}"#, json_content);
            let output = format!(r#""{}""#, output_content);

            // Create context with output (simulating successful query)
            let ctx = QueryContext::new(
                query.clone(),
                cursor_pos.min(query.len()),
                &json,
                Some(output.clone()),
                None,  // No error on success
            );

            // Verify success context is properly set
            prop_assert!(ctx.is_success, "Context should indicate success");
            prop_assert!(ctx.error.is_none(), "Error should be None on success");
            prop_assert!(ctx.output.is_some(), "Output should be present");
            prop_assert_eq!(
                ctx.output.as_ref().unwrap(),
                &output,
                "Output should match"
            );
            prop_assert!(ctx.output_sample.is_some(), "Output sample should be present");
        }

        #[test]
        fn prop_success_context_output_sample_bounded(
            query in "[.a-zA-Z0-9_]{1,50}",
            output_size in 0usize..3000
        ) {
            let json = r#"{"data": "test"}"#;
            let output = "x".repeat(output_size);

            // Create context with potentially large output
            let ctx = QueryContext::new(
                query.clone(),
                0,
                json,
                Some(output.clone()),
                None,
            );

            // Verify output_sample is bounded
            if let Some(ref sample) = ctx.output_sample {
                let max_with_suffix = MAX_JSON_SAMPLE_LENGTH + 15;
                prop_assert!(
                    sample.len() <= max_with_suffix,
                    "Output sample length {} exceeds max {} for output size {}",
                    sample.len(),
                    max_with_suffix,
                    output.len()
                );
            }
        }
    }
}
