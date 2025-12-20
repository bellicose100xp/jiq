use super::autocomplete_state::Suggestion;
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

    match context {
        SuggestionContext::FieldContext => {
            let char_before_dot = find_char_before_field_access(before_cursor, &partial);

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

            let needs_leading_dot = matches!(
                char_before_dot,
                Some('|')
                    | Some(';')
                    | Some(',')
                    | Some(':')
                    | Some('(')
                    | Some('[')
                    | Some('{')
                    | None
            ) || has_whitespace_before_dot;

            let suggestions = if let (Some(result), Some(typ)) = (result_parsed, result_type) {
                ResultAnalyzer::analyze_parsed_result(&result, typ, needs_leading_dot)
            } else {
                Vec::new()
            };

            if partial.is_empty() {
                suggestions
            } else {
                filter_suggestions_by_partial(suggestions, &partial)
            }
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

            let suggestions = if let (Some(result), Some(typ)) = (result_parsed, result_type) {
                ResultAnalyzer::analyze_parsed_result(&result, typ, false)
            } else {
                Vec::new()
            };

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
    let mut i = chars.len();

    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }

    if i == 0 {
        return (SuggestionContext::FunctionContext, String::new());
    }

    if i > 0 && chars[i - 1] == '.' {
        return (SuggestionContext::FieldContext, String::new());
    }

    let mut start = i;
    while start > 0 {
        let ch = chars[start - 1];

        if is_delimiter(ch) {
            break;
        }

        start -= 1;
    }

    let partial: String = chars[start..i].iter().collect();

    if let Some(stripped) = partial.strip_prefix('.') {
        let field_partial = if let Some(last_dot_pos) = partial.rfind('.') {
            partial[last_dot_pos + 1..].to_string()
        } else {
            stripped.to_string()
        };
        return (SuggestionContext::FieldContext, field_partial);
    } else if let Some(stripped) = partial.strip_prefix('?') {
        let field_partial = if let Some(after_dot) = stripped.strip_prefix('.') {
            after_dot.to_string()
        } else {
            String::new()
        };
        return (SuggestionContext::FieldContext, field_partial);
    }

    if start > 0 {
        let mut j = start;
        while j > 0 && chars[j - 1].is_whitespace() {
            j -= 1;
        }

        if j > 0 {
            let char_before = chars[j - 1];
            if char_before == '.' || char_before == '?' {
                return (SuggestionContext::FieldContext, partial);
            }

            if !partial.is_empty()
                && (char_before == '{' || char_before == ',')
                && brace_tracker.is_in_object(before_cursor.len())
            {
                return (SuggestionContext::ObjectKeyContext, partial);
            }
        }
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
