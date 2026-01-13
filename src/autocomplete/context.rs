use super::autocomplete_state::{JsonFieldType, Suggestion, SuggestionType};
use super::brace_tracker::BraceTracker;
use super::jq_functions::filter_builtins;
use super::result_analyzer::ResultAnalyzer;
use super::variable_extractor::extract_variables;
use crate::query::ResultType;
use serde_json::Value;
use std::sync::Arc;

/// Filters suggestions by matching the incomplete text the user is typing (case-insensitive).
///
/// # Parameters
/// - `suggestions`: List of available suggestions
/// - `partial`: The incomplete text being typed (e.g., "na" when typing ".na|")
fn filter_suggestions_by_partial(suggestions: Vec<Suggestion>, partial: &str) -> Vec<Suggestion> {
    let partial_lower = partial.to_lowercase();
    suggestions
        .into_iter()
        .filter(|s| s.text.to_lowercase().contains(&partial_lower))
        .collect()
}

/// Filters suggestions case-sensitively (for variables which are case-sensitive in jq).
fn filter_suggestions_case_sensitive(
    suggestions: Vec<Suggestion>,
    partial: &str,
) -> Vec<Suggestion> {
    suggestions
        .into_iter()
        .filter(|s| s.text.contains(partial))
        .collect()
}

/// Skips trailing whitespace backwards from a position in the character array.
///
/// # Parameters
/// - `chars`: Character array of the query text
/// - `start`: Position to skip backwards from (0-based index)
///
/// # Returns
/// Position of the last non-whitespace character before `start`.
fn skip_trailing_whitespace(chars: &[char], start: usize) -> usize {
    let mut i = start;
    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }
    i
}

/// Extracts the incomplete token the user is currently typing by walking backwards to a delimiter.
///
/// # Parameters
/// - `chars`: Character array of the query text before cursor
/// - `end`: Cursor position (0-based index, where extraction should end)
///
/// # Returns
/// Tuple of (token_start_position, partial_text)
///
/// # Example
/// Query: `map(.ser|` with cursor at position 8
/// Returns: (5, "ser")
fn extract_partial_token(chars: &[char], end: usize) -> (usize, String) {
    let mut start = end;
    while start > 0 {
        let ch = chars[start - 1];
        if is_delimiter(ch) {
            break;
        }
        start -= 1;
    }
    let partial: String = chars[start..end].iter().collect();
    (start, partial)
}

/// Determines context from field access prefixes (. or ?).
///
/// # Parameters
/// - `partial`: The incomplete token (e.g., ".name", "?.field", "?")
///
/// # Returns
/// Some(context, field_name) or None if no prefix match
///
/// # Examples
/// - ".name" → FieldContext with "name"
/// - ".user.name" → FieldContext with "name" (only last segment)
/// - "?.field" → FieldContext with "field"
/// - "?" → FunctionContext with ""
fn context_from_field_prefix(partial: &str) -> Option<(SuggestionContext, String)> {
    if let Some(stripped) = partial.strip_prefix('.') {
        let field_partial = if let Some(last_dot_pos) = partial.rfind('.') {
            partial[last_dot_pos + 1..].to_string()
        } else {
            stripped.to_string()
        };
        return Some((SuggestionContext::FieldContext, field_partial));
    } else if let Some(stripped) = partial.strip_prefix('?') {
        if let Some(after_dot) = stripped.strip_prefix('.') {
            return Some((SuggestionContext::FieldContext, after_dot.to_string()));
        } else {
            return Some((SuggestionContext::FunctionContext, String::new()));
        }
    }
    None
}

/// Infers context by examining the character before the partial token.
///
/// # Parameters
/// - `chars`: Character array of the query text
/// - `start`: Position where the partial token starts
/// - `partial`: The incomplete token text
/// - `before_cursor`: Full query text before cursor position
/// - `brace_tracker`: Tracks nested braces/brackets for object detection
///
/// # Returns
/// Some(context, partial) or None if no special context detected
///
/// # Examples
/// - "| name" → FunctionContext (pipe before token)
/// - ". name" → FieldContext (dot before token)
/// - "{name" → ObjectKeyContext (inside object literal)
fn infer_context_from_preceding_char(
    chars: &[char],
    start: usize,
    partial: &str,
    before_cursor: &str,
    brace_tracker: &BraceTracker,
) -> Option<(SuggestionContext, String)> {
    if start > 0 {
        let j = skip_trailing_whitespace(chars, start);

        if j > 0 {
            let char_before = chars[j - 1];
            if char_before == '.' || char_before == '?' {
                return Some((SuggestionContext::FieldContext, partial.to_string()));
            }

            if !partial.is_empty()
                && (char_before == '{' || char_before == ',')
                && brace_tracker.is_in_object(before_cursor.len())
            {
                return Some((SuggestionContext::ObjectKeyContext, partial.to_string()));
            }
        }
    }
    None
}

/// Determines if field suggestions should include a leading dot.
///
/// # Parameters
/// - `before_cursor`: Query text before the cursor position
/// - `partial`: The incomplete field name being typed
///
/// # Returns
/// true if suggestions need a leading dot (e.g., after |, ;, or at query start)
///
/// # Examples
/// - "| na" → true (after pipe delimiter)
/// - ".name .ag" → true (whitespace before dot)
/// - ".name.ag" → false (already has dot)
fn needs_leading_dot(before_cursor: &str, partial: &str) -> bool {
    let char_before_dot = find_char_before_field_access(before_cursor, partial);

    let dot_pos = if partial.is_empty() {
        before_cursor.len().saturating_sub(1)
    } else {
        before_cursor.len().saturating_sub(partial.len() + 1)
    };
    let has_immediate_dot =
        dot_pos < before_cursor.len() && before_cursor.chars().nth(dot_pos) == Some('.');

    let has_whitespace_before_dot = if dot_pos > 0 && has_immediate_dot {
        before_cursor[..dot_pos]
            .chars()
            .rev()
            .take_while(|c| c.is_whitespace())
            .count()
            > 0
    } else {
        false
    };

    matches!(
        char_before_dot,
        Some('|') | Some(';') | Some(',') | Some(':') | Some('(') | Some('[') | Some('{') | None
    ) || has_whitespace_before_dot
}

/// Gets field suggestions from parsed JSON result.
///
/// # Parameters
/// - `result_parsed`: Optional parsed JSON data to extract fields from
/// - `result_type`: Type of JSON result (Object, Array, etc.)
/// - `needs_leading_dot`: Whether suggestions should include leading dot
/// - `suppress_array_brackets`: Whether to suppress .[] suggestions (true inside map/select)
///
/// # Returns
/// List of field suggestions, or empty list if no result available.
fn get_field_suggestions(
    result_parsed: Option<Arc<Value>>,
    result_type: Option<ResultType>,
    needs_leading_dot: bool,
    suppress_array_brackets: bool,
) -> Vec<Suggestion> {
    if let (Some(result), Some(typ)) = (result_parsed, result_type) {
        ResultAnalyzer::analyze_parsed_result(
            &result,
            typ,
            needs_leading_dot,
            suppress_array_brackets,
        )
    } else {
        Vec::new()
    }
}

/// Injects .key and .value suggestions for with_entries() context.
///
/// # Parameters
/// - `suggestions`: Mutable list to inject special suggestions into
/// - `needs_leading_dot`: Whether suggestions should include leading dot
///
/// # Notes
/// Inserts .value first so .key ends up at position 0 (top of suggestion list).
fn inject_with_entries_suggestions(suggestions: &mut Vec<Suggestion>, needs_leading_dot: bool) {
    let prefix = if needs_leading_dot { "." } else { "" };

    suggestions.insert(
        0,
        Suggestion::new_with_type(format!("{}value", prefix), SuggestionType::Field, None)
            .with_description("Entry value from with_entries()"),
    );
    suggestions.insert(
        0,
        Suggestion::new_with_type(
            format!("{}key", prefix),
            SuggestionType::Field,
            Some(JsonFieldType::String),
        )
        .with_description("Entry key from with_entries()"),
    );
}

/// Filters suggestions by partial text only if partial is non-empty.
///
/// # Parameters
/// - `suggestions`: List of suggestions to filter
/// - `partial`: The incomplete text being typed
///
/// # Returns
/// Filtered suggestions if partial is non-empty, otherwise all suggestions.
fn filter_suggestions_by_partial_if_nonempty(
    suggestions: Vec<Suggestion>,
    partial: &str,
) -> Vec<Suggestion> {
    if partial.is_empty() {
        suggestions
    } else {
        filter_suggestions_by_partial(suggestions, partial)
    }
}

/// Checks if cursor is in a variable definition context where suggestions should not be shown.
/// This includes positions after `as `, `label `, or inside destructuring patterns.
fn is_in_variable_definition_context(before_cursor: &str) -> bool {
    let dollar_pos = before_cursor.rfind('$');
    let dollar_pos = match dollar_pos {
        Some(pos) => pos,
        None => return false,
    };

    let text_before_dollar = &before_cursor[..dollar_pos];
    let trimmed = text_before_dollar.trim_end();

    if is_after_definition_keyword(trimmed) {
        return true;
    }

    if is_in_destructuring_pattern(trimmed) {
        return true;
    }

    false
}

/// Checks if text ends with a definition keyword (as, label).
fn is_after_definition_keyword(trimmed: &str) -> bool {
    if trimmed.ends_with("as") {
        if trimmed.len() == 2 {
            return true;
        }
        let char_before = trimmed.chars().nth(trimmed.len() - 3);
        if let Some(ch) = char_before {
            return !ch.is_alphanumeric() && ch != '_';
        }
        return true;
    }

    if trimmed.ends_with("label") {
        if trimmed.len() == 5 {
            return true;
        }
        let char_before = trimmed.chars().nth(trimmed.len() - 6);
        if let Some(ch) = char_before {
            return !ch.is_alphanumeric() && ch != '_';
        }
        return true;
    }

    false
}

/// Checks if text indicates we're inside a destructuring pattern after `as`.
fn is_in_destructuring_pattern(trimmed: &str) -> bool {
    if trimmed.ends_with('[')
        || trimmed.ends_with('{')
        || trimmed.ends_with(',')
        || trimmed.ends_with(':')
    {
        return has_unclosed_as_destructure(trimmed);
    }
    false
}

/// Checks if there's an unclosed destructuring pattern after `as`.
fn has_unclosed_as_destructure(text: &str) -> bool {
    for pattern in &[" as [", " as[", " as {", " as{"] {
        if let Some(pos) = text.rfind(pattern) {
            let after_as = &text[pos + pattern.len()..];

            let open_brackets = after_as.chars().filter(|c| *c == '[').count();
            let closed_brackets = after_as.chars().filter(|c| *c == ']').count();
            let open_braces = after_as.chars().filter(|c| *c == '{').count();
            let closed_braces = after_as.chars().filter(|c| *c == '}').count();

            if pattern.contains('[') && open_brackets >= closed_brackets {
                return true;
            }
            if pattern.contains('{') && open_braces >= closed_braces {
                return true;
            }
        }
    }

    if text.ends_with("as [")
        || text.ends_with("as[")
        || text.ends_with("as {")
        || text.ends_with("as{")
    {
        return true;
    }

    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum SuggestionContext {
    FunctionContext,
    FieldContext,
    ObjectKeyContext,
    VariableContext,
}

pub fn get_suggestions(
    query: &str,
    cursor_pos: usize,
    result_parsed: Option<Arc<Value>>,
    result_type: Option<ResultType>,
    brace_tracker: &BraceTracker,
) -> Vec<Suggestion> {
    let before_cursor = &query[..cursor_pos.min(query.len())];
    let (context, partial) = analyze_context(before_cursor, brace_tracker);

    // Suppress .[].field suggestions inside element-context functions (map, select, etc.)
    // where iteration is already provided by the function
    let suppress_array_brackets = brace_tracker.is_in_element_context(cursor_pos);
    let in_with_entries = brace_tracker.is_in_with_entries_context(cursor_pos);

    match context {
        SuggestionContext::FieldContext => {
            let needs_dot = needs_leading_dot(before_cursor, &partial);
            let mut suggestions = get_field_suggestions(
                result_parsed,
                result_type,
                needs_dot,
                suppress_array_brackets,
            );

            if in_with_entries {
                inject_with_entries_suggestions(&mut suggestions, needs_dot);
            }

            filter_suggestions_by_partial_if_nonempty(suggestions, &partial)
        }
        SuggestionContext::FunctionContext => {
            if partial.is_empty() {
                Vec::new()
            } else {
                filter_builtins(&partial)
            }
        }
        SuggestionContext::ObjectKeyContext => {
            if partial.is_empty() {
                return Vec::new();
            }

            let suggestions = get_field_suggestions(result_parsed, result_type, false, true);
            filter_suggestions_by_partial(suggestions, &partial)
        }
        SuggestionContext::VariableContext => {
            let all_vars = extract_variables(query);
            let suggestions: Vec<Suggestion> = all_vars
                .into_iter()
                .map(|name| Suggestion::new_with_type(name, SuggestionType::Variable, None))
                .collect();
            filter_suggestions_case_sensitive(suggestions, &partial)
        }
    }
}

pub fn analyze_context(
    before_cursor: &str,
    brace_tracker: &BraceTracker,
) -> (SuggestionContext, String) {
    if before_cursor.is_empty() {
        return (SuggestionContext::FunctionContext, String::new());
    }

    let chars: Vec<char> = before_cursor.chars().collect();
    let end = skip_trailing_whitespace(&chars, chars.len());

    if end == 0 {
        return (SuggestionContext::FunctionContext, String::new());
    }

    if chars[end - 1] == '.' {
        return (SuggestionContext::FieldContext, String::new());
    }

    let (start, partial) = extract_partial_token(&chars, end);

    if let Some(result) = context_from_variable_prefix(&partial, before_cursor) {
        return result;
    }

    if let Some(result) = context_from_field_prefix(&partial) {
        return result;
    }

    if let Some(result) =
        infer_context_from_preceding_char(&chars, start, &partial, before_cursor, brace_tracker)
    {
        return result;
    }

    (SuggestionContext::FunctionContext, partial)
}

/// Determines context from variable prefix ($).
/// Returns VariableContext if typing a variable usage, None if defining a variable.
fn context_from_variable_prefix(
    partial: &str,
    before_cursor: &str,
) -> Option<(SuggestionContext, String)> {
    if !partial.starts_with('$') {
        return None;
    }

    if is_in_variable_definition_context(before_cursor) {
        return None;
    }

    let var_partial = partial.to_string();
    Some((SuggestionContext::VariableContext, var_partial))
}

pub fn find_char_before_field_access(before_cursor: &str, partial: &str) -> Option<char> {
    let search_end = if partial.is_empty() {
        before_cursor.len().saturating_sub(1)
    } else {
        before_cursor.len().saturating_sub(partial.len() + 1)
    };

    if search_end == 0 {
        return None;
    }

    let chars: Vec<char> = before_cursor[..search_end].chars().collect();
    for i in (0..chars.len()).rev() {
        let ch = chars[i];
        if !ch.is_whitespace() {
            return Some(ch);
        }
    }

    None
}

fn is_delimiter(ch: char) -> bool {
    matches!(
        ch,
        '|' | ';' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ' ' | '\t' | '\n' | '\r'
    )
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod context_tests;
