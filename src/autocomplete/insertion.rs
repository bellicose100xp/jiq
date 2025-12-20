//! Autocomplete suggestion insertion logic
//!
//! This module handles inserting autocomplete suggestions into the query,
//! managing cursor positioning, and executing the updated query.

use tui_textarea::TextArea;

use crate::app::App;
use crate::autocomplete::autocomplete_state::Suggestion;
use crate::autocomplete::{SuggestionContext, analyze_context, find_char_before_field_access};
use crate::query::{CharType, QueryState};

#[cfg(debug_assertions)]
use log::debug;

// Re-export sub-module functions
pub use self::cursor::move_cursor_to_column;
pub use self::query_manipulation::extract_middle_query;

// Module declarations
#[path = "insertion/cursor.rs"]
mod cursor;
#[path = "insertion/query_manipulation.rs"]
mod query_manipulation;

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
    let base_query = match &query_state.base_query_for_suggestions {
        Some(b) => b.clone(),
        None => textarea.lines()[0].clone(), // Fallback (shouldn't happen)
    };

    let query = textarea.lines()[0].clone();
    let cursor_pos = textarea.cursor().1;
    let before_cursor = &query[..cursor_pos.min(query.len())];

    #[cfg(debug_assertions)]
    debug!(
        "insert_suggestion: current_query='{}' base_query='{}' suggestion='{}' cursor_pos={}",
        query, base_query, suggestion_text, cursor_pos
    );

    let mut temp_tracker = crate::autocomplete::BraceTracker::new();
    temp_tracker.rebuild(before_cursor);
    let (context, partial) = analyze_context(before_cursor, &temp_tracker);

    #[cfg(debug_assertions)]
    debug!(
        "context_analysis: context={:?} partial='{}'",
        context, partial
    );

    // Function context: simple replacement without dots or complex formulas
    if context == SuggestionContext::FunctionContext {
        let replacement_start = cursor_pos.saturating_sub(partial.len());

        let insert_text = if suggestion.needs_parens {
            format!("{}(", suggestion_text)
        } else {
            suggestion_text.to_string()
        };

        let new_query = format!(
            "{}{}{}",
            &query[..replacement_start],
            insert_text,
            &query[cursor_pos..]
        );

        #[cfg(debug_assertions)]
        debug!(
            "function_context_replacement: partial='{}' new_query='{}'",
            partial, new_query
        );

        textarea.delete_line_by_head();
        textarea.insert_str(&new_query);

        let target_pos = replacement_start + insert_text.len();
        move_cursor_to_column(textarea, target_pos);

        return;
    }

    // ObjectKeyContext: handles cases like `{na` -> `{name` or `{name: .name, ag` -> `{name: .name, age`
    if context == SuggestionContext::ObjectKeyContext {
        let replacement_start = cursor_pos.saturating_sub(partial.len());

        let new_query = format!(
            "{}{}{}",
            &query[..replacement_start],
            suggestion_text,
            &query[cursor_pos..]
        );

        #[cfg(debug_assertions)]
        debug!(
            "object_key_context_replacement: partial='{}' new_query='{}'",
            partial, new_query
        );

        textarea.delete_line_by_head();
        textarea.insert_str(&new_query);

        let target_pos = replacement_start + suggestion_text.len();
        move_cursor_to_column(textarea, target_pos);

        return;
    }

    let char_before = find_char_before_field_access(before_cursor, &partial);
    let trigger_type = QueryState::classify_char(char_before);

    // Preserves complex expressions like if/then/else, functions between base and current field
    let mut middle_query = extract_middle_query(&query, &base_query, before_cursor, &partial);
    let mut adjusted_base = base_query.clone();
    let mut adjusted_suggestion = suggestion_text.to_string();

    #[cfg(debug_assertions)]
    debug!(
        "field_context: partial='{}' char_before={:?} trigger_type={:?} middle_query='{}'",
        partial, char_before, trigger_type, middle_query
    );

    // Nested arrays: .services[].capacityProviderStrategy[].field
    // Move [] from middle_query to base when user types []
    if trigger_type == CharType::CloseBracket && middle_query == "[]" {
        #[cfg(debug_assertions)]
        debug!("nested_array_adjustment: detected [] in middle_query, moving to base");

        adjusted_base = format!("{}{}", base_query, middle_query);
        middle_query = String::new();

        // Strip [] and leading dot from suggestion (already in query, formula adds dot)
        if let Some(stripped) = adjusted_suggestion.strip_prefix("[]") {
            adjusted_suggestion = stripped.strip_prefix('.').unwrap_or(stripped).to_string();

            #[cfg(debug_assertions)]
            debug!("nested_array_adjustment: stripped [] and leading dot from suggestion");
        }

        #[cfg(debug_assertions)]
        debug!(
            "nested_array_adjustment: adjusted_base='{}' adjusted_suggestion='{}' middle_query='{}'",
            adjusted_base, adjusted_suggestion, middle_query
        );
    }

    // Prevent double dots: "." + ".services" = "..services"
    let new_query = if adjusted_base == "."
        && adjusted_suggestion.starts_with('.')
        && middle_query.is_empty()
    {
        #[cfg(debug_assertions)]
        debug!("formula: root_replacement (special case for root '.')");

        adjusted_suggestion.to_string()
    } else {
        let formula_result = match trigger_type {
            CharType::NoOp => {
                // Add dot for path continuation unless suggestion starts with special char
                let needs_dot = !adjusted_suggestion.starts_with('[')
                    && !adjusted_suggestion.starts_with('{')
                    && !adjusted_suggestion.starts_with('.')
                    && adjusted_base != ".";

                if needs_dot {
                    #[cfg(debug_assertions)]
                    debug!("formula: NoOp -> base + middle + '.' + suggestion");

                    format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
                } else {
                    #[cfg(debug_assertions)]
                    debug!("formula: NoOp -> base + middle + suggestion (no dot added)");

                    format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
                }
            }
            CharType::CloseBracket => {
                #[cfg(debug_assertions)]
                debug!("formula: CloseBracket -> base + middle + '.' + suggestion");

                format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::PipeOperator | CharType::Semicolon | CharType::Comma | CharType::Colon => {
                #[cfg(debug_assertions)]
                debug!("formula: Separator -> base + middle + ' ' + suggestion");

                // Trim trailing space to avoid double spaces
                let trimmed_middle = middle_query.trim_end();
                format!(
                    "{}{} {}",
                    adjusted_base, trimmed_middle, adjusted_suggestion
                )
            }
            CharType::OpenParen => {
                #[cfg(debug_assertions)]
                debug!(
                    "formula: OpenParen -> base + middle + suggestion (paren already in middle)"
                );

                // ( already in middle_query
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::OpenBracket => {
                #[cfg(debug_assertions)]
                debug!(
                    "formula: OpenBracket -> base + middle + suggestion (bracket already in middle)"
                );

                // [ already in middle_query
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::OpenBrace => {
                #[cfg(debug_assertions)]
                debug!(
                    "formula: OpenBrace -> base + middle + suggestion (brace already in middle)"
                );

                // { already in middle_query
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::QuestionMark => {
                #[cfg(debug_assertions)]
                debug!("formula: QuestionMark -> base + middle + '.' + suggestion");

                format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::Dot => {
                #[cfg(debug_assertions)]
                debug!("formula: Dot -> base + middle + suggestion");

                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            CharType::CloseParen | CharType::CloseBrace => {
                #[cfg(debug_assertions)]
                debug!("formula: CloseParen/CloseBrace -> base + middle + '.' + suggestion");

                format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
            }
        };

        #[cfg(debug_assertions)]
        debug!(
            "formula_components: base='{}' middle='{}' suggestion='{}'",
            adjusted_base, middle_query, adjusted_suggestion
        );

        formula_result
    };

    #[cfg(debug_assertions)]
    debug!("new_query_constructed: '{}'", new_query);

    textarea.delete_line_by_head();
    textarea.insert_str(&new_query);

    #[cfg(debug_assertions)]
    debug!("query_after_insertion: '{}'", textarea.lines()[0]);

    let target_pos = new_query.len();
    move_cursor_to_column(textarea, target_pos);
}
