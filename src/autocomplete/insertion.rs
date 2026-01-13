//! Autocomplete suggestion insertion logic
//!
//! This module handles inserting autocomplete suggestions into the query,
//! managing cursor positioning, and executing the updated query.

use tui_textarea::TextArea;

use crate::app::App;
use crate::autocomplete::autocomplete_state::Suggestion;
use crate::autocomplete::{SuggestionContext, analyze_context};
use crate::query::QueryState;

// Re-export sub-module functions
pub use self::cursor::move_cursor_to_column;

// Module declarations
#[path = "insertion/cursor.rs"]
mod cursor;

/// Replace partial text at cursor, preserving text before and after
fn replace_partial_at_cursor(
    textarea: &mut TextArea<'_>,
    query: &str,
    cursor_pos: usize,
    replacement_start: usize,
    insert_text: &str,
) {
    let new_query = format!(
        "{}{}{}",
        &query[..replacement_start],
        insert_text,
        &query[cursor_pos..]
    );

    textarea.delete_line_by_head();
    textarea.delete_line_by_end();
    textarea.insert_str(&new_query);

    let target_pos = replacement_start + insert_text.len();
    move_cursor_to_column(textarea, target_pos);
}

/// Insert an autocomplete suggestion from App context
///
/// Executes the new query immediately (no debounce) for instant feedback.
/// Uses async execution to prevent race conditions with ongoing queries.
pub fn insert_suggestion_from_app(app: &mut App, suggestion: &Suggestion) {
    let query_state = match &mut app.query {
        Some(q) => q,
        None => return,
    };

    insert_suggestion(&mut app.input.textarea, query_state, suggestion);

    app.autocomplete.hide();
    app.results_scroll.reset();
    app.error_overlay_visible = false;

    let query = app.input.textarea.lines()[0].as_ref();
    app.input.brace_tracker.rebuild(query);
    query_state.execute_async(query);
}

/// Check if trailing separator should be replaced to avoid duplicates
fn should_replace_trailing_separator(char_before: Option<char>, suggestion: &str) -> bool {
    matches!(
        (char_before, suggestion),
        (Some('.'), s) if s.starts_with('.') || s.starts_with("[]") || s.starts_with("{}")
    ) || matches!(
        (char_before, suggestion.chars().next()),
        (Some('['), Some('[')) | (Some('{'), Some('{'))
    )
}

/// Calculate start position for array/object iteration syntax
fn calculate_iteration_syntax_start(
    cursor_pos: usize,
    partial_len: usize,
    before_cursor: &str,
    base_query: Option<&str>,
) -> usize {
    let base = base_query.expect("base_query always exists");

    if before_cursor == base {
        cursor_pos
    } else if cursor_pos > partial_len {
        cursor_pos - partial_len - 1
    } else {
        cursor_pos
    }
}

/// Insert function suggestion (e.g., "select", "map", "then", "else")
fn insert_function_suggestion(
    textarea: &mut TextArea<'_>,
    query: &str,
    cursor_pos: usize,
    partial: &str,
    suggestion: &Suggestion,
) {
    let replacement_start = cursor_pos.saturating_sub(partial.len());
    let insert_text = if suggestion.needs_parens {
        format!("{}(", suggestion.text)
    } else {
        suggestion.text.to_string()
    };

    replace_partial_at_cursor(textarea, query, cursor_pos, replacement_start, &insert_text);
}

/// Insert object key suggestion (e.g., keys in object literals)
fn insert_object_key_suggestion(
    textarea: &mut TextArea<'_>,
    query: &str,
    cursor_pos: usize,
    partial: &str,
    suggestion: &Suggestion,
) {
    let replacement_start = cursor_pos.saturating_sub(partial.len());
    replace_partial_at_cursor(
        textarea,
        query,
        cursor_pos,
        replacement_start,
        &suggestion.text,
    );
}

/// Insert variable suggestion (e.g., "$x", "$ENV")
fn insert_variable_suggestion(
    textarea: &mut TextArea<'_>,
    query: &str,
    cursor_pos: usize,
    partial: &str,
    suggestion: &Suggestion,
) {
    let replacement_start = cursor_pos.saturating_sub(partial.len());
    replace_partial_at_cursor(
        textarea,
        query,
        cursor_pos,
        replacement_start,
        &suggestion.text,
    );
}

/// Insert field suggestion (e.g., ".name", "[].price", "{}.key")
fn insert_field_suggestion(
    textarea: &mut TextArea<'_>,
    query: &str,
    cursor_pos: usize,
    partial: &str,
    suggestion: &Suggestion,
    before_cursor: &str,
    base_query: Option<&str>,
) {
    let suggestion_text = &suggestion.text;

    let replacement_start = if partial.is_empty() {
        if cursor_pos > 0 {
            let char_before = query.chars().nth(cursor_pos - 1);
            if should_replace_trailing_separator(char_before, suggestion_text) {
                cursor_pos - 1
            } else {
                cursor_pos
            }
        } else {
            cursor_pos
        }
    } else if suggestion_text.starts_with("[]") || suggestion_text.starts_with("{}") {
        calculate_iteration_syntax_start(cursor_pos, partial.len(), before_cursor, base_query)
    } else if suggestion_text.starts_with('[')
        || suggestion_text.starts_with('{')
        || suggestion_text.starts_with('.')
    {
        cursor_pos.saturating_sub(partial.len() + 1)
    } else {
        cursor_pos.saturating_sub(partial.len())
    };

    replace_partial_at_cursor(
        textarea,
        query,
        cursor_pos,
        replacement_start,
        suggestion_text,
    );
}

/// Insert an autocomplete suggestion at the current cursor position
pub fn insert_suggestion(
    textarea: &mut TextArea<'_>,
    query_state: &mut QueryState,
    suggestion: &Suggestion,
) {
    let query = textarea.lines()[0].clone();
    let cursor_pos = textarea.cursor().1;
    let before_cursor = &query[..cursor_pos.min(query.len())];

    let mut temp_tracker = crate::autocomplete::BraceTracker::new();
    temp_tracker.rebuild(before_cursor);
    let (context, partial) = analyze_context(before_cursor, &temp_tracker);

    let base_query = query_state.base_query_for_suggestions.as_deref();

    match context {
        SuggestionContext::FunctionContext => {
            insert_function_suggestion(textarea, &query, cursor_pos, &partial, suggestion);
        }
        SuggestionContext::ObjectKeyContext => {
            insert_object_key_suggestion(textarea, &query, cursor_pos, &partial, suggestion);
        }
        SuggestionContext::FieldContext => {
            insert_field_suggestion(
                textarea,
                &query,
                cursor_pos,
                &partial,
                suggestion,
                before_cursor,
                base_query,
            );
        }
        SuggestionContext::VariableContext => {
            insert_variable_suggestion(textarea, &query, cursor_pos, &partial, suggestion);
        }
    }
}
