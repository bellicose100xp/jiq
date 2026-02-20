use super::autocomplete_state::{JsonFieldType, Suggestion, SuggestionType};
use super::brace_tracker::{BraceTracker, BraceType};
use super::jq_functions::filter_builtins;
use super::json_navigator::navigate_multi;
use super::path_parser::{PathSegment, parse_path};
use super::result_analyzer::ResultAnalyzer;
use super::scan_state::ScanState;
use super::variable_extractor::extract_variables;
use crate::query::ResultType;
use serde_json::Value;
use std::collections::HashSet;
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
    array_sample_size: usize,
) -> Vec<Suggestion> {
    if let (Some(result), Some(typ)) = (result_parsed, result_type) {
        ResultAnalyzer::analyze_parsed_result(
            &result,
            typ,
            needs_leading_dot,
            suppress_array_brackets,
            array_sample_size,
        )
    } else {
        Vec::new()
    }
}

/// Converts all cached field names to suggestions for non-deterministic fallback.
fn get_all_field_suggestions(
    all_field_names: &HashSet<String>,
    needs_leading_dot: bool,
) -> Vec<Suggestion> {
    let prefix = if needs_leading_dot { "." } else { "" };
    all_field_names
        .iter()
        .map(|name| {
            Suggestion::new_with_type(format!("{}{}", prefix, name), SuggestionType::Field, None)
        })
        .collect()
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

/// Context when inside entry-transforming functions (to_entries, with_entries).
/// Determines whether to suggest .key/.value or fall back to all fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryContext {
    /// Not in an entry context
    None,
    /// Direct entry access - suggest .key and .value
    Direct,
    /// Navigated into .value with additional transformations - fall back to all fields
    OpaqueValue,
}

/// Detects entry context from transforming functions (to_entries, with_entries).
///
/// Returns:
/// - `EntryContext::Direct` - cursor is at direct entry access (suggest .key/.value)
/// - `EntryContext::OpaqueValue` - cursor is after `.value | nested(` (show all fields)
/// - `EntryContext::None` - not in entry context
pub fn detect_entry_context(query: &str, cursor_pos: usize) -> EntryContext {
    let before_cursor = &query[..cursor_pos.min(query.len())];

    // Check with_entries first (cursor inside function parentheses)
    if let Some(we_pos) = find_unclosed_with_entries(before_cursor) {
        // Find the actual opening paren position (may have whitespace after name)
        let after_name = &before_cursor[we_pos + "with_entries".len()..];
        let whitespace_len = after_name.len() - after_name.trim_start().len();
        let paren_pos = we_pos + "with_entries".len() + whitespace_len + 1; // +1 for '('
        let inside_we = &before_cursor[paren_pos..];
        return classify_entry_path(inside_we);
    }

    // Check to_entries
    if let Some(te_pos) = find_to_entries_outside_strings(before_cursor) {
        let after_te = &before_cursor[te_pos + "to_entries".len()..];
        if is_in_entry_element_context(after_te)
            && let Some(path_start) = find_entry_element_start(after_te)
        {
            return classify_entry_path(&after_te[path_start..]);
        }
    }

    EntryContext::None
}

/// Find the last occurrence of `to_entries` outside of string literals.
fn find_to_entries_outside_strings(query: &str) -> Option<usize> {
    let mut state = ScanState::default();
    let mut last_pos = None;

    for (pos, ch) in query.char_indices() {
        if !state.is_in_string() && query[pos..].starts_with("to_entries") {
            last_pos = Some(pos);
        }
        state = state.advance(ch);
    }
    last_pos
}

/// Find the innermost unclosed `with_entries(` position.
/// Handles optional whitespace between function name and opening paren.
fn find_unclosed_with_entries(before_cursor: &str) -> Option<usize> {
    let mut state = ScanState::default();
    let mut we_positions = Vec::new();

    for (pos, ch) in before_cursor.char_indices() {
        if !state.is_in_string() {
            // Check for with_entries followed by optional whitespace and (
            if before_cursor[pos..].starts_with("with_entries") {
                let after_name = &before_cursor[pos + "with_entries".len()..];
                let trimmed = after_name.trim_start();
                if trimmed.starts_with('(') {
                    we_positions.push(pos);
                }
            }
            if ch == ')' && !we_positions.is_empty() {
                we_positions.pop();
            }
        }
        state = state.advance(ch);
    }

    we_positions.last().copied()
}

/// Check if we're in an entry element context after to_entries.
/// This detects patterns like:
/// - `| .[]` (array iteration)
/// - `| map(` (mapping function)
fn is_in_entry_element_context(after_to_entries: &str) -> bool {
    let trimmed = after_to_entries.trim_start();

    // Check for pipe followed by iteration or map
    if let Some(pipe_pos) = trimmed.find('|') {
        let after_pipe = trimmed[pipe_pos + 1..].trim_start();

        // Array iteration: .[  or .[]
        if after_pipe.starts_with(".[") {
            return true;
        }

        // Map function
        if after_pipe.starts_with("map(") {
            return true;
        }
    }

    // Direct iteration without pipe: .[]
    trimmed.starts_with(".[")
}

/// Find the start position of entry element access in the after_to_entries string.
/// Returns the position where we start accessing individual entries.
fn find_entry_element_start(after_to_entries: &str) -> Option<usize> {
    let trimmed = after_to_entries.trim_start();
    let offset = after_to_entries.len() - trimmed.len();

    // Look for patterns that start element access
    if let Some(pipe_pos) = trimmed.find('|') {
        let after_pipe = trimmed[pipe_pos + 1..].trim_start();
        let pipe_offset = pipe_pos + 1 + (trimmed[pipe_pos + 1..].len() - after_pipe.len());

        // .[] pattern - find the closing ]
        if after_pipe.starts_with(".[]") {
            // Find position after .[]
            if let Some(bracket_end) = after_pipe[1..].find(']') {
                let pos_after_iteration = offset + pipe_offset + 1 + bracket_end + 1;
                // Skip any pipe after .[].
                let remainder = &after_to_entries[pos_after_iteration..];
                if let Some(dot_pos) = remainder.find('.') {
                    return Some(pos_after_iteration + dot_pos);
                }
            }
        }

        // map( pattern - find the opening paren
        if after_pipe.starts_with("map(") {
            let paren_pos = offset + pipe_offset + 4; // length of "map("
            return Some(paren_pos);
        }
    }

    // Direct .[] without pipe
    if trimmed.starts_with(".[]")
        && let Some(bracket_end) = trimmed[1..].find(']')
    {
        let pos_after_iteration = offset + 1 + bracket_end + 1;
        let remainder = &after_to_entries[pos_after_iteration..];
        if let Some(dot_pos) = remainder.find('.') {
            return Some(pos_after_iteration + dot_pos);
        }
    }

    None
}

/// Classify entry path to determine if we're at direct entry access or navigated into .value.
fn classify_entry_path(path: &str) -> EntryContext {
    // Find .value access outside strings
    let value_pos = match find_value_access_outside_strings(path) {
        Some(pos) => pos,
        None => return EntryContext::Direct,
    };

    let after_value = &path[value_pos + ".value".len()..];

    // Pipe after .value = opaque (can't determine structure)
    if contains_char_outside_strings(after_value, '|') {
        return EntryContext::OpaqueValue;
    }

    // Nested functions after .value = opaque
    let nested_functions = ["map(", "select(", "sort_by(", "group_by(", "unique_by("];
    for func in nested_functions {
        if contains_pattern_outside_strings(after_value, func) {
            return EntryContext::OpaqueValue;
        }
    }

    // Check if there's a dot immediately after .value (navigating into value)
    let trimmed_after = after_value.trim_start();
    if trimmed_after.starts_with('.') {
        // Direct .value.field navigation - not in entry context anymore
        return EntryContext::None;
    }

    // Just .value without further navigation - still in direct context
    EntryContext::Direct
}

/// Find the last `.value` access outside of string literals.
fn find_value_access_outside_strings(query: &str) -> Option<usize> {
    let mut state = ScanState::default();
    let mut last_pos = None;

    for (pos, ch) in query.char_indices() {
        if !state.is_in_string() && query[pos..].starts_with(".value") {
            // Verify it's not followed by more identifier chars (e.g., .values)
            let after_value = &query[pos + ".value".len()..];
            let next_char = after_value.chars().next();
            if !matches!(next_char, Some(c) if c.is_alphanumeric() || c == '_') {
                last_pos = Some(pos);
            }
        }
        state = state.advance(ch);
    }
    last_pos
}

/// Check if a character appears outside of string literals.
fn contains_char_outside_strings(query: &str, target: char) -> bool {
    let mut state = ScanState::default();

    for (_pos, ch) in query.char_indices() {
        if !state.is_in_string() && ch == target {
            return true;
        }
        state = state.advance(ch);
    }
    false
}

/// Check if a pattern appears outside of string literals.
fn contains_pattern_outside_strings(query: &str, pattern: &str) -> bool {
    let mut state = ScanState::default();

    for (pos, ch) in query.char_indices() {
        if !state.is_in_string() && query[pos..].starts_with(pattern) {
            return true;
        }
        state = state.advance(ch);
    }
    false
}

/// Injects .key and .value suggestions for entry context (to_entries, with_entries).
/// Removes any existing key/value suggestions first to avoid duplicates.
fn inject_entry_field_suggestions(suggestions: &mut Vec<Suggestion>, needs_leading_dot: bool) {
    let prefix = if needs_leading_dot { "." } else { "" };
    let key_text = format!("{}key", prefix);
    let value_text = format!("{}value", prefix);

    // Remove any existing key/value suggestions to avoid duplicates
    // (the result analyzer may have already found them from the entry structure)
    suggestions.retain(|s| s.text != key_text && s.text != value_text);

    suggestions.insert(
        0,
        Suggestion::new_with_type(value_text, SuggestionType::Field, None)
            .with_description("Entry value from to_entries/with_entries"),
    );
    suggestions.insert(
        0,
        Suggestion::new_with_type(key_text, SuggestionType::Field, Some(JsonFieldType::String))
            .with_description("Entry key from to_entries/with_entries"),
    );
}

#[allow(clippy::too_many_arguments)]
pub fn get_suggestions(
    query: &str,
    cursor_pos: usize,
    result_parsed: Option<Arc<Value>>,
    result_type: Option<ResultType>,
    original_json: Option<Arc<Value>>,
    all_field_names: Arc<HashSet<String>>,
    brace_tracker: &BraceTracker,
    array_sample_size: usize,
) -> Vec<Suggestion> {
    let before_cursor = &query[..cursor_pos.min(query.len())];
    let (context, partial) = analyze_context(before_cursor, brace_tracker);

    // Suppress .[].field suggestions inside element-context functions (map, select, etc.)
    // where iteration is already provided by the function
    let suppress_array_brackets = brace_tracker.is_in_element_context(cursor_pos);

    match context {
        SuggestionContext::FieldContext => {
            let needs_dot = needs_leading_dot(before_cursor, &partial);
            let is_at_end = is_cursor_at_logical_end(query, cursor_pos);
            let is_non_executing = brace_tracker.is_in_non_executing_context(cursor_pos);

            // Unified entry context detection for to_entries/with_entries
            let entry_context = detect_entry_context(query, cursor_pos);

            // If inside .value with nested transformations, fall back to all fields
            if entry_context == EntryContext::OpaqueValue {
                let suggestions = get_all_field_suggestions(&all_field_names, needs_dot);
                return filter_suggestions_by_partial_if_nonempty(suggestions, &partial);
            }

            // Phase 3: Path-aware suggestion logic
            let mut suggestions = if is_non_executing && is_at_end {
                // NON-EXECUTING CONTEXT + CURSOR AT END:
                // Cache is stale, extract path and navigate from cache or original
                let (path_context, is_after_pipe) =
                    extract_path_context_with_pipe_info(before_cursor, brace_tracker);

                // Try navigating from last_successful_result first
                if let Some(ref result) = result_parsed {
                    if let Some(nested_suggestions) = get_nested_field_suggestions(
                        result,
                        &path_context,
                        needs_dot,
                        suppress_array_brackets,
                        suppress_array_brackets, // is_in_element_context == suppress_array_brackets
                        is_after_pipe,
                        result_type.as_ref(),
                        array_sample_size,
                    ) {
                        nested_suggestions
                    } else if let Some(ref orig) = original_json {
                        // Navigation failed, fall back to original_json
                        get_nested_field_suggestions(
                            orig,
                            &path_context,
                            needs_dot,
                            suppress_array_brackets,
                            suppress_array_brackets,
                            is_after_pipe,
                            result_type.as_ref(),
                            array_sample_size,
                        )
                        .unwrap_or_else(|| {
                            // Non-deterministic: show all fields from original JSON
                            get_all_field_suggestions(&all_field_names, needs_dot)
                        })
                    } else {
                        // Non-deterministic: show all fields from original JSON
                        get_all_field_suggestions(&all_field_names, needs_dot)
                    }
                } else {
                    Vec::new()
                }
            } else if !is_at_end {
                // MIDDLE OF QUERY: Cache is "ahead" of cursor, navigate from original_json
                let (path_context, is_after_pipe) =
                    extract_path_context_with_pipe_info(before_cursor, brace_tracker);

                if let Some(ref orig) = original_json {
                    get_nested_field_suggestions(
                        orig,
                        &path_context,
                        needs_dot,
                        suppress_array_brackets,
                        suppress_array_brackets,
                        is_after_pipe,
                        result_type.as_ref(),
                        array_sample_size,
                    )
                    .unwrap_or_else(|| {
                        // Non-deterministic: show all fields from original JSON
                        get_all_field_suggestions(&all_field_names, needs_dot)
                    })
                } else {
                    // Non-deterministic: show all fields from original JSON
                    get_all_field_suggestions(&all_field_names, needs_dot)
                }
            } else {
                // EXECUTING CONTEXT + CURSOR AT END:
                // Cache is current, suggest its fields directly
                get_field_suggestions(
                    result_parsed.clone(),
                    result_type.clone(),
                    needs_dot,
                    suppress_array_brackets,
                    array_sample_size,
                )
            };

            // Inject .key/.value for direct entry context (to_entries/with_entries)
            if entry_context == EntryContext::Direct {
                inject_entry_field_suggestions(&mut suggestions, needs_dot);
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

            let suggestions =
                get_field_suggestions(result_parsed, result_type, false, true, array_sample_size);
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

/// Determine if cursor is at the "logical end" of the query
/// (at end, or only whitespace after cursor).
fn is_cursor_at_logical_end(query: &str, cursor_pos: usize) -> bool {
    if cursor_pos >= query.len() {
        return true;
    }
    query[cursor_pos..].chars().all(|c| c.is_whitespace())
}

/// Find where the current expression starts for path extraction.
/// Result of finding an expression boundary.
struct ExpressionBoundary {
    /// Position where the expression starts
    position: usize,
    /// Whether the boundary was a pipe (|) character
    is_after_pipe: bool,
}

/// Used in non-executing contexts to extract the path being typed.
fn find_expression_boundary(
    before_cursor: &str,
    brace_tracker: &BraceTracker,
) -> ExpressionBoundary {
    let innermost = brace_tracker.innermost_brace_info(before_cursor.len());

    match innermost {
        Some(info) => {
            let after_brace = &before_cursor[info.pos + 1..];

            // Within the brace context, find the last boundary character
            let boundary_chars: &[char] = match info.brace_type {
                BraceType::Paren => &['|', ';'],
                BraceType::Square => &['|', ';', ','],
                BraceType::Curly => &['|', ';', ',', ':'],
            };

            // Find last boundary within this context
            let last_boundary = after_brace.rfind(|c| boundary_chars.contains(&c));

            match last_boundary {
                Some(offset) => {
                    let boundary_char = after_brace.chars().nth(offset).unwrap_or(' ');
                    ExpressionBoundary {
                        position: info.pos + 1 + offset + 1, // +1 to skip the boundary char
                        is_after_pipe: boundary_char == '|',
                    }
                }
                None => ExpressionBoundary {
                    position: info.pos + 1, // Start after the opening brace
                    is_after_pipe: false,
                },
            }
        }
        None => {
            // Top-level: boundary at |, ;, or start
            let boundary_pos = before_cursor.rfind(['|', ';']);
            match boundary_pos {
                Some(pos) => {
                    let boundary_char = before_cursor.chars().nth(pos).unwrap_or(' ');
                    ExpressionBoundary {
                        position: pos + 1,
                        is_after_pipe: boundary_char == '|',
                    }
                }
                None => ExpressionBoundary {
                    position: 0,
                    is_after_pipe: false,
                },
            }
        }
    }
}

/// Extract path context from the expression boundary.
/// Returns the path string and whether we're after a pipe.
fn extract_path_context_with_pipe_info(
    before_cursor: &str,
    brace_tracker: &BraceTracker,
) -> (String, bool) {
    let boundary = find_expression_boundary(before_cursor, brace_tracker);
    let path = before_cursor[boundary.position..].trim_start().to_string();
    (path, boundary.is_after_pipe)
}

/// Get nested field suggestions by navigating the JSON tree.
/// This is the core Phase 3 integration point.
#[allow(clippy::too_many_arguments)]
fn get_nested_field_suggestions(
    json: &Value,
    path_context: &str,
    needs_leading_dot: bool,
    suppress_array_brackets: bool,
    is_in_element_context: bool,
    is_after_pipe: bool,
    result_type: Option<&ResultType>,
    array_sample_size: usize,
) -> Option<Vec<Suggestion>> {
    let mut parsed_path = parse_path(path_context);

    // Phase 7: Check if result is already from streaming (DestructuredObjects)
    // When the query has .services[] before select(), the cached result is already
    // an individual element, not an array. Don't prepend ArrayIterator in this case.
    let is_streaming = matches!(result_type, Some(ResultType::DestructuredObjects));

    // In element context (map, select), prepend ArrayIterator ONLY if the result
    // is not already from a streaming operation
    if is_in_element_context && !is_streaming {
        parsed_path.segments.insert(0, PathSegment::ArrayIterator);
    }

    // If path has no segments (just ".") AND we're after a pipe, return None.
    // After a pipe, "." refers to the pipe's input which we can't know without execution.
    // But at the start of an expression (not after pipe), "." means the root JSON.
    if parsed_path.segments.is_empty() && is_after_pipe {
        return None;
    }

    // Navigate with fan-out to collect values from multiple array elements
    let navigated_values = navigate_multi(json, &parsed_path.segments, array_sample_size);
    if navigated_values.is_empty() {
        return None;
    }

    let suggestions = ResultAnalyzer::analyze_multi_values(
        &navigated_values,
        needs_leading_dot,
        suppress_array_brackets,
        array_sample_size,
    );

    Some(suggestions)
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod context_tests;
