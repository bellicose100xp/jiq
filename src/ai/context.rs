//! Query context building
//!
//! Builds context from app state for AI requests including query, cursor position,
//! error messages, and JSON structure information.

use crate::stats::parser::StatsParser;
use crate::stats::types::ResultStats;

/// Maximum length for JSON sample in context (characters)
pub const MAX_JSON_SAMPLE_LENGTH: usize = 25_000;

/// Minification threshold ratio relative to MAX_JSON_SAMPLE_LENGTH
pub const MINIFY_THRESHOLD_RATIO: usize = 10;

/// Additional context parameters for AI queries
#[derive(Debug, Clone)]
pub struct ContextParams<'a> {
    /// Pre-computed JSON schema
    pub input_schema: Option<&'a str>,
    /// Last successful query (for error context)
    pub base_query: Option<&'a str>,
    /// Result of last successful query (for error context)
    pub base_query_result: Option<&'a str>,
    /// Whether current result is empty/null (from QueryState)
    pub is_empty_result: bool,
}

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
                        && rest.iter().find(|c| !c.is_whitespace()) == Some(&':');
                    if has_colon && let Some(start) = key_start {
                        let key: String = chars[start..key_end].iter().collect();
                        keys.push(key);
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
    /// Truncated sample of input JSON (max 25000 chars)
    pub input_sample: String,
    /// Current output (if successful)
    pub output: Option<String>,
    /// Truncated sample of output JSON (max 25000 chars) for successful queries
    pub output_sample: Option<String>,
    /// Error message (if query failed)
    pub error: Option<String>,
    /// JSON structure information
    pub json_type_info: JsonTypeInfo,
    /// Whether the query executed successfully
    pub is_success: bool,
    /// Whether current result is empty/null (valid query but no results)
    pub is_empty_result: bool,
    /// Truncated JSON schema (pre-computed, max 25000 chars)
    pub input_schema: Option<String>,
    /// Query that produced displayed result (error case, or success with empty/null result)
    pub base_query: Option<String>,
    /// Output of base_query (truncated to max 25000 chars)
    pub base_query_result: Option<String>,
}

impl QueryContext {
    /// Create a new QueryContext
    pub fn new(
        query: String,
        cursor_pos: usize,
        json_input: &str,
        output: Option<String>,
        error: Option<String>,
        params: ContextParams,
    ) -> Self {
        let input_sample = prepare_json_for_context(json_input, MAX_JSON_SAMPLE_LENGTH);
        let json_type_info = JsonTypeInfo::from_json(json_input);
        let is_success = error.is_none();
        let output_sample = output
            .as_ref()
            .map(|o| prepare_json_for_context(o, MAX_JSON_SAMPLE_LENGTH));

        let base_query_result = params
            .base_query_result
            .map(|r| prepare_json_for_context(r, MAX_JSON_SAMPLE_LENGTH));

        Self {
            query,
            cursor_pos,
            input_sample,
            output,
            output_sample,
            error,
            json_type_info,
            is_success,
            is_empty_result: params.is_empty_result,
            input_schema: params
                .input_schema
                .map(|s| prepare_json_for_context(s, MAX_JSON_SAMPLE_LENGTH)),
            base_query: params.base_query.map(|s| s.to_string()),
            base_query_result,
        }
    }

    /// Check if context has all required fields populated
    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        !self.query.is_empty() && !self.input_sample.is_empty()
    }
}

/// Attempt to minify JSON by parsing and re-serializing without whitespace
fn try_minify_json(json: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| serde_json::to_string(&v).ok())
}

/// Prepare JSON for context with smart minification and truncation
///
/// Logic:
/// 1. If size <= max_len: return as-is (no processing needed)
/// 2. If size <= max_len * MINIFY_THRESHOLD_RATIO: try minify, then truncate if needed
/// 3. If size > threshold OR minification fails: just truncate
pub fn prepare_json_for_context(json: &str, max_len: usize) -> String {
    let original_len = json.len();

    if original_len <= max_len {
        return json.to_string();
    }

    let minify_threshold = max_len * MINIFY_THRESHOLD_RATIO;

    if original_len <= minify_threshold
        && let Some(minified) = try_minify_json(json)
    {
        return truncate_json(&minified, max_len);
    }

    truncate_json(json, max_len)
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
#[path = "context_tests.rs"]
mod context_tests;
