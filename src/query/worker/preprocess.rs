//! Query Result Preprocessing
//!
//! Functions for preprocessing query results in the worker thread.
//! These operations are expensive and moved here to avoid blocking the main thread.

use std::sync::Arc;

use ansi_to_tui::IntoText;
use memchr::memchr;
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

    // Compute line metrics, widths, and is_only_nulls in a single pass
    if cancel_token.is_cancelled() {
        return Err(QueryError::Cancelled);
    }
    let (line_count, max_width, line_widths, is_only_nulls) = compute_line_metrics(&unformatted);

    // Parse ANSI to RenderedLine
    if cancel_token.is_cancelled() {
        return Err(QueryError::Cancelled);
    }
    let rendered_lines = parse_ansi_to_rendered_lines(&output, cancel_token)?;

    // Parse JSON and detect type in single pass
    if cancel_token.is_cancelled() {
        return Err(QueryError::Cancelled);
    }
    let (parsed, result_type) = parse_and_detect_type(&unformatted);
    let parsed = parsed.map(Arc::new);

    let base_query = normalize_base_query(query);

    Ok(ProcessedResult {
        output: Arc::new(output),
        unformatted: Arc::new(unformatted),
        rendered_lines,
        parsed,
        line_count,
        max_width,
        line_widths,
        result_type,
        query: base_query,
        execution_time_ms: None,
        is_only_nulls,
    })
}

/// Compute line count, max width, individual line widths, and is_only_nulls in a single pass
///
/// Returns (line_count, max_width, line_widths, is_only_nulls) to avoid multiple iterations.
/// is_only_nulls is true if all non-empty lines are "null" (including vacuous truth for empty output).
fn compute_line_metrics(output: &str) -> (u32, u16, Arc<Vec<u16>>, bool) {
    let mut line_count: u32 = 0;
    let mut max_width: usize = 0;
    let mut widths: Vec<u16> = Vec::new();
    let mut is_only_nulls = true;

    for line in output.lines() {
        line_count += 1;
        let width = line.len().min(u16::MAX as usize);
        widths.push(width as u16);
        if width > max_width {
            max_width = width;
        }

        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed != "null" {
            is_only_nulls = false;
        }
    }

    (
        line_count,
        max_width.min(u16::MAX as usize) as u16,
        Arc::new(widths),
        is_only_nulls,
    )
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

/// Strip ANSI escape codes from a string using SIMD-accelerated scanning
///
/// jq outputs colored results with ANSI codes like:
/// - `\x1b[0m` (reset)
/// - `\x1b[1;39m` (bold)
/// - `\x1b[0;32m` (green)
///
/// Uses memchr for fast byte-level scanning and bulk memory copies.
pub fn strip_ansi_codes(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let mut pos = 0;

    // Find each escape sequence using SIMD-accelerated search
    while let Some(esc_offset) = memchr(b'\x1b', &bytes[pos..]) {
        let esc_pos = pos + esc_offset;

        // Bulk copy everything before escape
        result.extend_from_slice(&bytes[pos..esc_pos]);

        // Skip escape sequence
        pos = skip_csi_sequence(bytes, esc_pos);
    }

    // Copy remaining content after last escape
    result.extend_from_slice(&bytes[pos..]);

    // Safe: we only copied valid UTF-8 byte sequences from valid input
    unsafe { String::from_utf8_unchecked(result) }
}

/// Skip a CSI (Control Sequence Introducer) sequence
///
/// CSI sequences have the format: ESC [ parameters m
/// where parameters are numbers and semicolons
fn skip_csi_sequence(bytes: &[u8], start: usize) -> usize {
    let mut pos = start + 1; // Skip ESC

    if pos < bytes.len() && bytes[pos] == b'[' {
        pos += 1; // Skip '['
        // Skip until 'm' (SGR terminator)
        while pos < bytes.len() {
            if bytes[pos] == b'm' {
                return pos + 1;
            }
            pos += 1;
        }
    }
    pos
}

/// Parse JSON and detect its type in a single pass
///
/// Returns both the parsed first value and the result type, avoiding duplicate parsing.
/// Handles both single values and destructured output (multiple JSON values).
///
/// Uses fast-path `from_str` for single values (common case), falling back to
/// streaming parser for destructured output like `{"a":1}\n{"b":2}`.
pub fn parse_and_detect_type(text: &str) -> (Option<Value>, ResultType) {
    let text = text.trim();
    if text.is_empty() {
        return (None, ResultType::Null);
    }

    // FAST PATH: Try full parse first (common case: single value)
    // from_str fails on destructured output (trailing content after first value)
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        let result_type = value_to_result_type(&value, false);
        return (Some(value), result_type);
    }

    // FALLBACK: Streaming parse for destructured output (multiple JSON values)
    let mut deserializer = serde_json::Deserializer::from_str(text).into_iter();

    let first_value = match deserializer.next() {
        Some(Ok(v)) => v,
        _ => return (None, ResultType::Null),
    };

    // Check for multiple values (destructured output)
    let has_multiple = deserializer.next().is_some();
    let result_type = value_to_result_type(&first_value, has_multiple);

    (Some(first_value), result_type)
}

/// Convert a parsed JSON value to its ResultType
fn value_to_result_type(value: &Value, has_multiple: bool) -> ResultType {
    match value {
        Value::Object(_) if has_multiple => ResultType::DestructuredObjects,
        Value::Object(_) => ResultType::Object,
        Value::Array(arr) => {
            if arr.is_empty() {
                ResultType::Array
            } else if matches!(arr.first(), Some(Value::Object(_))) {
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
