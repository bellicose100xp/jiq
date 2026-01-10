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

    // Delete entire line, not just up to cursor, to avoid leaving text after cursor
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
///
/// # Arguments
/// * `app` - Mutable reference to the App struct
/// * `suggestion` - The suggestion to insert
pub fn insert_suggestion_from_app(app: &mut App, suggestion: &Suggestion) {
    let query_state = match &mut app.query {
        Some(q) => q,
        None => return,
    };

    insert_suggestion(&mut app.input.textarea, query_state, suggestion);

    app.autocomplete.hide();
    app.results_scroll.reset();
    app.error_overlay_visible = false;

    // Execute immediately for instant feedback (no debounce delay)
    let query = app.input.textarea.lines()[0].as_ref();
    app.input.brace_tracker.rebuild(query);
    query_state.execute_async(query);

    // AI update happens in poll_query_response() when result arrives
}

/// Insert an autocomplete suggestion at the current cursor position
/// Uses explicit state-based formulas for each context type
///
/// Returns the new query string after insertion
pub fn insert_suggestion(
    textarea: &mut TextArea<'_>,
    query_state: &mut QueryState,
    suggestion: &Suggestion,
) {
    let suggestion_text = &suggestion.text;
    let query = textarea.lines()[0].clone();
    let cursor_pos = textarea.cursor().1;
    let before_cursor = &query[..cursor_pos.min(query.len())];

    let mut temp_tracker = crate::autocomplete::BraceTracker::new();
    temp_tracker.rebuild(before_cursor);
    let (context, partial) = analyze_context(before_cursor, &temp_tracker);

    // Get base_query for FieldContext append vs. replace logic
    let base_query = query_state.base_query_for_suggestions.as_deref();

    if context == SuggestionContext::FunctionContext {
        let replacement_start = cursor_pos.saturating_sub(partial.len());
        let insert_text = if suggestion.needs_parens {
            format!("{}(", suggestion_text)
        } else {
            suggestion_text.to_string()
        };
        replace_partial_at_cursor(
            textarea,
            &query,
            cursor_pos,
            replacement_start,
            &insert_text,
        );
        return;
    }

    if context == SuggestionContext::ObjectKeyContext {
        let replacement_start = cursor_pos.saturating_sub(partial.len());
        replace_partial_at_cursor(
            textarea,
            &query,
            cursor_pos,
            replacement_start,
            suggestion_text,
        );
        return;
    }

    // FieldContext: Calculate replacement_start based on suggestion prefix and partial
    let replacement_start = if partial.is_empty() {
        // No partial text - check if we need to replace a trailing separator
        if cursor_pos > 0 {
            let char_before_cursor = query.chars().nth(cursor_pos - 1);
            if (char_before_cursor == Some('.') && suggestion_text.starts_with('.'))
                || (char_before_cursor == Some('[') && suggestion_text.starts_with('['))
                || (char_before_cursor == Some('{') && suggestion_text.starts_with('{'))
                || (char_before_cursor == Some('.') && suggestion_text.starts_with("[]"))
                || (char_before_cursor == Some('.') && suggestion_text.starts_with("{}"))
            {
                // Replace the trailing separator to avoid double dots/brackets
                cursor_pos - 1
            } else {
                // Just append the suggestion at cursor
                cursor_pos
            }
        } else {
            cursor_pos
        }
    } else if suggestion_text.starts_with("[]") || suggestion_text.starts_with("{}") {
        // Array/object iteration syntax - append if query matches base, replace if user edited
        if let Some(base) = base_query {
            if before_cursor == base {
                // Query matches base exactly - append array syntax
                cursor_pos
            } else if cursor_pos > partial.len() {
                // User has edited - replace the ".partial" with array syntax
                let pos_before_partial = cursor_pos - partial.len();
                if pos_before_partial > 0 && query.chars().nth(pos_before_partial - 1) == Some('.')
                {
                    pos_before_partial - 1
                } else {
                    cursor_pos
                }
            } else {
                cursor_pos
            }
        } else {
            // No base query - just append
            cursor_pos
        }
    } else if suggestion_text.starts_with('[')
        || suggestion_text.starts_with('{')
        || suggestion_text.starts_with('.')
    {
        // Suggestion has separator - replace from trigger position (include the dot)
        cursor_pos.saturating_sub(partial.len() + 1)
    } else {
        // Suggestion needs separator - replace from partial position (keep the dot)
        cursor_pos.saturating_sub(partial.len())
    };

    replace_partial_at_cursor(
        textarea,
        &query,
        cursor_pos,
        replacement_start,
        suggestion_text,
    );
}
