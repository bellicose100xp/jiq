//! Query Result Preprocessing
//!
//! Functions for preprocessing query results in the worker thread.
//! These operations are expensive and moved here to avoid blocking the main thread.

use std::sync::Arc;

use ansi_to_tui::IntoText;
use ratatui::text::Text;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::types::{ProcessedResult, QueryError, RenderedLine, RenderedSpan};
use crate::query::query_state::ResultType;

/// Preprocess query result by performing all expensive operations
///
/// This includes:
/// - Stripping ANSI codes
/// - Computing line metrics
/// - Parsing ANSI to rendered lines
/// - Parsing JSON for autocomplete
///
/// Checks cancellation token between operations to allow fast cancellation.
pub fn preprocess_result(
    output: String,
    query: &str,
    cancel_token: &CancellationToken,
) -> Result<ProcessedResult, QueryError> {
    // Strip ANSI codes
    if cancel_token.is_cancelled() {
        return Err(QueryError::Cancelled);
    }
    let unformatted = strip_ansi_codes(&output);

    // Compute line metrics
    if cancel_token.is_cancelled() {
        return Err(QueryError::Cancelled);
    }
    let line_count = output.lines().count() as u32;
    let max_width = output
        .lines()
        .map(|l| l.len())
        .max()
        .unwrap_or(0)
        .min(u16::MAX as usize) as u16;

    // Parse ANSI to RenderedLine
    if cancel_token.is_cancelled() {
        return Err(QueryError::Cancelled);
    }
    let rendered_lines = parse_ansi_to_rendered_lines(&output, cancel_token)?;

    // Parse JSON for autocomplete
    if cancel_token.is_cancelled() {
        return Err(QueryError::Cancelled);
    }
    let parsed = parse_first_value(&unformatted).map(Arc::new);

    let result_type = detect_result_type(&unformatted);
    let base_query = normalize_base_query(query);

    Ok(ProcessedResult {
        output: Arc::new(output),
        unformatted: Arc::new(unformatted),
        rendered_lines,
        parsed,
        line_count,
        max_width,
        result_type,
        query: base_query,
    })
}

/// Parse ANSI text into rendered lines
///
/// Converts ANSI escape sequences to styled spans for rendering.
/// Checks cancellation every 10,000 lines for large files.
fn parse_ansi_to_rendered_lines(
    output: &str,
    cancel_token: &CancellationToken,
) -> Result<Vec<RenderedLine>, QueryError> {
    // Check cancellation before starting expensive operation
    if cancel_token.is_cancelled() {
        return Err(QueryError::Cancelled);
    }

    // Parse ANSI codes to Text
    let text: Text = output
        .as_bytes()
        .to_vec()
        .into_text()
        .unwrap_or_else(|_| Text::raw(output.to_string()));

    // Convert Text to Vec<RenderedLine>
    let rendered_lines = text
        .lines
        .into_iter()
        .enumerate()
        .map(|(idx, line)| {
            // Check cancellation every 10,000 lines
            if idx % 10000 == 0 && cancel_token.is_cancelled() {
                return Err(QueryError::Cancelled);
            }

            let spans = line
                .spans
                .into_iter()
                .map(|span| RenderedSpan {
                    content: span.content.to_string(),
                    style: span.style,
                })
                .collect();

            Ok(RenderedLine { spans })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rendered_lines)
}

/// Strip ANSI escape codes from a string
///
/// jq outputs colored results with ANSI codes like:
/// - `\x1b[0m` (reset)
/// - `\x1b[1;39m` (bold)
/// - `\x1b[0;32m` (green)
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Found escape character, skip until 'm' (end of ANSI sequence)
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                for c in chars.by_ref() {
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

/// Parse first JSON value from result text
///
/// Handles both single values and destructured output (multiple JSON values).
/// For destructured results like `{"a":1}\n{"b":2}`, parses just the first value.
fn parse_first_value(text: &str) -> Option<Value> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    // Try to parse the entire text first (common case: single value)
    if let Ok(value) = serde_json::from_str(text) {
        return Some(value);
    }

    // Fallback: use streaming parser to get first value (handles destructured output)
    let mut deserializer = serde_json::Deserializer::from_str(text).into_iter();
    deserializer.next().and_then(|r| r.ok())
}

/// Detect the type of a query result for type-aware autosuggestions
///
/// Examines the structure of the result to determine:
/// - Is it an array? Are elements objects or primitives?
/// - Is it multiple values (destructured)?
/// - Is it a single value? What type?
fn detect_result_type(result: &str) -> ResultType {
    use serde_json::Deserializer;

    // Use streaming parser to read first value
    let mut deserializer = Deserializer::from_str(result).into_iter();

    let first_value = match deserializer.next() {
        Some(Ok(v)) => v,
        _ => return ResultType::Null,
    };

    // Check if there's a second value (indicates destructured output)
    let has_multiple_values = deserializer.next().is_some();

    // Determine type based on first value and whether there are more
    match first_value {
        Value::Object(_) if has_multiple_values => ResultType::DestructuredObjects,
        Value::Object(_) => ResultType::Object,
        Value::Array(ref arr) => {
            if arr.is_empty() {
                ResultType::Array
            } else if matches!(arr[0], Value::Object(_)) {
                ResultType::ArrayOfObjects
            } else {
                ResultType::Array
            }
        }
        Value::String(_) => ResultType::String,
        Value::Number(_) => ResultType::Number,
        Value::Bool(_) => ResultType::Boolean,
        Value::Null => ResultType::Null,
    }
}

/// Normalize base query by stripping trailing incomplete operations
///
/// Strips patterns like:
/// - " | ." → pipe with identity (will be re-added by PipeOperator formula)
/// - "." at end → trailing dot (incomplete field access)
/// - Trailing whitespace
///
/// Examples:
/// - ".services | ." → ".services"
/// - ".services[]." → ".services[]"
/// - ".user " → ".user"
/// - "." → "." (keep root as-is)
fn normalize_base_query(query: &str) -> String {
    let mut base = query.trim_end().to_string();

    // Strip trailing " | ." pattern (pipe followed by identity)
    if base.ends_with(" | .") {
        base = base[..base.len() - 4].trim_end().to_string();
    }
    // Strip trailing " | " (incomplete pipe without operand)
    else if base.ends_with(" |") {
        base = base[..base.len() - 2].trim_end().to_string();
    }
    // Strip trailing "." if it's incomplete field access
    // But preserve "." if it's the root query
    else if base.ends_with('.') && base.len() > 1 {
        base = base[..base.len() - 1].to_string();
    }

    base
}

#[cfg(test)]
#[path = "preprocess_tests.rs"]
mod preprocess_tests;
