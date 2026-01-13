use super::autocomplete_state::{JsonFieldType, Suggestion, SuggestionType};
use super::brace_tracker::BraceTracker;
use super::jq_functions::filter_builtins;
use super::result_analyzer::ResultAnalyzer;
use crate::query::ResultType;
use serde_json::Value;
use std::sync::Arc;

fn filter_suggestions_by_partial(suggestions: Vec<Suggestion>, partial: &str) -> Vec<Suggestion> {
    let partial_lower = partial.to_lowercase();
    suggestions
        .into_iter()
        .filter(|s| s.text.to_lowercase().contains(&partial_lower))
        .collect()
}

fn skip_trailing_whitespace(chars: &[char], start: usize) -> usize {
    let mut i = start;
    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }
    i
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum SuggestionContext {
    FunctionContext,
    FieldContext,
    ObjectKeyContext,
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
