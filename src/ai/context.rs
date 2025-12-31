//! Query context building
//!
//! Builds context from app state for AI requests including query, cursor position,
//! error messages, and JSON structure information.

/// Maximum length for JSON sample in context (100KB characters)
pub const MAX_JSON_SAMPLE_LENGTH: usize = 100_000;

/// Maximum input size for attempting minification (5MB)
/// Files larger than this skip minification to bound parse cost
/// At 300MB/s parse speed, 5MB = ~17ms - imperceptible after 150ms debounce
pub const MINIFY_SIZE_LIMIT: usize = 5_000_000;

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

/// Context for AI queries built from app state
#[derive(Debug, Clone)]
pub struct QueryContext {
    /// Current query text
    pub query: String,
    /// Cursor position in the query
    pub cursor_pos: usize,
    /// Truncated sample of output JSON (max 25000 chars) for successful queries
    pub output_sample: Option<String>,
    /// Error message (if query failed)
    pub error: Option<String>,
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
        output: Option<String>,
        error: Option<String>,
        params: ContextParams,
        max_context_length: usize,
    ) -> Self {
        let is_success = error.is_none();

        // Use is_empty_result to determine if output should be shown
        // This flag correctly identifies empty output or output consisting entirely of nulls
        let output_sample = if params.is_empty_result {
            None
        } else {
            output
                .as_ref()
                .map(|o| prepare_json_for_context(o, max_context_length))
        };

        // base_query_result is now already processed
        let base_query_result = params.base_query_result.map(|s| s.to_string());

        Self {
            query,
            cursor_pos,
            output_sample,
            error,
            is_success,
            is_empty_result: params.is_empty_result,
            input_schema: params.input_schema.map(|s| s.to_string()),
            base_query: params.base_query.map(|s| s.to_string()),
            base_query_result,
        }
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
/// 1. For files under 5MB: always minify for consistent dense output
/// 2. For files >= 5MB: skip minification to bound parse cost
/// 3. Truncate result if needed to max_len
pub fn prepare_json_for_context(json: &str, max_len: usize) -> String {
    let content = if json.len() <= MINIFY_SIZE_LIMIT {
        try_minify_json(json).unwrap_or_else(|| json.to_string())
    } else {
        json.to_string()
    };

    truncate_json(&content, max_len)
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

/// Prepare schema for AI context
///
/// Schema is already minified from serde_json::to_string() in extract_json_schema.
/// This function just truncates to max length. Called once on file load since schema
/// never changes during the session.
pub fn prepare_schema_for_context(schema: &str, max_context_length: usize) -> String {
    truncate_json(schema, max_context_length)
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod context_tests;
